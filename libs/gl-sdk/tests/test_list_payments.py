"""Integration tests for list_invoices, list_pays, and list_payments."""

import uuid

import pytest
import glsdk
from gltesting.fixtures import *


MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon about"
)


class TestInvoiceTypes:
    """Test that invoice-related types exist in the bindings."""

    def test_invoice_status_enum_exists(self):
        assert hasattr(glsdk, "InvoiceStatus")
        assert hasattr(glsdk.InvoiceStatus, "UNPAID")
        assert hasattr(glsdk.InvoiceStatus, "PAID")
        assert hasattr(glsdk.InvoiceStatus, "EXPIRED")

    def test_invoice_record_exists(self):
        invoice = glsdk.Invoice(
            label="test",
            description="coffee",
            payment_hash=b"\x00" * 32,
            status=glsdk.InvoiceStatus.UNPAID,
            amount_msat=1000,
            amount_received_msat=None,
            bolt11="lnbc1...",
            bolt12=None,
            paid_at=None,
            expires_at=1234567890,
            payment_preimage=None,
            destination_pubkey=None,
        )
        assert invoice.label == "test"
        assert invoice.status == glsdk.InvoiceStatus.UNPAID
        assert invoice.expires_at == 1234567890

    def test_invoice_record_with_preimage(self):
        invoice = glsdk.Invoice(
            label="paid",
            description="coffee",
            payment_hash=b"\x00" * 32,
            status=glsdk.InvoiceStatus.PAID,
            amount_msat=1000,
            amount_received_msat=1000,
            bolt11="lnbc1...",
            bolt12=None,
            paid_at=1234567890,
            expires_at=1234567900,
            payment_preimage=b"\xab" * 32,
            destination_pubkey=b"\x02" * 33,
        )
        assert invoice.payment_preimage == b"\xab" * 32
        assert invoice.destination_pubkey == b"\x02" * 33

    def test_list_invoices_response_exists(self):
        response = glsdk.ListInvoicesResponse(invoices=[])
        assert response.invoices == []


class TestListInvoicesMethod:
    """Test list_invoices() method."""

    def test_node_has_list_invoices_method(self):
        assert hasattr(glsdk.Node, "list_invoices")

    def test_list_invoices_empty(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)
        result = node.list_invoices(
            label=None, invstring=None, payment_hash=None,
            offer_id=None, index=None, start=None, limit=None,
        )
        assert isinstance(result, glsdk.ListInvoicesResponse)
        assert result.invoices == []
        node.disconnect()


class TestPayTypes:
    """Test that pay-related types exist in the bindings."""

    def test_pay_record_exists(self):
        pay = glsdk.Pay(
            payment_hash=b"\x00" * 32,
            status=glsdk.PayStatus.COMPLETE,
            destination_pubkey=b"\x02" * 33,
            amount_msat=1000,
            amount_sent_msat=1010,
            label="test",
            bolt11="lnbc1...",
            description="coffee",
            bolt12=None,
            preimage=b"\x01" * 32,
            created_at=1234567890,
            completed_at=1234567900,
            number_of_parts=1,
        )
        assert pay.status == glsdk.PayStatus.COMPLETE
        assert pay.created_at == 1234567890
        assert pay.destination_pubkey == b"\x02" * 33

    def test_list_pays_response_exists(self):
        response = glsdk.ListPaysResponse(pays=[])
        assert response.pays == []


class TestListPaysMethod:
    """Test list_pays() method."""

    def test_node_has_list_pays_method(self):
        assert hasattr(glsdk.Node, "list_pays")

    def test_list_pays_empty(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)
        result = node.list_pays(
            bolt11=None, payment_hash=None, status=None,
            index=None, start=None, limit=None,
        )
        assert isinstance(result, glsdk.ListPaysResponse)
        assert result.pays == []
        node.disconnect()


