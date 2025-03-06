use crate::awaitables::{AwaitableChannel, AwaitablePeer};
use crate::pb;
use anyhow::{anyhow, Result};
use cln_rpc::{
    primitives::{Amount, PublicKey, ShortChannelId},
    ClnRpc,
};
use futures::{future::join_all, FutureExt};
use gl_client::bitcoin::hashes::hex::ToHex;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::{path::Path, time::Duration};
use tokio::time::{timeout_at, Instant};

// Feature bit used to signal trampoline support.
const TRAMPOLINE_FEATURE_BIT: usize = 427;
// BOLT#4 default value:
// https://github.com/lightning/bolts/blob/master/04-onion-routing.md#max-htlc-cltv-selection
const MAX_DELAY_DEFAULT: u32 = 2016;
// The amount we overpay to allow the trampoline node to spend some fees.
const DEFAULT_OVERPAY_PERCENT: f32 = 0.5;
// Type used to address bolt11 in the onion payload.
const TLV_BOLT11: u64 = 33001;
// Type used to address the amount in msat in the onion payload, in case
// that the bolt11 does not have an amount set.
const TLV_AMT_MSAT: u64 = 33003;
// Error Message that CLN returns on an unknown onion error. This is the
// case when the trampoline server rejected with a custom error type.
const PAY_UNPARSEABLE_ONION_MSG: &str = "Malformed error reply";
const PAY_UNPARSEABLE_ONION_CODE: i32 = 202;
// How long do we wait for channels to re-establish?
const AWAIT_CHANNELS_TIMEOUT_SEC: u64 = 20;

fn feature_guard(features: impl Into<Vec<u8>>, feature_bit: usize) -> Result<()> {
    let mut features = features.into();
    features.reverse();
    let byte_pos = feature_bit / 8;
    let bit_pos = feature_bit % 8;
    if byte_pos >= features.len() || (features[byte_pos] & (1 << bit_pos)) == 0 {
        return Err(anyhow!(
            "Features {:?} do not contain feature bit {}",
            hex::encode(features),
            feature_bit
        ));
    }
    Ok(())
}

fn as_option<T>(v: T) -> Option<T>
where
    T: Default + PartialEq,
{
    if v == T::default() {
        None
    } else {
        Some(v)
    }
}

