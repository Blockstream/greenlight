from fixtures import *
from glclient.pairing import NewDeviceClient, AttestationDeviceClient

@pytest.fixture
def attestation_device(clients):
    c = clients.new()
    c.register()
    yield c

def test_pairing_session(attestation_device, creds):
    name = "new_device"
    desc = "my_description"
    restrs = "method^list"
    ps = NewDeviceClient(TlsConfig(creds))
    session = ps.pair_device(name, desc, restrs)
    session_iter = iter(session)

    # check that qr data is returned.
    m = next(session_iter)
    assert(m.data)

    # check for pairing data.
    session_id = m.data.split(':')[1]
    ac = AttestationDeviceClient(creds=attestation_device.creds())
    m = ac.get_pairing_data(session_id)
    assert(m.session_id)
    assert(m.csr)
    assert(m.device_name == name)
    assert(m.desc == desc)
    assert(restrs in m.restrs)
    
    m = next(session_iter)
    assert(m.device_cert)
    assert(m.device_key)
    # assert(m.rune) fixme: enable once we pass back a rune during the tests.
    assert(m.creds)