class TestPaymentTypes:
    """Test that unified payment types exist in the bindings."""

    def test_payment_direction_enum_exists(self):
        assert hasattr(glsdk, "PaymentDirection")
        assert hasattr(glsdk.PaymentDirection, "SENT")
        assert hasattr(glsdk.PaymentDirection, "RECEIVED")

    def test_payment_status_enum_exists(self):
        assert hasattr(glsdk, "PaymentStatus")
        assert hasattr(glsdk.PaymentStatus, "PENDING")
        assert hasattr(glsdk.PaymentStatus, "COMPLETE")
        assert hasattr(glsdk.PaymentStatus, "FAILED")
        assert hasattr(glsdk.PaymentStatus, "EXPIRED")

    def test_payment_record_with_invoice(self):
        inv = glsdk.Invoice(
            label="test",
            description="coffee",
            payment_hash=b"\x00" * 32,
            status=glsdk.InvoiceStatus.PAID,
            amount_msat=1000,
            amount_received_msat=1000,
            bolt11="lnbc1...",
            bolt12=None,
            paid_at=1234567890,
            expires_at=1234567900,
            payment_preimage=b"\xab" * 32,
            destination_pubkey=b"\x02" * 33,
        )
        payment = glsdk.Payment(
            payment_hash=b"\x00" * 32,
            direction=glsdk.PaymentDirection.RECEIVED,
            status=glsdk.PaymentStatus.COMPLETE,
            invoice_status=glsdk.InvoiceStatus.PAID,
            pay_status=None,
            amount_msat=1000,
            fee_msat=None,
            amount_total_msat=1000,
            preimage=b"\xab" * 32,
            destination_pubkey=b"\x02" * 33,
            description="coffee",
            bolt11="lnbc1...",
            label="test",
            created_at=1234567890,
            invoice=inv,
            pay=None,
        )
        assert payment.direction == glsdk.PaymentDirection.RECEIVED
        assert payment.status == glsdk.PaymentStatus.COMPLETE
        assert payment.invoice_status == glsdk.InvoiceStatus.PAID
        assert payment.pay_status is None
        assert payment.fee_msat is None
        assert payment.amount_total_msat == payment.amount_msat
        assert payment.preimage == b"\xab" * 32
        assert payment.destination_pubkey == b"\x02" * 33
        assert payment.invoice is not None
        assert payment.invoice.label == "test"
        assert payment.pay is None

    def test_payment_record_with_pay(self):
        p = glsdk.Pay(
            payment_hash=b"\x00" * 32,
            status=glsdk.PayStatus.COMPLETE,
            destination_pubkey=b"\x02" * 33,
            amount_msat=1000,
            amount_sent_msat=1010,
            label="test",
            bolt11="lnbc1...",
            description="coffee",
            bolt12=None,
            preimage=b"\x01" * 32,
            created_at=1234567890,
            completed_at=1234567900,
            number_of_parts=1,
        )
        payment = glsdk.Payment(
            payment_hash=b"\x00" * 32,
            direction=glsdk.PaymentDirection.SENT,
            status=glsdk.PaymentStatus.COMPLETE,
            invoice_status=None,
            pay_status=glsdk.PayStatus.COMPLETE,
            amount_msat=1000,
            fee_msat=10,
            amount_total_msat=1010,
            preimage=b"\x01" * 32,
            destination_pubkey=b"\x02" * 33,
            description="coffee",
            bolt11="lnbc1...",
            label="test",
            created_at=1234567890,
            invoice=None,
            pay=p,
        )
        assert payment.direction == glsdk.PaymentDirection.SENT
        assert payment.amount_msat == 1000
        assert payment.fee_msat == 10
        assert payment.amount_total_msat == 1010
        assert payment.preimage == b"\x01" * 32
        assert payment.destination_pubkey == b"\x02" * 33
        assert payment.invoice is None
        assert payment.pay is not None
        assert payment.pay.created_at == 1234567890

    def test_list_payments_response_exists(self):
        response = glsdk.ListPaymentsResponse(payments=[])
        assert response.payments == []


class TestListPaymentsMethod:
    """Test list_payments() method."""

    def test_node_has_list_payments_method(self):
        assert hasattr(glsdk.Node, "list_payments")

    def test_list_payments_empty(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)
        result = node.list_payments(status=None)
        assert isinstance(result, glsdk.ListPaymentsResponse)
        assert result.payments == []
        node.disconnect()


class TestListInvoicesIntegration:
    """Test that created invoices appear in list_invoices and list_payments."""

    def test_created_invoice_appears_in_list_invoices(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)

        label = str(uuid.uuid4())
        node.receive(label=label, description="coffee", amount_msat=10_000_000)

        result = node.list_invoices(
            label=None, invstring=None, payment_hash=None,
            offer_id=None, index=None, start=None, limit=None,
        )
        assert len(result.invoices) >= 1
        matching = [i for i in result.invoices if i.label == label]
        assert len(matching) == 1
        assert matching[0].status == glsdk.InvoiceStatus.UNPAID
        assert matching[0].description == "coffee"
        assert matching[0].bolt11 is not None
        assert matching[0].destination_pubkey is not None
        node.disconnect()

    def test_default_filter_excludes_unpaid(self, scheduler, nobody_id):
        """Default (None) filters to COMPLETE, so an unpaid invoice should not appear."""
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)

        label = str(uuid.uuid4())
        node.receive(label=label, description="tea", amount_msat=5_000_000)

        result = node.list_payments(status=None)
        matching = [p for p in result.payments if p.label == label]
        assert len(matching) == 0
        node.disconnect()

    def test_pending_filter_includes_unpaid_invoice(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)

        label = str(uuid.uuid4())
        node.receive(label=label, description="tea", amount_msat=5_000_000)

        result = node.list_payments(status=glsdk.PaymentStatus.PENDING)
        assert len(result.payments) >= 1
        matching = [p for p in result.payments if p.label == label]
        assert len(matching) == 1
        assert matching[0].direction == glsdk.PaymentDirection.RECEIVED
        assert matching[0].status == glsdk.PaymentStatus.PENDING
        assert matching[0].invoice_status == glsdk.InvoiceStatus.UNPAID
        assert matching[0].pay_status is None
        assert matching[0].fee_msat is None
        assert matching[0].invoice is not None
        assert matching[0].invoice.label == label
        assert matching[0].pay is None
        node.disconnect()

    def test_list_payments_explicit_status_filter(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)

        label = str(uuid.uuid4())
        node.receive(label=label, description="filtered", amount_msat=1_000_000)

        # Explicit COMPLETE — same as default, should not include UNPAID invoice
        result = node.list_payments(status=glsdk.PaymentStatus.COMPLETE)
        matching = [p for p in result.payments if p.label == label]
        assert len(matching) == 0

        # Explicit EXPIRED — should not include UNPAID invoice either
        result = node.list_payments(status=glsdk.PaymentStatus.EXPIRED)
        matching = [p for p in result.payments if p.label == label]
        assert len(matching) == 0

        # Explicit PENDING — should include it
        result = node.list_payments(status=glsdk.PaymentStatus.PENDING)
        matching = [p for p in result.payments if p.label == label]
        assert len(matching) == 1
        node.disconnect()
