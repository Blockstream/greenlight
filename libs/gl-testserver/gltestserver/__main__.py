from dataclasses import dataclass
from gltesting import fixtures
from inspect import isgeneratorfunction
from pathlib import Path
from pyln.testing.utils import BitcoinD
from rich.console import Console
from rich.logging import RichHandler
from rich.pretty import pprint
from typing import Any, List
import click
import gltesting
import json
import logging
import tempfile
import time


console = Console()
logging.basicConfig(
    level="DEBUG",
    format="%(message)s",
    datefmt="[%X]",
    handlers=[
        RichHandler(rich_tracebacks=True, tracebacks_suppress=[click], console=console)
    ],
)
logger = logging.getLogger("gltestserver")


@dataclass
class TestServer:
    directory: Path
    bitcoind: BitcoinD
    scheduler: gltesting.scheduler.Scheduler
    finalizers: List[Any]
    clients: gltesting.clients.Clients
    grpc_web_proxy: gltesting.grpcweb.GrpcWebProxy

    def stop(self):
        for f in self.finalizers[::-1]:
            try:
                f()
            except StopIteration:
                continue
            except Exception as e:
                logger.warn(f"Unexpected exception tearing down server: {e}")

    def metadata(self):
        """Construct a dict of config values for this TestServer."""
        return {
            "scheduler_grpc_uri": self.scheduler.grpc_addr,
            "grpc_web_proxy_uri": f"http://localhost:{self.grpc_web_proxy.web_port}",
            "bitcoind_rpc_uri": f"http://rpcuser:rpcpass@localhost:{self.bitcoind.rpcport}",
        }


def build(base_dir: Path):
    # List of teardown functions to call in reverse order.
    finalizers = []

    def callfixture(f, *args, **kwargs):
        """Small shim to bypass the pytest decorator."""
        F = f.__pytest_wrapped__.obj

        if isgeneratorfunction(F):
            it = F(*args, **kwargs)
            v = it.__next__()
            finalizers.append(it.__next__)
            return v
        else:
            return F(*args, **kwargs)

    directory = base_dir / "gl-testserver"

    cert_directory = callfixture(fixtures.cert_directory, directory)
    _root_id = callfixture(fixtures.root_id, cert_directory)
    _users_id = callfixture(fixtures.users_id)
    nobody_id = callfixture(fixtures.nobody_id, cert_directory)
    scheduler_id = callfixture(fixtures.scheduler_id, cert_directory)
    _paths = callfixture(fixtures.paths)
    bitcoind = callfixture(
        fixtures.bitcoind,
        directory=directory,
        teardown_checks=None,
    )
    scheduler = callfixture(
        fixtures.scheduler, scheduler_id=scheduler_id, bitcoind=bitcoind
    )

    clients = callfixture(
        fixtures.clients, directory=directory, scheduler=scheduler, nobody_id=nobody_id
    )

    node_grpc_web_server = callfixture(
        fixtures.node_grpc_web_proxy, scheduler=scheduler
    )

    return TestServer(
        directory=directory,
        bitcoind=bitcoind,
        finalizers=finalizers,
        scheduler=scheduler,
        clients=clients,
        grpc_web_proxy=node_grpc_web_server,
    )


@click.group()
def cli():
    pass


@cli.command()
@click.option(
    "--directory",
    type=click.Path(),
    help="""
      Set the top-level directory for the testserver. This can be used to run
      multiple instances isolated from each other, by giving each isntance a
      different top-level directory. Defaults to '/tmp/'
    """,
)
def run(directory):
    """Start a gl-testserver instance to test against."""
    if not directory:
        directory = Path(tempfile.gettempdir())
    else:
        directory = Path(directory)

    gl = build(base_dir=directory)
    try:
        meta = gl.metadata()
        metafile = gl.directory / "metadata.json"
        metafile.parent.mkdir(parents=True, exist_ok=True)
        logger.debug(f"Writing testserver metadata to {metafile}")
        with metafile.open(mode="w") as f:
            json.dump(meta, f)

        pprint(meta)
        logger.info(
            "Server is up and running with the above config values. To stop press Ctrl-C."
        )
        time.sleep(1800)
    except Exception as e:
        logger.warning(f"Caught exception running testserver: {e}")
        pass
    finally:
        logger.info("Stopping gl-testserver")
        # Now tear things down again.
        gl.stop()


if __name__ == "__main__":
    cli()
