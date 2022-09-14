from binascii import hexlify, unhexlify
from glcli import environment as env
from glclient import TlsConfig, Scheduler, Amount, nodepb as pb
from google.protobuf.descriptor import FieldDescriptor
from pathlib import Path
from threading import Thread
import click
import functools
import json
import logging
import os
import glclient
import re
import secrets
import struct
import sys
import threading
import time
from filelock import FileLock


logger = logging.getLogger("glapi.cli")
logger.setLevel(logging.DEBUG)
handler = logging.StreamHandler(sys.stderr)
handler.setLevel(logging.DEBUG)
formatter = logging.Formatter("[%(asctime)s - %(levelname)s] %(message)s")
handler.setFormatter(formatter)
logger.addHandler(handler)


class Tls:
    """Encapsulation of the fs convention followed by glcli
    """
    def __init__(self):
        self.tls = TlsConfig()
        cert_path = Path('device.crt')
        key_path = Path('device-key.pem')
        have_certs = cert_path.exists() and key_path.exists()
        if have_certs:
            device_cert = open('device.crt', 'rb').read()
            device_key = open('device-key.pem', 'rb').read()
            self.tls = self.tls.identity(device_cert, device_key)
            logger.info("Configuring client with user identity.")
        else:
            logger.info("Configuring client with the NOBODY identity.")

class Signer:
    def __init__(self, tls: Tls, network='testnet'):
        """Initialize the signer based on the cwd
        """
        self.lock = FileLock("hsm_secret.lock")

        self.lock.acquire()
        secrets_file = Path("hsm_secret")
        if not secrets_file.exists():
            logger.info(f"No {secrets_file} file found, generating a new secret key")
            secret = secrets.token_bytes(32)
            with open(
                os.open(secrets_file, os.O_CREAT | os.O_WRONLY, 0o600), "wb"
            ) as f:
                f.write(secret)
        else:
            logger.debug(f"Found existing {secrets_file} file, loading secret from it")
            with open(secrets_file, "rb") as f:
                secret = f.read(32)

        self.inner = glclient.Signer(secret, network, tls.tls)


class Context:
    def load_metadata(self, tls):
        """Load the metadata from the metadata.toml file if it exists.

        If it doesn't exist, create it by instantiating a signer
        briefly, extract the information and then destroy the signer
        again, unlocking the state directory for other signers.

        """
        path = Path("metadata.json")
        if not path.exists():
            signer = Signer(self.tls)
            node_id = signer.inner.node_id()
            self.metadata = {
                'hex_node_id': signer.inner.node_id().hex(),
                'signer_version': signer.inner.version(),
            }
            del signer
            json.dump(self.metadata, path.open(mode='w'), sort_keys=True, indent=2)
            self.metadata['node_id'] = node_id
        else:
            self.metadata = json.load(path.open(mode='r'))
            self.metadata['node_id'] = bytes.fromhex(self.metadata['hex_node_id'])

    def __init__(self, network='testnet', start_hsmd=False):
        self.tls = Tls().tls
        self.load_metadata(self.tls)
        self.scheduler = Scheduler(self.metadata['node_id'], network, self.tls)
        self.scheduler.tls = self.tls
        self.node = None
        self.scheduler_chan = None
        self.node_id = self.metadata['node_id']

    def get_node(self):
        if self.node is None:
            self.node = self.scheduler.node()
        return self.node


class AmountType(click.ParamType):
    name = 'amount'

    def convert(self, value, param, ctx):
        if isinstance(value, pb.Amount):
            return value

        if value.lower() == 'any':
            return pb.Amount(any=True)
        elif value.lower() == 'all':
            return pb.Amount(all=True)

        g = re.match(r'([0-9\.]+)(btc|msat|sat)?', value)
        if g is None:
            raise ValueError(f'Unable to parse {value} as an amount.')

        num, suffix = g.groups()

        suffix2unit = {
            'btc': 'bitcoin',
            'sat': 'satoshi',
            'msat': 'millisatoshi',
            None: 'millisatoshi',
        }

        if suffix is None:
            logger.warn(
                "Unit-less amounts are deprecated to avoid implicit conversion to millisatoshi"
            )

        args = {suffix2unit[suffix]: int(num)}
        return pb.Amount(**args)


