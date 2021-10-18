from . import scheduler_pb2 as schedpb
from . import greenlight_pb2 as nodepb
from . import glclient as native
from .greenlight_pb2 import Amount
from binascii import hexlify, unhexlify
from typing import Optional, List


class TlsConfig(object):
    def __init__(self):
        # We wrap the TlsConfig since some calls cannot yet be routed
        # through the rust library (streaming calls)
        self.inner = native.TlsConfig()
        self.ca = None
        self.id = (None, None)

    def identity(self, cert_pem, key_pem):
        c = TlsConfig()
        c.inner = self.inner.identity(cert_pem, key_pem)
        c.ca = self.ca
        c.id = (cert_pem, key_pem)
        return c

    def with_ca_certificate(self, ca):
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

    def run_in_thread(self):
        return self.inner.run_in_thread()

    def run_in_foreground(self):
        return self.inner.run_in_foreground()

    def node_id(self):
        return self.inner.node_id()

    def sign_challenge(self, challenge):
        return bytes(self.inner.sign_challenge(challenge))


class Scheduler(object):

    def __init__(self, node_id: bytes, network: str):
        self.inner = native.Scheduler(node_id, network)
        self.node_id = node_id
        self.network = network
        self.tls = TlsConfig()

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

    def get_info(self) -> nodepb.GetInfoResponse:
        return nodepb.GetInfoResponse.FromString(
            bytes(self.inner.get_info())
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

    def list_invoices(self) -> nodepb.ListInvoicesResponse:
        return nodepb.ListInvoicesResponse.FromString(
            bytes(self.inner.list_invoices())
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

    def new_address(self, address_type: str) -> nodepb.NewAddrResponse:
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

    def fund_channel(self, node_id, amount, announce=False, minconf=1) -> nodepb.FundChannelResponse:
        if len(node_id) == 66:
            node_id = unhexlify(node_id)

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

    def close_channel(peer_id, timeout=None, address=None) -> nodepb.CloseChannelResponse:
        if len(node_id) == 66:
            node_id = unhexlify(node_id)

        if len(node_id) != 33:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        if isinstance(node_id, str):
            node_id = node_id.encode('ASCII')

        return nodepb.CloseChannelResponse.FromString(bytes(self.inner.close_channel(
            peer_id,
            timeout,
            address,
        )))


    def create_invoice(self, label: str, amount=None, description=None) -> nodepb.Invoice:
        req = nodepb.InvoiceRequest(
            amount=amount,
            label=label,
            description=description,
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

    def _direct_stub(self):
        """Streaming API methods are not yet supported by our middleware. So
        we need to actually create a new direct connection instead.

        """
        # Need to create a python-based stub since we cannot yield
        # across FFI boundaries (yet).
        from . import greenlight_pb2_grpc as nodegrpc
        import grpc
        creds = grpc.ssl_channel_credentials(
            root_certificates=bytes(self.tls.inner.ca_certificate()),
            private_key=self.tls.id[1],
            certificate_chain=self.tls.id[0],
        )
        grpc_uri = self.grpc_uri
        if grpc_uri.startswith("https://"):
            grpc_uri = grpc_uri[8:]

        chan = grpc.secure_channel(
            grpc_uri,
            creds,
            options=(
                (
                    "grpc.ssl_target_name_override",
                    "localhost",
                ),
            ),
        )
        stub = nodegrpc.NodeStub(chan)
        return stub

    def stream_log(self):
        """Stream logs as they get generated on the server side.
        """
        stub = self._direct_stub()
        yield from stub.StreamLog(nodepb.StreamLogRequest())

    def stream_incoming(self):
        stub = self._direct_stub()
        yield from stub.StreamIncoming(nodepb.StreamIncomingFilter())


def normalize_node_id(node_id, string=False):
    if len(node_id) == 66:
        node_id = unhexlify(node_id)

    if len(node_id) != 33:
        raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

    if isinstance(node_id, str):
        node_id = node_id.encode('ASCII')
    return node_id if not string else hexlify(node_id).encode('ASCII')
