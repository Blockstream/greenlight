import pytest
from gltesting.fixtures import *
from glclient import Signer, Scheduler, Credentials, TlsConfig


@pytest.fixture
def creds(nobody_id):
    """Nobody credentials for the tests."""
    creds = Credentials.nobody_with(
        nobody_id.cert_chain, nobody_id.private_key, nobody_id.caroot
    )
    return creds


@pytest.fixture
def tls(creds):
    """Just a preconfigured TlsConfig."""
    return TlsConfig(creds=creds)


@pytest.fixture
def signer(scheduler, tls):
    secret = b"\x00" * 32
    network = "regtest"
    return Signer(secret, network=network, tls=tls)


@pytest.fixture
def sclient(signer, creds):
    """Just a preconfigured scheduler client.

    This scheduler client is configured with a secret for easy
    registration and recovery, but no mTLS certificate yet.
    """
    network = "regtest"
    return Scheduler(signer.node_id(), network=network, creds=creds)
