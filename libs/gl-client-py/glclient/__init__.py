from . import scheduler_pb2 as schedpb
from . import greenlight_pb2 as nodepb
from . import glclient as native
from .greenlight_pb2 import Amount
from binascii import hexlify, unhexlify
from typing import Optional, List
import logging


# Keep in sync with the libhsmd version, this is tested in unit tests.
__version__ = "v0.11.0.1"

class TlsConfig(object):
    def __init__(self):
        # We wrap the TlsConfig since some calls cannot yet be routed
        # through the rust library (streaming calls)
        self.inner = native.TlsConfig()
        self.ca = None
        self.id = (None, None)

    def identity(self, cert_pem, key_pem):
        if isinstance(cert_pem, str):
            cert_pem = cert_pem.encode('ASCII')

        if isinstance(key_pem, str):
            key_pem = key_pem.encode('ASCII')

        c = TlsConfig()
        c.inner = self.inner.identity(cert_pem, key_pem)
        c.ca = self.ca
        c.id = (cert_pem, key_pem)
        return c

    def with_ca_certificate(self, ca):
        if isinstance(ca, str):
            ca = ca.encode('ASCII')

        c = TlsConfig()
        c.inner = self.inner.with_ca_certificate(ca)
        c.ca = ca
        c.id = self.id
        return c


def _convert(cls, res):
    return cls.FromString(bytes(res))


class Signer(object):
    def __init__(self, secret: bytes, network: str, tls: TlsConfig):
        self.inner = native.Signer(secret, network, tls.inner)
        self.tls = tls
        self.handle = None

    def run_in_thread(self):
        if self.handle is not None:
            raise ValueError("This signer is already running, please shut it down before starting it again")
        self.handle = self.inner.run_in_thread()
        return self.handle

    def run_in_foreground(self):
        return self.inner.run_in_foreground()

    def node_id(self):
        return bytes(self.inner.node_id())

    def version(self):
        return self.inner.version()

    def sign_challenge(self, message: bytes) -> bytes:
        return bytes(self.inner.sign_challenge(message))

    def shutdown(self):
        if self.handle is None:
            raise ValueError("Attempted to shut down a signer that is not running")
        self.handle.shutdown()
        self.handle = None

    def is_running(self):
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
    def __init__(self, node_id, network, tls, grpc_uri):
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
        req = bytes(nodepb.GetInfoRequest().SerializeToString())
        return nodepb.GetInfoResponse.FromString(
            bytes(self.call("GetInfo", req))
        )

    def stop(self) -> nodepb.StopResponse:
        return self.inner.stop()

    def list_funds(self) -> nodepb.ListFundsResponse:
        return nodepb.ListFundsResponse.FromString(
            bytes(self.inner.list_funds())
        )

    def list_peers(self) -> nodepb.ListPeersResponse:
        return nodepb.ListPeersResponse.FromString(
            bytes(self.inner.list_peers())
        )

    def list_payments(self) -> nodepb.ListPaymentsResponse:
        return nodepb.ListPaymentsResponse.FromString(
            bytes(self.inner.list_payments())
        )

    def list_invoices(self, label: str = None, invstring: str = None, payment_hash: bytes = None) -> nodepb.ListInvoicesResponse:
        if sum([bool(a) for a in [label, invstring, payment_hash]]) >= 2:
            raise ValueError(
                f"Cannot specify multiple filters for list_invoices: "
                f"label={label}, invstring={invstring}, or "
                f"paymen_hash={payment_hash}"
            )

        if label is not None:
            f = nodepb.InvoiceIdentifier(label=label)
        elif invstring is not None:
            f = nodepb.InvoiceIdentifier(invstring=invstring)
        elif payment_hash is not None:
            f = nodepb.InvoiceIdentifier(payment_hash=unhexlify(payment_hash))
        else:
            f = None

        req = bytes(nodepb.ListInvoicesRequest(identifier=f).SerializeToString())
        return nodepb.ListInvoicesResponse.FromString(
            bytes(self.call("ListInvoices", req))
        )

    def connect_peer(self, node_id, addr=None) -> nodepb.ConnectResponse:
        if len(node_id) == 33:
            node_id = hexlify(node_id)

        if len(node_id) != 66:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        if isinstance(node_id, bytes):
            node_id = node_id.decode('ASCII')

        return nodepb.ConnectResponse.FromString(
            bytes(self.inner.connect_peer(node_id, addr))
        )

    def disconnect_peer(self, peer_id, force=False) -> nodepb.DisconnectResponse:
        return nodepb.ConnectResponse.FromString(
            bytes(self.inner.disconnect_peer(peer_id, force))
        )

    def new_address(self) -> nodepb.NewAddrResponse:
        return nodepb.NewAddrResponse.FromString(
            bytes(self.inner.new_address())
        )

    def withdraw(self, destination, amount: Amount, minconf: int=0) -> nodepb.WithdrawResponse:
        req = nodepb.WithdrawRequest(
            destination=destination,
            amount=amount,
            minconf=nodepb.Confirmation(blocks=minconf),
        ).SerializeToString()

        return nodepb.WithdrawResponse.FromString(
            bytes(self.inner.withdraw(req))
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

        req = nodepb.FundChannelRequest(
            node_id=node_id,
            amount=amount,
            announce=announce,
            minconf=minconf,
        ).SerializeToString()

        return nodepb.FundChannelResponse.FromString(
            bytes(self.inner.fund_channel(req))
        )

    def close_channel(self, peer_id, timeout=None, address=None) -> nodepb.CloseChannelResponse:
        if len(peer_id) == 66:
            peer_id = unhexlify(peer_id)

        if len(peer_id) != 33:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        if isinstance(peer_id, str):
            node_id = peer_id.encode('ASCII')

        return nodepb.CloseChannelResponse.FromString(bytes(self.inner.close(
            peer_id,
            timeout,
            address,
        )))


    def create_invoice(
            self,
            label: str,
            amount=None,
            description: Optional[str]=None,
            preimage: Optional[bytes]=None
    ) -> nodepb.Invoice:
        req = nodepb.InvoiceRequest(
            amount=amount,
            label=label,
            description=description,
            preimage=bytes.fromhex(preimage) if preimage is not None else None,
        ).SerializeToString()

        return nodepb.Invoice.FromString(
            bytes(self.inner.create_invoice(req))
        )

    def pay(self, bolt11: str, amount=None, timeout: int=0) -> nodepb.Payment:
        req = nodepb.PayRequest(
            bolt11=bolt11,
            amount=amount,
            timeout=timeout,
        ).SerializeToString()

        return nodepb.Payment.FromString(
            bytes(self.inner.pay(req))
        )

    def keysend(
            self,
            node_id,
            amount: nodepb.Amount,
            label: Optional[str]=None,
            routehints: Optional[List[nodepb.Routehint]]=None,
            extratlvs: Optional[List[nodepb.TlvField]]=None
    ) -> nodepb.Payment:
        req = nodepb.KeysendRequest(
            node_id=normalize_node_id(node_id),
            amount=amount,
            label=label,
            routehints=routehints,
            extratlvs=extratlvs,
        ).SerializeToString()

        return nodepb.Payment.FromString(
            bytes(self.inner.keysend(req))
        )

    def call(self, method: str, request: bytes) -> bytes:
        self.logger.debug(f"Calling {method} with request {request}")
        return bytes(self.inner.call(method, request))

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
