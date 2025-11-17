from gltesting.fixtures import *
from pyln.testing.utils import wait_for
from pyln import grpc as clnpb
import grpc
import pytest
import secrets
from pathlib import Path
from flaky import flaky

trmp_plugin_path = Path(__file__).parent / "plugins" / "trmp_htlc_hook.py"


def test_big_size_requests(clients):
    """We want to test if we can pass through big size (up to ~4MB)
    requests. These requests are handled by the gl-plugin to extract
    the request context that is passed to the signer.
    We need to test if the request is fully captured and passed to
    cln-grpc.
    """
    c1: Client = clients.new()
    c1.register()
    n1 = c1.node()
    # Size is roughly 4MB with some room for grpc overhead.
    size = 3990000

    # Write large data to the datastore.
    n1.datastore("some-key", hex=bytes.fromhex(secrets.token_hex(size)))


def test_max_message_size(clients):
    """Tests that the maximum message size is ensured by the plugin.
    This is currently hard-coded to 4194304bytes. The plugin should
    return with an grpc error.
    """
    c1: Client = clients.new()
    c1.register()
    n1 = c1.node()
    size = 4194304 + 1

    # Send message too large.
    with pytest.raises(ValueError):
        n1.datastore("some-key", hex=bytes.fromhex(secrets.token_hex(size)))


@flaky(max_runs=10)
def test_trampoline_pay(bitcoind, clients, node_factory):
    c1 = clients.new()
    c1.register()
    s1 = c1.signer()
    s1.run_in_thread()
    n1 = c1.node()

    # Fund greenlight node.
    addr = n1.new_address().bech32
    print(f"Send to address {addr}")
    txid = bitcoind.rpc.sendtoaddress(addr, 0.1)
    print(f"Generate a block to confirm {txid}")
    bitcoind.generate_block(1, wait_for_mempool=[txid])

    wait_for(lambda: txid in [o.txid.hex() for o in n1.list_funds().outputs])

    # Fund channel between nodes.
    l2 = node_factory.get_node(
        options={
            "plugin": trmp_plugin_path,
            "disable-plugin": "cln-grpc",
        }
    )
    n1.connect_peer(l2.info["id"], f"localhost:{l2.port}")
    n1.fund_channel(
        bytes.fromhex(l2.info["id"]),
        clnpb.AmountOrAll(amount=clnpb.Amount(msat=1000000000)),
    )
    bitcoind.generate_block(6, wait_for_mempool=1)

    wait_for(
        lambda: l2.info["id"]
        in [c.peer_id.hex() for c in n1.list_funds().channels if c.state == 2]
    )

    # create invoice and pay via trampoline. Trampoline is actually the
    # same node as the destination but we don't care as we just want to
    # test the business logic.
    invoice_preimage = (
        "17b08f669513b7379728fc1abcea5eaf3448bc1eba55a68ca2cd1843409cdc04"
    )
    inv = l2.rpc.invoice(
        amount_msat=50000000,
        label="trampoline-pay-test",
        description="trampoline-pay-test",
        preimage=invoice_preimage,
    )
    l2.rpc.setpaymentkey(invoice_preimage)
    l2.rpc.setcheckinvoice(inv["bolt11"])
    l2.rpc.setcheckamount(50000000)

    res = n1.trampoline_pay(inv["bolt11"], bytes.fromhex(l2.info["id"]))
    assert res
    assert len(res.payment_hash) == 32  # There was a confusion about hex/bytes return.

    l2.rpc.unsetchecks()

    # settle channel htlcs
    bitcoind.generate_block(10)
    wait_for(
        lambda: len(
            n1.list_peer_channels(bytes.fromhex(l2.info["id"])).channels[0].htlcs
        )
        == 0
    )

    # `trampoline_pay` is idempotent. A second invocation should return
    # the same result but must not send any htlc.
    res2 = n1.trampoline_pay(inv["bolt11"], bytes.fromhex(l2.info["id"]))
    ch = n1.list_peer_channels(bytes.fromhex(l2.info["id"])).channels[0]
    assert res2 == res

    assert ch.to_us_msat.msat == (1000000000 - (50000000 + 0.005 * 50000000))
    assert len(ch.htlcs) == 0

    # new unknown unconnected node without the trampoline featurebit.
    l3 = node_factory.get_node(options={"disable-plugin": "cln-grpc"})
    inv = l2.rpc.invoice(
        amount_msat=1000000,
        label="trampoline-pay-test-2",
        description="trampoline-pay-test-2",
    )

    # calling `trampoline_pay` with an unkown tmrp_node_id must fail.
    with pytest.raises(
        expected_exception=ValueError,
        match=f"Unknown peer {l3.info['id']}",
    ):
        res = n1.trampoline_pay(inv["bolt11"], bytes.fromhex(l3.info["id"]))

    n1.connect_peer(l3.info["id"], f"localhost:{l3.port}")

    # calling `trampoline_pay` with a trmp_node that does not support
    # trampoline payments must fail.
    with pytest.raises(
        expected_exception=ValueError,
        match="Peer doesn't support trampoline payments",
    ):
        res = n1.trampoline_pay(inv["bolt11"], bytes.fromhex(l3.info["id"]))


