"""Tests for Node methods like get_info, list_peers, list_peer_channels, list_funds.

These tests verify that the UniFFI bindings correctly expose the new Node methods
and their response types.
"""

import json

import pytest
import glsdk
from gltesting.fixtures import *


MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon about"
)


class TestResponseTypes:
    """Test that response types are properly defined in the bindings."""

    def test_get_info_response_type_exists(self):
        """Test that GetInfoResponse type is available."""
        assert hasattr(glsdk, "GetInfoResponse")

    def test_list_peers_response_type_exists(self):
        """Test that ListPeersResponse type is available."""
        assert hasattr(glsdk, "ListPeersResponse")

    def test_list_peer_channels_response_type_exists(self):
        """Test that ListPeerChannelsResponse type is available."""
        assert hasattr(glsdk, "ListPeerChannelsResponse")

    def test_list_funds_response_type_exists(self):
        """Test that ListFundsResponse type is available."""
        assert hasattr(glsdk, "ListFundsResponse")

    def test_peer_type_exists(self):
        """Test that Peer type is available."""
        assert hasattr(glsdk, "Peer")

    def test_peer_channel_type_exists(self):
        """Test that PeerChannel type is available."""
        assert hasattr(glsdk, "PeerChannel")

    def test_fund_output_type_exists(self):
        """Test that FundOutput type is available."""
        assert hasattr(glsdk, "FundOutput")

    def test_fund_channel_type_exists(self):
        """Test that FundChannel type is available."""
        assert hasattr(glsdk, "FundChannel")

    def test_channel_state_enum_exists(self):
        """Test that ChannelState enum is available."""
        assert hasattr(glsdk, "ChannelState")
        # Check some enum variants
        assert hasattr(glsdk.ChannelState, "OPENINGD")
        assert hasattr(glsdk.ChannelState, "CHANNELD_NORMAL")
        assert hasattr(glsdk.ChannelState, "ONCHAIN")

    def test_output_status_enum_exists(self):
        """Test that OutputStatus enum is available."""
        assert hasattr(glsdk, "OutputStatus")
        # Check enum variants
        assert hasattr(glsdk.OutputStatus, "UNCONFIRMED")
        assert hasattr(glsdk.OutputStatus, "CONFIRMED")
        assert hasattr(glsdk.OutputStatus, "SPENT")


class TestNodeMethods:
    """Test that Node has the expected methods."""

    def test_node_has_get_info_method(self):
        """Test that Node class has get_info method."""
        assert hasattr(glsdk.Node, "get_info")

    def test_node_has_list_peers_method(self):
        """Test that Node class has list_peers method."""
        assert hasattr(glsdk.Node, "list_peers")

    def test_node_has_list_peer_channels_method(self):
        """Test that Node class has list_peer_channels method."""
        assert hasattr(glsdk.Node, "list_peer_channels")

    def test_node_has_list_funds_method(self):
        """Test that Node class has list_funds method."""
        assert hasattr(glsdk.Node, "list_funds")


class TestSendResponseFields:
    """Test that SendResponse includes payment_hash and destination_pubkey."""

    def test_send_response_has_payment_hash(self):
        preimage_hex = "01" * 32
        payment_hash_hex = "00" * 32
        destination_hex = "02" * 33
        response = glsdk.SendResponse(
            status=glsdk.PayStatus.COMPLETE,
            preimage=preimage_hex,
            payment_hash=payment_hash_hex,
            destination_pubkey=destination_hex,
            amount_msat=1000,
            amount_sent_msat=1010,
            parts=1,
        )
        assert response.payment_hash == payment_hash_hex
        assert response.destination_pubkey == destination_hex

    def test_send_response_destination_pubkey_is_optional(self):
        response = glsdk.SendResponse(
            status=glsdk.PayStatus.COMPLETE,
            preimage="01" * 32,
            payment_hash="00" * 32,
            destination_pubkey=None,
            amount_msat=1000,
            amount_sent_msat=1010,
            parts=1,
        )
        assert response.destination_pubkey is None


