from binascii import hexlify, unhexlify
from glapi import environment as env
from glapi import greenlight_pb2 as pb
from glapi import scheduler_pb2 as schedpb
from glapi.greenlight_pb2 import GetInfoRequest, StopRequest, ConnectRequest
from glapi.greenlight_pb2_grpc import NodeStub
from glapi.identity import Identity
from glapi.libhsmd import init as libhsmd_init, handle as libhsmd_handle
from glapi.scheduler_pb2_grpc import SchedulerStub
from google.protobuf.json_format import MessageToJson
from pathlib import Path
from threading import Thread
import click
import functools
import grpc
import logging
import os
import secrets
import struct
import sys
import time


logger = logging.getLogger("glapi.cli")
logger.setLevel(logging.DEBUG)
handler = logging.StreamHandler(sys.stderr)
handler.setLevel(logging.DEBUG)
formatter = logging.Formatter("[%(asctime)s - %(levelname)s] %(message)s")
handler.setFormatter(formatter)
logger.addHandler(handler)


class HSM:
    """Simple wrapper around the keeper of keys."""

    def __init__(self):
        secrets_file = Path("hsm_secret")
        if not secrets_file.exists():
            logging.info(f"No {secrets_file} file found, generating a new secret key")
            secret = secrets.token_bytes(32)
            with open(
                os.open(secrets_file, os.O_CREAT | os.O_WRONLY, 0o600), "wb"
            ) as f:
                f.write(secret)
        else:
            logger.debug(f"Found existing {secrets_file} file, loading secret from it")
            with open(secrets_file, "rb") as f:
                secret = f.read(32)
        logger.debug(f"Initializing libhsmd with secret")
        msg = unhexlify(libhsmd_init(secret.hex(), "bitcoin"))
        self.node_id = msg[2:35]
        self.bip32_key = msg[35:-32]
        self.bolt12_key = msg[-32:]
        self.scheduler_chan = None
        logger.debug(f"libhsmd initialized for node_id={self.node_id.hex()}")

    def sign_challenge(self, challenge: bytes) -> bytes:
        """Sign a 32 byte challenge."""
        assert len(challenge) == 32
        msg = struct.pack(f"!HH{len(challenge)}s", 23, len(challenge), challenge)
        res = libhsmd_handle(1024, 0, None, msg.hex())
        assert len(res) == 2 * 2 + 2 * 64 + 2
        type, signature, recid = struct.unpack("!H64ss", unhexlify(res))
        assert type == 123
        return signature

    def _scheduler_stub(self, grpc_uri):
        if self.scheduler_chan is None:
            identity = get_identity()
            cred = identity.to_channel_credentials()
            self.scheduler_chan = grpc.secure_channel(
                grpc_uri,
                cred,
                options=(
                    (
                        "grpc.ssl_target_name_override",
                        "localhost",
                    ),
                ),
            )

        return SchedulerStub(self.scheduler_chan)

    def run(self):
        grpc_uri = os.environ.get("GL_SCHEDULER_GRPC_URI")
        if grpc_uri.startswith("https://"):
            grpc_uri = grpc_uri[8:]

        logger.debug(
            f"Contacting scheduler at {grpc_uri} to wait for the node to be scheduled."
        )

        scheduler = self._scheduler_stub(grpc_uri)
        while True:
            # Outer loop: wait for our node to get scheduled.
            logger.debug(f"Waiting for node {self.node_id.hex()} to be scheduled")
            res = scheduler.GetNodeInfo(
                schedpb.NodeInfoRequest(
                    node_id=self.node_id,
                    wait=True,
                )
            )
            logger.info(
                f"Node was scheduled at {res.grpc_uri}, opening direct connection"
            )
            self.run_once(res.grpc_uri)

    def run_once(self, uri):
        identity = get_identity()
        cred = identity.to_channel_credentials()
        if uri.startswith("https://"):
            uri = uri[8:]

        chan = grpc.secure_channel(
            uri,
            cred,
            options=(
                (
                    "grpc.ssl_target_name_override",
                    "localhost",
                ),
            ),
        )
        node = NodeStub(chan)
        logger.debug(f"Streaming HSM requests")
        try:
            for r in node.StreamHsmRequests(pb.Empty()):
                if r.context.ByteSize() != 0:
                    capabilities = r.context.capabilities
                    dbid = r.context.dbid
                    node_id = r.context.node_id.hex()
                else:
                    capabilities = 1024 | 2 | 1
                    dbid = 0
                    node_id = None
                response = libhsmd_handle(capabilities, dbid, node_id, r.raw.hex())
                node.RespondHsmRequest(
                    pb.HsmResponse(
                        request_id=r.request_id,
                        raw=unhexlify(response),
                    )
                )
        except grpc.RpcError:
            logger.debug(
                "Error streaming hsm requests from node, likely just got disconnected."
            )
            time.sleep(1)

    def ping(self):
        grpc_uri = os.environ.get("GL_SCHEDULER_GRPC_URI")
        if grpc_uri.startswith("https://"):
            grpc_uri = grpc_uri[8:]

        logger.debug(f"Pinging scheduler at {grpc_uri}")
        scheduler = self._scheduler_stub(grpc_uri)
        scheduler.GetNodeInfo(schedpb.NodeInfoRequest(node_id=self.node_id))


