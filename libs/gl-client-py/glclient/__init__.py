from . import scheduler_pb2 as schedpb
from . import greenlight_pb2 as nodepb
from pyln import grpc as clnpb  # type: ignore
from pyln.grpc import Amount, AmountOrAll, AmountOrAny  # noqa: F401
from . import glclient as native
from glclient.glclient import backup_decrypt_with_seed  # noqa: F401
from .tls import TlsConfig
from google.protobuf.message import Message as PbMessage
from binascii import hexlify, unhexlify
from typing import Optional, List, Iterable, Any, Type, TypeVar
import logging
from glclient.lsps import LspClient


backup_decrypt_with_seed = native.backup_decrypt_with_seed


# Keep in sync with the libhsmd version, this is tested in unit tests.
__version__ = "v23.08"


E = TypeVar('E', bound=PbMessage)
def _convert(cls: Type[E], res: Iterable[Any]) -> E:
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
        self.tls = tls
        self.inner = native.Scheduler(node_id, network, tls.inner)

    def get_node_info(self) -> schedpb.NodeInfoResponse:
        return _convert(
            schedpb.NodeInfoResponse,
            self.inner.get_node_info()
        )

    def schedule(self) -> schedpb.NodeInfoResponse:
        res = self.inner.schedule()
        return schedpb.NodeInfoResponse.FromString(bytes(res))

    def register(self, signer: Signer, invite_code: Optional[str] = None) -> schedpb.RegistrationResponse:
        res = self.inner.register(signer.inner, invite_code)
        return schedpb.RegistrationResponse.FromString(bytes(res))

    def recover(self, signer: Signer) -> schedpb.RecoveryResponse:
        res = self.inner.recover(signer.inner)
        return schedpb.RecoveryResponse.FromString(bytes(res))

    def export_node(self) -> schedpb.ExportNodeResponse:
        res = schedpb.ExportNodeResponse
        return res.FromString(bytes(self.inner.export_node()))

    def node(self) -> "Node":
        res = self.schedule()
        return Node(
            node_id=self.node_id,
            network=self.network,
            tls=self.tls,
            grpc_uri=res.grpc_uri
        )

    def get_invite_codes(self) -> schedpb.ListInviteCodesResponse:
        res = self.inner.get_invite_codes()
        return res


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

    def get_info(self) -> clnpb.GetinfoResponse:
        uri = "/cln.Node/Getinfo"
        req = clnpb.GetinfoRequest().SerializeToString()
        res = clnpb.GetinfoResponse

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def stop(self) -> None:
        uri = "/cln.Node/Stop"
        req = clnpb.StopRequest().SerializeToString()

        try:
            # This fails, since we just get disconnected, but that's
            # on purpose, so drop the error silently.
            self.inner.call(uri, bytes(req))
        except ValueError as e:
            self.logger.debug(f"Caught an expected exception: {e}. Don't worry it's expected.")

    def list_funds(
            self,
            spent: Optional[bool] = None,
    ) -> clnpb.ListfundsResponse:
        uri = "/cln.Node/ListFunds"
        res = clnpb.ListfundsResponse
        req = clnpb.ListfundsRequest(
            spent=spent,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def list_peers(self) -> clnpb.ListpeersResponse:
        uri = "/cln.Node/ListPeers"
        req = clnpb.ListpeersRequest().SerializeToString()
        res = clnpb.ListpeersResponse

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def list_closed_channels(self) -> clnpb.ListclosedchannelsResponse:
        uri = "/cln.Node/ListClosedChannels"
        req = clnpb.ListclosedchannelsRequest().SerializeToString()
        res = clnpb.ListclosedchannelsResponse

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )
    
    def list_channels(
            self,
            short_channel_id: Optional[str] = None,
            source: Optional[bytes] = None,
            destination: Optional[bytes] = None 
    ) -> clnpb.ListchannelsResponse:
        uri = "/cln.Node/ListChannels"
        req = clnpb.ListchannelsRequest(
            short_channel_id=short_channel_id,
            source=source,
            destination=destination
        ).SerializeToString()
        res = clnpb.ListchannelsResponse

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def listpays(self) -> clnpb.ListpaysResponse:
        uri = "/cln.Node/ListPays"
        req = clnpb.ListpaysRequest().SerializeToString()
        res = clnpb.ListpaysResponse

        return res.FromString(
            bytes(self.inner.call(uri, req))
        )

    def list_invoices(
            self,
            label: Optional[str] = None,
            invstring: Optional[str] = None,
            payment_hash: Optional[bytes] = None
    ) -> clnpb.ListinvoicesResponse:
        uri = "/cln.Node/ListInvoices"
        res = clnpb.ListinvoicesResponse
        req = clnpb.ListinvoicesRequest(
        ).SerializeToString()
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def connect_peer(
            self,
            node_id,
            host: Optional[str]=None,
            port: Optional[int]=None
    ) -> clnpb.ConnectResponse:
        if len(node_id) == 33:
            node_id = hexlify(node_id)

        if isinstance(node_id, bytes):
            node_id = node_id.decode('ASCII')

        uri = "/cln.Node/ConnectPeer"
        res = clnpb.ConnectResponse
        req = clnpb.ConnectRequest(
            id=node_id,
            host=host,
            port=port,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def decode (self, string: str) -> clnpb.DecodeResponse:
        uri = "/cln.Node/Decode"
        res = clnpb.DecodeResponse
        req = clnpb.DecodeRequest(
            string=string,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )
    
    def decodepay (
            self,
            bolt11: str,
            description: Optional[str]
    ) -> clnpb.DecodepayResponse:
        uri = "/cln.Node/DecodePay"
        res = clnpb.DecodepayResponse
        req = clnpb.DecodepayRequest(
            bolt11=bolt11,
            description=description,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def disconnect_peer(self, peer_id: str, force=False) -> clnpb.DisconnectResponse:
        uri = "/cln.Node/Disconnect"
        res = clnpb.DisconnectResponse
        req = clnpb.DisconnectRequest(
            id=bytes.fromhex(peer_id),
            force=force,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def new_address(self) -> clnpb.NewaddrResponse:
        uri = "/cln.Node/NewAddr"
        req = clnpb.NewaddrRequest().SerializeToString()
        res = clnpb.NewaddrResponse

        return res.FromString(
            bytes(self.inner.call(uri, req))
        )

    def withdraw(
            self,
            destination,
            amount: AmountOrAll,
            minconf: int=0
    ) -> clnpb.WithdrawResponse:
        uri = "/cln.Node/Withdraw"
        res = clnpb.WithdrawResponse
        req = clnpb.WithdrawRequest(
            destination=destination,
            satoshi=amount,
            minconf=minconf
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def fund_channel(
            self,
            id: bytes,
            amount,
            announce: Optional[bool] = False,
            minconf: Optional[int] = 1,
    ) -> clnpb.FundchannelResponse:

        if len(id) != 33:
            raise ValueError("id is not 33 bytes long")

        uri = "/cln.Node/FundChannel"
        res = clnpb.FundchannelResponse
        req = clnpb.FundchannelRequest(
            id=id,
            amount=amount,
            announce=announce,
            minconf=minconf,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def close(
            self,
            id: bytes,
            unilateraltimeout=None,
            destination=None
    ) -> clnpb.CloseResponse:
        if len(id) != 33:
            raise ValueError("node_id is not 33 bytes long")

        uri = "/cln.Node/Close"
        res = clnpb.CloseResponse
        req = clnpb.CloseRequest(
            id=id.hex(),
            unilateraltimeout=unilateraltimeout,
            destination=destination,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def invoice(
            self,
            amount_msat: clnpb.AmountOrAny,
            label: str,
            description: str,
            expiry: Optional[int]=None,
            fallbacks: Optional[List[str]]=None,
            preimage: Optional[bytes]=None,
            cltv: Optional[int]=None,
            deschashonly: Optional[bool]=None
    ) -> clnpb.InvoiceResponse:
        if preimage and len(preimage) != 32:
            raise ValueError("Preimage must be 32 bytes in length")

        uri = "/cln.Node/Invoice"
        res = clnpb.InvoiceResponse
        req = clnpb.InvoiceRequest(
            amount_msat=amount_msat,
            label=label,
            description=description,
            preimage=preimage,
            expiry=expiry,
            fallbacks=fallbacks,
            cltv=cltv,
            deschashonly=deschashonly,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def pay(
            self,
            bolt11: str,
            amount_msat: Optional[clnpb.Amount]=None,
            retry_for: int=0,
            maxfee: Optional[clnpb.Amount]=None,
            maxfeepercent: Optional[float]=None
    ) -> clnpb.PayResponse:
        uri = "/cln.Node/Pay"
        res = clnpb.PayResponse
        req = clnpb.PayRequest(
            bolt11=bolt11,
            amount_msat=amount_msat,
            retry_for=retry_for,
            maxfeepercent=maxfeepercent,
            maxfee=maxfee,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def keysend(
            self,
            destination: bytes,
            amount: clnpb.Amount,
            label: Optional[str]=None,
            routehints: Optional[clnpb.RoutehintList]=None,
            extratlvs: Optional[clnpb.TlvStream]=None
    ) -> clnpb.KeysendResponse:
        uri = "/cln.Node/KeySend"
        res = clnpb.KeysendResponse
        req = clnpb.KeysendRequest(
            destination=normalize_node_id(destination, string=False),
            amount_msat=amount,
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

    def stream_custommsg(self):
        stream = self.inner.stream_custommsg(b"")
        while True:
            n = stream.next()
            if n is None:
                break
            yield nodepb.Custommsg.FromString(bytes(n))

    def send_custommsg(
            self,
            node_id: str,
            msg: bytes
    ) -> clnpb.SendcustommsgResponse:
        uri = "/cln.Node/SendCustomMsg"
        res = clnpb.SendcustommsgResponse
        req = clnpb.SendcustommsgRequest(
            node_id=normalize_node_id(node_id),
            msg=msg,
        ).SerializeToString()

        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def datastore(
            self,
            key,
            string=None,
            hex=None,
            mode=None,
            generation=None
    ):
        uri = "/cln.Node/Datastore"
        req = clnpb.DatastoreRequest(
            key=key,
            string=string,
            hex=hex,
            mode=mode,
            generation=generation
        ).SerializeToString()
        res = clnpb.DatastoreResponse
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def del_datastore(
            self,
            key,
            generation=None
    ):
        uri = "/cln.Node/DelDatastore"
        req = clnpb.DeldatastoreRequest(
            key=key,
            generation=generation
        ).SerializeToString()
        res = clnpb.DeldatastoreResponse
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def list_datastore(
            self,
            key=None
    ):
        uri = "/cln.Node/ListDatastore"
        req = clnpb.ListdatastoreRequest(
            key=key
        ).SerializeToString()
        res = clnpb.ListdatastoreResponse
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def get_lsp_client(
        self,
    ) -> LspClient:
        native_lsps = self.inner.get_lsp_client()
        return LspClient(native_lsps)

    def configure(self, close_to_addr: str) -> None:
        req = nodepb.GlConfig(
            close_to_addr=close_to_addr
        ).SerializeToString()

        return self.inner.configure(req)

    def wait_blockheight(
            self,
            blockheight: int,
            timeout: Optional[int] = None,
    ):
        """Wait until the blockchain has reached the specified blockheight."""
        uri = "/cln.Node/WaitBlockheight"
        req = clnpb.WaitblockheightRequest(
            blockheight=blockheight,
            timeout=timeout
        ).SerializeToString()
        res = clnpb.WaitblockheightResponse
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def fetch_invoice(
            self,
            offer: str,
            amount_msat: Optional[Amount] = None,
            quantity: Optional[int] = None,
            recurrence_counter: Optional[int] = None,
            recurrence_start: Optional[int] = None,
            recurrence_label: Optional[str] = None,
            timeout: Optional[int] = None,
            payer_note: Optional[str] = None,
    ) -> clnpb.FetchinvoiceResponse:
        """Fetch an invoice based on an offer.

        Contact the issuer of an offer to get an actual invoice that
        can be paid. It highlights any changes between the offer and
        the returned invoice.

        """
        uri = "/cln.Node/FetchInvoice"
        req = clnpb.FetchinvoiceRequest(
            offer=offer,
            amount_msat=amount_msat,
            quantity=quantity,
            recurrence_counter=recurrence_counter,
            recurrence_start=recurrence_start,
            recurrence_label=recurrence_label,
            timeout=timeout,
            payer_note=payer_note,
        ).SerializeToString()
        res = clnpb.FetchinvoiceResponse
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )

    def wait(
            self,
            subsystem,
            indexname,
            nextvalue: int
    ) -> clnpb.WaitResponse:
        """Wait for the next event in the provided subsystem.

        Returns once the index given by indexname in subsystem reaches
        or exceeds nextvalue.

        """
        uri = "/cln.Node/Wait"
        req = clnpb.WaitRequest(
            subsystem=subsystem,
            indexname=indexname,
            nextvalue=nextvalue,
        ).SerializeToString()
        res = clnpb.WaitResponse
        return res.FromString(
            bytes(self.inner.call(uri, bytes(req)))
        )


def normalize_node_id(node_id, string=False):
    if len(node_id) == 66:
        node_id = unhexlify(node_id)

    if len(node_id) != 33:
        raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

    if isinstance(node_id, str):
        node_id = node_id.encode('ASCII')
    return node_id if not string else hexlify(node_id).encode('ASCII')
