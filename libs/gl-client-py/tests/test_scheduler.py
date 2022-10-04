from gltesting.fixtures import *
from glclient import TlsConfig, Signer, Scheduler, Node
from binascii import hexlify
import unittest


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


def test_connect(scheduler, tls):
    """Test that we can connect to the scheduler.
    """
    sig = Signer(b'\x00'*32, network='regtest', tls=tls)
    node_id = sig.node_id()
    s = Scheduler(node_id, network='regtest', tls=tls)
    with pytest.raises(ValueError):
        s.get_node_info()


def test_register(sclient, signer):
    res = sclient.register(signer)
    assert(res.device_cert)
    assert(res.device_key)


def test_recover(sclient, signer):
    sclient.register(signer)
    rec = sclient.recover(signer)
    assert(rec.device_cert)
    assert(rec.device_key)


@unittest.skip("Scheduler is being reworked")
def test_schedule_call(sclient, signer, tls):
    req = sclient.register(signer)
    res = sclient.schedule()
    tls = tls.identity(req.device_cert, req.device_key)
    node = Node(signer.node_id(), 'regtest', tls, res.grpc_uri)
    info = node.get_info()


def test_sign_challenge(signer):
    """Check that we can sign a challenge
    """
    res = signer.sign_challenge(b'\x00' * 32)
    print(res, len(res))
    res = hexlify(res)
    assert res == b'cdd553f30964056a855556b2d4635c6f8872fdc145de0dd336020886a56377a150f70a2a8bc428fabe9be87ede610999af8a14a64f7e9ef73836d78e59d28d92'

def test_signer_version(signer):
    import glclient
    assert glclient.__version__ == signer.version()
