"""Tests for Node methods like get_info, list_peers, list_peer_channels, list_funds.

These tests verify that the UniFFI bindings correctly expose the new Node methods
and their response types.
"""

import pytest
import glsdk


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


class TestResponseTypeFields:
    """Test that response types have expected fields."""

    def test_get_info_response_has_expected_fields(self):
        """Test GetInfoResponse has expected field attributes."""
        # Create instance to check fields
        response = glsdk.GetInfoResponse(
            id=b"\x02" * 33,
            alias="test-node",
            color=b"\xff\x00\x00",
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
        assert response.id == b"\x02" * 33
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
        peer = glsdk.Peer(
            id=b"\x03" * 33,
            connected=True,
            num_channels=1,
            netaddr=["127.0.0.1:9735"],
            remote_addr="192.168.1.1:9735",
            features=b"\x00",
        )
        assert peer.id == b"\x03" * 33
        assert peer.connected is True
        assert peer.num_channels == 1
        assert "127.0.0.1:9735" in peer.netaddr

    def test_peer_channel_record_has_expected_fields(self):
        """Test PeerChannel record has expected fields."""
        channel = glsdk.PeerChannel(
            peer_id=b"\x03" * 33,
            peer_connected=True,
            state=glsdk.ChannelState.CHANNELD_NORMAL,
            short_channel_id="123x1x0",
            channel_id=b"\x00" * 32,
            funding_txid=b"\xab" * 32,
            funding_outnum=0,
            to_us_msat=500000000,
            total_msat=1000000000,
            spendable_msat=400000000,
            receivable_msat=400000000,
        )
        assert channel.peer_id == b"\x03" * 33
        assert channel.peer_connected is True
        assert channel.state == glsdk.ChannelState.CHANNELD_NORMAL
        assert channel.total_msat == 1000000000

    def test_fund_output_record_has_expected_fields(self):
        """Test FundOutput record has expected fields."""
        output = glsdk.FundOutput(
            txid=b"\xab" * 32,
            output=0,
            amount_msat=1000000000,
            status=glsdk.OutputStatus.CONFIRMED,
            address="bcrt1qtest",
            blockheight=100,
        )
        assert output.txid == b"\xab" * 32
        assert output.amount_msat == 1000000000
        assert output.status == glsdk.OutputStatus.CONFIRMED

    def test_fund_channel_record_has_expected_fields(self):
        """Test FundChannel record has expected fields."""
        channel = glsdk.FundChannel(
            peer_id=b"\x03" * 33,
            our_amount_msat=500000000,
            amount_msat=1000000000,
            funding_txid=b"\xab" * 32,
            funding_output=0,
            connected=True,
            state=glsdk.ChannelState.CHANNELD_NORMAL,
            short_channel_id="123x1x0",
            channel_id=b"\x00" * 32,
        )
        assert channel.peer_id == b"\x03" * 33
        assert channel.our_amount_msat == 500000000
        assert channel.connected is True
