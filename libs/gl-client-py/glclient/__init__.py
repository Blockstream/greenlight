from . import scheduler_pb2 as schedpb
from . import greenlight_pb2 as nodepb
from . import node_pb2 as clnpb
from . import glclient as native
from google.protobuf.message import Message as PbMessage

from .greenlight_pb2 import Amount
from binascii import hexlify, unhexlify
from typing import Optional, List, Union, Tuple, Iterable, SupportsIndex, Type, Any, TypeVar
import logging


# Keep in sync with the libhsmd version, this is tested in unit tests.
__version__ = "v0.11.0.1"

class TlsConfig(object):
    def __init__(self) -> None:
        # We wrap the TlsConfig since some calls cannot yet be routed
        # through the rust library (streaming calls)
        self.inner = native.TlsConfig()
        self.ca: Optional[bytes] = None
        self.id: Tuple[Optional[bytes], Optional[bytes]] = (None, None)

    def identity(self, cert_pem: Union[str, bytes], key_pem: Union[str, bytes]) -> "TlsConfig":
        if isinstance(cert_pem, str):
            cert_pem = cert_pem.encode('ASCII')

        if isinstance(key_pem, str):
            key_pem = key_pem.encode('ASCII')

        c = TlsConfig()
        c.inner = self.inner.identity(cert_pem, key_pem)
        c.ca = self.ca
        c.id = (cert_pem, key_pem)
        return c

    def with_ca_certificate(self, ca: Union[str, bytes]) -> "TlsConfig":
        if isinstance(ca, str):
            ca = ca.encode('ASCII')

        c = TlsConfig()
        c.inner = self.inner.with_ca_certificate(ca)
        c.ca = ca
        c.id = self.id
        return c


E = TypeVar('E', bound=PbMessage)
def _convert(cls: Type[E], res: Iterable[SupportsIndex]) -> E:
    return cls.FromString(bytes(res))


class Signer(object):
    def __init__(self, secret: bytes, network: str, tls: TlsConfig):
        self.inner = native.Signer(secret, network, tls.inner)
        self.tls = tls
        self.handle: Optional[native.SignerHandle] = None

    def run_in_thread(self) -> "native.SignerHandle":
        if self.handle is not None:
            raise ValueError("This signer is already running, please shut it down before starting it again")
        self.handle = self.inner.run_in_thread()
        return self.handle

    def run_in_foreground(self) -> None:
        return self.inner.run_in_foreground()

    def node_id(self) -> bytes:
        return bytes(self.inner.node_id())

    def version(self) -> str:
        return self.inner.version()

    def sign_challenge(self, message: bytes) -> bytes:
        return bytes(self.inner.sign_challenge(message))

    def shutdown(self) -> None:
        if self.handle is None:
            raise ValueError("Attempted to shut down a signer that is not running")
        self.handle.shutdown()
        self.handle = None

    def is_running(self) -> bool:
        return self.handle is not None


class Scheduler(object):

    def __init__(self, node_id: bytes, network: str, tls: TlsConfig):
        self.node_id = node_id
        self.network = network
        self.inner = native.Scheduler(node_id, network)
        self.tls = tls

    def get_node_info(self) -> schedpb.NodeInfoResponse:
        return _convert(
            schedpb.NodeInfoResponse,
            self.inner.get_node_info()
        )

    def schedule(self) -> schedpb.NodeInfoResponse:
        res = self.inner.schedule()
        return schedpb.NodeInfoResponse.FromString(bytes(res))

    def register(self, signer: Signer) -> schedpb.RegistrationResponse:
        res = self.inner.register(signer.inner)
        return schedpb.RegistrationResponse.FromString(bytes(res))

    def recover(self, signer: Signer) -> schedpb.RecoveryResponse:
        res = self.inner.recover(signer.inner)
        return schedpb.RecoveryResponse.FromString(bytes(res))

    def node(self) -> "Node":
        res = self.schedule()
        return Node(
            node_id=self.node_id,
            network=self.network,
            tls=self.tls,
            grpc_uri=res.grpc_uri
        )