pub async fn trampolinepay(
    req: pb::TrampolinePayRequest,
    rpc_path: impl AsRef<Path>,
) -> Result<cln_rpc::model::responses::PayResponse> {
    let node_id = cln_rpc::primitives::PublicKey::from_slice(&req.trampoline_node_id[..])?;

    let mut rpc = ClnRpc::new(&rpc_path).await?;

    // Extract the amount from the bolt11 or use the set amount field
    // Return an error if there is a mismatch.
    let decoded = rpc
        .call_typed(&cln_rpc::model::requests::DecodepayRequest {
            bolt11: req.bolt11.clone(),
            description: None,
        })
        .await?;

    let send_pays = rpc
        .call_typed(&cln_rpc::model::requests::ListsendpaysRequest {
            payment_hash: Some(decoded.payment_hash.clone()),
            bolt11: None,
            index: None,
            limit: None,
            start: None,
            status: None,
        })
        .await?;
    if send_pays
        .payments
        .iter()
        .any(|p| p.status != cln_rpc::model::responses::ListsendpaysPaymentsStatus::FAILED)
    {
        let resp = rpc
            .call_typed(&cln_rpc::model::requests::WaitsendpayRequest {
                payment_hash: decoded.payment_hash.clone(),
                groupid: None,
                partid: None,
                timeout: None,
            })
            .await?;

        let preimage = match resp.payment_preimage {
            Some(preimage) => preimage,
            None => return Err(anyhow!("got completed payment part without preimage")),
        };
        return Ok(cln_rpc::model::responses::PayResponse {
            amount_msat: resp.amount_msat.unwrap_or(Amount::from_msat(0)),
            amount_sent_msat: resp.amount_sent_msat,
            created_at: 0.,
            destination: resp.destination,
            parts: match resp.partid {
                Some(0) => 1,
                Some(partid) => partid as u32,
                None => 1,
            },
            payment_hash: resp.payment_hash,
            payment_preimage: preimage,
            status: match resp.status {
                cln_rpc::model::responses::WaitsendpayStatus::COMPLETE => {
                    cln_rpc::model::responses::PayStatus::COMPLETE
                }
            },
            warning_partial_completion: None,
        });
    }

    let max_group_id = send_pays
        .payments
        .iter()
        .map(|p| p.groupid)
        .max()
        .unwrap_or(0);
    log::debug!(
        "New trampoline payment via {}: {} ",
        node_id.to_hex(),
        req.bolt11.clone()
    );

    // Wait for the peer connection to re-establish.
    log::debug!("Await peer connection to {}", node_id.to_hex());
    AwaitablePeer::new(node_id, rpc_path.as_ref().to_path_buf())
        .wait()
        .await?;

    // Check if peer has signaled that they support forward trampoline pays:

    let features = rpc
        .call_typed(&cln_rpc::model::requests::ListpeersRequest {
            id: Some(node_id),
            level: None,
        })
        .await?
        .peers
        .first()
        .ok_or_else(|| anyhow!("node with id {} is unknown", node_id.to_hex()))?
        .features
        .as_ref()
        .map(|feat| hex::decode(feat))
        .ok_or_else(|| anyhow!("Could not extract features from peer object."))??;

    feature_guard(features, TRAMPOLINE_FEATURE_BIT)?;

    let amount_msat = match (as_option(req.amount_msat), decoded.amount_msat) {
        (None, None) => {
            return Err(anyhow!(
                "Bolt11 does not contain an amount and no amount was set either"
            ));
        }
        (None, Some(amt)) => amt.msat(),
        (Some(amt), None) => amt,
        (Some(set_amt), Some(bolt11_amt)) => {
            if set_amt != bolt11_amt.msat() {
                return Err(anyhow!(
                    "Bolt11 amount {}msat and given amount {}msat do not match.",
                    bolt11_amt.msat(),
                    set_amt,
                ));
            }
            bolt11_amt.msat()
        }
    };

    // We need to add some sats to the htlcs to allow the trampoline node
    // to pay fees on routing.
    let tlv_amount_msat = amount_msat;
    let overpay = amount_msat as f64
        * (as_option(req.maxfeepercent).unwrap_or(DEFAULT_OVERPAY_PERCENT) as f64 / 100 as f64);
    let amount_msat = amount_msat + overpay as u64;

    debug!("overpay={}, total_amt={}", overpay, amount_msat);

    let mut channels: Vec<Channel> = rpc
        .call_typed(&cln_rpc::model::requests::ListpeerchannelsRequest { id: Some(node_id) })
        .await?
        .channels
        .unwrap_or_default()
        .into_iter()
        .filter_map(|ch| {
            let short_channel_id = ch.short_channel_id.or(ch.alias.and_then(|a| a.local));
            let short_channel_id = match short_channel_id {
                Some(scid) => scid,
                None => {
                    warn!("Missing short channel id on a channel to {}", &node_id);
                    return None;
                }
            };
            let spendable_msat = match ch.spendable_msat {
                Some(s) => s.msat(),
                None => {
                    warn!(
                        "Missing missing spendable_msat on channel with scid={}",
                        short_channel_id.to_string()
                    );
                    return None;
                }
            };
            let min_htlc_out_msat = match ch.minimum_htlc_out_msat {
                Some(m) => m.msat(),
                None => {
                    warn!(
                        "Missing missing minimum_htlc_out_msat on channel with scid={}",
                        short_channel_id.to_string()
                    );
                    return None;
                }
            };
            return Some(Channel {
                short_channel_id,
                spendable_msat,
                min_htlc_out_msat,
            });
        })
        .filter(|ch| ch.spendable_msat > 0)
        .filter(|ch| ch.spendable_msat > ch.min_htlc_out_msat)
        .collect();

    channels.sort_by(|a, b| b.spendable_msat.cmp(&a.spendable_msat));

    // Check if we actually got a channel to the trampoline node.
    if channels.is_empty() {
        return Err(anyhow!("Has no channels with trampoline node"));
    }

    // Await and filter out re-established channels.
    let deadline = Instant::now() + Duration::from_secs(AWAIT_CHANNELS_TIMEOUT_SEC);
    let channels =
        reestablished_channels(channels, node_id, rpc_path.as_ref().to_path_buf(), deadline)
            .await?;

    // Note: We can also do this inside the reestablished_channels function
    // but as we want to be greedy picking our channels we don't want to
    // introduce a race of the chosen channels for now.
    let alloc = match find_minimal_allocation(&channels, amount_msat) {
        Some(alloc) if !alloc.is_empty() => alloc,
        _ => {
            return Err(anyhow!(
                "could not allocate enough funds accross channels {}msat<{}msat",
                channels.iter().map(|ch| ch.spendable_msat).sum::<u64>(),
                amount_msat
            ));
        }
    };

    // All set we can preapprove the invoice
    let _ = rpc
        .call_typed(&cln_rpc::model::requests::PreapproveinvoiceRequest {
            bolt11: Some(req.bolt11.clone()),
        })
        .await?;

    // Create TLV payload.
    use crate::tlv::{SerializedTlvStream, ToBytes};
    let mut payload: SerializedTlvStream = SerializedTlvStream::new();
    payload.set_bytes(TLV_BOLT11, req.bolt11.as_bytes());
    payload.set_tu64(TLV_AMT_MSAT, tlv_amount_msat);
    let payload_hex = hex::encode(SerializedTlvStream::to_bytes(payload));

    let mut part_id = if alloc.len() == 1 { 0 } else { 1 };
    let group_id = max_group_id + 1;
    let mut handles: Vec<
        tokio::task::JoinHandle<
            std::result::Result<cln_rpc::model::responses::WaitsendpayResponse, anyhow::Error>,
        >,
    > = vec![];
    for ch in &alloc {
        let bolt11 = req.bolt11.clone();
        let label = req.label.clone();
        let part_amt = ch.contrib_msat.clone();
        let scid = ch.channel.short_channel_id.clone();
        let description = decoded.description.clone();
        let payload_hex = payload_hex.clone();
        let mut rpc = ClnRpc::new(&rpc_path).await?;
        let handle = tokio::spawn(async move {
            do_pay(
                &mut rpc,
                node_id,
                bolt11,
                label,
                description,
                part_amt,
                scid,
                part_id,
                group_id,
                decoded.payment_hash,
                cln_rpc::primitives::Amount::from_msat(amount_msat),
                decoded
                    .payment_secret
                    .map(|e| e[..].to_vec())
                    .ok_or(anyhow!("missing payment secret"))?
                    .try_into()?,
                payload_hex,
                as_option(req.maxdelay),
            )
            .await
        });
        part_id += 1;
        handles.push(handle);
    }

    let results = join_all(handles).await;
    let mut payment_preimage = None;
    for result in results {
        let response = result??;
        if let Some(preimage) = response.payment_preimage {
            payment_preimage = Some(preimage);
        }
    }

    if let Some(payment_preimage) = payment_preimage {
        Ok(cln_rpc::model::responses::PayResponse {
            destination: Some(decoded.payee),
            warning_partial_completion: None,
            status: cln_rpc::model::responses::PayStatus::COMPLETE,
            amount_msat: cln_rpc::primitives::Amount::from_msat(amount_msat),
            amount_sent_msat: cln_rpc::primitives::Amount::from_msat(amount_msat),
            created_at: 0.,
            parts: alloc.len() as u32,
            payment_hash: decoded.payment_hash,
            payment_preimage,
        })
    } else {
        Err(anyhow!("missing payment_preimage"))
    }
}

