// Instrumented tests for list_invoices, list_pays, and list_payments.
// Tests type construction, empty lists on fresh nodes, invoice creation
// visibility, and type filtering.

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
    fun payment_type_enum_values() {
        assertNotNull(PaymentType.SENT)
        assertNotNull(PaymentType.RECEIVED)
    }

    @Test
    fun payment_type_filter_enum_values() {
        assertNotNull(PaymentTypeFilter.SENT)
        assertNotNull(PaymentTypeFilter.RECEIVED)
    }

    @Test
    fun payment_status_enum_values() {
        assertNotNull(PaymentStatus.PENDING)
        assertNotNull(PaymentStatus.COMPLETE)
        assertNotNull(PaymentStatus.FAILED)
    }

    // ============================================================
    // Invoice appears in list_invoices and list_payments
    // ============================================================

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun created_invoice_appears_in_list_invoices() {
        val config = Config()
        NodeBuilder(config).registerOrRecover(testMnemonic, null).use { node ->
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
    fun unpaid_invoices_excluded() {
        val config = Config()
        NodeBuilder(config).registerOrRecover(testMnemonic, null).use { node ->
            val label = Uuid.random().toString()
            node.receive(label = label, description = "Tea", amountMsat = 5_000_000uL)

            // listPayments returns paid invoices only on the received
            // side. The unpaid invoice we just created must not appear.
            val result = node.listPayments()
            for (p in result) {
                if (p.paymentType == PaymentType.RECEIVED) {
                    assertEquals(
                        "non-Complete received payment: $p",
                        PaymentStatus.COMPLETE,
                        p.status,
                    )
                }
            }
        }
    }

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun type_filter_received_only() {
        val config = Config()
        NodeBuilder(config).registerOrRecover(testMnemonic, null).use { node ->
            val label = Uuid.random().toString()
            node.receive(label = label, description = "Tea", amountMsat = 5_000_000uL)

            val result = node.listPayments(filters = listOf(PaymentTypeFilter.RECEIVED))
            for (p in result) {
                assertEquals(PaymentType.RECEIVED, p.paymentType)
            }
        }
    }
}