class Context:
    def __init__(self):
        self.hsm = HSM()
        self.identity = get_identity()

        self.scheduler_chan = None

    def get_node(self):
        uri = self.get_node_grpc_uri(self.hsm.node_id)

        if uri.startswith("https://"):
            uri = uri[8:]

        cred = self.identity.to_channel_credentials()
        chan = grpc.secure_channel(
            uri,
            cred,
            options=(
                (
                    "grpc.ssl_target_name_override",
                    "localhost",
                ),
            ),
        )
        return NodeStub(chan)

    @functools.lru_cache(1)
    def get_scheduler(self):
        if self.scheduler_chan is None:
            grpc_uri = os.environ.get("GL_SCHEDULER_GRPC_URI")
            if grpc_uri.startswith("https://"):
                grpc_uri = grpc_uri[8:]
            identity = get_identity(default=True)
            cred = identity.to_channel_credentials()
            self.scheduler_chan = grpc.secure_channel(
                grpc_uri,
                cred,
                options=(
                    (
                        "grpc.ssl_target_name_override",
                        "localhost",
                    ),
                ),
            )
        return SchedulerStub(self.scheduler_chan)

    @functools.lru_cache(1)
    def get_node_grpc_uri(self, node_id):
        logger.debug(f"Contacting scheduler to find grp_uri of node_id={node_id.hex()}")
        scheduler = self.get_scheduler()
        res = scheduler.Schedule(schedpb.ScheduleRequest(node_id=node_id))
        logger.debug(f"Node {node_id.hex()} is running at {res.grpc_uri}")
        return res.grpc_uri


def pbprint(pb):
    print(
        MessageToJson(
            pb,
            including_default_value_fields=True,
            preserving_proto_field_name=True,
            indent=2,
            sort_keys=True,
        )
    )


def get_identity(default=False):
    p = Path("device-key.pem")
    if p.exists() and not default:
        private_key = p.read_bytes()
        cert_chain = Path("device.crt").read_bytes()
        caroot = (Path(os.environ.get("GL_CERT_PATH")) / Path("ca.pem")).read_bytes()
        return Identity(pem=cert_chain, crt=cert_chain, caroot=caroot, key=private_key)
    elif env is not None:
        logger.debug("Using certificate and keys from the environment.py file")
        return Identity(
            pem=env.users_nobody_crt,
            crt=env.users_nobody_crt,
            key=env.users_nobody_key,
            caroot=env.ca_pem,
        )
    else:
        logger.debug("Loading generic installation keys")
        return Identity.from_path("/users/nobody")


