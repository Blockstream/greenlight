# Tests that use a grpc-web client, without a client certificate, but
# payload signing for authentication.

from gltesting.fixtures import *
from gltesting.test_pb2_grpc import GreeterStub
from gltesting.test_pb2 import HelloRequest
import sonora.client

def test_start(grpc_web_proxy):
    with sonora.client.insecure_web_channel(
        f"http://localhost:{grpc_web_proxy.web_port}"
    ) as channel:
        stub = GreeterStub(channel)
        req = HelloRequest(name="greenlight")
        print(stub.SayHello(req))

