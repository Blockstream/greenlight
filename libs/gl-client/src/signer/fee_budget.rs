//! Fee limits for invoice preapproval and on-chain signing. VLS retains
//! payment accounting; routing_policy supplies its amount-dependent allowance.

use crate::pb::HsmRequestContext;
use anyhow::{anyhow, bail, Context, Result};
use lightning_signer::bitcoin::Transaction;
use lightning_signer::channel::{Channel, ChannelId, ChannelSlot};
use lightning_signer::node::Node;
use vls_protocol::model::Htlc;
use vls_protocol::msgs::Message;

/// Fee ceilings chosen by the signer operator. On-chain caps are independent
/// of policy filters.
/// The routing cap uses VLS's payment-balance policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeBudgetLimits {
    /// Minimum routing fee allowance per payment hash, in msat.
    pub payment_fee_exempt_msat: u64,
    /// Percentage routing fee allowance, in parts per million (5,000 = 0.5%).
    /// The effective allowance is the larger of this and the exemption.
    pub max_payment_fee_ppm: u32,
    /// Actual miner fee when accepting a commitment, including trimmed HTLC
    /// value.
    pub max_commitment_fee_sat: u64,
    /// Actual miner fee per cooperative close. Zero permits no miner fee.
    pub max_mutual_close_fee_sat: u64,
}

impl Default for FeeBudgetLimits {
    fn default() -> Self {
        Self {
            payment_fee_exempt_msat: 1_000_000,
            max_payment_fee_ppm: Self::DEFAULT_PAYMENT_FEE_PPM,
            // Provisional miner-fee ceiling: e.g. 49,600 sat base fee for an anchor
            // commitment with five HTLC outputs at 100 sat/vB, plus 50k dust.
            max_commitment_fee_sat: 100_000,
            // Provisional miner-fee ceiling: e.g. 200 vB at 250 sat/vB.
            max_mutual_close_fee_sat: 50_000,
        }
    }
}

impl FeeBudgetLimits {
    pub const DEFAULT_PAYMENT_FEE_PPM: u32 = 5_000;

    /// Maximum fee for an approved principal, shared by the signer and clients.
    pub fn payment_fee_msat(&self, amount_msat: u64) -> Result<u64> {
        Ok(self.payment_fee_exempt_msat.max(Self::fee_for_ppm_msat(
            amount_msat,
            self.max_payment_fee_ppm,
        )?))
    }

    fn fee_for_ppm_msat(amount_msat: u64, ppm: u32) -> Result<u64> {
        u64::try_from(u128::from(amount_msat) * u128::from(ppm) / 1_000_000)
            .context("routing fee overflow")
    }

    /// Normalize the wire percentage to the nearest ppm; truncate fees to msat.
    pub fn fee_for_percent_msat(amount_msat: u64, percent: f32) -> Result<u64> {
        let ppm = (f64::from(percent) * 10_000.0).round();
        if percent < 0.0 || !ppm.is_finite() || ppm > f64::from(u32::MAX) {
            bail!("invalid routing fee percentage");
        }

        Self::fee_for_ppm_msat(amount_msat, ppm as u32)
    }
}

fn cln_fee_limit_msat(amount: u64, maxfee: Option<u64>, percent: f64, exempt: u64) -> Result<u64> {
    if let Some(fee) = maxfee {
        return Ok(fee); // CLN's absolute limit replaces percentage and exemption defaults.
    }
    // CLN parses millionths of a percent, then truncates to whole ppm.
    let ppm = (percent * 1_000_000.0).round() as u64 / 100;
    if percent < 0.0 || !percent.is_finite() || ppm > u64::from(u32::MAX) {
        bail!("invalid payment fee percentage");
    }

    Ok(exempt.max(FeeBudgetLimits::fee_for_ppm_msat(amount, ppm as u32)?))
}

