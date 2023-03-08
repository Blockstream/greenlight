from gltesting.identity import Identity
from gltesting.fixtures import *
from pyln.testing.utils import wait_for
from rich.pretty import pprint
from glclient import nodepb

import unittest

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


def test_cln_grpc_interface(clients):
    """Test that we can talk to the cln-grpc interface.

    Temporarily bypasses the Rust library, and is not signed
    therefore, until we map the methods into the Rust library.

    """
    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()

    gl1 = c.node()

    # Reach into the node configuration
    from glclient import node_pb2_grpc as clngrpc
    from glclient import node_pb2 as clnpb
    import grpc
    cred = grpc.ssl_channel_credentials(
        root_certificates=gl1.tls.ca,
        private_key=gl1.tls.id[1],
        certificate_chain=gl1.tls.id[0]
    )
    grpc_uri = gl1.grpc_uri[8:]  # Strip the `https://` prefix
    chan = grpc.secure_channel(grpc_uri, cred)
    client = clngrpc.NodeStub(chan)

    info = client.Getinfo(clnpb.GetinfoRequest())
    print(info)


def test_node_invoice_amountless(bitcoind, node_factory, clients):
    """Test that the request is being mapped correctly.
    ```dot
    l1 -> gl1
    ```
    """
    l1 = node_factory.get_node()
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
    s = c.signer().run_in_thread()

    # Now open a channel from l2 <- gl1 (this could be easier...)
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')
    addr = gl1.new_address().address
    txid = bitcoind.rpc.sendtoaddress(addr, 1)
    bitcoind.generate_block(1, wait_for_mempool=[txid])
    wait_for(lambda: len(gl1.list_funds().outputs) == 1)
    gl1.fund_channel(node_id=l1.info['id'], amount=nodepb.Amount(satoshi=10**6))
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(lambda: gl1.list_peers().peers[0].channels[0].state == "CHANNELD_NORMAL")

    # Generate an invoice without amount:
    inv = l1.rpc.call('invoice', payload={
        'label': 'test',
        'amount_msat': 'any',
        'description': 'desc'
    })['bolt11']
    print(inv)
    print(l1.rpc.decodepay(inv))
    p = gl1.pay(inv, amount=nodepb.Amount(millisatoshi=31337))
    invs = l1.rpc.listinvoices()['invoices']

    assert(len(invs) == 1)
    assert(invs[0]['status'] == 'paid')


def test_node_listpays_preimage(clients, node_factory, bitcoind):
    """Test that GL nodes correctly return incoming payment details.
    """
    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()
    gl1 = c.node()
    l1 = node_factory.get_node()
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')
    addr = gl1.new_address().address
    txid = bitcoind.rpc.sendtoaddress(addr, 1)
    bitcoind.generate_block(1, wait_for_mempool=[txid])
    wait_for(lambda: len(gl1.list_funds().outputs) == 1)
    gl1.fund_channel(node_id=l1.info['id'], amount=nodepb.Amount(satoshi=10**6))
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(lambda: gl1.list_peers().peers[0].channels[0].state == "CHANNELD_NORMAL")

    preimage = "00"*32

    i = l1.rpc.call("invoice", {
        'amount_msat': '2100sat',
        'label': 'lbl',
        'description': 'desc',
        'preimage': preimage,
    })

    gl1.pay(i['bolt11'])

    pay = gl1.listpays()
    assert len(pay.pays) == 1
    assert pay.pays[0].preimage.hex() == preimage

    # TODO: Remove as soon as we deprecate `list_payments`
    pay = gl1.list_payments()
    assert len(pay.payments) == 1
    assert pay.payments[0].payment_preimage.hex() == preimage
