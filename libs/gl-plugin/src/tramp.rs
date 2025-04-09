use crate::awaitables::{AwaitableChannel, AwaitablePeer};
use crate::pb;
use anyhow::{anyhow, Result};
use cln_rpc::{
    primitives::{Amount, PublicKey, ShortChannelId},
    ClnRpc,
};
use futures::{future::join_all, FutureExt};
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
    let hex_node_id = hex::encode(node_id.serialize());

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
        hex_node_id,
        req.bolt11.clone()
    );

    // Wait for the peer connection to re-establish.
    log::debug!("Await peer connection to {}", hex_node_id);
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
        .ok_or_else(|| anyhow!("node with id {} is unknown", hex_node_id))?
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

    let channels: Vec<Channel> = rpc
        .call_typed(&cln_rpc::model::requests::ListpeerchannelsRequest { id: Some(node_id) })
        .await?
        .channels
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

    // Check if we actually got a channel to the trampoline node.
    if channels.is_empty() {
        return Err(anyhow!("Has no channels with trampoline node"));
    }

    // Await and filter out re-established channels.
    let deadline = Instant::now() + Duration::from_secs(AWAIT_CHANNELS_TIMEOUT_SEC);
    let mut channels =
        reestablished_channels(channels, node_id, rpc_path.as_ref().to_path_buf(), deadline)
            .await?;

    // Try different allocation strategies in sequence. First try in ascending
    // order of spendable_msat, giving us most drained channels first. Then
    // try in descending order of spendable_msat giving us the channels with the
    // biggest local balance first.
    debug!(
        "Trying to allocate {}msat accross {} channels in ascending order",
        amount_msat,
        channels.len()
    );
    let alloc = match find_allocation_ascending_order(&mut channels, amount_msat)
        .filter(|alloc| !alloc.is_empty())
    {
        Some(alloc) => alloc,
        None => {
            debug!("Failed to allocate {}msat in ascending channel order {:?}, trying in descending order",amount_msat, &channels);
            match find_allocation_descending_order(&mut channels, amount_msat)
                .filter(|alloc| !alloc.is_empty())
            {
                Some(alloc) => alloc,
                None => {
                    return Err(anyhow!(
                        "could not allocate enough funds across channels {}msat<{}msat",
                        channels.iter().map(|ch| ch.spendable_msat).sum::<u64>(),
                        amount_msat
                    ));
                }
            }
        }
    };

    // All set we can preapprove the invoice
    let _ = rpc
        .call_typed(&cln_rpc::model::requests::PreapproveinvoiceRequest {
            bolt11: req.bolt11.clone(),
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
        delay: max_delay.unwrap_or(MAX_DELAY_DEFAULT),
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

// Finds a payment allocation by sorting channels in descending order of
// spendable amount.
///
/// This strategy prioritizes channels with the most funds first, which tends to
/// minimize the number of channels used for large payments. For each spendable
/// amount, it further prioritizes channels with smaller minimum HTLC
/// requirements.
fn find_allocation_descending_order<'a>(
    channels: &'a mut [Channel],
    target_msat: u64,
) -> Option<Vec<ChannelContribution<'a>>> {
    // We sort in descending order for spendable_msat and ascending for the
    // min_htlc_out_msat, which means that we process the channels with the
    // biggest local funds first.
    channels.sort_by(|a, b| {
        b.spendable_msat
            .cmp(&a.spendable_msat)
            .then_with(|| a.min_htlc_out_msat.cmp(&b.min_htlc_out_msat))
    });

    find_allocation(channels, target_msat)
}

/// Finds a payment allocation by sorting channels in ascending order of
/// spendable amount.
///
/// This strategy prioritizes draining smaller channels first, which can help
/// consolidate funds into fewer channels. For each spendable amount,
/// it further prioritizes channels with smaller minimum HTLC requirements.
fn find_allocation_ascending_order<'a>(
    channels: &'a mut [Channel],
    target_msat: u64,
) -> Option<Vec<ChannelContribution<'a>>> {
    // We sort in ascending order for spendable_msat and min_htlc_out_msat,
    // which means that we process the smallest channels first.
    channels.sort_by(|a, b| {
        a.spendable_msat
            .cmp(&b.spendable_msat)
            .then_with(|| a.min_htlc_out_msat.cmp(&b.min_htlc_out_msat))
    });

    find_allocation(channels, target_msat)
}