class TestResponseTypeFields:
    """Test that response types have expected fields."""

    def test_get_info_response_has_expected_fields(self):
        """Test GetInfoResponse has expected field attributes."""
        node_id_hex = "02" * 33
        response = glsdk.GetInfoResponse(
            id=node_id_hex,
            alias="test-node",
            color="ff0000",
            num_peers=0,
            num_pending_channels=0,
            num_active_channels=0,
            num_inactive_channels=0,
            version="v24.11",
            lightning_dir="/tmp/lightning",
            blockheight=100,
            network="regtest",
            fees_collected_msat=0,
        )
        assert response.id == node_id_hex
        assert response.alias == "test-node"
        assert response.network == "regtest"
        assert response.blockheight == 100

    def test_list_peers_response_has_peers_field(self):
        """Test ListPeersResponse has peers field."""
        response = glsdk.ListPeersResponse(peers=[])
        assert response.peers == []

    def test_list_peer_channels_response_has_channels_field(self):
        """Test ListPeerChannelsResponse has channels field."""
        response = glsdk.ListPeerChannelsResponse(channels=[])
        assert response.channels == []

    def test_list_funds_response_has_expected_fields(self):
        """Test ListFundsResponse has outputs and channels fields."""
        response = glsdk.ListFundsResponse(outputs=[], channels=[])
        assert response.outputs == []
        assert response.channels == []

    def test_peer_record_has_expected_fields(self):
        """Test Peer record has expected fields."""
        peer_id_hex = "03" * 33
        peer = glsdk.Peer(
            id=peer_id_hex,
            connected=True,
            num_channels=1,
            netaddr=["127.0.0.1:9735"],
            remote_addr="192.168.1.1:9735",
            features=b"\x00",
        )
        assert peer.id == peer_id_hex
        assert peer.connected is True
        assert peer.num_channels == 1
        assert "127.0.0.1:9735" in peer.netaddr

    def test_peer_channel_record_has_expected_fields(self):
        """Test PeerChannel record has expected fields."""
        peer_id_hex = "03" * 33
        channel_id_hex = "00" * 32
        funding_txid_hex = "ab" * 32
        channel = glsdk.PeerChannel(
            peer_id=peer_id_hex,
            peer_connected=True,
            state=glsdk.ChannelState.CHANNELD_NORMAL,
            short_channel_id="123x1x0",
            channel_id=channel_id_hex,
            funding_txid=funding_txid_hex,
            funding_outnum=0,
            to_us_msat=500000000,
            total_msat=1000000000,
            spendable_msat=400000000,
            receivable_msat=400000000,
            closer=None,
            status=[],
        )
        assert channel.peer_id == peer_id_hex
        assert channel.peer_connected is True
        assert channel.state == glsdk.ChannelState.CHANNELD_NORMAL
        assert channel.total_msat == 1000000000
        assert channel.closer is None
        assert channel.status == []

    def test_fund_output_record_has_expected_fields(self):
        """Test FundOutput record has expected fields."""
        txid_hex = "ab" * 32
        output = glsdk.FundOutput(
            txid=txid_hex,
            output=0,
            amount_msat=1000000000,
            status=glsdk.OutputStatus.CONFIRMED,
            address="bcrt1qtest",
            blockheight=100,
            reserved=False,
        )
        assert output.txid == txid_hex
        assert output.amount_msat == 1000000000
        assert output.status == glsdk.OutputStatus.CONFIRMED
        assert output.reserved is False

    def test_fund_channel_record_has_expected_fields(self):
        """Test FundChannel record has expected fields."""
        peer_id_hex = "03" * 33
        funding_txid_hex = "ab" * 32
        channel_id_hex = "00" * 32
        channel = glsdk.FundChannel(
            peer_id=peer_id_hex,
            our_amount_msat=500000000,
            amount_msat=1000000000,
            funding_txid=funding_txid_hex,
            funding_output=0,
            connected=True,
            state=glsdk.ChannelState.CHANNELD_NORMAL,
            short_channel_id="123x1x0",
            channel_id=channel_id_hex,
        )
        assert channel.peer_id == peer_id_hex
        assert channel.our_amount_msat == 500000000
        assert channel.connected is True


