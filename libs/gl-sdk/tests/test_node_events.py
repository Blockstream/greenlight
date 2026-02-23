"""Tests for Node event streaming functionality.

These tests verify that the UniFFI bindings correctly expose the NodeEventStream
and related types.
"""

import pytest
import glsdk


class TestEventStreamTypes:
    """Test that event streaming types are properly defined in the bindings."""

    def test_node_event_stream_type_exists(self):
        """Test that NodeEventStream type is available."""
        assert hasattr(glsdk, "NodeEventStream")

    def test_node_event_type_exists(self):
        """Test that NodeEvent type is available."""
        assert hasattr(glsdk, "NodeEvent")

    def test_invoice_paid_event_type_exists(self):
        """Test that InvoicePaidEvent type is available."""
        assert hasattr(glsdk, "InvoicePaidEvent")

    def test_node_event_has_invoice_paid_variant(self):
        """Test that NodeEvent has INVOICE_PAID variant."""
        assert hasattr(glsdk.NodeEvent, "INVOICE_PAID")

    def test_node_event_has_unknown_variant(self):
        """Test that NodeEvent has UNKNOWN variant."""
        assert hasattr(glsdk.NodeEvent, "UNKNOWN")

    def test_node_has_stream_node_events_method(self):
        """Test that Node class has stream_node_events method."""
        assert hasattr(glsdk.Node, "stream_node_events")

    def test_node_event_stream_has_next_method(self):
        """Test that NodeEventStream class has next method."""
        assert hasattr(glsdk.NodeEventStream, "next")


class TestInvoicePaidEventFields:
    """Test that InvoicePaidEvent has expected fields."""

    def test_invoice_paid_event_can_be_constructed(self):
        """Test InvoicePaidEvent can be constructed with all fields."""
        event = glsdk.InvoicePaidEvent(
            payment_hash=b"\x00" * 32,
            bolt11="lnbcrt1...",
            preimage=b"\x01" * 32,
            label="test-invoice",
            amount_msat=100000,
        )
        assert event.payment_hash == b"\x00" * 32
        assert event.bolt11 == "lnbcrt1..."
        assert event.preimage == b"\x01" * 32
        assert event.label == "test-invoice"
        assert event.amount_msat == 100000

    def test_invoice_paid_event_str(self):
        """Test InvoicePaidEvent has a reasonable string representation."""
        event = glsdk.InvoicePaidEvent(
            payment_hash=b"\x00" * 32,
            bolt11="lnbcrt1...",
            preimage=b"\x01" * 32,
            label="test-invoice",
            amount_msat=100000,
        )
        str_repr = str(event)
        assert "InvoicePaidEvent" in str_repr
        assert "test-invoice" in str_repr


class TestNodeEventVariants:
    """Test NodeEvent enum variants and their behavior."""

    def test_invoice_paid_variant_construction(self):
        """Test that INVOICE_PAID variant can be constructed."""
        details = glsdk.InvoicePaidEvent(
            payment_hash=b"\x00" * 32,
            bolt11="lnbcrt1...",
            preimage=b"\x01" * 32,
            label="test-invoice",
            amount_msat=100000,
        )
        event = glsdk.NodeEvent.INVOICE_PAID(details=details)
        assert event.details == details

    def test_unknown_variant_construction(self):
        """Test that UNKNOWN variant can be constructed."""
        event = glsdk.NodeEvent.UNKNOWN()
        assert event is not None

    def test_invoice_paid_is_invoice_paid(self):
        """Test is_invoice_paid() method on INVOICE_PAID variant."""
        details = glsdk.InvoicePaidEvent(
            payment_hash=b"\x00" * 32,
            bolt11="lnbcrt1...",
            preimage=b"\x01" * 32,
            label="test-invoice",
            amount_msat=100000,
        )
        event = glsdk.NodeEvent.INVOICE_PAID(details=details)
        assert event.is_invoice_paid()
        assert not event.is_unknown()

    def test_unknown_is_unknown(self):
        """Test is_unknown() method on UNKNOWN variant."""
        event = glsdk.NodeEvent.UNKNOWN()
        assert event.is_unknown()
        assert not event.is_invoice_paid()



