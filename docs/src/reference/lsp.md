# Lightning Service Provider integration

Greenlight includes a client for the JIT LSP protocol. In particular
the `gl-plugin` implements support for LSP JIT fees.  These are fees
leveraged as payment for a channel being opened, by forwarding a
reduced amount, i.e., holding back a non-LN native fee. This is
implemented on the LSP by intercepting and modifying the payload for
the HTLC being forwarded, but this also causes the onion payload to
mismatch the HTLC parameters (forwarded amount and total amount the
sender intended to send). This would normally cause the recipient to
fail the payment, however they are aware of this fee being leveraged,
so we need to get the onion payload to match the HTLC and the
corresponding invoice.

The opt-in currently is determined by a matching invoice being
present (payment_hash) and the sender values in the onion not matching
the invoice. Hence the LSP needs to store a reduced invoice a the
recipient node, but give out the original invoice to the prospective
sender.

## Overview
!!! todo 

## Caveats

 1. The LSP has to have the ability to know the amount that the
   destination expects in order to satisfy that expectation. In order
   to do so we need to have a couple of values match up:
   
     1. Each HTLC value has to match or exceed the `amt_to_forward`
        field in the destination's onion payload.
	  
     2. The `total_msat` value at the end of the `payment_secret` onion
        payload field has to match the sum of `amt_to_forward` values
        communicated in the HTLCs (on overpayment we reject, since the
        sender should have matched the expected amount exactly).
	  
    The first is simple to patch in `gl-plugin` since we just need to
    inspect the two values and adjust the one in the onion payload to
    match. The latter is not, but we can glimpse the expected value by
    looking up the invoice that is being paid. That does not work in
    the following two cases:
	
	  - Spontaneous payments: don't have a matching invoice, and we
        can't decrypt the payload on the LSP to learn the amount the
        sender intended to deliver. Since the LSP uses routehints in
        invoices the sender would likely just not find a path.
	  - Amount-less invoices: same as above.
