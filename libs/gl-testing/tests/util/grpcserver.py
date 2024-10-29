# This is a simple grpc server serving the `gltesting/test.proto`
# protocol. It is used to test whether the grpc-web to grpc/h2
# proxying is working.

from gltesting.test_pb2 import HelloRequest, HelloReply
from gltesting.test_grpc import GreeterServicer
from ephemeral_port_reserve import reserve
import purerpc
from threading import Thread
import anyio



class Server(GreeterServicer):
    def __init__(self, *args, **kwargs):
        GreeterServicer.__init__(self, *args, **kwargs)
        self.grpc_port = reserve()
        self.inner = purerpc.Server(self.grpc_port)
        self.thread: Thread | None = None
        self.inner.add_service(self.service)

    async def SayHello(self, message):
        return HelloReply(message="Hello, " + message.name)

    def start(self):
        def target():
            try:
                anyio.run(self.inner.serve_async)
            except Exception as e:
                print("Error starting the grpc backend")

        self.thread = Thread(target=target, daemon=True)
        self.thread.start()

    def stop(self):
        self.inner.aclose
