from gltesting.fixtures import *
from glclient import TlsConfig, Signer, Scheduler

@pytest.fixture
def tls(nobody_id):
    """Just a preconfigured TlsConfig.
    """
    return TlsConfig().with_ca_certificate(
        nobody_id.caroot
    ).identity(
        nobody_id.cert_chain,
        nobody_id.private_key
    )


@pytest.fixture
def signer(scheduler, tls):
    secret = b'\x00'*32
    network='regtest'
    return Signer(secret, network=network, tls=tls)


@pytest.fixture
def sclient(signer, tls):
    """Just a preconfigured scheduler client.

    This scheduler client is configured with a secret for easy
    registration and recovery, but no mTLS certificate yet.

    """
    network = 'regtest'
    return Scheduler(signer.node_id(), network=network, tls=tls)