def pb2dict(p):
    res = {}
    defaults = True
    for desc, val in p.ListFields():
        # If the parent of any field is contained in a oneof, then we
        # all are.
        defaults = defaults and desc.containing_oneof is None

        if desc.type == FieldDescriptor.TYPE_MESSAGE:
            if desc.label == FieldDescriptor.LABEL_REPEATED:
                val = [pb2dict(v) for v in val]
            else:
                val = pb2dict(val)
        res[desc.name] = val

    # Fill in the default variables so we don't end up with changing
    # keys all the time. If we are in a oneof we don't show them since
    # they are just redundant.
    if defaults:
        for desc in p.DESCRIPTOR.fields:
            if desc.name in res:
                continue
            if desc.label == FieldDescriptor.LABEL_REPEATED:
                res[desc.name] = []
            elif desc.type == FieldDescriptor.TYPE_MESSAGE:
                res[desc.name] = {}
            else:
                res[desc.name] = desc.default_value

    return res


def dict2jsondict(d):
    """Hexlify all binary fields so they can be serialized with `json.dumps`
    """
    if isinstance(d, list):
        return [dict2jsondict(e) for e in d]
    elif isinstance(d, bytes):
        return hexlify(d).decode('ASCII')
    elif isinstance(d, dict):
        return {k: dict2jsondict(v) for k, v in d.items()}
    else:
        return d


def pbprint(pb):
    print(json.dumps(dict2jsondict(pb2dict(pb)), indent=2))


@click.group()
@click.option('--testenv', is_flag=True)
@click.option('--hsmd/--no-hsmd', default=True)
@click.pass_context
def cli(ctx, testenv, hsmd):

    # Disable hsmd if we explicitly get told to run it in the
    # foreground
    nohsmd_subcmds = ['scheduler', 'hsmd']
    hsmd = hsmd and (ctx.invoked_subcommand not in nohsmd_subcmds)

    if testenv:
        os.environ.update(env.test)
    else:
        os.environ.update(env.prod)

    if ctx.obj is None:
        ctx.obj = Context(start_hsmd=hsmd)


@cli.group()
def scheduler():
    pass


@scheduler.command()
@click.option("--network", type=str, default="testnet")
@click.pass_context
def register(ctx, network):
    # Reinitialize the signer with the right network, so register will pick that up
    signer = ctx.obj.signer
    node_id = ctx.obj.node_id
    hex_node_id = hexlify(node_id).decode("ASCII")
    logger.debug(f"Registering new node with node_id={hex_node_id} for {network}")
    scheduler = ctx.obj.scheduler
    res = scheduler.register(signer)

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
    signer = ctx.obj.signer
    node_id = ctx.obj.node_id
    logger.debug(f"Recovering access to node_id={hexlify(node_id).decode('ASCII')}")
    scheduler = ctx.obj.scheduler

    res = scheduler.recover(signer)

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
    node_id = ctx.obj.node_id
    pbprint(ctx.obj.scheduler.get_node_info())


@scheduler.command()
@click.pass_context
def schedule(ctx):
    grpc_uri = os.environ.get("GL_SCHEDULER_GRPC_URI")
    node_id = ctx.obj.node_id
    logger.info(f"Scheduling {node_id.hex()} with scheduler {grpc_uri}")
    scheduler = ctx.obj.scheduler
    try:
        res = scheduler.schedule()
        pbprint(res)
    except ValueError as e:
        raise click.ClickException(message=e.args[0])

@cli.command()
@click.pass_context
def hsmd(ctx):
    """Run the hsmd against the scheduler."""
    signer = Signer(Tls())
    hsm = signer.inner.run_in_foreground()


@cli.command()
@click.pass_context
def getinfo(ctx):
    node = ctx.obj.get_node()
    res = node.get_info()
    pbprint(res)


@cli.command()
@click.pass_context
def stop(ctx):
    node = ctx.obj.get_node()
    res = node.stop()
    print("Node shut down")


@cli.command()
@click.argument("node_id")
@click.argument("addr", required=False)
@click.pass_context
def connect(ctx, node_id, addr):
    node = ctx.obj.get_node()
    res = node.connect_peer(node_id=node_id, addr=addr)
    pbprint(res)


@cli.command()
#@click.argument("node_id", required=False)
@click.pass_context
def listpeers(ctx):
    node = ctx.obj.get_node()
    res = node.list_peers()
    pbprint(res)


@cli.command()
@click.argument("node_id")
@click.pass_context
def disconnect(ctx, node_id):
    node = ctx.obj.get_node()
    res = node.disconnect_peer(node_id)
    pbprint(res)


@cli.command()
@click.pass_context
def newaddr(ctx):
    node = ctx.obj.get_node()
    res = node.new_address()
    pbprint(res)


@cli.command()
#@click.option("--minconf", required=False, type=int)
@click.pass_context
def listfunds(ctx):
    node = ctx.obj.get_node()
    res = node.list_funds()
    pbprint(res)


