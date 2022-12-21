from .tls import TlsConfig
from . import glclient as native

class BaseNode(object):
    """Common functionality shared by both legacy and cln nodes.
    """
    def __init__(
            self,
            node_id: bytes,
            network: str,
            tls: TlsConfig,
            grpc_uri: str
    ):
        self.tls = tls
        self.grpc_uri = grpc_uri
        self.inner = native.Node(
            node_id=node_id,
            network=network,
            tls=tls.inner,
            grpc_uri=grpc_uri
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