/// Finds an allocation that covers the target amount while respecting channel
/// constraints.
///
/// This function implements a recursive backtracking algorithm that attempts to
/// allocate funds from channels in the order they are provided. It handles
/// complex scenarios where channel minimum requirements may need cascading
/// adjustments to find a valid solution.
///
/// # Algorithm Details
///
/// The algorithm works by:
/// 1. Trying to allocate the maximum possible from each channel
/// 2. If a channel's minimum exceeds the remaining target, it tries to skip
///    that channel
/// 3. When a channel minimum can't be met, it backtracks and adjusts previous
///    allocations
/// 4. It uses a cascading approach to free up just enough space from previous
///    channels
fn find_allocation<'a>(
    channels: &'a [Channel],
    target_msat: u64,
) -> Option<Vec<ChannelContribution<'a>>> {
    // We can not allocate channels for a zero amount.
    if target_msat == 0 {
        return None;
    }

    /// Result type for the recursive allocation function
    enum AllocResult {
        /// Allocation succeeded
        Success,
        /// Allocation is impossible with current channels
        Impossible,
        /// Need more space (in msat) to satisfy minimum requirements
        NeedSpace(u64),
    }

    /// Recursive helper function that tries to find a valid allocation
    ///
    /// # Arguments
    /// * `channels` - Remaining channels to consider
    /// * `target_msat` - Remaining amount to allocate
    /// * `allocations` - Current allocation state (modified in-place)
    fn try_allocate<'a>(
        channels: &'a [Channel],
        target_msat: u64,
        allocations: &mut Vec<ChannelContribution<'a>>,
    ) -> AllocResult {
        // Base case: If we've exactly allocated the target, we found a solution.
        if target_msat == 0 {
            return AllocResult::Success;
        }

        // Check that we have channels left to allocate from.
        if channels.is_empty() {
            return AllocResult::Impossible;
        }

        // Try to use the current channel (smallest amount) first.
        let ch = &channels[0];

        // Channel is drained or unusable, skip it.
        if ch.spendable_msat < ch.min_htlc_out_msat || ch.spendable_msat == 0 {
            return try_allocate(&channels[1..], target_msat, allocations);
        }

        // Each channel has an upper and a lower bound defined by the minimum
        // HTLC amount and the spendable amount.
        let lower = ch.min_htlc_out_msat;
        let upper = ch.spendable_msat.min(target_msat);

        // We need a higher target amount.
        if target_msat < lower {
            // First we try skipping this channel to see if later channels can
            // handle it.
            match try_allocate(&channels[1..], target_msat, allocations) {
                AllocResult::Success => return AllocResult::Success,
                // If that doesn't work, we need space from earlier allocations
                _ => return AllocResult::NeedSpace(lower - target_msat),
            }
        }

        // We can allocate from this channel - try max amount first.
        allocations.push(ChannelContribution {
            channel: ch,
            contrib_msat: upper,
        });

        // Try to allocate the remaining amount from subsequent channels.
        match try_allocate(&channels[1..], target_msat - upper, allocations) {
            // Success! We're done.
            AllocResult::Success => return AllocResult::Success,

            // No solution possible with current allocations
            AllocResult::Impossible => return AllocResult::Impossible,

            // Need to free up space
            AllocResult::NeedSpace(shortfall) => {
                // Calculate how much we can free from this allocation.
                let free = upper - lower;
                if shortfall <= free {
                    // We can cover the shortfall with free space in this channel
                    allocations.pop();
                    let adjusted_amount = upper - shortfall;
                    allocations.push(ChannelContribution {
                        channel: ch,
                        contrib_msat: adjusted_amount,
                    });

                    // Try allocation with the adjusted amount.
                    match try_allocate(&channels[1..], target_msat - adjusted_amount, allocations) {
                        AllocResult::Success => return AllocResult::Success,
                        _ => {
                            // If that still don't work skip this channel completely.
                            // NOTE: We could also try to skip the next channel.
                            allocations.pop();
                            return try_allocate(&channels[1..], target_msat, allocations);
                        }
                    }
                } else {
                    // We can't fully cover the shortfall, need to pass up a remainder.
                    allocations.pop();
                    return AllocResult::NeedSpace(shortfall - free);
                }
            }
        };
    }

    let mut allocations = Vec::with_capacity(channels.len());
    match try_allocate(channels, target_msat, &mut allocations) {
        AllocResult::Success => Some(allocations),
        _ => None,
    }
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

    fn create_channel(
        short_channel_id: ShortChannelId,
        spendable_msat: u64,
        min_htlc_out_msat: u64,
    ) -> Channel {
        Channel {
            short_channel_id,
            spendable_msat,
            min_htlc_out_msat,
        }
    }

    fn scid(s: &str) -> ShortChannelId {
        ShortChannelId::from_str(s).unwrap()
    }

    fn verify_allocation(
        allocations: &[ChannelContribution],
        target_msat: u64,
        expected_scids: &[ShortChannelId],
    ) {
        // Check that the total amount matches.
        let total: u64 = allocations.iter().map(|ch| ch.contrib_msat).sum();
        assert_eq!(total, target_msat);

        // Check that the expected order matches.
        for (i, alloc) in allocations.iter().enumerate() {
            let ch = alloc.channel;
            if i < expected_scids.len() {
                assert_eq!(ch.short_channel_id, expected_scids[i]);
            }

            // Check that the lower and upper limits have been respected.
            assert!(alloc.contrib_msat >= ch.min_htlc_out_msat);
            assert!(alloc.contrib_msat <= ch.spendable_msat);
        }
    }

    #[test]
    fn zero_target_amount() {
        // A target sum of 0 should return None.
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
        ];

        let result = find_allocation(&channels, 0);
        assert_eq!(result, None, "Zero target should return None");
    }

    #[test]
    fn single_channel_exact_amount() {
        // A single channel that can take the exact amount.
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
        ];

        let result = find_allocation(&channels, 1_000);
        assert!(result.is_some(), "Should find an allocation");

        let allocations = result.unwrap();
        verify_allocation(&allocations, 1_000, &[scid("1x1x1")]);
    }

    #[test]
    fn single_channel_partial() {
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
        ];

        let result = find_allocation(&channels, 500);
        assert!(result.is_some(), "Should find an allocation");

        let allocations = result.unwrap();
        verify_allocation(&allocations, 500, &[scid("1x1x1")]);
        assert_eq!(allocations[0].contrib_msat, 500);
    }

    #[test]
    fn multiple_channels_simple() {
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
            create_channel(scid("3x1x1"), 3_000, 300),
        ];

        let result = find_allocation(&channels, 2_500);
        assert!(result.is_some(), "Should find an allocation");

        let allocations = result.unwrap();
        verify_allocation(&allocations, 2_500, &[scid("1x1x1"), scid("2x1x1")]);
        assert_eq!(allocations[0].contrib_msat, 1_000); // Use all of first channel
        assert_eq!(allocations[1].contrib_msat, 1_500); // Use part of second channel
    }

    #[test]
    fn minimum_constraint() {
        // Target is below channel's minimum
        let channels = vec![create_channel(scid("1x1x1"), 1_000, 500)];

        let result = find_allocation(&channels, 400);
        assert_eq!(result, None, "Can't allocate below minimum");
    }

    #[test]
    fn not_enough_funds() {
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
        ];

        let result = find_allocation(&channels, 5_000);
        assert_eq!(result, None, "Can't allocate more than total available");
    }

    #[test]
    fn adjusting_for_minimum_simple() {
        // Need to adjust allocation to meet minimum of next channel
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 600),
        ];

        // Target 1500 would normally use 1000 from channel 1,
        // leaving 500 for channel 2, but channel 2 needs at least 600
        let result = find_allocation(&channels, 1_500);
        assert!(result.is_some(), "Should find an allocation by adjusting");

        let allocations = result.unwrap();
        verify_allocation(&allocations, 1_500, &[scid("1x1x1"), scid("2x1x1")]);
        assert_eq!(allocations[0].contrib_msat, 900); // Reduced from 1000
        assert_eq!(allocations[1].contrib_msat, 600); // Minimum of channel 2
    }

    #[test]
    fn cascading_adjustment() {
        // Need to adjust multiple channels to satisfy constraints
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 1_500, 1_300),
            create_channel(scid("3x1x1"), 2_000, 800),
        ];

        let result = find_allocation(&channels, 3_000);
        assert!(
            result.is_some(),
            "Should find an allocation by cascading adjustment"
        );

        let allocations = result.unwrap();
        verify_allocation(
            &allocations,
            3_000,
            &[scid("1x1x1"), scid("2x1x1"), scid("3x1x1")],
        );

        // Verify channel 3 gets at least its minimum
        assert_eq!(allocations.len(), 3);
        assert_eq!(allocations[0].contrib_msat, 900);
        assert_eq!(allocations[1].contrib_msat, 1_300);
        assert_eq!(allocations[2].contrib_msat, 800);
    }

    #[test]
    fn complex_adjustment() {
        // Complex case requiring multiple adjustments
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 300),
            create_channel(scid("2x1x1"), 1_200, 500),
            create_channel(scid("3x1x1"), 1_500, 700),
            create_channel(scid("4x1x1"), 2_000, 1_000),
        ];

        let result = find_allocation(&channels, 4_000);
        assert!(
            result.is_some(),
            "Should find an allocation for complex case"
        );

        let allocations = result.unwrap();
        verify_allocation(
            &allocations,
            4_000,
            &[scid("1x1x1"), scid("2x1x1"), scid("3x1x1"), scid("4x1x1")],
        );

        assert_eq!(allocations.len(), 4);
        assert_eq!(allocations[0].contrib_msat, 1_000);
        assert_eq!(allocations[1].contrib_msat, 1_200);
        assert_eq!(allocations[2].contrib_msat, 800);
        assert_eq!(allocations[3].contrib_msat, 1_000);
    }

    #[test]
    fn skip_channel() {
        // Case where we need to skip a channel with higher minimum
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 900),
            create_channel(scid("2x1x1"), 1_500, 800),
            create_channel(scid("3x1x1"), 2_000, 200),
        ];

        // For target 1500, we should skip channel 2 and use 1 and 3
        let result = find_allocation(&channels, 1_500);
        assert!(
            result.is_some(),
            "Should find an allocation by skipping a channel"
        );

        let allocations = result.unwrap();

        // We expect to use channels 1 and 3, not channel 2
        verify_allocation(&allocations, 1_500, &[scid("1x1x1"), scid("3x1x1")]);
        // Skip specific ID check
    }

    #[test]
    fn exact_minimum_allocation() {
        let channels = vec![
            create_channel(scid("1x1x1"), 999, 500),
            create_channel(scid("2x1x1"), 1_000, 500),
        ];
        let result = find_allocation(&channels, 1_000);
        assert!(result.is_some());
        let allocations = result.unwrap();
        verify_allocation(&allocations, 1_000, &[scid("1x1x1"), scid("2x1x1")]);
        // Both should be at their minimums
        assert_eq!(allocations[0].contrib_msat, 500);
        assert_eq!(allocations[1].contrib_msat, 500);
    }

    #[test]
    fn all_channels_at_maximum() {
        let channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
            create_channel(scid("3x1x1"), 3_000, 300),
        ];
        let result = find_allocation(&channels, 6_000);
        assert!(result.is_some());
        let allocations = result.unwrap();
        verify_allocation(
            &allocations,
            6_000,
            &[scid("1x1x1"), scid("2x1x1"), scid("3x1x1")],
        );
        assert_eq!(allocations[0].contrib_msat, 1_000);
        assert_eq!(allocations[1].contrib_msat, 2_000);
        assert_eq!(allocations[2].contrib_msat, 3_000);
    }

    #[test]
    fn drained_channel_skip() {
        let channels = vec![
            create_channel(scid("1x1x1"), 50, 100), // Spendable < min_htlc
            create_channel(scid("2x1x1"), 1_000, 100),
        ];
        let result = find_allocation(&channels, 500);
        assert!(result.is_some());
        let allocations = result.unwrap();
        verify_allocation(&allocations, 500, &[scid("2x1x1")]);
    }

    #[test]
    fn zero_spendable_skip() {
        // We have some channels with 0 spendable_msat and we do not want to
        // allocate them with 0 amount HTLCs.
        let channels = [
            create_channel(scid("1x1x1"), 0, 0),
            create_channel(scid("2x1x1"), 5_000, 0),
        ];
        let target_msat = 5_000;
        let allocations =
            find_allocation(&channels, target_msat).expect("Should be able to allocate");
        verify_allocation(&allocations, 5_000, &[scid("2x1x1")]);
    }

    #[test]
    fn respects_channel_order() {
        // Same channels but different order should produce different allocations
        let channels1 = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
        ];
        let channels2 = vec![
            create_channel(scid("2x1x1"), 2_000, 200),
            create_channel(scid("1x1x1"), 1_000, 100),
        ];

        let result1 = find_allocation(&channels1, 1_500);
        let result2 = find_allocation(&channels2, 1_500);

        assert!(result1.is_some());
        assert!(result2.is_some());

        let alloc1 = result1.unwrap();
        let alloc2 = result2.unwrap();

        // The allocations should be different because channel order is different
        assert_eq!(alloc1[0].channel.short_channel_id, scid("1x1x1"));
        assert_eq!(alloc2[0].channel.short_channel_id, scid("2x1x1"));
    }

    #[test]
    fn test_ascending_order() {
        let mut channels = vec![
            create_channel(scid("2x1x1"), 2_000, 200),
            create_channel(scid("1x1x1"), 1_000, 100),
        ];

        let result = find_allocation_ascending_order(&mut channels, 1_500);
        assert!(result.is_some());
        let allocations = result.unwrap();

        // Should use the smallest channel first
        assert_eq!(allocations[0].channel.short_channel_id, scid("1x1x1"));
    }

    #[test]
    fn test_descending_order() {
        let mut channels = vec![
            create_channel(scid("1x1x1"), 1_000, 100),
            create_channel(scid("2x1x1"), 2_000, 200),
        ];

        let result = find_allocation_descending_order(&mut channels, 1_500);
        assert!(result.is_some());
        let allocations = result.unwrap();

        // Should use the largest channel first
        assert_eq!(allocations[0].channel.short_channel_id, scid("2x1x1"));
    }
}