// Outside permissive authentication fallback: reject unsafe fee limits before
// CLN offers HTLCs, rather than rejecting the commitment that adds them.
pub(super) fn check_payment_preapproval(
    message: &Message,
    requests: &[super::model::Request],
    limits: FeeBudgetLimits,
) -> Result<()> {
    use lightning_signer::invoice::InvoiceAttributes;
    let Message::PreapproveInvoice(message) = message else {
        return Ok(());
    };
    let invoice = super::auth::parse_invoice(std::str::from_utf8(&message.invstring.0)?)?;
    let amount = |chosen| match invoice.amount_milli_satoshis() {
        0 => chosen,
        amount => amount,
    };
    for request in requests {
        use super::model::Request;
        let matches = |candidate: &str| super::auth::same_invoice(&message.invstring.0, candidate);
        let (amount, fee) = match request {
            Request::Pay(r) if matches(&r.bolt11) => {
                let amount = amount(r.amount_msat.as_ref().map_or(0, |a| a.msat));

                (
                    amount,
                    cln_fee_limit_msat(
                        amount,
                        r.maxfee.as_ref().map(|a| a.msat),
                        r.maxfeepercent.unwrap_or(0.5),
                        r.exemptfee.as_ref().map_or(5_000, |a| a.msat),
                    )?,
                )
            }
            Request::Xpay(r) if matches(&r.invstring) => {
                let amount = amount(r.amount_msat.as_ref().map_or(0, |a| a.msat));

                (
                    amount,
                    cln_fee_limit_msat(
                        r.partial_msat.as_ref().map_or(amount, |a| a.msat),
                        r.maxfee.as_ref().map(|a| a.msat),
                        1.0,
                        5_000,
                    )?,
                )
            }
            Request::TrampolinePay(r) if matches(&r.bolt11) => {
                let amount = amount(r.amount_msat);
                let percent = if r.maxfeepercent == 0.0 {
                    FeeBudgetLimits::DEFAULT_PAYMENT_FEE_PPM as f32 / 10_000.0
                } else {
                    r.maxfeepercent
                };

                (
                    amount,
                    FeeBudgetLimits::fee_for_percent_msat(amount, percent)?,
                )
            }
            _ => continue,
        };
        let cap = limits.payment_fee_msat(amount)?;
        if fee > cap {
            bail!("requested fee limit {fee} msat exceeds payment allowance {cap} msat");
        }
    }
    Ok(())
}

fn sum(values: impl IntoIterator<Item = u64>) -> Result<u64> {
    values.into_iter().try_fold(0u64, |a, b| {
        a.checked_add(b)
            .ok_or_else(|| anyhow!("fee accounting overflow"))
    })
}

fn check_fee(input_sat: u64, output_sat: u64, cap_sat: u64, kind: &str) -> Result<()> {
    let fee_sat = input_sat
        .checked_sub(output_sat)
        .ok_or_else(|| anyhow!("{kind} outputs exceed input"))?;
    if fee_sat > cap_sat {
        bail!("{kind} fee {fee_sat} sat exceeds hard budget {cap_sat} sat");
    }
    Ok(())
}

// Count the actual miner fee, including trimmed outputs. The hsmd protocol
// supplies only untrimmed HTLCs, so dust cannot be subtracted reliably here.
fn check_tx(tx: &Transaction, input_sat: u64, cap_sat: u64, kind: &str) -> Result<()> {
    check_fee(
        input_sat,
        sum(tx.output.iter().map(|o| o.value.to_sat()))?,
        cap_sat,
        kind,
    )
}

// VLS builds phase-2 outputs directly from these values, including only the
// anchor outputs whose side has a balance or an untrimmed HTLC. Keys and
// commitment numbers do not affect output values and remain VLS's concern.
fn check_commitment(
    channel: &Channel,
    local_sat: u64,
    remote_sat: u64,
    htlcs: impl IntoIterator<Item = u64>,
    cap: u64,
) -> Result<()> {
    let mut htlcs = htlcs.into_iter().peekable();
    let has_htlcs = htlcs.peek().is_some();
    let anchors = if channel.setup.features().supports_anchors_zero_fee_htlc_tx() {
        330 * (u64::from(local_sat > 0 || has_htlcs) + u64::from(remote_sat > 0 || has_htlcs))
    } else {
        0
    };
    check_fee(
        channel.setup.channel_value_sat,
        sum([local_sat, remote_sat, sum(htlcs)?, anchors])?,
        cap,
        "commitment",
    )
}

fn check_phase2(
    channel: &Channel,
    local: u64,
    remote: u64,
    htlcs: &[Htlc],
    cap: u64,
) -> Result<()> {
    if htlcs
        .iter()
        .any(|h| h.side != Htlc::LOCAL && h.side != Htlc::REMOTE)
    {
        bail!("invalid HTLC side");
    }
    check_commitment(
        channel,
        local,
        remote,
        htlcs.iter().map(|h| h.amount / 1000),
        cap,
    )
}

