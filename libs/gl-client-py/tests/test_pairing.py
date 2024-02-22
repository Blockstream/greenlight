from fixtures import *
from glclient.pairing import NewDeviceClient

def test_pairing_session(scheduler, creds):
    ps = NewDeviceClient(TlsConfig(creds))
    session = ps.pair_device("new_device", "my_description", "")
    session_iter = iter(session)

    # check that qr data is returned.
    m = next(session_iter)
    assert(m.data)

    # check that response is returned.
    m = next(session_iter)
    assert(m.session_id)
    assert(m.device_cert)
    assert(m.device_key)
    # assert(m.rune) fixme: enable once we pass back a rune during the tests.
    assert(m.creds)