async fn do_pay(
    rpc: &mut ClnRpc,
    node_id: PublicKey,
    bolt11: String,
    label: String,
    description: Option<String>,
    part_amt: u64,
    scid: ShortChannelId,
    part_id: u64,
    group_id: u64,
    payment_hash: cln_rpc::primitives::Sha256,
    total_amount: Amount,
    payment_secret: cln_rpc::primitives::Secret,
    payment_metadata: String,
    max_delay: Option<u32>,
) -> Result<cln_rpc::model::responses::WaitsendpayResponse> {
    let route = cln_rpc::model::requests::SendpayRoute {
        amount_msat: cln_rpc::primitives::Amount::from_msat(part_amt),
        id: node_id.clone(),
        delay: max_delay.unwrap_or(MAX_DELAY_DEFAULT) as u16,
        channel: scid,
    };

    debug!(
        "Trampoline payment part_id={} with amount={}, using route={:?}",
        part_id, part_amt, route
    );

    let _r: serde_json::Value = rpc
        .call_raw(
            "sendpay",
            &SendpayRequest {
                route: vec![route],
                payment_hash,
                label: as_option(label),
                amount_msat: Some(total_amount),
                bolt11: Some(bolt11),
                payment_secret: Some(payment_secret),
                partid: Some(part_id),
                localinvreqid: None,
                groupid: Some(group_id),
                description,
                payment_metadata: Some(payment_metadata),
            },
        )
        .await?;

    match rpc
        .call_typed(&cln_rpc::model::requests::WaitsendpayRequest {
            payment_hash: payment_hash,
            timeout: None,
            partid: Some(part_id),
            groupid: Some(group_id),
        })
        .await
    {
        Ok(v) => Ok(v),
        Err(e) => {
            if let Some(code) = e.code {
                if code == PAY_UNPARSEABLE_ONION_CODE {
                    return Err(anyhow!("trampoline payment failed by the server"));
                }
            } else if e.message == PAY_UNPARSEABLE_ONION_MSG {
                return Err(anyhow!("trampoline payment failed by the server"));
            }
            Err(e.into())
        }
    }
}

