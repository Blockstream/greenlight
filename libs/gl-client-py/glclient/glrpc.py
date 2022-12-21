from .baserpc import BaseNode
from .tls import TlsConfig
from . import greenlight_pb2 as nodepb
from .greenlight_pb2 import Amount
from typing import Optional, List, Union, Tuple, Iterable, SupportsIndex, Type, Any, TypeVar
import logging
from binascii import hexlify, unhexlify
from . import node_pb2 as clnpb  # type: ignore


class Node(BaseNode):
    def __init__(self, node_id: bytes, network: str, tls: TlsConfig, grpc_uri: str) -> None:
        BaseNode.__init__(self, node_id, network, tls, grpc_uri)
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
            node_id=peer_id,
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

    def close_channel(
            self,
            peer_id,
            unilateraltimeout=None,
            destination=None
    ) -> nodepb.CloseChannelResponse:
        if len(peer_id) == 66:
            peer_id = unhexlify(peer_id)

        if len(peer_id) != 33:
            raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

        if isinstance(peer_id, str):
            node_id = peer_id.encode('ASCII')

        uri = "/greenlight.Node/CloseChannel"
        res = nodepb.CloseChannelResponse
        req = nodepb.CloseChannelRequest(
            node_id=peer_id,
            unilateraltimeout=unilateraltimeout,
            destination=destination,
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
