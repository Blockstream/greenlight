from gltesting.identity import Identity
from gltesting.fixtures import *
from pyln.testing.utils import wait_for
from rich.pretty import pprint
from glclient import nodepb

def test_node_start(scheduler, clients):
    c = clients.new()
    res = c.register(configure=True)
    pprint(res)

    node_info = c.scheduler().schedule()
    pprint(node_info)
    assert node_info.grpc_uri is not None


def test_node_connect(scheduler, clients, bitcoind):
    """Register and schedule a node, then connect to it.
    """
    c = clients.new()
    c.register(configure=True)
    n = c.node()
    info = n.get_info()
    pprint(info)


def test_node_signer(clients, executor):
    """Ensure we can attach a signer to the node and sign an invoice.
    """
    c = clients.new()
    c.register(configure=True)
    n = c.node()

    # Running the `invoice` invocation in a separate thread since
    # it'll block until the signer connects.
    fi = executor.submit(n.create_invoice, 'test', nodepb.Amount(millisatoshi=42000))

    # Now attach the signer and the above call should return
    h = c.signer().run_in_thread()

    inv = fi.result(10)
    pprint(inv)
    h.shutdown()


def test_node_network(node_factory, clients, bitcoind):
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

    # Now open a channel from l2 -> gl1
    l2.fundwallet(sats=2*10**6)
    l2.rpc.fundchannel(c.node_id.hex(), 'all')
    bitcoind.generate_block(1, wait_for_mempool=1)

    # Now wait for the channel to confirm
    wait_for(lambda: gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL')
    import time
    time.sleep(5)

    inv = gl1.create_invoice('test', nodepb.Amount(millisatoshi=10000)).bolt11
    decoded = l1.rpc.decodepay(inv)
    pprint(decoded)
    l1.rpc.pay(inv)


def test_node_invoice_preimage(clients):
    """Test that we can create an invoice with a specific preimage
    """
    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()
    gl1 = c.node()

    preimage = "00"*32
    expected = '66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925'

    i = gl1.create_invoice(
        label='lbl',
        amount=nodepb.Amount(millisatoshi=21000000),
        description="desc",
        preimage=preimage,
    )

    assert i.payment_hash.hex() == expected
