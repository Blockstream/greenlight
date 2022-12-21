from gltesting.identity import Identity
from gltesting.fixtures import *
from pyln.testing.utils import wait_for
from rich.pretty import pprint
from glclient import nodepb
from glclient import clnpb
import time

def test_node_network_gl_fund(node_factory, clients, bitcoind):
    """Setup a small network and check that we can send/receive payments.

    ```dot
    l1 -> l2 -> gl1
    ```
    """
    l1, l2 = node_factory.line_graph(2)
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()

    # Handshake needs signer for ECDH of Noise_XK exchange
    s = c.signer().run_in_thread()
    gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')

    # Fund gl1
    gl1_address = gl1.new_addr().bech32
    bitcoind.rpc.sendtoaddress(gl1_address, 1)
    bitcoind.generate_block(1, wait_for_mempool=1)
    wait_for(lambda: len(gl1.list_funds().outputs) > 0 )

    # Now open a channel from gl1 -> l2
    l1_node_id = l1.rpc.getinfo()['id']
    channel_result = gl1.fund_channel(
        id=bytes.fromhex(l1_node_id),
        amount=clnpb.AmountOrAll(amount=clnpb.Amount(msat=50000000))
    )
    bitcoind.generate_block(1, wait_for_mempool=1)
