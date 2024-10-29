import purerpc
import gltesting.test_pb2 as gltesting_dot_test__pb2


class GreeterServicer(purerpc.Servicer):
    async def SayHello(self, input_message):
        raise NotImplementedError()

    @property
    def service(self) -> purerpc.Service:
        service_obj = purerpc.Service(
            "gltesting.Greeter"
        )
        service_obj.add_method(
            "SayHello",
            self.SayHello,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                gltesting_dot_test__pb2.HelloRequest,
                gltesting_dot_test__pb2.HelloReply,
            )
        )
        return service_obj


class GreeterStub:
    def __init__(self, channel):
        self._client = purerpc.Client(
            "gltesting.Greeter",
            channel
        )
        self.SayHello = self._client.get_method_stub(
            "SayHello",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                gltesting_dot_test__pb2.HelloRequest,
                gltesting_dot_test__pb2.HelloReply,
            )
        )