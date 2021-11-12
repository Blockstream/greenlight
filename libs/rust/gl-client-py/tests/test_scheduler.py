from gltesting.fixtures import *
from glclient import TlsConfig, Signer, Scheduler, Node


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
def sclient(signer):
    """Just a preconfigured scheduler client.

    This scheduler client is configured with a secret for easy
    registration and recovery, but no mTLS certificate yet.

    """
    network = 'regtest'
    return Scheduler(signer.node_id(), network)


def test_connect(scheduler, tls):
    """Test that we can connect to the scheduler.
    """
    sig = Signer(b'\x00'*32, network='regtest', tls=tls)
    node_id = sig.node_id()
    s = Scheduler(node_id, network='regtest')
    assert(s.get_node_info().node_id == node_id)


def test_register(sclient, signer):
    res = sclient.register(signer)
    assert(res.device_cert)
    assert(res.device_key)


def test_recover(sclient, signer):
    res = sclient.register(signer)
    rec = sclient.recover(signer)
    assert(res.device_cert)
    assert(res.device_key)


def test_schedule_call(sclient, signer, tls):
    req = sclient.register(signer)
    res = sclient.schedule()
    tls = tls.identity(req.device_cert, req.device_key)
    node = Node(signer.node_id(), 'regtest', tls, res.grpc_uri)
    info = node.get_info()