@click.group()
@click.pass_context
def cli(ctx):
    for k, v in env.env.items():
        os.environ[k] = v
    ctx.obj = Context()


@cli.group()
def scheduler():
    pass


@scheduler.command()
@click.option("--network", type=str, default="testnet")
@click.pass_context
def register(ctx, network):
    hsm = ctx.obj.hsm
    node_id = hsm.node_id
    bip32_key = hsm.bip32_key + hsm.bolt12_key
    hex_node_id = hexlify(node_id).decode("ASCII")
    logger.debug(f"Registering new node with node_id={hex_node_id} for {network}")
    scheduler = ctx.obj.get_scheduler()

    ch = scheduler.GetChallenge(
        schedpb.ChallengeRequest(
            scope=schedpb.ChallengeScope.REGISTER,
            node_id=node_id,
        )
    )

    # Now have the hsmd sign the challenge
    res = hsm.sign_challenge(ch.challenge)

    res = scheduler.Register(
        schedpb.RegistrationRequest(
            node_id=node_id,
            bip32_key=bip32_key,
            email=None,
            network=network,
            challenge=ch.challenge,
            signature=res,
        )
    )

    with open("device-key.pem", "w") as f:
        f.write(res.device_key)

    with open("device.crt", "w") as f:
        f.write(res.device_cert)

    with open("ca.pem", "wb") as f:
        f.write(env.ca_pem)

    pbprint(res)


@scheduler.command()
@click.pass_context
def recover(ctx):
    node_id = ctx.obj.hsm.node_id
    logger.debug(f"Recovering access to node_id={hexlify(node_id).decode('ASCII')}")
    scheduler = ctx.obj.get_scheduler()

    ch = scheduler.GetChallenge(
        schedpb.ChallengeRequest(
            scope=schedpb.ChallengeScope.RECOVER,
            node_id=node_id,
        )
    )

    # Now have the hsmd sign the challenge
    res = ctx.obj.hsm.sign_challenge(ch.challenge)

    res = scheduler.Recover(
        schedpb.RecoveryRequest(
            node_id=node_id,
            challenge=ch.challenge,
            signature=res,
        )
    )

    with open("device-key.pem", "w") as f:
        f.write(res.device_key)

    with open("device.crt", "w") as f:
        f.write(res.device_cert)

    with open("ca.pem", "wb") as f:
        f.write(env.ca_pem)

    pbprint(res)


@scheduler.command()
@click.pass_context
def ping(ctx):
    node_id = ctx.obj.hsm.node_id
    scheduler = ctx.obj.get_scheduler()
    res = scheduler.GetNodeInfo(schedpb.NodeInfoRequest(node_id=node_id, wait=False))
    pbprint(res)


@scheduler.command()
@click.pass_context
def schedule(ctx):
    grpc_uri = os.environ.get("GL_SCHEDULER_GRPC_URI")
    node_id = ctx.obj.hsm.node_id
    print(f"Scheduling {node_id.hex()} with scheduler {grpc_uri}")
    scheduler = ctx.obj.get_scheduler()

    res = scheduler.Schedule(schedpb.ScheduleRequest(node_id=node_id))
    pbprint(res)


@cli.command()
@click.pass_context
def hsmd(ctx):
    """Run the hsmd against the scheduler."""
    hsm = ctx.obj.hsm

    def ping_loop():
        while True:
            time.sleep(10)
            hsm.ping()

    #Thread(target=ping_loop, daemon=True).start()

    hsm.run()


@cli.command()
@click.pass_context
def getinfo(ctx):
    node = ctx.obj.get_node()
    res = node.GetInfo(GetInfoRequest())
    pbprint(res)


@cli.command()
@click.pass_context
def stop(ctx):
    node = ctx.obj.get_node()
    try:
        res = node.Stop(StopRequest())
        print(res)
    except Exception:
        print("No response received, node was shut down")


