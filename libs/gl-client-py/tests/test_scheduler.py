from fixtures import *
from glclient import Signer, Scheduler, Node, Credentials
from binascii import hexlify
import unittest


def test_connect(scheduler, creds):
    """Test that we can connect to the scheduler."""
    sig = Signer(b"\x00" * 32, network="regtest", creds=creds)
    node_id = sig.node_id()
    s = Scheduler(node_id, network="regtest", creds=creds)
    with pytest.raises(ValueError):
        s.recover(sig)


def test_register(sclient, signer):
    res = sclient.register(signer)
    assert res.device_cert
    assert res.device_key
    assert res.rune
    assert res.creds


def test_recover(sclient, signer):
    sclient.register(signer)
    res = sclient.recover(signer)
    assert res.device_cert
    assert res.device_key
    assert res.rune
    assert res.creds


def test_schedule_call(sclient, signer):
    req = sclient.register(signer)
    
    # This fails now as the scheduler needs to be authenticated to 
    # schedule a node.
    with pytest.raises(ValueError):
        res = sclient.schedule()

    # Authenticate the scheduler client.
    creds = Credentials.from_bytes(req.creds)
    sclient.authenticate(creds)
    node = sclient.node()
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


def test_get_invite_codes(scheduler, sclient, device_creds):
    scheduler.add_invite_codes(
        [{"code": "ABC", "is_redeemed": False}, {"code": "HELLO", "is_redeemed": True}]
    )
    invite_codes = sclient.authenticate(device_creds).get_invite_codes()
    print(f"Got codes: {invite_codes}")


def test_register_with_invite_code(scheduler, sclient, signer):
    sclient.register(signer, "some-invite-code")
    assert scheduler.received_invite_code == "some-invite-code"

