# Tests that use a grpc-web client, without a client certificate, but
# payload signing for authentication.

from gltesting.fixtures import *
from gltesting.test_pb2_grpc import GreeterStub
from gltesting.test_pb2 import HelloRequest
import sonora.client
from pyln import grpc as clnpb
from base64 import b64encode
from time import time
import struct
from typing import Any


class GrpcWebClient:
    """A simple grpc-web client that implements the calling convention."""

    def __init__(self, node_grpc_web_proxy_uri, node_id: bytes):
        self.node_id = node_id
        self.node_grpc_web_proxy_uri = node_grpc_web_proxy_uri
        self.channel = sonora.client.insecure_web_channel(node_grpc_web_proxy_uri)
        self.stub = clnpb.NodeStub(self.channel)

    def call(self, method_name: str, req: Any) -> Any:
        ts = struct.pack("!Q", int(time() * 1000))
        metadata = [
            ("glauthpubkey", b64encode(self.node_id).decode("ASCII")),
            ("glauthsig", b64encode(b"\x00" * 64).decode("ASCII")),
            ("glts", b64encode(ts).decode("ASCII")),
        ]
        func = self.stub.__dict__.get(method_name)
        return func(req, metadata=metadata)


def test_start(grpc_web_proxy):
    with sonora.client.insecure_web_channel(
        f"http://localhost:{grpc_web_proxy.web_port}"
    ) as channel:
        stub = GreeterStub(channel)
        req = HelloRequest(name="greenlight")
        print(stub.SayHello(req))


def test_node_grpc_web(scheduler, node_grpc_web_proxy, clients):
    """Ensure that the"""
    # Start by creating a node
    c = clients.new()
    c.register(configure=True)
    n = c.node()
    _s = c.signer().run_in_thread()
    info = n.get_info()

    # Now extract the TLS certificates, so we can sign the payload.
    # TODO Configure the web client to sign its requests too
    node_id = info.id
    key_path = c.directory / "device-key.pem"
    ca_path = c.directory / "ca.pem"

    proxy_uri = f"http://localhost:{node_grpc_web_proxy.web_port}"
    web_client = GrpcWebClient(proxy_uri, node_id)

    # Issue a request to the node through the proxy.
    req = clnpb.GetinfoRequest()
    info = web_client.call("Getinfo", req)
    print(info)

    # Ask for a new address
    req = clnpb.NewaddrRequest()
    addr = web_client.call("NewAddr", req)
    print(addr)
