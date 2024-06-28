from . import TlsConfig
from . import scheduler_pb2 as schedpb
import glclient.glclient as native
from glclient.glclient import Credentials
from typing import Optional, Generator


class NewDeviceClient(object):
    """A Pairing Client for the "new device" flow."""

    def __init__(self, creds: Credentials, uri: Optional[str] = None):
        self._inner = native.NewDeviceClient(creds=creds, uri=uri)

    def _recv(self, m):
        msgs = {
            1: schedpb.PairDeviceResponse,
            2: schedpb.PairingQr,
        }

        typ, msg = m[0], m[1:]
        conv = msgs.get(typ, None)

        if conv is None:
            raise ValueError(f"Unknown message type {typ}")

        yield conv.FromString(bytes(msg))

    def pair_device(
        self, name: str, desc: str, restrs: str
    ) -> Generator[schedpb.PairDeviceResponse, None, None]:
        """Starts a new paring session and yields for next message.

        First message would be the "QRData" data that must be presented to
        the attestation device.

        Second message is either the pairing response or an error.
        """
        for m in self._inner.pair_device(name, desc, restrs):
            yield from self._recv(m)


class AttestationDeviceClient(object):
    def __init__(self, creds: Credentials, uri: Optional[str] = None):
        self.inner = native.AttestationDeviceClient(creds=creds, uri=uri)

    def get_pairing_data(self, session_id: str) -> schedpb.GetPairingDataResponse:
        res = self.inner.get_pairing_data(session_id=session_id)
        return schedpb.GetPairingDataResponse.FromString(bytes(res))

    def approve_pairing(
        self, session_id: str, node_id: bytes, device_name: str, restrs: str
    ):
        self.inner.approve_pairing(session_id, node_id, device_name, restrs)

    def verify_pairing_data(self, data: schedpb.GetPairingDataRequest) -> Optional[str]:
        try:
            self.inner.verify_pairing_data(data.SerializeToString())
            return None
        except Exception as e:
            return str(e)
