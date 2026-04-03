// Kotlin extension functions for Node that provide default parameter values.
// The UniFFI-generated bindings require all parameters explicitly;
// these extensions give Kotlin consumers idiomatic optional-parameter APIs.

package com.blockstream.glsdk

@Throws(Exception::class)
public fun Node.listInvoices(
    label: String? = null,
    invstring: String? = null,
    paymentHash: ByteArray? = null,
    offerId: String? = null,
    index: ListIndex? = null,
    start: ULong? = null,
    limit: UInt? = null,
): ListInvoicesResponse = listInvoices(label, invstring, paymentHash, offerId, index, start, limit)

@Throws(Exception::class)
public fun Node.listPays(
    bolt11: String? = null,
    paymentHash: ByteArray? = null,
    status: PayStatus? = null,
    index: ListIndex? = null,
    start: ULong? = null,
    limit: UInt? = null,
): ListPaysResponse = listPays(bolt11, paymentHash, status, index, start, limit)

@Throws(Exception::class)
public fun Node.listPayments(
    filters: List<PaymentTypeFilter>? = null,
    fromTimestamp: ULong? = null,
    toTimestamp: ULong? = null,
    includeFailures: Boolean? = null,
    offset: UInt? = null,
    limit: UInt? = null,
): List<Payment> = listPayments(ListPaymentsRequest(filters, fromTimestamp, toTimestamp, includeFailures, offset, limit))
