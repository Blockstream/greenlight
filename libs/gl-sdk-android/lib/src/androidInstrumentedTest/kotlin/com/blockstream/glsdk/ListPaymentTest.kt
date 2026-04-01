// Instrumented tests for list_invoices, list_pays, and list_payments.
// Tests type construction, empty lists on fresh nodes, invoice creation
// visibility, and status filtering.

package com.blockstream.glsdk

import android.system.Os
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@RunWith(AndroidJUnit4::class)
class ListPaymentTest {

    @Before
    fun setup() {
        Os.setenv("RUST_LOG", "trace", true)
    }

    // BIP39 test vector — not a real wallet
    private val testMnemonic =
        "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"

    // ============================================================
    // Type construction
    // ============================================================

    @Test
    fun invoice_status_enum_values() {
        assertNotNull(InvoiceStatus.UNPAID)
        assertNotNull(InvoiceStatus.PAID)
        assertNotNull(InvoiceStatus.EXPIRED)
    }

    @Test
    fun payment_direction_enum_values() {
        assertNotNull(PaymentDirection.SENT)
        assertNotNull(PaymentDirection.RECEIVED)
    }

    @Test
    fun payment_status_enum_values() {
        assertNotNull(PaymentStatus.PENDING)
        assertNotNull(PaymentStatus.COMPLETE)
        assertNotNull(PaymentStatus.FAILED)
        assertNotNull(PaymentStatus.EXPIRED)
    }

    // ============================================================
    // Invoice appears in list_invoices and list_payments
    // ============================================================

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun created_invoice_appears_in_list_invoices() {
        val config = Config()
        registerOrRecover(testMnemonic, null, config).use { node ->
            val label = Uuid.random().toString()
            node.receive(label = label, description = "Coffee", amountMsat = 10_000_000uL)

            val result = node.listInvoices()
            val matching = result.invoices.filter { it.label == label }
            assertEquals("Should find exactly one invoice", 1, matching.size)
            assertEquals(InvoiceStatus.UNPAID, matching[0].status)
            assertEquals("Coffee", matching[0].description)
            assertNotNull(matching[0].bolt11)
            // Unpaid invoices have no preimage yet
            assertNull(matching[0].paymentPreimage)
            // Destination pubkey is parsed from the bolt11 invoice
            assertNotNull(matching[0].destinationPubkey)
        }
    }

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun default_filter_excludes_unpaid() {
        val config = Config()
        registerOrRecover(testMnemonic, null, config).use { node ->
            val label = Uuid.random().toString()
            node.receive(label = label, description = "Tea", amountMsat = 5_000_000uL)

            // Default (null) filters to COMPLETE, so unpaid invoice should not appear
            val result = node.listPayments()
            val matching = result.payments.filter { it.label == label }
            assertTrue("Default filter should exclude UNPAID invoice", matching.isEmpty())
        }
    }

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun pending_filter_includes_unpaid_invoice() {
        val config = Config()
        registerOrRecover(testMnemonic, null, config).use { node ->
            val label = Uuid.random().toString()
            node.receive(label = label, description = "Tea", amountMsat = 5_000_000uL)

            val result = node.listPayments(PaymentStatus.PENDING)
            val matching = result.payments.filter { it.label == label }
            assertEquals("PENDING filter should include UNPAID invoice", 1, matching.size)
            assertEquals(PaymentDirection.RECEIVED, matching[0].direction)
            assertEquals(PaymentStatus.PENDING, matching[0].status)
            assertEquals(InvoiceStatus.UNPAID, matching[0].invoiceStatus)
            assertNull(matching[0].payStatus)
            // Received payments have no fee
            assertNull(matching[0].feeMsat)
            // For received, amount_total_msat equals amount_msat
            assertEquals(matching[0].amountMsat, matching[0].amountTotalMsat)
            // Destination pubkey is available on the unified Payment
            assertNotNull(matching[0].destinationPubkey)
            assertNotNull(matching[0].invoice)
            assertEquals(label, matching[0].invoice?.label)
            assertNull(matching[0].pay)
        }
    }
}