class TestNodeStateType:
    """Test that NodeState type is properly defined in the bindings."""

    def test_node_state_type_exists(self):
        assert hasattr(glsdk, "NodeState")

    def test_node_state_record_has_expected_fields(self):
        node_id_hex = "02" * 33
        peer_id_hex = "03" * 33
        state = glsdk.NodeState(
            id=node_id_hex,
            block_height=800000,
            network="regtest",
            version="v24.11",
            alias="test-node",
            color="ff0000",
            num_active_channels=2,
            num_pending_channels=1,
            num_inactive_channels=0,
            channels_balance_msat=500_000_000,
            max_payable_msat=450_000_000,
            total_channel_capacity_msat=1_000_000_000,
            max_chan_reserve_msat=50_000_000,
            onchain_balance_msat=100_000_000,
            unconfirmed_onchain_balance_msat=50_000_000,
            immature_onchain_balance_msat=0,
            pending_onchain_balance_msat=0,
            max_receivable_single_payment_msat=400_000_000,
            total_inbound_liquidity_msat=800_000_000,
            connected_channel_peers=[peer_id_hex],
            utxos=[],
            total_onchain_msat=150_000_000,
            total_balance_msat=650_000_000,
            spendable_balance_msat=550_000_000,
        )
        assert state.id == node_id_hex
        assert state.block_height == 800000
        assert state.network == "regtest"
        assert state.version == "v24.11"
        assert state.channels_balance_msat == 500_000_000
        assert state.max_payable_msat == 450_000_000
        assert state.total_channel_capacity_msat == 1_000_000_000
        assert state.max_chan_reserve_msat == 50_000_000
        assert state.onchain_balance_msat == 100_000_000
        assert state.unconfirmed_onchain_balance_msat == 50_000_000
        assert state.immature_onchain_balance_msat == 0
        assert state.total_onchain_msat == 150_000_000
        assert state.total_balance_msat == 650_000_000
        assert state.spendable_balance_msat == 550_000_000
        assert len(state.connected_channel_peers) == 1
        assert state.connected_channel_peers[0] == peer_id_hex
        assert state.utxos == []


class TestNodeStateMethod:
    """Test node_state() integration."""

    def test_node_has_node_state_method(self):
        assert hasattr(glsdk.Node, "node_state")

    def test_node_state_returns_valid_snapshot(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register_or_recover(MNEMONIC, None)
        state = node.node_state()
        assert isinstance(state, glsdk.NodeState)
        # Node id is a lowercase hex pubkey (33 bytes → 66 chars).
        assert isinstance(state.id, str)
        assert len(state.id) == 66
        assert state.block_height > 0
        assert state.network == "regtest"
        assert state.version != ""
        assert state.channels_balance_msat == 0
        assert state.max_payable_msat == 0
        assert state.total_channel_capacity_msat == 0
        assert state.onchain_balance_msat == 0
        assert state.total_inbound_liquidity_msat == 0
        node.disconnect()


class TestGenerateDiagnosticData:
    """Test generate_diagnostic_data() on a freshly registered node."""

    def test_node_has_generate_diagnostic_data_method(self):
        assert hasattr(glsdk.Node, "generate_diagnostic_data")

    def test_generate_diagnostic_data_returns_well_formed_envelope(
        self, scheduler, nobody_id
    ):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register_or_recover(MNEMONIC, None)

        blob = node.generate_diagnostic_data()
        assert isinstance(blob, str)

        parsed = json.loads(blob)
        assert isinstance(parsed["timestamp"], int)
        assert parsed["timestamp"] > 0

        assert "sdk" in parsed
        assert isinstance(parsed["sdk"]["version"], str)
        assert parsed["sdk"]["version"] != ""
        # node_state should serialize as a JSON object on a healthy node.
        assert isinstance(parsed["sdk"]["node_state"], dict)
        assert parsed["sdk"]["node_state"]["network"] == "regtest"

        assert "node" in parsed
        for key in ("getinfo", "listpeerchannels", "listfunds"):
            assert key in parsed["node"], f"missing node section: {key}"
        # Payment/invoice history is intentionally excluded from the dump.
        assert "listpays" not in parsed["node"]
        assert "listinvoices" not in parsed["node"]

        # Healthy node: getinfo should serialize as an object, not as
        # {"error": "..."}.
        getinfo = parsed["node"]["getinfo"]
        assert isinstance(getinfo, dict)
        assert "error" not in getinfo
        assert isinstance(getinfo["id"], str)
        assert len(getinfo["id"]) == 66

        node.disconnect()
