from gltesting.identity import Identity
from gltesting.fixtures import *
from pyln.testing.utils import wait_for, NodeFactory, LightningNode
from rich.pretty import pprint
from glclient import nodepb
from pyln import grpc as clngrpc
from pyln import grpc as clnpb
import grpc
from glclient import scheduler_pb2 as schedpb
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
    gl1_address = gl1.new_address().bech32
    bitcoind.rpc.sendtoaddress(gl1_address, 1)
    bitcoind.generate_block(1, wait_for_mempool=1)
    wait_for(lambda: len(gl1.list_funds().outputs) > 0 )

    # Now open a channel from gl1 -> l2
    l1_node_id = bytes.fromhex(l1.rpc.getinfo()['id'])
    channel_result = gl1.fund_channel(
        l1_node_id,
        clnpb.AmountOrAll(
            amount=clnpb.Amount(msat=50000000)
        )
    )
    bitcoind.generate_block(1, wait_for_mempool=1)


def test_peerlist_datastore_add(node_factory: NodeFactory, clients: Clients):
    """Check that connected peers are written to the datastore. We can
    connect from the gl node to the peer node and the peer should be
    written to the datastore.
    """
    l1: LightningNode = node_factory.get_node()
    c: Client = clients.new()
    c.register()
    gl1 = c.node()

    # We need the signer for the handshake
    s = c.signer().run_in_thread()

    # Connect from the gl node to a peer node.
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')
    # Check that the peer id and address is written to the database
    res = gl1.list_datastore(key=["greenlight", "peerlist"])
    assert l1.info['id'] in res.datastore[0].key


def test_peerlist_datastore_remove(node_factory: NodeFactory, clients: Clients):
    """Check that peers are removed from the peerlist datastore on the
    call of disconnect.
    """
    l1: LightningNode = node_factory.get_node()
    c: Client = clients.new()
    c.register()
    gl1 = c.node()

    # We need the signer for the handshake
    s = c.signer().run_in_thread()
    
    # Connect from the gl node to a peer node.
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')
    # Check that the peerlist has expected len.
    res = gl1.list_datastore(key=["greenlight", "peerlist"])
    assert len(res.datastore) == 1

    # Disconnect
    gl1.disconnect_peer(l1.info['id'])

    # Check that the peer entry is removed
    res = gl1.list_datastore(key=["greenlight", "peerlist"])
    assert len(res.datastore) == 0


def test_reconnect_peers_on_startup(node_factory: NodeFactory, clients: Clients, scheduler: Scheduler):
    """Check that the node reconnects to the peers from the datastore
    peerlist.
    """
    l1: LightningNode = node_factory.get_node()
    c: Client = clients.new()
    c.register()
    gl1 = c.node()

    # We need the signer for the handshake
    s = c.signer().run_in_thread()

    # Connect from the gl node to a peer node.
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')
    # Check that the peerlist has expected len.
    res = gl1.list_datastore(key=["greenlight", "peerlist"])
    assert len(res.datastore) == 1

    # Stop node to disconnect from peer.
    c.schedsvc.nodes[0].process.stop()
    c.schedsvc.nodes[0].process = None

    # Reschedule node and check that we reconnected to the peer
    gl1 = c.node()

    def check(gl1):
        """Little helper to wait for and verify
        """
        res = gl1.list_peers()
        print(res)
        return len(res.peers) == 1 and res.peers[0].connected

    wait_for(lambda: check(gl1))

def test_configure_close_to_addr(node_factory, clients, bitcoind):
    l1, l2 = node_factory.line_graph(2)
    l2.fundwallet(sats=2*10**6)

    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()

    s = c.signer().run_in_thread()
    gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')
    
    close_to_addr = bitcoind.getnewaddress()
    gl1.configure(close_to_addr)

    l2.rpc.fundchannel(c.node_id.hex(), 'all')
    bitcoind.generate_block(1, wait_for_mempool=1)

    wait_for(lambda:
        gl1.list_peers().peers[0].channels[0].state == 2
    )
    
    assert gl1.list_peers().peers[0].channels[0].close_to_addr == close_to_addr
