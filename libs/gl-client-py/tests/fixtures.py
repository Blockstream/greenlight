import pytest
from gltesting.fixtures import *
from glclient import Signer, Scheduler, Credentials, TlsConfig


@pytest.fixture
def creds(nobody_id):
    """Nobody credentials for the tests."""
    creds = Credentials.nobody_with(
        nobody_id.cert_chain, nobody_id.private_key
    ).with_ca(nobody_id.caroot)
    return creds


@pytest.fixture
def signer(scheduler, creds, tmp_path):
    secret = b"\x00" * 32
    network = "regtest"
    return Signer(
        secret,
        network=network,
        creds=creds,
        device_storage_path=str(tmp_path / "device-state"),
    )


@pytest.fixture
def sclient(signer, creds):
    """Just a preconfigured scheduler client.

    This scheduler client is configured with a secret for easy
    registration and recovery, but no mTLS certificate yet.
    """
    network = "regtest"
    return Scheduler(network=network, creds=creds)


@pytest.fixture
def device_creds(signer, creds, sclient):
    """An authenticated set of credentials.
    """

    res = sclient.register(signer)
    return Credentials.from_bytes(res.creds)