class Node(object):
    def __init__(self, node_id: bytes, network: str, tls: TlsConfig, grpc_uri: str) -> None:
        self.tls = tls
        self.grpc_uri = grpc_uri
        self.inner = native.Node(
            node_id=node_id,
            network=network,
            tls=tls.inner,
            grpc_uri=grpc_uri
        )
        self.logger = logging.getLogger("glclient.Node")

    def get_info(self) -> nodepb.GetInfoResponse:
        uri = "/cln.Node/Getinfo"
        req = clnpb.GetinfoRequest().SerializeToString()
        res = clnpb.GetinfoResponse

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def stop(self) -> None:
        uri = "/greenlight.Node/Stop"
        req = nodepb.StopRequest().SerializeToString()

        try:
            # This fails, since we just get disconnected, but that's
            # on purpose, so drop the error silently.
            self.inner.call(uri, bytes(req))
        except ValueError as e:
            self.logger.debug(f"Caught an expected exception: {e}. Don't worry it's expected.")

    def list_funds(
            self,
            minconf: Union[nodepb.Confirmation, int]=1
    ) -> nodepb.ListFundsResponse:
        if isinstance(minconf, int):
            minconf = nodepb.Confirmation(
                blocks=minconf
            )

        uri = "/greenlight.Node/ListFunds"
        res = nodepb.ListFundsResponse
        req = nodepb.ListFundsRequest(
            minconf=minconf,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def list_peers(self) -> nodepb.ListPeersResponse:
        uri = "/greenlight.Node/ListPeers"
        req = nodepb.ListPeersRequest().SerializeToString()
        res = nodepb.ListPeersResponse

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def list_payments(self) -> nodepb.ListPaymentsResponse:
        uri = "/greenlight.Node/ListPayments"
        req = nodepb.ListPaymentsRequest().SerializeToString()
        res = nodepb.ListPaymentsResponse

        return res.FromString(
            bytes(self.inner.call(uri, req))
        )

    def list_invoices(
            self,
            label: str = None,
            invstring: str = None,
            payment_hash: bytes = None
    ) -> nodepb.ListInvoicesResponse:
        if sum([bool(a) for a in [label, invstring, payment_hash]]) >= 2:
            hexhash = hexlify(payment_hash).decode('ASCII') if payment_hash else None
            raise ValueError(
                f"Cannot specify multiple filters for list_invoices: "
                f"label={label}, invstring={invstring}, or "
                f"paymen_hash={hexhash}"
            )

        if label is not None:
            f = nodepb.InvoiceIdentifier(label=label)
        elif invstring is not None:
            f = nodepb.InvoiceIdentifier(invstring=invstring)
        elif payment_hash is not None:
            f = nodepb.InvoiceIdentifier(payment_hash=unhexlify(payment_hash))
        else:
            f = None

        uri = "/greenlight.Node/ListInvoices"
        res = nodepb.ListInvoicesResponse
        req = nodepb.ListInvoicesRequest(
            identifier=f
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def connect_peer(self, node_id, addr=None) -> nodepb.ConnectResponse:
        if len(node_id) == 33:
            node_id = hexlify(node_id)

        if len(node_id) != 66:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        if isinstance(node_id, bytes):
            node_id = node_id.decode('ASCII')

        uri = "/greenlight.Node/ConnectPeer"
        res = nodepb.ConnectResponse
        req = nodepb.ConnectRequest(
            node_id=node_id,
            addr=addr,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def disconnect_peer(self, peer_id, force=False) -> nodepb.DisconnectResponse:
        uri = "/greenlight.Node/Disconnect"
        res = nodepb.DisconnectResponse
        req = nodepb.DisconnectRequest(
            node_id=node_id,
            force=force,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def new_address(self) -> nodepb.NewAddrResponse:
        uri = "/greenlight.Node/NewAddr"
        req = nodepb.NewAddrRequest().SerializeToString()
        res = nodepb.NewAddrResponse

        return res.FromString(
            bytes(self.inner.call(uri, req))
        )

    def withdraw(
            self,
            destination,
            amount: Amount,
            minconf: int=0
    ) -> nodepb.WithdrawResponse:
        uri = "/greenlight.Node/Withdraw"
        res = nodepb.WithdrawResponse
        req = nodepb.WithdrawRequest(
            destination=destination,
            amount=amount,
            minconf=nodepb.Confirmation(blocks=minconf),
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def fund_channel(
            self,
            node_id,
            amount,
            announce=False,
            minconf=1) -> nodepb.FundChannelResponse:
        if len(node_id) == 66:
            node_id = unhexlify(node_id)

        if isinstance(minconf, int):
            minconf = nodepb.Confirmation(blocks=minconf)
        elif isinstance(minconf, nodepb.Confirmation):
            pass
        elif not isinstance(minconf, nodepb.Confirmation):
            raise ValueError("'minconf' is neither an int nor a Confirmation")


        if len(node_id) != 33:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        uri = "/greenlight.Node/FundChannel"
        res = nodepb.FundChannelResponse
        req = nodepb.FundChannelRequest(
            node_id=node_id,
            amount=amount,
            announce=announce,
            minconf=minconf,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def close_channel(self, peer_id, timeout=None, address=None) -> nodepb.CloseChannelResponse:
        if len(peer_id) == 66:
            peer_id = unhexlify(peer_id)

        if len(peer_id) != 33:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        if isinstance(peer_id, str):
            node_id = peer_id.encode('ASCII')

        uri = "/greenlight.Node/CloseChannel"
        res = nodepb.CloseChannelResponse
        req = nodepb.FundChannelRequest(
            node_id=peer_id,
            timeout=timeout,
            address=address,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )


    def create_invoice(
            self,
            label: str,
            amount=None,
            description: Optional[str]=None,
            preimage: Optional[Union[str,bytes]]=None
    ) -> nodepb.Invoice:
        if preimage is None:
            preimage = b""
        else:
            if isinstance(preimage, str) and len(preimage) == 64:
                preimage = bytes.fromhex(preimage)
            elif isinstance(preimage, bytes) and len(preimage) == 32:
                pass
            elif isinstance(preimage, str):
                raise ValueError("Preimage must be 32 bytes, either as bytes or as hex-encoded string.")

        uri = "/greenlight.Node/CreateInvoice"
        res = nodepb.Invoice
        req = nodepb.InvoiceRequest(
            amount=amount,
            label=label,
            description=description if description else "",
            preimage=preimage
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def pay(self, bolt11: str, amount=None, timeout: int=0) -> nodepb.Payment:
        uri = "/greenlight.Node/Pay"
        res = nodepb.Payment
        req = nodepb.PayRequest(
            bolt11=bolt11,
            amount=amount,
            timeout=timeout,
        ).SerializeToString()

        return nodepb.Payment.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def keysend(
            self,
            node_id,
            amount: nodepb.Amount,
            label: Optional[str]=None,
            routehints: Optional[List[nodepb.Routehint]]=None,
            extratlvs: Optional[List[nodepb.TlvField]]=None
    ) -> nodepb.Payment:
        uri = "/greenlight.Node/Keysend"
        res = nodepb.Payment
        req = nodepb.KeysendRequest(
            node_id=normalize_node_id(node_id),
            amount=amount,
            label=label if label else "",
            routehints=routehints,
            extratlvs=extratlvs,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def stream_log(self):
        """Stream logs as they get generated on the server side.
        """
        stream = self.inner.stream_log(b"")
        while True:
            n = stream.next()
            if n is None:
                break
            yield nodepb.LogEntry.FromString(bytes(n))

    def stream_incoming(self):
        stream = self.inner.stream_incoming(b"")
        while True:
            n = stream.next()
            if n is None:
                break
            yield nodepb.IncomingPayment.FromString(bytes(n))


def normalize_node_id(node_id, string=False):
    if len(node_id) == 66:
        node_id = unhexlify(node_id)

    if len(node_id) != 33:
        raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

    if isinstance(node_id, str):
        node_id = node_id.encode('ASCII')
    return node_id if not string else hexlify(node_id).encode('ASCII')
