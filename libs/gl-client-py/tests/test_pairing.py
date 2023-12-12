from fixtures import *
from glclient.pairing import NewDeviceClient, AttestationDeviceClient
from glclient.tls import TlsConfig


@pytest.fixture
def attestation_device(clients):
    c = clients.new()
    c.register()
    yield c

def test_pairing_session(attestation_device):
    name = "new_device"
    desc = "my_description"
    restrs = "method^list"
    ps = NewDeviceClient(TlsConfig())
    session = ps.pair_device(name, desc, restrs)
    session_iter = iter(session)

    # check that qr data str is returned.
    m = next(session_iter)
    assert(m)

    # check for pairing data.
    session_id = m.split(':')[1]
    ac = AttestationDeviceClient(creds=attestation_device.creds())
    m = ac.get_pairing_data(session_id)
    assert(m.session_id)
    assert(m.csr)
    assert(m.device_name == name)
    assert(m.description == desc)
    assert(restrs in m.restrictions)
    
    m = next(session_iter)
    assert(m.device_cert)
    assert(m.device_key)
    # assert(m.rune) fixme: enable once we pass back a rune during the tests.
    assert(m.creds)
