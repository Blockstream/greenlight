from fixtures import *
from glclient.pairing import NewDeviceClient
from glclient.tls import TlsConfig

def test_pairing_session(scheduler, nobody_id):
    ps = NewDeviceClient(TlsConfig())
    session = ps.pair_device("new_device", "my_description", "")
    session_iter = iter(session)
    m = next(session_iter)
    assert(m.session_id)
    assert(m.device_cert)
    assert(m.device_key)
    # assert(m.rune) fixme: enable once we pass back a rune during the tests.
    assert(m.creds)