async fn reestablished_channels(
    channels: Vec<Channel>,
    node_id: PublicKey,
    rpc_path: PathBuf,
    deadline: Instant,
) -> Result<Vec<Channel>> {
    // Wait for channels to re-establish.
    crate::awaitables::assert_send(AwaitableChannel::new(
        node_id,
        ShortChannelId::from_str("1x1x1")?,
        rpc_path.clone(),
    ));
    let mut futures = Vec::new();
    for c in &channels {
        let rp = rpc_path.clone();
        futures.push(
            async move {
                timeout_at(
                    deadline,
                    AwaitableChannel::new(node_id, c.short_channel_id, rp),
                )
                .await
            }
            .boxed(),
        );
    }

    log::info!(
        "Starting {} tasks to wait for channels to be ready",
        futures.len()
    );

    let results = join_all(futures).await;
    Ok(results
        .into_iter()
        .zip(channels)
        .filter_map(|(result, channel_data)| match result {
            Ok(_amount) => Some(channel_data),
            _ => None,
        })
        .collect::<Vec<Channel>>())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Channel {
    short_channel_id: cln_rpc::primitives::ShortChannelId,
    spendable_msat: u64,
    min_htlc_out_msat: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChannelContribution<'a> {
    channel: &'a Channel,
    contrib_msat: u64,
}

/// Finds an allocation that covers the target amount using as few channels as
/// possible.
fn find_minimal_allocation<'a>(
    channels: &'a [Channel],
    target_msat: u64,
) -> Option<Vec<ChannelContribution<'a>>> {
    // We can not allocate channels for a zero amount.
    if target_msat == 0 {
        return None;
    }

    // We'll use recursion with backtracking.
    // - `idx` is the current channel index.
    // - `target_msat` is the remaining amount.
    // - `current` collects the allocations so far.
    // - `best` stores the best solution found.
    fn rec<'a>(
        channels: &'a [Channel],
        idx: usize,
        target_msat: u64,
        current: &mut Vec<ChannelContribution<'a>>,
        best: &mut Option<Vec<ChannelContribution<'a>>>,
    ) {
        // Base case: If we've exactly allocated the target, record the solution.
        if target_msat == 0 {
            if best.is_none() || current.len() < best.as_ref().unwrap().len() {
                *best = Some(current.clone())
            }
            return;
        }

        // Base case: We have processed all channels and could not find a
        // solution.
        if idx >= channels.len() {
            return;
        }

        // Option 1: Skip the current channel.
        rec(channels, idx + 1, target_msat, current, best);

        // Option 2: Use the current channel.
        let ch = &channels[idx];

        // Each channel has an upper and a lower bound defined by the minimum
        // htlc amount and the spendable amount.
        let lower = ch.min_htlc_out_msat;
        let upper = ch.spendable_msat.min(target_msat);

        // If the remaining target amount is below the channel's lower bound we
        // can not use it.
        if target_msat < lower {
            return;
        }

        // Try allocations from the upper bound down to the lower bound to
        // maximize channel usage.
        for alloc in (lower..=upper).rev() {
            current.push(ChannelContribution {
                channel: ch,
                contrib_msat: alloc,
            });
            rec(channels, idx + 1, target_msat - alloc, current, best);
            current.pop();
            // Early exit: If we've found a one-channel solution, that's
            // optimal.
            if best.as_ref().map_or(false, |s| s.len() == 1) {
                return;
            }
        }
    }

    let mut best = None;
    let mut current = Vec::new();
    rec(channels, 0, target_msat, &mut current, &mut best);
    best
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendpayRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_msat: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bolt11: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groupid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localinvreqid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_secret: Option<cln_rpc::primitives::Secret>,
    pub payment_hash: cln_rpc::primitives::Sha256,
    pub route: Vec<cln_rpc::model::requests::SendpayRoute>,
}

