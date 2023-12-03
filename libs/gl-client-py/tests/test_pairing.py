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
    device_id = m.split(':')[1]
    ac = AttestationDeviceClient(creds=attestation_device.creds())
    m = ac.get_pairing_data(device_id)
    assert(m.device_id)
    assert(m.csr)
    assert(m.device_name == name)
    assert(m.description == desc)
    assert(restrs in m.restrictions)

    # We are happy with the pairing_data and want to approve the 
    # request. Therefor we need a PairingService with our tls cert
    # and with our rune.
    ac.approve_pairing(
        m.session_id,
        attestation_device.node_id,
        m.device_name,
        m.restrictions
    )

    # check that response is returned.
    m = next(session_iter)
    assert(m.session_id)
    assert(m.device_cert)
    assert(m.device_key)
    # assert(m.rune) fixme: enable once we pass back a rune during the tests.
    assert(m.creds)