@cli.command()
@click.argument("destination")
@click.argument("amount", type=AmountType())
@click.option("--minconf", required=False, type=int)
@click.pass_context
def withdraw(ctx, destination, amount, minconf=1):
    node = ctx.obj.get_node()
    res = node.withdraw(destination, amount, minconf)
    pbprint(res)


@cli.command()
@click.argument("nodeid")
@click.argument("amount", type=AmountType())
@click.option("--minconf", required=False, type=int, default=1)
@click.pass_context
def fundchannel(ctx, nodeid, amount, minconf):
    node = ctx.obj.get_node()
    res = node.fund_channel(
        node_id=nodeid,
        amount=amount,
        announce=False,
        minconf=minconf,
    )
    pbprint(res)


@cli.command()
@click.argument("peer_id")
@click.option("--timeout", required=False, type=int)
@click.option("--address", required=False, type=str)
@click.pass_context
def close(ctx, peer_id, timeout=None, address=None):
    node = ctx.obj.get_node()
    res = node.close_channel(peer_id, timeout=timeout, address=address)
    pbprint(res)


@cli.command()
@click.argument("label")
@click.argument("amount", required=False, type=AmountType())
@click.option("--description", required=False, type=str)
@click.pass_context
def invoice(ctx, label, amount=None, description=None):
    node = ctx.obj.get_node()
    if amount is None:
        amount = Amount(any=True)

    args = {
        "label": label,
        "amount": amount,
    }
    args["description"] = description if description is not None else ""

    res = node.create_invoice(**args)
    pbprint(res)


@cli.command()
@click.argument("invoice")
@click.pass_context
@click.option("--amount", required=False, type=AmountType())
@click.option("--timeout", required=False, type=int)
def pay(ctx, invoice, amount=None, timeout=0):
    node = ctx.obj.get_node()
    res = node.pay(bolt11=invoice, amount=amount, timeout=timeout)
    pbprint(res)


@cli.command()
@click.pass_context
def stream_incoming(ctx):
    """Listen for incoming payments and print details to stdout.
    """
    node = ctx.obj.get_node()
    for e in node.stream_incoming():
        pbprint(e)
        sys.stdout.flush()


@cli.command()
@click.argument('node_id')
@click.argument('amount', type=AmountType())
@click.option("--label", required=False)
@click.option("--routehints", required=False)
@click.option("--extratlvs", required=False)
@click.pass_context
def keysend(ctx, node_id, amount, label, routehints, extratlvs):
    """Send a spontaneous payment to the specified node.
    """
    # Convert the advanced arguments.
    if routehints is not None:
        arr = json.loads(routehints)
        routehints = []
        if not isinstance(arr, list):
            raise click.UsageError("Routehints must be a JSON encoded list of lists of routehint hops")
        for rharr in arr:
            routehint = pb.Routehint(hops=[])
            if not isinstance(rharr, list):
                raise click.UsageError("Routehints must be a JSON encoded list of lists of routehint hops")
            for rh in rharr:
                rh['node_id'] = unhexlify(rh['node_id'])
                r = pb.RoutehintHop(
                    **rh
                )
                routehint.hops.append(r)
            routehints.append(routehint)

    if extratlvs is not None:
        arr = json.loads(extratlvs)
        extratlvs = []
        if not isinstance(extratlvs, list):
            raise click.UsageError('--extratlvs must be a JSON encoded list of `{"type": 1234, "value": "DECAFBAD"}` entries')
        for a in arr:
            t = a['type']
            v = unhexlify(a['value'])
            extratlvs.append(pb.TlvField(
                type=t,
                value=v,
            ))

    print(extratlvs)
    node = ctx.obj.get_node()
    res = node.keysend(node_id=node_id, amount=amount, label=label,
                       routehints=routehints, extratlvs=extratlvs)
    pbprint(res)


@cli.command()
@click.pass_context
def log(ctx):
    node = ctx.obj.get_node()

    def tail(node):
        for entry in node.stream_log():
            print(entry.line.strip())

    t = threading.Thread(target=tail, args=(node, ), daemon=True)
    t.start()
    t.join()


@cli.command()
@click.option('--payment-hash', '-h', required=False)
@click.option('--label', '-l', required=False)
@click.option('--invoice', '-i', required=False)
@click.pass_context
def listinvoices(ctx, payment_hash=None, label=None, invoice=None):
    node = ctx.obj.get_node()
    res = node.list_invoices(
        payment_hash=payment_hash,
        invstring=invoice,
        label=label
    )
    pbprint(res)


@cli.command()
@click.pass_context
def listpays(ctx):
    node = ctx.obj.get_node()
    res = node.list_payments()
    pbprint(res)


if __name__ == "__main__":
    cli()