#[cfg(test)]
mod channel_allocation_tests {
    use super::*;

    #[test]
    fn single_channel_exact_amount() {
        // A single channel that can exactly cover the amount.
        let channels = [Channel {
            short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
            spendable_msat: 5_000,
            min_htlc_out_msat: 1_000,
        }];
        let target_msat = 5_000;
        let allocation =
            find_minimal_allocation(&channels, target_msat).expect("Allocation should succeed");
        assert_eq!(allocation.len(), 1, "Only one channel needed");
        assert_eq!(
            allocation[0].contrib_msat, 5_000,
            "Full amount should be allocated"
        );
        // Verify constraints.
        assert!(
            allocation[0].contrib_msat >= channels[0].min_htlc_out_msat
                && allocation[0].contrib_msat <= channels[0].spendable_msat,
        );
    }

    #[test]
    fn multiple_channels_exact_sum() {
        // Two channels whose spendable amounts sum exactlyu to the target
        // amount.
        let channels = [
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
                spendable_msat: 8_000,
                min_htlc_out_msat: 500,
            },
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x101x0").unwrap(),
                spendable_msat: 7_000,
                min_htlc_out_msat: 500,
            },
        ];
        let target_msat = 15_000;
        let allocation =
            find_minimal_allocation(&channels, target_msat).expect("Allocation should succeed");
        assert_eq!(
            allocation.len(),
            2,
            "Both channels should be used to cover the amount"
        );
        assert_eq!(
            allocation[0].contrib_msat, 8_000,
            "Channel 0 should contribute 8_000 msat"
        );
        assert_eq!(
            allocation[1].contrib_msat, 7_000,
            "Channel 1 should contribute 7_000 msat"
        );
        // Verify constraints.
        let total_allocated: u64 = allocation.iter().map(|ch| ch.contrib_msat).sum();
        assert_eq!(
            total_allocated, target_msat,
            "Total allocated should equal the target amount"
        );
        assert!(
            allocation[0].contrib_msat >= channels[0].min_htlc_out_msat
                && allocation[0].contrib_msat <= channels[0].spendable_msat,
        );
        assert!(
            allocation[1].contrib_msat >= channels[1].min_htlc_out_msat
                && allocation[1].contrib_msat <= channels[1].spendable_msat,
        );
    }

    #[test]
    fn single_channel_below_min_htlc() {
        // Channel has enough spendable, but target_msat is below channel's
        // minimum HTLC value.
        let channels = [Channel {
            short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
            spendable_msat: 20_000,
            min_htlc_out_msat: 15_000,
        }];
        let target_msat = 10_000;
        assert!(
            find_minimal_allocation(&channels, target_msat).is_none(),
            "Should fail to allocate below minimum HTLC"
        );
    }

    #[test]
    fn min_htlc_constraints_multiple_channels() {
        // Two channels where each has a significant min_htlc_out_msat.
        // Total liquidity is enough, but we must split carefully to meet each
        // min.
        let channels = [
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
                spendable_msat: 200,
                min_htlc_out_msat: 110,
            },
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x101x0").unwrap(),
                spendable_msat: 150,
                min_htlc_out_msat: 100,
            },
        ];
        let target_msat = 250;
        // Channel 0 alone cannot cover (200 < 250), channel 1 alone cannot
        // (150 < 250). Both needed, but each must send at least their min
        // (100 and 110) respectively.
        // One valid allocation is channel0 -> 150, channel1 -> 100 (total 250).
        let allocation =
            find_minimal_allocation(&channels, target_msat).expect("Allocation should succeed");
        assert_eq!(
            allocation.len(),
            2,
            "Both channels are required for this payment"
        );
        // Verify constraints.
        allocation.iter().for_each(|ch| {
            assert!(ch.contrib_msat <= ch.channel.spendable_msat);
            assert!(ch.contrib_msat >= ch.channel.min_htlc_out_msat);
        });
        // Verify total equals target
        let total: u64 = allocation.iter().map(|ch| ch.contrib_msat).sum();
        assert_eq!(total, target_msat);
    }

    #[test]
    fn zero_target_amount() {
        // A zero amount should not allocate from any channel.
        let channels = [Channel {
            short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
            spendable_msat: 20_000,
            min_htlc_out_msat: 15_000,
        }];
        let target_msat = 0;
        assert!(
            find_minimal_allocation(&channels, target_msat).is_none(),
            "Should fail to allocate channels for an amount of 0 msat"
        );
    }

    #[test]
    fn insufficient_total_liquidity() {
        // Total available(2_000) is less than target_msat (2_500)
        let channels = [
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
                spendable_msat: 1_500,
                min_htlc_out_msat: 1,
            },
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x101x0").unwrap(),
                spendable_msat: 500,
                min_htlc_out_msat: 1,
            },
        ];
        let target_msat = 2_500;
        assert!(find_minimal_allocation(&channels, target_msat).is_none());
    }

    #[test]
    fn all_channels_min_htlc_too_high() {
        // Each channel's min_htlc_out_msat is higher than the target amount,
        // so none can send out an HTLC.
        let channels = [
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
                spendable_msat: 10_000,
                min_htlc_out_msat: 6_000,
            },
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x101x0").unwrap(),
                spendable_msat: 8_000,
                min_htlc_out_msat: 6_000,
            },
        ];
        let target_msat = 5_000;
        assert!(find_minimal_allocation(&channels, target_msat).is_none());
    }

    #[test]
    fn channel_has_zero_spendable() {
        // We have some channels with 0 spendable_msat and we do not want to
        // allocate them with 0 amount HTLCs.
        let channels = [
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x100x0").unwrap(),
                spendable_msat: 0,
                min_htlc_out_msat: 0,
            },
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x102x0").unwrap(),
                spendable_msat: 5_000,
                min_htlc_out_msat: 0,
            },
            Channel {
                short_channel_id: ShortChannelId::from_str("100000x101x0").unwrap(),
                spendable_msat: 0,
                min_htlc_out_msat: 0,
            },
        ];
        let target_msat = 5_000;
        let alloc =
            find_minimal_allocation(&channels, target_msat).expect("Should be able to allocate");
        assert_eq!(alloc.len(), 1, "Should have only included one channel");
    }
}
