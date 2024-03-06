from gltesting.fixtures import *
from glclient import (
    TlsConfig,
    Signer,
    Scheduler,
    Node,
    Credentials,
)

# from glclient.credentials import Credentials, CBuilder
from binascii import hexlify
import unittest


@pytest.fixture
def creds(nobody_id):
    """Nobody credentials for the tests."""
    creds = (
        Credentials.as_nobody()
        .with_identity(nobody_id.cert_chain, nobody_id.private_key)
        .with_ca(nobody_id.caroot)
        .build()
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
def sclient(signer, tls):
    """Just a preconfigured scheduler client.

    This scheduler client is configured with a secret for easy
    registration and recovery, but no mTLS certificate yet.
    """
    network = "regtest"
    return Scheduler(signer.node_id(), network=network, tls=tls)


def test_connect(scheduler, tls):
    """Test that we can connect to the scheduler."""
    sig = Signer(b"\x00" * 32, network="regtest", tls=tls)
    node_id = sig.node_id()
    s = Scheduler(node_id, network="regtest", tls=tls)
    with pytest.raises(ValueError):
        s.get_node_info()


def test_register(sclient, signer):
    res = sclient.register(signer)
    assert res.device_cert
    assert res.device_key
    assert res.creds


def test_recover(sclient, signer):
    sclient.register(signer)
    res = sclient.recover(signer)
    assert res.device_cert
    assert res.device_key
    assert res.creds


def test_schedule_call(sclient, signer):
    req = sclient.register(signer)
    res = sclient.schedule()
    creds = Credentials.as_device().from_bytes(req.creds).build()
    node = Node(signer.node_id(), "regtest", res.grpc_uri, creds=creds)
    info = node.get_info()
    assert info


def test_sign_challenge(signer):
    """Check that we can sign a challenge"""
    res = signer.sign_challenge(b"\x00" * 32)
    print(res, len(res))
    res = hexlify(res)
    assert (
        res
        == b"cdd553f30964056a855556b2d4635c6f8872fdc145de0dd336020886a56377a150f70a2a8bc428fabe9be87ede610999af8a14a64f7e9ef73836d78e59d28d92"
    )


def test_signer_version(signer):
    import glclient
    assert glclient.__version__ == signer.version()


def test_get_invite_codes(scheduler, sclient):
    scheduler.add_invite_codes(
        [{"code": "ABC", "is_redeemed": False}, {"code": "HELLO", "is_redeemed": True}]
    )
    invite_codes = sclient.get_invite_codes()
    print(f"Got codes: {invite_codes}")


def test_register_with_invite_code(scheduler, sclient, signer):
    sclient.register(signer, "some-invite-code")
    assert scheduler.received_invite_code == "some-invite-code"

