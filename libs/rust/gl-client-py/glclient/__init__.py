from . import scheduler_pb2 as schedpb
from . import greenlight_pb2 as nodepb
from . import glclient as native
from binascii import hexlify

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
        return Node(self.node_id, self.network, self.tls.inner, res.grpc_uri)


class Node(object):
    def __init__(self, *args):
        self.inner = native.Node(*args)

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

    def disconnect(self, peer_id, force=False) -> nodepb.DisconnectResponse:
        return nodepb.ConnectResponse.FromString(
            bytes(self.inner.disconnect_peer(peer_id, force))
        )

    def newaddr(NewAddrRequest) -> nodepb.NewAddrResponse:
        raise NotImplementedError

    def Withdraw(WithdrawRequest) -> nodepb.WithdrawResponse:
        raise NotImplementedError

    def FundChannel(FundChannelRequest) -> nodepb.FundChannelResponse:
        raise NotImplementedError

    def CloseChannel(CloseChannelRequest) -> nodepb.CloseChannelResponse:
        raise NotImplementedError

    def CreateInvoice(InvoiceRequest) -> nodepb.Invoice:
        raise NotImplementedError

    def Pay(PayRequest) -> nodepb.Payment:
        raise NotImplementedError

    def Keysend(KeysendRequest) -> nodepb.Payment:
        raise NotImplementedError

"""
    def StreamIncoming(StreamIncomingFilter) returns (stream IncomingPayment)
        raise NotImplementedError

    def StreamLog(StreamLogRequest) returns (stream LogEntry)
        raise NotImplementedError
"""
