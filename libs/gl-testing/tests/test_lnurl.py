"""End-to-end integration tests for LNURL flows.

These tests spin up real CLN nodes, a CLN-backed LNURL server, and a
Greenlight SDK node, then exercise the full LNURL-pay and
LNURL-withdraw protocols.

Network topology:
    gl_sdk_node ── channel ── relay ── channel ── service_node (LNURL server)
"""

from gltesting.fixtures import *  # noqa: F401, F403
from pyln.testing.utils import wait_for

import glsdk


MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon about"
)

CHANNEL_SATS = 1_000_000  # 1M sats


def make_sdk_node(nobody_id, scheduler):
    """Register a GL node via the SDK and return it with signer running."""
    dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
    config = glsdk.Config().with_developer_cert(dev_cert)
    node = glsdk.register(MNEMONIC, None, config)
    return node, config


def fund_and_connect(node_factory, bitcoind, lnurl_service):
    """Create a relay node with channels to the LNURL service node.

    Returns the relay node, already funded and with a NORMAL channel to
    the service.
    """
    relay = node_factory.get_node(options={"disable-plugin": "cln-grpc"})
    service_node = lnurl_service.cln_node
    service_id = service_node.info["id"]

    # Connect relay <-> service
    relay.rpc.connect(service_id, "127.0.0.1", service_node.daemon.port)

    # Fund relay
    addr = relay.rpc.newaddr()["bech32"]
    bitcoind.rpc.sendtoaddress(addr, 1)
    bitcoind.generate_block(1, wait_for_mempool=1)
    wait_for(lambda: len(relay.rpc.listfunds()["outputs"]) > 0)

    # Open channel relay -> service
    relay.rpc.fundchannel(service_id, CHANNEL_SATS)
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(
        lambda: any(
            ch["state"] == "CHANNELD_NORMAL"
            for ch in relay.rpc.listpeerchannels(service_id)["channels"]
        )
    )

    return relay


def test_resolve_lnurl_pay(
    scheduler, nobody_id, node_factory, bitcoind, lnurl_service
):
    """Resolve an LNURL-pay endpoint via the SDK."""
    sdk_node, config = make_sdk_node(nobody_id, scheduler)

    try:
        resolved = sdk_node.resolve_lnurl(lnurl_service.pay_url)

        assert isinstance(resolved, glsdk.ResolvedLnUrl.PAY)
        data = resolved.data
        assert data.min_sendable == lnurl_service.min_sendable
        assert data.max_sendable == lnurl_service.max_sendable
        assert len(data.description) > 0
        assert data.callback.startswith(lnurl_service.base_url)
    finally:
        sdk_node.disconnect()


def test_resolve_lnurl_withdraw(
    scheduler, nobody_id, node_factory, bitcoind, lnurl_service
):
    """Resolve an LNURL-withdraw endpoint via the SDK."""
    sdk_node, config = make_sdk_node(nobody_id, scheduler)

    try:
        resolved = sdk_node.resolve_lnurl(lnurl_service.withdraw_url)

        assert isinstance(resolved, glsdk.ResolvedLnUrl.WITHDRAW)
        data = resolved.data
        assert data.min_withdrawable == lnurl_service.min_withdrawable
        assert data.max_withdrawable == lnurl_service.max_withdrawable
        assert len(data.k1) > 0
    finally:
        sdk_node.disconnect()


def test_resolve_lightning_address_url(
    scheduler, nobody_id, node_factory, bitcoind, lnurl_service
):
    """Resolve a Lightning Address well-known URL (LUD-16)."""
    sdk_node, config = make_sdk_node(nobody_id, scheduler)

    try:
        resolved = sdk_node.resolve_lnurl(lnurl_service.lightning_address_url)

        assert isinstance(resolved, glsdk.ResolvedLnUrl.PAY)
        data = resolved.data
        assert data.min_sendable == lnurl_service.min_sendable
    finally:
        sdk_node.disconnect()


