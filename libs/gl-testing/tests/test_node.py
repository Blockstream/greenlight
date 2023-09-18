from gltesting.identity import Identity
from gltesting.fixtures import *
from pyln.testing.utils import wait_for
from rich.pretty import pprint
from glclient import nodepb
from pyln import grpc as clnpb
from flaky import flaky

import struct
import time
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


@pytest.mark.skip(reason="routehints seem to be missing in regtest")
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
    bitcoind.generate_block(6, wait_for_mempool=1)

    # Now wait for the channel to confirm
    wait_for(lambda: gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL')
    wait_for(lambda: len(l1.rpc.listchannels()['channels']) == 2)

    inv = gl1.invoice(
        amount_msat=clnpb.AmountOrAny(amount=clnpb.Amount(msat=10000)),
        description="desc",
        label="lbl"
    ).bolt11

    decoded = l1.rpc.decodepay(inv)
    pprint(decoded)
    l1.rpc.pay(inv)

    print(c.list_closed_channels())


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


def test_node_invoice_expiration(clients):
    """Test that we can set the invoice expiry

    The invoice should expire after the set expiry.
    """
    c1: Client = clients.new()
    c1.register()
    s = c1.signer().run_in_thread()
    n1 = c1.node()

    now = int(time.time())
    res = n1.invoice(
        amount_msat=clnpb.AmountOrAny(any=True),
        description="desc",
        label="lbl",
        expiry=100,
    )
    assert now <= res.expires_at <= now + 160
    

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
    from pyln import grpc as clngrpc
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
    p = gl1.pay(
        inv,
        clnpb.Amount(msat=31337)
    )
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

    from rich.rule import Rule
    from rich.console import Console
    console = Console()
    console.rule("[bold red]<pay>")
    gl1.pay(i['bolt11'])
    console.rule("[bold red]</pay>")

    pay = gl1.listpays()
    assert len(pay.pays) == 1
    assert pay.pays[0].preimage.hex() == preimage


def test_lsp_jit_fee(clients, node_factory, bitcoind):
    """Test that the LSP (our peer) is allowed to alter the amount to
    deduct its fee.

    The scenario is simple: l1 -> gl1, with l1 being the LSP,
    forwarding less than it is supposed to according to the onion
    packet. We explicitly opt-in to this deduction by having a
    matching invoice stashed with the node. Upon receiving the
    incoming HTLC the plugin fetches the invoice, checks that indeed
    the expected `total_msat` is lower than the sender thought, and
    then we modify the onion payload in order to get `lightningd` to
    accept it and settle the invoice with it.

    We test multiple parts and overpay slightly to verify that even
    that works out ok.

    """
    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()
    gl1 = c.node()
    l1 = node_factory.get_node()
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')
    l1.fundwallet(10**6)
    wait_for(lambda: len(l1.rpc.listfunds()['outputs']) > 0)
    l1.rpc.fundchannel(c.node_id.hex(), 'all')
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(lambda: l1.rpc.listpeerchannels()['channels'][0]['state'] == 'CHANNELD_NORMAL')

    # Create an invoice for 10k
    preimage = '00' * 32
    payment_hash = '66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925'
    parts = 2
    p1, p2 = 300000, 700000  # The two parts we're going to use
    fee = 100000  # Fee leverage on each part
    inv = gl1.create_invoice(
        label='lbl',
        amount=nodepb.Amount(millisatoshi=p1 + p2 - parts * fee),
        description="desc",
        preimage=preimage,
    ).bolt11

    decoded = l1.rpc.decodepay(inv)

    # So we have an invoice for 100k, now send it in two parts:
    o1 = l1.rpc.createonion(hops=[{
        "pubkey": c.node_id.hex(),
        "payload": (
            "30" +
            "0203" + "0493e0" +  # amt_to_forward: 30k
            "04016e" +  # 110 blocks CLTV
            "0823" + decoded['payment_secret'] + "0f4240" +  # Payment_secret + total_msat
            "FB0142"  # Typ 251 payload 0x42 (testing we don't lose TLVs)
        )
    }], assocdata=payment_hash)

    o2 = l1.rpc.createonion(hops=[{
        "pubkey": c.node_id.hex(),
        "payload": (
            "30" +
            "0203" + "0aae60" +  # amt_to_forward: 70k
            "04016e" +  # 110 blocks CLTV
            "0823" + decoded['payment_secret'] + "0f4240" + # Payment_secret + total_msat
            "FB0142"  # Typ 251 payload 0x42 (testing we don't lose TLVs)
        )
    }], assocdata=payment_hash)

    l1.rpc.call('sendonion', {
        'onion': o1['onion'],
        'first_hop': {
            "id": c.node_id.hex(),
            "amount_msat": f"{p1 - fee}msat",
            "delay": 21,
        },
        'payment_hash': payment_hash,
        'partid': 1,
        'groupid': 1,
        'shared_secrets': o1['shared_secrets'],
    })
    l1.rpc.call('sendonion', {
        'onion': o2['onion'],
        'first_hop': {
            "id": c.node_id.hex(),
            "amount_msat": f"{p2 - fee}msat",
            "delay": 21,
        },
        'payment_hash': payment_hash,
        'partid': 2,
        'groupid': 1,
        'shared_secrets': o1['shared_secrets'],
    })

    # Check that custom payloads are preserved. See the type=251 field
    # at the end of the onion-construction above.
    c.find_node().process.wait_for_log(r'Serialized payload: .*fb0142')

    l1.rpc.waitsendpay(
        payment_hash=payment_hash,
        partid=1,
        timeout=10
    )
    l1.rpc.waitsendpay(
        payment_hash=payment_hash,
        partid=2,
        timeout=10
    )


def test_custommsg(clients, node_factory, bitcoind, executor):
    """Connect a GL node and a CLN node and have them talk.
    """
    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()
    gl1 = c.node()
    l1 = node_factory.get_node()
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')

    # Part 1: CLN -> GL
    m = gl1.stream_custommsg()
    f = executor.submit(next, m)

    # Give the executor time to actually register itself with the
    # notification
    import time
    time.sleep(1)
    l1.rpc.sendcustommsg(node_id=c.node_id.hex(), msg="FFFFDEADBEEF")

    res = f.result(1)
    assert res.payload == b'\xff\xff\xde\xad\xbe\xef'
    assert res.peer_id.hex() == l1.info['id']

    # Part 2: GL -> CLN
    gl1.send_custommsg(bytes.fromhex(l1.info['id']), b"\xff\xffhello")

    l1.daemon.wait_for_logs([
        r'connectd: peer_in INVALID 65535',
        r'Calling custommsg hook of plugin chanbackup',
    ])


def test_node_reconnect(clients, scheduler, node_factory, bitcoind):
    """Connect from GL to a peer, then restart and we should reconnect.
    """
    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()
    gl1 = c.node()

    l1 = node_factory.get_node()
    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')

    time.sleep(1)
    node = scheduler.nodes[0]
    node.process.stop()
    node.process = None

    gl1 = c.node()

    rpc = scheduler.nodes[0].rpc()
    wait_for(lambda: rpc.listpeers()['peers'] != [])
    peer = rpc.listpeers()['peers'][0]
    assert peer['connected']
    assert peer['id'] == l1.info['id']


def test_vls_crash_repro(
        clients: Clients,
        scheduler: Scheduler,
        node_factory,
        bitcoind) -> None:
    """Reproduce an overflow panic in VLS v0.10.0. """
    l1, = node_factory.line_graph(1, opts={'experimental-anchors': None})
    assert(l1.rpc.getinfo()['version'] == 'v23.08gl1')

    c = clients.new()
    c.register(configure=True)
    s = c.signer().run_in_thread()
    gl1 = c.node()

    gl1.connect_peer(l1.info['id'], f'127.0.0.1:{l1.daemon.port}')

    l1.fundwallet(10**7)
    l1.rpc.fundchannel(c.node_id.hex(), 'all')
    bitcoind.generate_block(1, wait_for_mempool=1)

    wait_for(lambda: l1.rpc.listpeerchannels()['channels'][0]['state'] == 'CHANNELD_NORMAL')

    # Roei reports that the issue can be triggered by sending n from
    # l1 to n1 and then (n-1)msat back to l1

    inv = gl1.invoice(
        amount_msat=clnpb.AmountOrAny(amount=clnpb.Amount(msat=2500000)),
        description="desc",
        label="lbl"
    ).bolt11

    l1.rpc.pay(inv)
    inv = l1.rpc.invoice(amount_msat=2499000, label="lbl", description="desc")
