from . import scheduler_pb2 as schedpb
from . import greenlight_pb2 as nodepb
from . import node_pb2 as clnpb  # type: ignore
from . import glclient as native
from .tls import TlsConfig
from .rpc import Node as ClnNode
from .glrpc import Node as LegacyNode
from google.protobuf.message import Message as PbMessage

from .greenlight_pb2 import Amount
from binascii import hexlify, unhexlify
from typing import Optional, List, Union, Tuple, Iterable, SupportsIndex, Type, Any, TypeVar
import logging


# Keep in sync with the libhsmd version, this is tested in unit tests.
__version__ = "v0.11.0.1"


# Switch from legacy to cln API, eventually
Node = ClnNode

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

    def register(self, signer: Signer, invite_code: Optional[str] = None) -> schedpb.RegistrationResponse:
        res = self.inner.register(signer.inner, invite_code)
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

    def get_invite_codes(self) -> schedpb.ListInviteCodesResponse:
        res = self.inner.get_invite_codes()
        return schedpb.ListInviteCodesResponse.FromString(bytes(res))


def normalize_node_id(node_id, string=False):
    if len(node_id) == 66:
        node_id = unhexlify(node_id)

    if len(node_id) != 33:
        raise ValueError("node_id is not 33 (binary) or 66 (hex) bytes long")

    if isinstance(node_id, str):
        node_id = node_id.encode('ASCII')
    return node_id if not string else hexlify(node_id).encode('ASCII')