@cli.command()
@click.argument("node_id")
@click.argument("addr", required=False)
@click.pass_context
def connect(ctx, node_id, addr):
    node = ctx.obj.get_node()
    res = node.ConnectPeer(ConnectRequest(node_id=node_id, addr=addr))
    pbprint(res)


@cli.command()
@click.argument("node_id", required=False)
@click.pass_context
def listpeers(ctx, node_id=None):
    node = ctx.obj.get_node()
    res = node.ListPeers(pb.ListPeersRequest(node_id=node_id))
    pbprint(res)


@cli.command()
@click.argument("node_id")
@click.pass_context
def disconnect(ctx, node_id):
    node = ctx.obj.get_node()
    res = node.Disconnect(pb.DisconnectRequest(node_id=node_id))
    pbprint(res)


@cli.command()
@click.pass_context
def newaddr(ctx):
    node = ctx.obj.get_node()
    res = node.NewAddr(pb.NewAddrRequest(address_type=pb.BtcAddressType.BECH32))
    pbprint(res)


@cli.command()
@click.option("--minconf", required=False, type=int)
@click.pass_context
def listfunds(ctx, minconf=1):
    node = ctx.obj.get_node()
    res = node.ListFunds(pb.ListFundsRequest())
    pbprint(res)


@cli.command()
@click.argument("destination")
@click.argument("amount", type=int)
@click.option("--minconf", required=False, type=int)
@click.pass_context
def withdraw(ctx, destination, amount, minconf=1):
    node = ctx.obj.get_node()
    res = node.Withdraw(
        pb.WithdrawRequest(
            destination=destination,
            amount=pb.Amount(millisatoshi=amount),
            minconf=pb.Confirmation(blocks=minconf),
        )
    )
    pbprint(res)


@cli.command()
@click.argument("nodeid")
@click.argument("amount", type=int)
@click.option("--minconf", required=False, type=int)
@click.pass_context
def fundchannel(ctx, nodeid, amount, minconf=1):
    node = ctx.obj.get_node()
    res = node.FundChannel(
        pb.FundChannelRequest(
            node_id=unhexlify(nodeid),
            amount=pb.Amount(millisatoshi=amount),
        )
    )
    pbprint(res)


@cli.command()
@click.argument("nodeid")
@click.option("--timeout", required=False, type=int)
@click.option("--address", required=False, type=str)
@click.pass_context
def close(ctx, nodeid, timeout=None, address=None):
    node = ctx.obj.get_node()
    args = {
        "node_id": unhexlify(nodeid),
    }
    if timeout is not None:
        args["unilateraltimeout"] = pb.Timeout(seconds=timeout)

    if address is not None:
        args["destination"] = pb.BitcoinAddress(address=address)

    res = node.CloseChannel(pb.CloseChannelRequest(**args))
    pbprint(res)


@cli.command()
@click.argument("label")
@click.argument("amount", required=False, type=int)
@click.option("--description", required=False, type=str)
@click.pass_context
def invoice(ctx, label, amount=None, description=None):
    node = ctx.obj.get_node()
    args = {
        "label": label,
    }
    if amount is not None:
        args["amount"] = pb.Amount(millisatoshi=amount)

    args["description"] = description if description is not None else ""

    res = node.CreateInvoice(pb.InvoiceRequest(**args))
    pbprint(res)


@cli.command()
@click.argument("invoice")
@click.pass_context
def pay(ctx, invoice):
    node = ctx.obj.get_node()
    res = node.Pay(pb.PayRequest(bolt11=invoice))
    pbprint(res)


@cli.command()
def destroy():
    os.unlink("device-key.pem")
    os.unlink("device.crt")
    os.unlink("ca.pem")
    os.unlink("hsm_secret")


if __name__ == "__main__":
    cli()
