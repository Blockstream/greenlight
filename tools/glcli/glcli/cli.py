from binascii import hexlify, unhexlify
from glcli import environment as env
from glclient import Scheduler, Credentials
from pyln.grpc import Amount, AmountOrAll, AmountOrAny
from google.protobuf.descriptor import FieldDescriptor
from pathlib import Path
from typing import Optional
import click
import json
import logging
import os
import glclient
import re
import secrets
import sys
import threading
import signal


logger = logging.getLogger("glcli")
logger.setLevel(logging.DEBUG)
handler = logging.StreamHandler(sys.stderr)
handler.setLevel(logging.DEBUG)
formatter = logging.Formatter("[%(asctime)s - %(levelname)s] %(message)s")
handler.setFormatter(formatter)
logger.addHandler(handler)


class Creds:
    """Encapsulation of the fs convention followed by glcli."""

    def __init__(self) -> "Creds":
        """Initialize the credentials."""
        self.creds = Credentials()
        creds_path = Path("credentials.gfs")
        # legacy paths used by TlsConfig
        cert_path = Path("device.crt")
        key_path = Path("device-key.pem")
        ca_path = Path("ca.pem")
        have_certs = cert_path.exists() and key_path.exists() and ca_path.exists()
        if creds_path.exists():
            self.creds = Credentials.from_path(str(creds_path))
            logger.info("Configuring client with device credentials")
        elif have_certs:
            logger.info("Configuring client with device credentials (legacy)")
            device_cert = open(str(cert_path), "rb").read()
            device_key = open(str(key_path), "rb").read()
            ca = open(str(ca_path), "rb").read()
            rune = ""
            if Path("rune").exists():
                rune = open("rune", "r").read()
            self.creds = Credentials.from_parts(device_cert, device_key, ca, rune)
        else:
            logger.info("Configuring client with NOBODY credentials.")


class Signer:
    def __init__(self, creds: Creds, network='testnet'):
        """Initialize the signer based on the cwd
        """
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

        self.inner = glclient.Signer(secret, network, creds.creds)


class Context:
    def load_metadata(self, creds):
        """Load the metadata from the metadata.toml file if it exists.

        If it doesn't exist, create it by instantiating a signer
        briefly, extract the information and then destroy the signer
        again, unlocking the state directory for other signers.

        """
        path = Path("metadata.json")
        if not path.exists():
            signer = Signer(creds)
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
        self.creds = Creds()
        self.load_metadata(self.creds)
        self.scheduler = Scheduler(self.metadata['node_id'], network, creds=self.creds.creds)
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
        if isinstance(value, Amount):
            return value

        g = re.match(r'([0-9\.]+)(btc|msat|sat)?', value)
        if g is None:
            raise ValueError(f'Unable to parse {value} as an amount.')

        num, suffix = g.groups()

        unit = {
            'btc': 10**8 * 10**3,
            'sat': 1000,
            'msat': 1,
        }

        return Amount(msat=int(num) * unit[suffix])


class AmountOrAnyType(click.ParamType):
    name = 'amount'

    def convert(self, value, param, ctx):
        if isinstance(value, AmountOrAny):
            return value

        if value.lower() == 'any':
            return AmountOrAny(any=True)
        
        g = re.match(r'([0-9\.]+)(btc|msat|sat)?', value)

        if g is None:
            raise ValueError(f'Unable to parse {value} as an amount.')

        num, suffix = g.groups()

        unit = {
            'btc': 10**8 * 10**3,
            'sat': 1000,
            'msat': 1,
        }

        return AmountOrAny(
            amount=Amount(msat=int(num) * unit[suffix])
        )


class AmountOrAllType(click.ParamType):
    name = 'amount'

    def convert(self, value, param, ctx):
        if isinstance(value, AmountOrAll):
            return value

        if value.lower() == 'all':
            return AmountOrAll(all=True)

        g = re.match(r'([0-9\.]+)(btc|msat|sat)?', value)

        if g is None:
            raise ValueError(f'Unable to parse {value} as an amount.')

        num, suffix = g.groups()

        unit = {
            'btc': 10**8 * 10**3,
            'sat': 1000,
            'msat': 1,
        }

        return AmountOrAll(
            amount=Amount(msat=int(num) * unit[suffix])
        )


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
        if isinstance(val, bytes):
            val = val.hex()
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
    elif isinstance(d, dict):
        return {k: dict2jsondict(v) for k, v in d.items()}
    elif isinstance(d, (bytes, bytearray)):
        return d.hex()
    else:
        return d


def pbprint(pb):
    dta = pb2dict(pb)
    dta = dict2jsondict(dta)
    print(json.dumps(dta))


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
@click.option("--invite", type=str, default=None)
@click.pass_context
def register(ctx, network, invite):
    # Reinitialize the signer with the right network, so register will pick that up
    signer = Signer(creds=ctx.obj.creds, network=network)
    node_id = ctx.obj.node_id
    hex_node_id = hexlify(node_id).decode("ASCII")
    logger.debug(f"Registering new node with node_id={hex_node_id} for {network}")
    # Reinitialize the Scheduler with the passed network for register.
    scheduler = Scheduler(node_id, network=network, creds=ctx.obj.scheduler.creds)
    res = scheduler.register(signer.inner, invite_code=invite)

    with open("credentials.gfs", "wb") as f:
        f.write(res.creds)

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
    signer = Signer(Creds())
    node_id = ctx.obj.node_id
    logger.debug(f"Recovering access to node_id={hexlify(node_id).decode('ASCII')}")
    scheduler = ctx.obj.scheduler

    res = scheduler.recover(signer.inner)

    with open("credentials.gfs", "wb") as f:
        f.write(res.creds)

    with open("device-key.pem", "w") as f:
        f.write(res.device_key)

    with open("device.crt", "w") as f:
        f.write(res.device_cert)

    with open("ca.pem", "wb") as f:
        f.write(env.ca_pem)

    pbprint(res)