fn check_holder_close(channel: &Channel, number: u64, cap: u64) -> Result<()> {
    let state = &channel.enforcement_state;
    let info = if number == state.next_holder_commit_num {
        state.next_holder_commit_info.as_ref().map(|(info, _)| info)
    } else if Some(number) == state.next_holder_commit_num.checked_sub(1) {
        state.current_holder_commit_info.as_ref()
    } else {
        None
    }
    .ok_or_else(|| anyhow!("no validated holder commitment for close number {number}"))?;
    if let Err(e) = check_commitment(
        channel,
        info.to_broadcaster_value_sat,
        info.to_countersigner_value_sat,
        info.offered_htlcs
            .iter()
            .chain(&info.received_htlcs)
            .map(|h| h.value_sat),
        cap,
    ) {
        // The fee was fixed when this commitment was accepted. Upgrades or
        // lower caps must not prevent recovering funds by force-closing it.
        log::warn!("Signing previously accepted holder commitment: {e:#}");
    }
    Ok(())
}

/// Check on-chain fees before VLS can sign or advance commitment state.
/// SignRemoteHtlcTx spends the peer's claim and is deliberately left to VLS.
pub(super) fn check_request(
    message: &Message,
    context: Option<&HsmRequestContext>,
    node: &Node,
    limits: FeeBudgetLimits,
) -> Result<()> {
    // A valid SegWit HTLC signature commits to the input amount. A node
    // supplying a false witness_utxo value cannot turn it into a spendable
    // signature. Check the matching output, including SIGHASH_SINGLE anchor
    // transactions, rather than letting unrelated inputs hide our fee share.
    let htlc = match message {
        Message::SignLocalHtlcTx(m) => Some((
            &m.tx.0,
            0,
            m.psbt
                .0
                .inner
                .inputs
                .first()
                .and_then(|i| i.witness_utxo.as_ref())
                .map(|o| o.value.to_sat()),
        )),
        Message::SignAnyLocalHtlcTx(m) => Some((
            &m.tx.0,
            m.input as usize,
            m.psbt
                .0
                .inner
                .inputs
                .get(m.input as usize)
                .and_then(|i| i.witness_utxo.as_ref())
                .map(|o| o.value.to_sat()),
        )),
        Message::SignLocalHtlcTx2(m) => {
            Some((&m.tx.0, m.input as usize, Some(m.htlc_amount_msat / 1000)))
        }
        _ => None,
    };
    if let Some((tx, input, amount)) = htlc {
        let amount = amount.ok_or_else(|| anyhow!("HTLC fee check requires input amount"))?;
        let output = tx
            .output
            .get(input)
            .ok_or_else(|| anyhow!("HTLC fee check requires matching output"))?;
        check_fee(
            amount,
            output.value.to_sat(),
            limits.max_commitment_fee_sat,
            "HTLC input",
        )?;
        return Ok(());
    }
    let channel_id = match message {
        Message::SignCommitmentTx(m) => ChannelId::new_from_peer_id_and_oid(&m.peer_id.0, m.dbid),
        Message::SignRemoteCommitmentTx(_)
        | Message::SignRemoteCommitmentTx2(_)
        | Message::ValidateCommitmentTx(_)
        | Message::ValidateCommitmentTx2(_)
        | Message::SignLocalCommitmentTx2(_)
        | Message::SignMutualCloseTx(_)
        | Message::SignMutualCloseTx2(_) => {
            let context = context
                .filter(|c| c.dbid != 0)
                .ok_or_else(|| anyhow!("fee check requires channel context"))?;
            let peer: [u8; 33] = context
                .node_id
                .as_slice()
                .try_into()
                .context("invalid channel peer id")?;
            ChannelId::new_from_peer_id_and_oid(&peer, context.dbid)
        }
        _ => return Ok(()),
    };
    let slot = node.get_channel(&channel_id).map_err(|e| anyhow!("{e}"))?;
    let slot = slot.lock().map_err(|_| anyhow!("channel lock poisoned"))?;
    let channel = match &*slot {
        ChannelSlot::Ready(channel) => channel,
        ChannelSlot::Stub(_) => bail!("fee check requires a ready channel"),
    };
    let funding_sat = channel.setup.channel_value_sat;
    let commitment_cap = limits.max_commitment_fee_sat;
    let close_cap = limits.max_mutual_close_fee_sat;
    match message {
        Message::SignRemoteCommitmentTx(m) => {
            check_tx(&m.tx.0, funding_sat, commitment_cap, "commitment")
        }
        Message::ValidateCommitmentTx(m) => {
            check_tx(&m.tx.0, funding_sat, commitment_cap, "commitment")
        }
        Message::SignRemoteCommitmentTx2(m) => check_phase2(
            channel,
            m.to_local_value_sat,
            m.to_remote_value_sat,
            &m.htlcs,
            commitment_cap,
        ),
        Message::ValidateCommitmentTx2(m) => check_phase2(
            channel,
            m.to_local_value_sat,
            m.to_remote_value_sat,
            &m.htlcs,
            commitment_cap,
        ),
        Message::SignCommitmentTx(m) if m.tx.0.lock_time.to_consensus_u32() == 0 => {
            check_tx(&m.tx.0, funding_sat, close_cap, "mutual close")
        }
        Message::SignCommitmentTx(m) => {
            // VLS signs its stored current or pending holder commitment,
            // rather than the transaction supplied in this message.
            check_holder_close(channel, m.commitment_number, commitment_cap)
        }
        Message::SignLocalCommitmentTx2(m) => {
            check_holder_close(channel, m.commitment_number, commitment_cap)
        }
        Message::SignMutualCloseTx(m) => check_tx(&m.tx.0, funding_sat, close_cap, "mutual close"),
        Message::SignMutualCloseTx2(m) => check_fee(
            funding_sat,
            sum([m.to_local_value_sat, m.to_remote_value_sat])?,
            close_cap,
            "mutual close",
        ),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::{auth, model, record_invoice, Signer};
    use lightning_signer::bitcoin::hashes::{sha256, Hash};
    use lightning_signer::bitcoin::secp256k1::{Message as SecpMessage, Secp256k1, SecretKey};
    use lightning_signer::bitcoin::Network;
    use lightning_signer::invoice::InvoiceAttributes;
    use lightning_signer::lightning::types::payment::{
        PaymentHash, PaymentPreimage, PaymentSecret,
    };
    use lightning_signer::policy::validator::CommitmentSignatures;
    use lightning_signer::tx::tx::{CommitmentInfo2, HTLCInfo2};
    use std::sync::Arc;
    use vls_protocol::serde_bolt::{Octets, WithSize};
    use vls_protocol_signer::handler::Handler;

    fn fixture() -> (Signer, Arc<Node>) {
        let signer = Signer::new(
            vec![1; 32],
            Network::Bitcoin,
            crate::credentials::Nobody::default(),
        )
        .unwrap();
        let node = signer.handler().unwrap().node().clone();
        (signer, node)
    }

    fn channel_fixture() -> (Signer, Arc<Node>, HsmRequestContext) {
        use lightning_signer::bitcoin::{bip32::DerivationPath, OutPoint};
        use lightning_signer::channel::{ChannelSetup, CommitmentType};
        let (signer, node) = fixture();
        let peer = node.get_id().serialize();
        let (id, _) = node.new_channel(1, &peer, &node).unwrap();
        let points = node
            .with_channel_base(&id, |c| Ok(c.get_channel_basepoints()))
            .unwrap();
        node.setup_channel(
            id,
            None,
            ChannelSetup {
                is_outbound: true,
                channel_value_sat: 1_000_000,
                push_value_msat: 0,
                funding_outpoint: OutPoint::null(),
                holder_selected_contest_delay: 144,
                holder_shutdown_script: None,
                counterparty_points: points,
                counterparty_selected_contest_delay: 144,
                counterparty_shutdown_script: None,
                commitment_type: CommitmentType::StaticRemoteKey,
            },
            &DerivationPath::master(),
        )
        .unwrap();
        (
            signer,
            node,
            HsmRequestContext {
                node_id: peer.to_vec(),
                dbid: 1,
                capabilities: 0,
            },
        )
    }

    fn tx(output_sat: u64) -> Transaction {
        use lightning_signer::bitcoin::{
            absolute::LockTime, transaction::Version, Amount, ScriptBuf, TxIn, TxOut,
        };
        Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn::default()],
            output: vec![TxOut {
                value: Amount::from_sat(output_sat),
                script_pubkey: ScriptBuf::new(),
            }],
        }
    }

    fn amountless_payment() -> (model::cln::PayRequest, PaymentHash) {
        use lightning_signer::lightning_invoice::{Currency, InvoiceBuilder};
        let invoice = InvoiceBuilder::new(Currency::Bitcoin)
            .description("amountless payment".to_string())
            .payment_hash(sha256::Hash::hash(&[3; 32]))
            .payment_secret(PaymentSecret([4; 32]))
            .current_timestamp()
            .min_final_cltv_expiry_delta(18)
            .build_signed(|m| {
                Secp256k1::new()
                    .sign_ecdsa_recoverable(m, &SecretKey::from_slice(&[2; 32]).unwrap())
            })
            .unwrap();
        let hash = auth::parse_invoice(&invoice.to_string())
            .unwrap()
            .payment_hash();
        (
            model::cln::PayRequest {
                bolt11: invoice.to_string(),
                amount_msat: Some(model::cln::Amount { msat: 2_000_000 }),
                ..Default::default()
            },
            hash,
        )
    }

    fn commitment_signatures() -> CommitmentSignatures {
        CommitmentSignatures(
            Secp256k1::new().sign_ecdsa(
                &SecpMessage::from_digest([1; 32]),
                &SecretKey::from_slice(&[2; 32]).unwrap(),
            ),
            vec![],
        )
    }

    #[test]
    fn routing_budget_aggregates_parts_across_channels() {
        use lightning_signer::{policy::validator::BalanceDelta, prelude::Map};
        let (signer, node, context) = channel_fixture();
        let id = ChannelId::new_from_peer_id_and_oid(&node.get_id().serialize(), context.dbid);
        let other = ChannelId::new_from_peer_id_and_oid(&node.get_id().serialize(), 2);
        let (mut request, hash) = amountless_payment();
        request.amount_msat = Some(model::cln::Amount {
            msat: 1_000_000_000,
        });
        record_invoice(
            &model::Request::Pay(request),
            &node,
            signer.services.persister.as_ref(),
            signer.services.clock.now().as_secs(),
        )
        .unwrap();
        let validator = signer.services.validator_factory.make_validator(
            Network::Bitcoin,
            node.get_id(),
            Some(id.clone()),
        );
        let mut state = node.get_state();
        state
            .payments
            .get_mut(&hash)
            .unwrap()
            .outgoing
            .insert(other, 500_000);
        // A 1,000,000-sat principal gets 5,000 sat once, shared by both channels.
        for (part_sat, allowed) in [(505_000, true), (505_001, false)] {
            let outgoing = [(hash, part_sat)].into_iter().collect();
            assert_eq!(
                state
                    .validate_payments(
                        &id,
                        &Map::new(),
                        &outgoing,
                        &BalanceDelta(0, 0),
                        validator.clone()
                    )
                    .is_ok(),
                allowed
            );
        }
        // Exercise a provided trait method whose default would silently disable
        // the stock validator's configured balance enforcement.
        let mut policy = lightning_signer::policy::simple_validator::make_default_simple_policy(
            Network::Bitcoin,
        );
        policy.enforce_balance = true;
        let strict = super::super::routing_policy::RoutingValidatorFactory::new(
            policy,
            FeeBudgetLimits::default(),
        );
        use lightning_signer::policy::validator::ValidatorFactory;
        assert!(strict
            .make_validator(Network::Bitcoin, node.get_id(), None)
            .enforce_balance());
        assert!(validator.validate_payment_cltv(100, 99).is_err());
        assert!(validator
            .validate_payment_balance(1_000, 1_000, None)
            .is_ok());
        assert!(validator
            .validate_payment_balance(1_000, 1_001, None)
            .is_err());
        assert!(validator
            .validate_payment_balance(0, 0, Some(u64::MAX))
            .is_err());
    }

    #[test]
    fn trampoline_preapproval_checks_the_configured_amount_dependent_budget() {
        let (payment, _) = amountless_payment();
        let message = Message::PreapproveInvoice(vls_protocol::msgs::PreapproveInvoice {
            invstring: vls_protocol::serde_bolt::WireString(payment.bolt11.as_bytes().to_vec()),
        });
        for (percent, ppm, allowed) in [
            (0.0, 5_000, true),
            (0.25, 5_000, true),
            (1.0, 5_000, false),
            (1.0, 10_000, true),
            (f32::NAN, 5_000, false),
            (-0.00001, 5_000, false),
        ] {
            let request = model::Request::TrampolinePay(crate::pb::TrampolinePayRequest {
                bolt11: payment.bolt11.clone(),
                amount_msat: 1_000_000_000,
                maxfeepercent: percent,
                ..Default::default()
            });
            let limits = FeeBudgetLimits {
                max_payment_fee_ppm: ppm,
                ..Default::default()
            };
            assert_eq!(
                check_payment_preapproval(&message, &[request], limits).is_ok(),
                allowed
            );
        }
        let limits = FeeBudgetLimits::default();
        for (amount, fee) in [
            (1, 1_000_000),
            (200_000_000, 1_000_000),
            (200_200_000, 1_001_000),
            (1_000_000_000, 5_000_000),
        ] {
            assert_eq!(limits.payment_fee_msat(amount).unwrap(), fee);
        }
        assert!(FeeBudgetLimits {
            max_payment_fee_ppm: u32::MAX,
            ..limits
        }
        .payment_fee_msat(u64::MAX)
        .is_err());
    }

    #[test]
    fn pay_and_xpay_limits_are_checked_before_htlcs() {
        let (payment, _) = amountless_payment();
        let message = Message::PreapproveInvoice(vls_protocol::msgs::PreapproveInvoice {
            invstring: vls_protocol::serde_bolt::WireString(payment.bolt11.as_bytes().to_vec()),
        });
        let amount = 1_000_000_000;
        let cap = 5_000_000;
        assert_eq!(cln_fee_limit_msat(amount, None, 0.07, 0).unwrap(), 700_000);
        for (maxfee, percent, exempt, allowed) in [
            (None, None, None, true),
            (Some(cap), None, None, true),
            (Some(cap + 1), None, None, false),
            (None, Some(1.0), None, false),
            (None, None, Some(cap + 1), false),
            (None, Some(0.5000001), None, true),
            (None, Some(f64::NAN), None, false),
        ] {
            let request = model::Request::Pay(model::cln::PayRequest {
                bolt11: payment.bolt11.clone(),
                amount_msat: Some(model::cln::Amount { msat: amount }),
                maxfee: maxfee.map(|msat| model::cln::Amount { msat }),
                maxfeepercent: percent,
                exemptfee: exempt.map(|msat| model::cln::Amount { msat }),
                ..Default::default()
            });
            assert_eq!(
                check_payment_preapproval(&message, &[request], FeeBudgetLimits::default()).is_ok(),
                allowed
            );
        }
        for (amount, partial, maxfee, allowed) in [
            (amount, None, None, false),
            (50_000_000, None, None, true),
            (amount, Some(amount / 2), None, true),
            (amount, None, Some(cap), true),
            (amount, None, Some(cap + 1), false),
        ] {
            let request = model::Request::Xpay(model::cln::XpayRequest {
                invstring: payment.bolt11.clone(),
                amount_msat: Some(model::cln::Amount { msat: amount }),
                partial_msat: partial.map(|msat| model::cln::Amount { msat }),
                maxfee: maxfee.map(|msat| model::cln::Amount { msat }),
                ..Default::default()
            });
            assert_eq!(
                check_payment_preapproval(&message, &[request], FeeBudgetLimits::default()).is_ok(),
                allowed
            );
        }
    }

    #[test]
    fn amountless_retries_preserve_pending_and_fulfilled_amounts() {
        use lightning_signer::node::RoutedPayment;
        let (signer, node, context) = channel_fixture();
        let id =
            ChannelId::new_from_peer_id_and_oid(&context.node_id.try_into().unwrap(), context.dbid);
        let (mut request, hash) = amountless_payment();
        let record = |request: &model::cln::PayRequest| {
            record_invoice(
                &model::Request::Pay(request.clone()),
                &node,
                signer.services.persister.as_ref(),
                signer.services.clock.now().as_secs(),
            )
        };
        record(&request).unwrap();
        assert_eq!(node.get_state().invoices[&hash].amount_msat, 2_000_000);
        request.amount_msat = Some(model::cln::Amount { msat: 3_000_000 });
        record(&request).unwrap(); // No HTLCs: retrying with a new amount is allowed.

        for (incoming, outgoing, fulfilled) in [(0, 3_000, false), (3_000, 0, false), (0, 0, true)]
        {
            let mut payment = RoutedPayment::new();
            payment.apply(&id, incoming, outgoing, None, None);
            payment.preimage = fulfilled.then_some(PaymentPreimage([3; 32]));
            node.get_state().payments.insert(hash, payment);
            record(&request).unwrap(); // Reusing the existing approval does not reset its amount.
            let mut retry = request.clone();
            retry.amount_msat = Some(model::cln::Amount { msat: 4_000_000 });
            assert!(record(&retry)
                .unwrap_err()
                .to_string()
                .contains("pending or fulfilled"));
            assert_eq!(node.get_state().invoices[&hash].amount_msat, 3_000_000);
            assert_eq!(node.get_state().payments[&hash].is_fulfilled(), fulfilled);
        }

        node.get_state().payments.remove(&hash);
        node.with_channel(&id, |channel| {
            channel.enforcement_state.next_holder_commit_info = Some((
                CommitmentInfo2::new(
                    false,
                    0,
                    996_000,
                    vec![HTLCInfo2 {
                        value_sat: 3_000,
                        payment_hash: hash,
                        cltv_expiry: 100,
                    }],
                    vec![],
                    253,
                ),
                commitment_signatures(),
            ));
            Ok(())
        })
        .unwrap();
        request.amount_msat = Some(model::cln::Amount { msat: 4_000_000 });
        assert!(record(&request)
            .unwrap_err()
            .to_string()
            .contains("pending payment"));
        assert_eq!(node.get_state().invoices[&hash].amount_msat, 3_000_000);
        node.with_channel(&id, |channel| {
            channel.enforcement_state.next_holder_commit_info = None;
            Ok(())
        })
        .unwrap();
        record(&request).unwrap();
        assert_eq!(node.get_state().invoices[&hash].amount_msat, 4_000_000);
    }

    #[tokio::test]
    async fn failed_preapprovals_carry_negative_protocol_replies() {
        use vls_protocol::msgs::{
            PreapproveInvoice, PreapproveInvoiceReply, PreapproveKeysend, PreapproveKeysendReply,
            SerBolt,
        };
        let (signer, _) = fixture();
        for (raw, expected) in [
            (
                PreapproveInvoice {
                    invstring: vls_protocol::serde_bolt::WireString(b"invalid invoice".to_vec()),
                }
                .as_vec(),
                PreapproveInvoiceReply { result: false }.as_vec(),
            ),
            (
                PreapproveKeysend {
                    destination: vls_protocol::model::PubKey([0; 33]),
                    payment_hash: vls_protocol::model::Sha256([0; 32]),
                    amount_msat: 1_000,
                }
                .as_vec(),
                PreapproveKeysendReply { result: false }.as_vec(),
            ),
        ] {
            let error = signer
                .process_request(crate::pb::HsmRequest {
                    request_id: 42,
                    raw,
                    ..Default::default()
                })
                .await
                .unwrap_err();
            match error {
                crate::signer::Error::PreapprovalDeclined { reply, .. } => {
                    assert_eq!(reply, expected)
                }
                other => panic!("unexpected error: {other}"),
            }
        }
    }

    #[test]
    fn xpay_context_records_the_amountless_principal() {
        use prost::Message;
        let (signer, node) = fixture();
        let (payment, hash) = amountless_payment();
        let wire = model::cln::XpayRequest {
            invstring: payment.bolt11,
            amount_msat: payment.amount_msat,
            ..Default::default()
        }
        .encode_to_vec();
        let request = model::cln::decode_request("/cln.Node/Xpay", &wire).unwrap();
        record_invoice(
            &request,
            &node,
            signer.services.persister.as_ref(),
            signer.services.clock.now().as_secs(),
        )
        .unwrap();
        assert_eq!(node.get_state().invoices[&hash].amount_msat, 2_000_000);
    }

    #[test]
    fn amountless_sendpay_defaults_to_final_hop_only_for_single_part() {
        let (payment, hash) = amountless_payment();
        for (partid, explicit, expected) in [
            (None, None, Some(2_000_000)),
            (Some(0), None, Some(2_000_000)),
            (None, Some(4_000_000), Some(4_000_000)),
            (Some(1), Some(4_000_000), Some(4_000_000)),
            (Some(1), None, None),
        ] {
            let (signer, node) = fixture();
            let request = model::Request::SendPay(model::cln::SendpayRequest {
                bolt11: Some(payment.bolt11.clone()),
                partid,
                amount_msat: explicit.map(|msat| model::cln::Amount { msat }),
                route: [3_000_000, 2_000_000]
                    .into_iter()
                    .map(|msat| model::cln::SendpayRoute {
                        amount_msat: Some(model::cln::Amount { msat }),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            });
            let result = record_invoice(
                &request,
                &node,
                signer.services.persister.as_ref(),
                signer.services.clock.now().as_secs(),
            );
            match expected {
                Some(amount) => {
                    result.unwrap();
                    assert_eq!(node.get_state().invoices[&hash].amount_msat, amount);
                }
                None => {
                    assert!(result
                        .unwrap_err()
                        .to_string()
                        .contains("authenticated payment amount"));
                    assert!(!node.get_state().invoices.contains_key(&hash));
                }
            }
        }
    }

    #[test]
    fn stored_force_close_remains_signable_above_current_budget() {
        let (_, node, context) = channel_fixture();
        let id = ChannelId::new_from_peer_id_and_oid(
            &context.node_id.clone().try_into().unwrap(),
            context.dbid,
        );
        node.with_channel(&id, |channel| {
            channel.enforcement_state.next_holder_commit_num = 1;
            channel.enforcement_state.current_holder_commit_info =
                Some(CommitmentInfo2::new(false, 0, 899_999, vec![], vec![], 253));
            channel.enforcement_state.next_holder_commit_info = Some((
                CommitmentInfo2::new(false, 0, 899_998, vec![], vec![], 253),
                commitment_signatures(),
            ));
            Ok(())
        })
        .unwrap();
        for number in [0, 1] {
            let message =
                Message::SignLocalCommitmentTx2(vls_protocol::msgs::SignLocalCommitmentTx2 {
                    commitment_number: number,
                });
            check_request(&message, Some(&context), &node, FeeBudgetLimits::default()).unwrap();
        }
    }

    #[test]
    fn close_disguised_as_sign_commitment_uses_close_cap() {
        use lightning_signer::bitcoin::psbt::Psbt;
        let (_, node, context) = channel_fixture();
        let tx = tx(949_999); // 50,001 sat: within commitment cap, outside close cap.
        let message = Message::SignCommitmentTx(vls_protocol::msgs::SignCommitmentTx {
            peer_id: vls_protocol::model::PubKey(context.node_id.try_into().unwrap()),
            dbid: context.dbid,
            psbt: WithSize(Psbt::from_unsigned_tx(tx.clone()).unwrap().into()),
            tx: WithSize(tx),
            remote_funding_key: vls_protocol::model::PubKey(node.get_id().serialize()),
            commitment_number: 0,
        });
        let error = check_request(&message, None, &node, FeeBudgetLimits::default()).unwrap_err();
        assert!(error.to_string().contains("mutual close fee 50001 sat"));
    }

    #[test]
    fn phase2_fee_matches_vls_outputs_for_each_channel_type() {
        use lightning_signer::channel::CommitmentType;
        let (_, node, context) = channel_fixture();
        let id =
            ChannelId::new_from_peer_id_and_oid(&context.node_id.try_into().unwrap(), context.dbid);
        node.with_channel(&id, |channel| {
            for kind in [
                CommitmentType::StaticRemoteKey,
                CommitmentType::Anchors,
                CommitmentType::AnchorsZeroFeeHtlc,
            ] {
                channel.setup.commitment_type = kind;
                for (local, remote) in [(900_000, 0), (0, 900_000), (450_000, 450_000)] {
                    let tx = channel.make_counterparty_commitment_tx(
                        &node.get_id(),
                        0,
                        253,
                        local,
                        remote,
                        vec![],
                    );
                    let fee = channel.setup.channel_value_sat
                        - tx.trust()
                            .built_transaction()
                            .transaction
                            .output
                            .iter()
                            .map(|o| o.value.to_sat())
                            .sum::<u64>();
                    assert!(check_phase2(channel, local, remote, &[], fee).is_ok());
                    assert!(check_phase2(channel, local, remote, &[], fee - 1).is_err());
                }
            }
            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn hard_cap_is_enforced_at_handler_entry_even_in_permissive_mode() {
        use lightning_signer::bitcoin::{psbt::Psbt, Amount, ScriptBuf, TxOut};
        let (signer, _) = fixture();
        let tx = tx(100_000);
        let mut psbt = Psbt::from_unsigned_tx(tx.clone()).unwrap();
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(200_001),
            script_pubkey: ScriptBuf::new(),
        });
        let message = Message::SignAnyLocalHtlcTx(vls_protocol::msgs::SignAnyLocalHtlcTx {
            commitment_number: 0,
            tx: WithSize(tx),
            psbt: WithSize(psbt.into()),
            wscript: Octets(vec![]),
            option_anchors: false,
            input: 0,
            peer_id: vls_protocol::model::PubKey(
                signer.handler().unwrap().node().get_id().serialize(),
            ),
            dbid: 1,
        });
        let error = signer
            .process_request(crate::pb::HsmRequest {
                request_id: 1,
                context: None,
                raw: message.inner().as_vec(),
                signer_state: vec![],
                requests: vec![],
            })
            .await
            .unwrap_err();
        assert!(
            matches!(error, crate::signer::Error::FeeBudget(_)),
            "{error}"
        );
        assert!(error.to_string().contains("HTLC input fee 100001 sat"));
    }
}