def test_trampoline_multi_htlc(bitcoind, clients, node_factory):
    c1 = clients.new()
    c1.register()
    s1 = c1.signer()
    s1.run_in_thread()
    n1 = c1.node()

    # Fund greenlight node.
    addr = n1.new_address().bech32
    print(f"Send o address {addr}")
    txid = bitcoind.rpc.sendtoaddress(addr, 0.1)
    print(f"Generate a block to confirm {txid}")
    bitcoind.generate_block(1, wait_for_mempool=[txid])

    wait_for(lambda: txid in [o.txid.hex() for o in n1.list_funds().outputs])

    # Fund channel between nodes.
    l2 = node_factory.get_node(
        options={
            "plugin": trmp_plugin_path,
        }
    )
    n1.connect_peer(l2.info["id"], f"localhost:{l2.port}")

    # Fund first channel
    n1.fund_channel(
        bytes.fromhex(l2.info["id"]),
        clnpb.AmountOrAll(amount=clnpb.Amount(msat=70000000)),
    )
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(lambda: len([c for c in n1.list_funds().channels if c.state == 2]) == 1)

    # Fund second channel
    n1.fund_channel(
        bytes.fromhex(l2.info["id"]),
        clnpb.AmountOrAll(amount=clnpb.Amount(msat=30000000)),
    )
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(lambda: len([c for c in n1.list_funds().channels if c.state == 2]) == 2)

    spendable = max(
        [
            c.spendable_msat.msat
            for c in n1.list_peer_channels(bytes.fromhex(l2.info["id"])).channels
        ]
    )
    print(f"spendable_msat: {spendable}")

    # Create an invoice with an amount larger than the capacity of the
    # bigger channel.
    inv = l2.rpc.invoice(
        amount_msat=spendable + 100000,
        label="trampoline-multi-htlc-test",
        description="trampoline-multi-htlc-test",
    )

    res = n1.trampoline_pay(inv["bolt11"], bytes.fromhex(l2.info["id"]))
    assert res
    assert res.parts == 2


def test_lsps_plugin_calls(clients, bitcoind, node_factory, lsps_server):
    """Test that we can call the `lsps-jitchannel` method from a
    variety of places.

    ```ditaa
    S --- LSP --- R1
    ```

    """
    # Bootstrap a very simple network
    lsp_id = lsps_server.info["id"]
    (r1,) = node_factory.get_nodes(1, opts=[{"disable-plugin": "cln-grpc"}])

    # Get the GL node
    c = clients.new()
    c.register()
    s1 = c.signer()
    s1.run_in_thread()
    s = c.node()
    s.connect_peer(lsps_server.info["id"], f"localhost:{lsps_server.port}")

    # Try the pyo3 bindings from gl-client-py
    res = s.lsp_invoice(label="lbl2", description="description", amount_msat=31337)
    from pyln.proto import Invoice
    inv = Invoice.decode(res.bolt11)
    pprint(inv)
    # Only one routehint, with only one hop, the LSP to the destination
    assert len(inv.route_hints.route_hints) == 1
    rh = inv.route_hints.route_hints[0]
    assert rh.pubkey == bytes.fromhex(lsp_id)
