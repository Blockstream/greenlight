import time
from fixtures import *
from glclient.pairing import NewDeviceClient, AttestationDeviceClient
from glclient import Credentials
import glclient.scheduler_pb2 as schedpb
import pytest


@pytest.fixture
def attestation_device(clients):
    c = clients.new()
    c.register()
    yield c


def test_pairing_session(sclient, signer, creds):
    # register attestation device.
    res = sclient.register(signer)

    # Run the signer in the background
    signer.run_in_thread()

    name = "new_device"
    desc = "my_description"
    restrs = "method^list"
    ps = NewDeviceClient()
    session = ps.pair_device(name, desc, restrs)
    session_iter = iter(session)

    # check that qr data str is returned.
    m = next(session_iter)
    assert m

    creds = Credentials.from_bytes(res.creds)
    scheduler = Scheduler(network="regtest", creds=creds)
    scheduler.schedule()

    # check for pairing data.
    assert isinstance(m, str)
    device_id = m.split(":")[1]
    ac = AttestationDeviceClient(creds=creds)
    m = ac.get_pairing_data(device_id)
    assert isinstance(m, schedpb.GetPairingDataResponse)
    assert m.device_id
    assert m.csr
    assert m.device_name == name
    assert m.description == desc
    assert restrs in m.restrictions

    # We are happy with the pairing_data and want to approve the
    # request. Therefor we need a PairingService with our tls cert
    # and with our rune.
    ac.approve_pairing(m.device_id, m.device_name, m.restrictions)

    # check that response is returned.
    m = next(session_iter)
    assert m.device_id
    assert m.device_cert
    assert m.device_key
    # assert(m.rune) fixme: enable once we pass back a rune during the tests.
    assert m.creds

    signer.shutdown()
    # FIXME: add a blocking shutdown call that waits for the signer to shutdown.
    time.sleep(2)


def test_paring_data_validation(attestation_device, creds):
    """A simple test to ensure that data validation works as intended.

    If the data is valid, the public key belongs to the private key that was
    used to sign the csr subject.
    """
    name = "new_device"
    desc = "my description"
    restrs = "method^list"

    dc = NewDeviceClient(creds)
    session = dc.pair_device(name, desc, restrs)
    session_iter = iter(session)
    m = next(session_iter)

    ac = AttestationDeviceClient(creds=attestation_device.creds())
    # check for pairing data.
    device_id = m.split(":")[1]
    m = ac.get_pairing_data(device_id)

    assert ac.verify_pairing_data(m) is None

    # Change the public key and try again
    pk = "01" + m.device_id[2:] if m.device_id[0:1] == "00" else "00" + m.device_id[2:]
    m.device_id = pk

    with pytest.raises(
        expected_exception=ValueError, match="could not verify pairing data"
    ):
        ac.verify_pairing_data(m)
