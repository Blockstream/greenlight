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

    def test_list_payments_request_exists(self):
        req = glsdk.ListPaymentsRequest(
            filters=None,
            from_timestamp=None,
            to_timestamp=None,
            include_failures=None,
            offset=None,
            limit=None,
        )
        assert req.filters is None
        assert req.include_failures is None

    def test_list_payments_request_with_filters(self):
        req = glsdk.ListPaymentsRequest(
            filters=[glsdk.PaymentTypeFilter.SENT],
            from_timestamp=1000,
            to_timestamp=2000,
            include_failures=True,
            offset=10,
            limit=50,
        )
        assert len(req.filters) == 1
        assert req.from_timestamp == 1000
        assert req.to_timestamp == 2000
        assert req.include_failures is True
        assert req.offset == 10
        assert req.limit == 50

    def test_payment_type_enum_exists(self):
        assert hasattr(glsdk, "PaymentType")
        assert hasattr(glsdk.PaymentType, "SENT")
        assert hasattr(glsdk.PaymentType, "RECEIVED")

    def test_payment_type_filter_enum_exists(self):
        assert hasattr(glsdk, "PaymentTypeFilter")
        assert hasattr(glsdk.PaymentTypeFilter, "SENT")
        assert hasattr(glsdk.PaymentTypeFilter, "RECEIVED")

    def test_payment_status_enum_exists(self):
        assert hasattr(glsdk, "PaymentStatus")
        assert hasattr(glsdk.PaymentStatus, "PENDING")
        assert hasattr(glsdk.PaymentStatus, "COMPLETE")
        assert hasattr(glsdk.PaymentStatus, "FAILED")

    def test_payment_record_received(self):
        payment = glsdk.Payment(
            id="00" * 32,
            payment_type=glsdk.PaymentType.RECEIVED,
            payment_time=1234567890,
            amount_msat=1000,
            fee_msat=0,
            status=glsdk.PaymentStatus.COMPLETE,
            description="coffee",
            bolt11="lnbc1...",
            preimage=b"\xab" * 32,
            destination=None,
        )
        assert payment.payment_type == glsdk.PaymentType.RECEIVED
        assert payment.status == glsdk.PaymentStatus.COMPLETE
        assert payment.fee_msat == 0
        assert payment.amount_msat == 1000
        assert payment.preimage == b"\xab" * 32
        assert payment.destination is None

    def test_payment_record_sent(self):
        payment = glsdk.Payment(
            id="00" * 32,
            payment_type=glsdk.PaymentType.SENT,
            payment_time=1234567890,
            amount_msat=1000,
            fee_msat=10,
            status=glsdk.PaymentStatus.COMPLETE,
            description="coffee",
            bolt11="lnbc1...",
            preimage=b"\x01" * 32,
            destination=b"\x02" * 33,
        )
        assert payment.payment_type == glsdk.PaymentType.SENT
        assert payment.amount_msat == 1000
        assert payment.fee_msat == 10
        assert payment.preimage == b"\x01" * 32
        assert payment.destination == b"\x02" * 33

    def test_payment_record_pending(self):
        payment = glsdk.Payment(
            id="00" * 32,
            payment_type=glsdk.PaymentType.RECEIVED,
            payment_time=1234567890,
            amount_msat=0,
            fee_msat=0,
            status=glsdk.PaymentStatus.PENDING,
            description=None,
            bolt11=None,
            preimage=None,
            destination=None,
        )
        assert payment.status == glsdk.PaymentStatus.PENDING
        assert payment.amount_msat == 0


class TestListPaymentsMethod:
    """Test list_payments() method."""

    def test_node_has_list_payments_method(self):
        assert hasattr(glsdk.Node, "list_payments")

    def test_list_payments_empty(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)
        req = glsdk.ListPaymentsRequest(
            filters=None, from_timestamp=None, to_timestamp=None,
            include_failures=None, offset=None, limit=None,
        )
        result = node.list_payments(req)
        assert isinstance(result, list)
        assert result == []
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

    def test_default_excludes_failures(self, scheduler, nobody_id):
        """Default list_payments excludes failed/expired, so unpaid invoices appear as Pending."""
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)

        label = str(uuid.uuid4())
        node.receive(label=label, description="tea", amount_msat=5_000_000)

        req = glsdk.ListPaymentsRequest(
            filters=None, from_timestamp=None, to_timestamp=None,
            include_failures=None, offset=None, limit=None,
        )
        result = node.list_payments(req)
        # Pending invoices should appear (they are not failures)
        matching = [p for p in result if p.status == glsdk.PaymentStatus.PENDING]
        assert len(matching) >= 1
        node.disconnect()

    def test_type_filter_received_only(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.register_or_recover(MNEMONIC, None, config)

        label = str(uuid.uuid4())
        node.receive(label=label, description="tea", amount_msat=5_000_000)

        req = glsdk.ListPaymentsRequest(
            filters=[glsdk.PaymentTypeFilter.RECEIVED],
            from_timestamp=None, to_timestamp=None,
            include_failures=None, offset=None, limit=None,
        )
        result = node.list_payments(req)
        for p in result:
            assert p.payment_type == glsdk.PaymentType.RECEIVED
        node.disconnect()
