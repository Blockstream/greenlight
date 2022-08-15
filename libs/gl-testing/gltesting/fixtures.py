# Pytest fixtures
import tempfile
from .scheduler import Scheduler
from gltesting.clients import Clients, Client
from ephemeral_port_reserve import reserve
import pytest
from gltesting import certs
from gltesting.identity import Identity
import os
from pathlib import Path
import logging
import sys
from pyln.testing.fixtures import bitcoind, teardown_checks, node_factory, node_cls, test_name, executor, db_provider, test_base_dir, throttler, jsonschemas

logging.basicConfig(level=logging.DEBUG, stream=sys.stdout)
logging.getLogger().addHandler(logging.StreamHandler(sys.stdout))
logging.getLogger("sh").setLevel(logging.ERROR)
logger = logging.getLogger(__name__)


@pytest.fixture()
def directory():
    """Root directory in which we'll generate all dependent files."""

    with tempfile.TemporaryDirectory() as d:
        yield Path(d)


@pytest.fixture()
def cert_directory(directory):
    yield directory / "certs"


@pytest.fixture()
def root_id(cert_directory):
    os.environ.update(
        {
            "GL_CERT_PATH": str(cert_directory),
            "GL_CA_CRT": str(cert_directory / "ca.pem"),
        }
    )

    identity = certs.genca("/")

    yield identity


@pytest.fixture()
def scheduler_id(root_id):
    certs.genca("/services")
    id = certs.gencert("/services/scheduler")
    yield id


@pytest.fixture()
def users_id():
    yield certs.genca("/users")


@pytest.fixture()
def nobody_id(users_id):
    identity = certs.gencert("/users/nobody")
    os.environ.update(
        {
            "GL_NOBODY_CRT": str(identity.cert_chain_path),
            "GL_NOBODY_KEY": str(identity.private_key_path),
        }
    )

    yield identity


@pytest.fixture()
def scheduler(scheduler_id, bitcoind):
    grpc_port = reserve()
    s = Scheduler(bitcoind=bitcoind, grpc_port=grpc_port, identity=scheduler_id)
    logger.debug(f"Scheduler is running at {s.grpc_addr}")
    os.environ.update(
        {
            "GL_SCHEDULER_GRPC_URI": s.grpc_addr,
        }
    )
    s.start()
    yield s

    del os.environ["GL_SCHEDULER_GRPC_URI"]
    s.stop()


@pytest.fixture()
def clients(directory, scheduler, nobody_id):
    clients = Clients(
        directory=directory / "clients", scheduler=scheduler, nobody_id=nobody_id
    )
    yield clients