def test_lnurl_pay_end_to_end(
    scheduler, nobody_id, clients, node_factory, bitcoind, lnurl_service
):
    """Full LNURL-pay flow: resolve → pay → verify.

    Uses a GL SDK node with outbound liquidity to pay an LNURL service.
    """
    # Use the low-level client to set up channels, since the SDK node
    # doesn't expose connect_peer / fund_channel directly.
    relay = fund_and_connect(node_factory, bitcoind, lnurl_service)

    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
    s = c.signer().run_in_thread()

    relay_id = relay.info["id"]

    # Connect GL node to relay and open channel
    gl1.connect_peer(relay_id, f"127.0.0.1:{relay.daemon.port}")
    gl_addr = gl1.new_address().bech32
    bitcoind.rpc.sendtoaddress(gl_addr, 0.5)
    bitcoind.generate_block(1, wait_for_mempool=1)
    wait_for(lambda: len(gl1.list_funds().outputs) > 0)

    from pyln import grpc as clnpb

    gl1.fund_channel(
        bytes.fromhex(relay_id),
        clnpb.AmountOrAll(amount=clnpb.Amount(msat=CHANNEL_SATS * 1000)),
    )
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(
        lambda: any(
            ch.state == 2  # CHANNELD_NORMAL
            for ch in gl1.list_peer_channels().channels
        )
    )

    # Now build an SDK-level Node for LNURL operations
    creds_bytes = c.creds().to_bytes()
    sdk_node = glsdk.Node(glsdk.Credentials.load(creds_bytes))

    try:
        # Resolve
        resolved = sdk_node.resolve_lnurl(lnurl_service.pay_url)
        assert isinstance(resolved, glsdk.ResolvedLnUrl.PAY)
        pay_data = resolved.data

        amount_msat = 50_000  # 50 sats

        # Pay
        result = sdk_node.lnurl_pay(
            glsdk.LnUrlPayRequest(
                data=pay_data,
                amount_msat=amount_msat,
                comment=None,
            )
        )

        assert isinstance(result, glsdk.LnUrlPayResult.ENDPOINT_SUCCESS)
        assert len(result.data.payment_preimage) == 64  # hex-encoded 32 bytes

        # Verify the LNURL server saw the callback
        assert len(lnurl_service.pay_callbacks) == 1
        assert lnurl_service.pay_callbacks[0]["amount_msat"] == amount_msat
    finally:
        sdk_node.disconnect()


def test_lnurl_pay_with_message_success_action(
    scheduler, nobody_id, clients, node_factory, bitcoind, lnurl_service
):
    """LNURL-pay with a message-type success action (LUD-09)."""
    lnurl_service.success_action = {
        "tag": "message",
        "message": "Thank you for your payment!",
    }

    relay = fund_and_connect(node_factory, bitcoind, lnurl_service)

    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
    s = c.signer().run_in_thread()

    relay_id = relay.info["id"]
    gl1.connect_peer(relay_id, f"127.0.0.1:{relay.daemon.port}")
    gl_addr = gl1.new_address().bech32
    bitcoind.rpc.sendtoaddress(gl_addr, 0.5)
    bitcoind.generate_block(1, wait_for_mempool=1)
    wait_for(lambda: len(gl1.list_funds().outputs) > 0)

    from pyln import grpc as clnpb

    gl1.fund_channel(
        bytes.fromhex(relay_id),
        clnpb.AmountOrAll(amount=clnpb.Amount(msat=CHANNEL_SATS * 1000)),
    )
    bitcoind.generate_block(6, wait_for_mempool=1)
    wait_for(
        lambda: any(
            ch.state == 2  # CHANNELD_NORMAL
            for ch in gl1.list_peer_channels().channels
        )
    )

    creds_bytes = c.creds().to_bytes()
    sdk_node = glsdk.Node(glsdk.Credentials.load(creds_bytes))

    try:
        resolved = sdk_node.resolve_lnurl(lnurl_service.pay_url)
        pay_data = resolved.data

        result = sdk_node.lnurl_pay(
            glsdk.LnUrlPayRequest(
                data=pay_data,
                amount_msat=50_000,
                comment=None,
            )
        )

        assert isinstance(result, glsdk.LnUrlPayResult.ENDPOINT_SUCCESS)
        sa = result.data.success_action
        assert sa is not None
        assert isinstance(sa, glsdk.SuccessActionProcessed.MESSAGE)
        assert sa.message == "Thank you for your payment!"
    finally:
        sdk_node.disconnect()
