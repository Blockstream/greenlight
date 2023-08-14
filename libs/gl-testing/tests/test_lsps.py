from gltesting.fixtures import *
from pyln.testing.utils import NodeFactory, BitcoinD, LightningNode
import json

from glclient.lsps import ProtocolList

import threading

logger = logging.getLogger(__name__)

class AwaitResult:
    """ A very poor implementation of an awaitable in python

    It is inefficient and uses Threads under the hood.
    But it gives something like the `await` syntax which
    makes it easier to write tests.
    """
    def __init__(self, function, args=None, kwargs=None):
        self._result = None
        self._thread = None
        self._exception = None

        if args is None:
            args = []
        if kwargs is None:
            kwargs = dict()

        def wrap_function(*args, **kwargs):
            try:
                self._result = function(*args, **kwargs)
            except Exception as e:
                self._exception = e

        self._thread = threading.Thread(target=wrap_function, args=args, kwargs=kwargs)
        self._thread.start()

    def await_result(self, timeout_seconds : float=30.0):
        self._thread.join(timeout=timeout_seconds)
        if self._thread.is_alive():
            raise TimeoutError()

        if self._exception:
            raise self._exception

        return self._result


def test_lsps_list_protocol(clients : Clients, node_factory : NodeFactory, bitcoind : BitcoinD):
    # Create the LSP
    n1 : LightningNode = node_factory.get_node()

    # Create and configure the greenlight client and connect it to the LSP
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
    s = c.signer().run_in_thread()

    # Connect our greenlight node (client ot the LSP)
    lsp_ip = n1.info["binding"][0]["address"]
    lsp_port = n1.info["binding"][0]["port"]
    gl1.connect_peer(n1.info['id'], addr=f"{lsp_ip}:{lsp_port}")


    # Get the lsp-client and do list-protocols
    lsp_client = gl1.get_lsp_client(n1.info['id'])

    # The client sends a message
    json_rpc_id = "abcdef"
    protocol_fut = AwaitResult(lambda: lsp_client.list_protocols(json_rpc_id=json_rpc_id))

    # The n1.rpc.sendcustommsg expects that both the node_id and msg are hex encoded strings
    msg_content = {
        "jsonrpc" : "2.0",
        "id" : json_rpc_id,
        "result" : {"protocols" : [1,2]}
    }

    json_str = json.dumps(msg_content)
    json_bytes = json_str.encode("utf-8")
    msg_str = "9419" + json_bytes.hex()

    n1.rpc.sendcustommsg(
        node_id = gl1.get_info().id.hex(),
        msg = msg_str
    )

    result = protocol_fut.await_result()
    assert result == ProtocolList([1,2])