from gltesting.fixtures import *
from gltesting.scheduler import Scheduler
from pyln.testing.utils import NodeFactory, BitcoinD, LightningNode
import json

from glclient.lsps import ProtocolList
import time

import threading
import subprocess

logger = logging.getLogger(__name__)


def get_lsps_dummy_plugin_path() -> str:
    # Find the fully specified path to the LSPS-plugin
    # This plugin sets the feature flags and makes the node appear as an LSP

    base_path, _ = os.path.split(__file__)
    return os.path.join(base_path, "util", "dummy_lsps_plugin.sh")


@pytest.fixture
def dummy_plugin(scope="session"):
    logger.info(
        f"Configure {get_lsps_dummy_plugin_path()} to become executable using chmod +x"
    )
    subprocess.run(["chmod", "+x", get_lsps_dummy_plugin_path()], check=True)


class AwaitResult:
    """A very poor implementation of an awaitable in python

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

    def await_result(self, timeout_seconds: float = 30.0):
        self._thread.join(timeout=timeout_seconds)
        if self._thread.is_alive():
            raise TimeoutError()

        if self._exception:
            raise self._exception

        return self._result


def test_lsps_list_protocol(
    clients: Clients, node_factory: NodeFactory, bitcoind: BitcoinD
):
    # Create the LSP
    n1: LightningNode = node_factory.get_node()

    # Create and configure the greenlight client and connect it to the LSP
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
    s = c.signer().run_in_thread()

    # Connect our greenlight node (client ot the LSP)
    lsp_ip = n1.info["binding"][0]["address"]
    lsp_port = n1.info["binding"][0]["port"]
    gl1.connect_peer(n1.info["id"], addr=f"{lsp_ip}:{lsp_port}")

    # Get the lsp-client and do list-protocols
    lsp_client = gl1.get_lsp_client()

    # The client sends a message
    json_rpc_id = "abcdef"
    protocol_fut = AwaitResult(
        lambda: lsp_client.list_protocols(
            peer_id=n1.info["id"], json_rpc_id=json_rpc_id
        )
    )

    # The sleep ensures the lsp-client has actually send the message and is ready to receive
    # the response
    time.sleep(1.0)

    # The n1.rpc.sendcustommsg expects that both the node_id and msg are hex encoded strings
    msg_content = {"jsonrpc": "2.0", "id": json_rpc_id, "result": {"protocols": [1, 2]}}

    json_str = json.dumps(msg_content)
    json_bytes = json_str.encode("utf-8")
    msg_str = "9419" + json_bytes.hex()

    n1.rpc.sendcustommsg(node_id=gl1.get_info().id.hex(), msg=msg_str)

    result = protocol_fut.await_result()
    assert result == ProtocolList([1, 2])


def test_list_lsp_server(
    clients: Clients, node_factory: NodeFactory, bitcoind: BitcoinD, dummy_plugin: None
):
    # Create a network
    n1: LightningNode = node_factory.get_node(
        options=dict(plugin=get_lsps_dummy_plugin_path())
    )
    n2: LightningNode = node_factory.get_node(
        options=dict(plugin=get_lsps_dummy_plugin_path())
    )
    n3: LightningNode = node_factory.get_node()

    # Fund all nodes so they can open a channel
    n1.fundwallet(100_000_000)
    n2.fundwallet(100_000_000)
    n3.fundwallet(100_000_000)

    # Create a basic channel graph
    n1.openchannel(n2, 500_000, connect=True)
    n2.openchannel(n3, 500_000, connect=True)

    # Initiate the greenlight node
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
    s = c.signer().run_in_thread()

    n1_full_address = (
        f"{n1.info['binding'][0]['address']}:{n1.info['binding'][0]['port']}"
    )
    _ = gl1.connect_peer(node_id=n1.info["id"], addr=n1_full_address)

    # Await gossip
    time.sleep(1.0)

    lsp_client = gl1.get_lsp_client()
    lsp_servers = lsp_client.list_lsp_servers()

    assert len(lsp_servers) == 2, "There are 2 lsp-servers defined"
    assert n1.info["id"] in lsp_servers
    assert n2.info["id"] in lsp_servers
    assert n3.info["id"] not in lsp_servers
