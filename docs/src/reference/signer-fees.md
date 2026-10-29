# Signer Fee Budgets

The signer limits routing fees on outgoing payments and miner fees on channel
transactions. Set these limits when starting or embedding the signer.

## Set fee limits

| CLI flag | Default | Controls |
|----------|---------|----------|
| `--payment-fee-exempt-msat` | 1,000,000 msat (1,000 sat) | Routing fee allowance for small payments. |
| `--max-payment-fee-ppm` | 5,000 ppm (0.5%) | Routing fee allowance as a percentage of the payment amount. |
| `--max-commitment-fee-sat` | 100,000 sat | Maximum miner fee per new commitment or signed local HTLC input. |
| `--max-mutual-close-fee-sat` | 50,000 sat | Maximum miner fee per cooperative channel close. |

For example, to set the routing percentage to 1% while keeping the other defaults:

```sh
glcli signer run --max-payment-fee-ppm 10000
```

In Rust, set `SignerConfig::fee_budget_limits`:

```rust
use gl_client::signer::{FeeBudgetLimits, SignerConfig};

let config = SignerConfig {
    fee_budget_limits: FeeBudgetLimits {
        max_payment_fee_ppm: 10_000, // 1%
        ..Default::default()
    },
    ..Default::default()
};
```

## Routing fees

The signer allows the larger of the fixed allowance and the percentage allowance:

```text
max(payment_fee_exempt_msat,
    floor(payment_amount_msat * max_payment_fee_ppm / 1_000_000))
```

With the defaults:

| Payment amount | Maximum routing fee |
|----------------|---------------------|
| 100,000 sat | 1,000 sat |
| 1,000,000 sat | 5,000 sat |

The allowance is shared across all parts and channels used by the payment.
For invoices with no amount, it uses the amount supplied in the payment request.
You can retry an amountless invoice with a different amount once the previous
attempt has no outstanding HTLCs.

Routing checks use whole satoshi. Rounding can allow up to 999 msat extra per
HTLC, including when both routing settings are zero. These limits apply to
outstanding HTLCs per payment hash; they do not track lifetime spending.

## Dealing with different payment types

Keep the payment request's fee limit at or below the signer's routing allowance.
For `pay`, direct-invoice `xpay`, and trampoline payments, the signer rejects
higher requested limits before CLN offers any HTLCs. A cheaper available route
does not prevent this rejection.

### SDK `send` and LNURL pay

The SDK sets `maxfee` automatically to the larger of 1,000 sat and 0.5% of the
payment amount. It does not read custom signer settings or expose a fee override.
Use the CLN RPCs directly if your application needs a different fee limit.

### CLN `pay`

Set `maxfee` in millisatoshi to choose an absolute fee limit.

Without `maxfee`, CLN uses the larger of:

- The percentage fee set by `maxfeepercent` (default: 0.5%).
- The fixed allowance set by `exemptfee` (default: 5,000 msat, or 5 sat).

### CLN `xpay`

Set `maxfee` explicitly. Xpay's default is the larger of 1% of the payment amount
and 5 sat. For large payments, this exceeds the signer's default allowance.

If you omit `maxfee` and set `partial_msat`, Xpay calculates the default fee from
the partial amount. The signer's allowance still uses the full payment amount.

The early fee check covers direct invoices. It does not cover invoices obtained
through offer or BIP353 name resolution.

### Trampoline

Trampoline payments prepay the requested percentage surcharge in full. Set
`maxfeepercent` to choose the percentage; omitting it or setting it to zero uses
0.5%. The entire surcharge counts toward the signer's routing allowance.

### CLN `sendpay`

When supplying your own routes, keep their combined routing fees within the
signer's allowance. These fees are checked at commitment signing, after the
HTLCs have been offered.

## Miner fees

The signer checks these caps before accepting a new commitment or signing a
local HTLC input or cooperative close. The default caps may reject channel
updates when Bitcoin fees are high or many HTLCs are pending. Commitment fees
include the value of small HTLCs omitted from the transaction outputs as dust.

A previously accepted commitment can still be signed for a force-close if its
fee exceeds the current cap. The signer logs a warning. Lowering a cap therefore
does not prevent force-closing a stored commitment.

Wallet withdrawals and sweeps retain their existing 150,000 sat/kW (600 sat/vB)
fee-rate ceiling.

For payment authorization, see [Security](security.md#signer-fee-budgets).