@scheduler.command()
@click.pass_context
def upgradecreds(ctx):
    signer = Signer(Creds())
    creds = ctx.obj.creds.creds.upgrade(ctx.obj.scheduler.inner, signer.inner.inner)

    with open("credentials.gfs", "wb") as f:
        f.write(creds.to_bytes())


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
    signer = Signer(Creds())
    signer_handle = signer.inner.run_in_thread()
    
    def signal_handler(_signal, _frame):
        signer_handle.shutdown()
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    
    while True:
        signal.pause()


@cli.command()
@click.pass_context
def getinfo(ctx) -> None:
    node = ctx.obj.get_node()
    res = node.get_info()
    pbprint(res)


@cli.command()
@click.pass_context
def stop(ctx) -> None:
    node = ctx.obj.get_node()
    res = node.stop()
    print("Node shut down")


@cli.command()
@click.argument("id", type=str)
@click.argument("host", required=False, type=str)
@click.argument("port", required=False, type=int)
@click.pass_context
def connect(ctx, id: str, host: Optional[str], port: Optional[int]):
    node = ctx.obj.get_node()
    res = node.connect_peer(node_id=id, host=host, port=port)
    pbprint(res)


@cli.command()
#@click.argument("node_id", required=False)
@click.pass_context
def listpeers(ctx):
    node = ctx.obj.get_node()
    res = node.list_peers()
    pbprint(res)


@cli.command()
@click.pass_context
def listclosedchannels(ctx):
    node = ctx.obj.get_node()
    res = node.list_closed_channels()
    pbprint(res)


@cli.command()
@click.argument("string", type=str)
@click.pass_context
def decode(ctx, string: str):
    node = ctx.obj.get_node()
    res = node.decode(string)
    pbprint(res)


@cli.command()
@click.argument("bolt11", type=str)
@click.argument("description", required=False, type=str)
@click.pass_context
def decodepay(ctx, bolt11: str, description: Optional[str] = None):
    node = ctx.obj.get_node()
    res = node.decodepay(bolt11=bolt11, description=description)
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
@click.argument("amount", type=AmountOrAllType())
@click.option("--minconf", required=False, type=int, default=1)
@click.pass_context
def fundchannel(ctx, nodeid, amount, minconf):
    node = ctx.obj.get_node()
    nodeid = bytes.fromhex(nodeid)
    res = node.fund_channel(
        id=nodeid,
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
    res = node.close_channel(peer_id, unilateraltimeout=timeout, destination=address)
    pbprint(res)


@cli.command()
@click.argument("label")
@click.argument("amount", required=False, type=AmountOrAnyType())
@click.option("--description", required=False, type=str)
@click.pass_context
def invoice(ctx, label, amount=None, description=None):
    node = ctx.obj.get_node()
    if amount is None:
        amount = Amount(any=True)

    args = {
        "label": label,
        "amount_msat": amount,
    }
    args["description"] = description if description is not None else ""

    res = node.invoice(**args)
    pbprint(res)


@cli.command()
@click.argument("invoice")
@click.pass_context
@click.option("--amount", required=False, type=AmountType())
@click.option("--timeout", required=False, type=int)
def pay(ctx, invoice, amount=None, timeout=0):
    node = ctx.obj.get_node()
    res = node.pay(bolt11=invoice, amount_msat=amount, retry_for=timeout)
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
@click.argument('destination')
@click.argument('amount', type=AmountType())
@click.option("--label", required=False)
@click.option("--routehints", required=False)
@click.option("--extratlvs", required=False)
@click.pass_context
def keysend(ctx, destination, amount, label, routehints, extratlvs):
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
    res = node.keysend(
        destination=destination,
        amount=amount,
        label=label,
        routehints=routehints,
        extratlvs=extratlvs
    )
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
@click.pass_context
def listinvoices(ctx):
    node = ctx.obj.get_node()
    res = node.list_invoices()
    pbprint(res)


@cli.command()
@click.pass_context
def listpays(ctx):
    node = ctx.obj.get_node()
    res = node.listpays()
    pbprint(res)


@cli.command()
def version():
    import glclient
    print(glclient.__version__)


@scheduler.command()
@click.pass_context
def export(ctx):
    print("""
    You are about to export your node from greenlight. This will disable the node
    on greenlight, create an encrypted backup and serve that backup back to you.

    This is intended for users that would like to offboard and run their node on
    their own infrastructure. Due to the security model of Lightning Nodes, we
    will not be able to reactivate the node on greenlight. Please configure and
    test your infrastructure before exporting, to prevent prolonged downtime.
    """)
    answer = input("Do you want to continue (y/N)?")
    if answer.lower() != "y":
        print("Cancelling the export...")
        return

    scheduler = ctx.obj.scheduler
    exp = scheduler.export_node()
    print(exp)
    # TODO Download
    # TODO Decrypt
    # TODO Print about next steps


if __name__ == "__main__":
    cli()
