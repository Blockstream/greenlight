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
// Minimum amount we can send through a channel.
const MIN_HTLC_AMOUNT: u64 = 1;

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
    // introduce a race of the choosen channels for now.
    let choosen = pick_channels(amount_msat, channels)?;

    // FIXME should not be neccessary as we already check on the amount.
    let parts = choosen.len();
    if parts == 0 {
        return Err(anyhow!("no channels found to send"));
    }

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

    let mut part_id = if choosen.len() == 1 { 0 } else { 1 };
    let group_id = max_group_id + 1;
    let mut handles: Vec<
        tokio::task::JoinHandle<
            std::result::Result<cln_rpc::model::responses::WaitsendpayResponse, anyhow::Error>,
        >,
    > = vec![];
    for (scid, part_amt) in choosen {
        let bolt11 = req.bolt11.clone();
        let label = req.label.clone();
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
            parts: parts as u32,
            payment_hash: decoded.payment_hash,
            payment_preimage,
        })
    } else {
        Err(anyhow!("missing payment_preimage"))
    }
}

fn pick_channels(
    amount_msat: u64,
    mut channels: Vec<ChannelData>,
) -> Result<Vec<(cln_rpc::primitives::ShortChannelId, u64)>> {
    let mut acc = 0;
    let mut choosen = vec![];
    while let Some(channel) = channels.pop() {
        if acc == amount_msat {
            break;
        }

        // Filter out channels that lack minimum funds and can not send an htlc.
        if std::cmp::max(MIN_HTLC_AMOUNT, channel.min_htlc_out_msat) > channel.spendable_msat {
            debug!("Skip channel {}: has spendable_msat={} and minimum_htlc_out_msat={} and can not send htlc.",
                channel.short_channel_id,
                channel.spendable_msat,
                channel.min_htlc_out_msat,
            );
            continue;
        }

        if (channel.spendable_msat + acc) <= amount_msat {
            choosen.push((channel.short_channel_id, channel.spendable_msat));
            acc += channel.spendable_msat;
        } else {
            let rest = amount_msat - acc;
            choosen.push((channel.short_channel_id, rest));
            acc += rest;
            break;
        }
    }

    if acc < amount_msat {
        return Err(anyhow!("missing balance {}msat<{}msat", acc, amount_msat));
    }

    Ok(choosen)
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
mod tests {
    use super::*;

    #[test]
    fn test_picking_channels() {
        let scid1 = ShortChannelId::from_str("100000x100x0").unwrap();
        let scid2 = ShortChannelId::from_str("100000x101x0").unwrap();
        let scid3 = ShortChannelId::from_str("100000x102x0").unwrap();
        let scid4 = ShortChannelId::from_str("100000x103x0").unwrap();
        let channels = vec![
            ChannelData {
                short_channel_id: scid1,
                spendable_msat: 100000,
                min_htlc_out_msat: 0,
            },
            // Below MIN_HTLC_AMOUNT.
            ChannelData {
                short_channel_id: scid2,
                spendable_msat: 0,
                min_htlc_out_msat: 0,
            },
            // min_htlc_out_msat is larger than spendable_msat.
            ChannelData {
                short_channel_id: scid3,
                spendable_msat: 1,
                min_htlc_out_msat: 2,
            },
            ChannelData {
                short_channel_id: scid4,
                spendable_msat: 55000,
                min_htlc_out_msat: 55000,
            },
        ];

        let amount_msat = 150000;
        let choosen = pick_channels(amount_msat, channels).unwrap();

        assert_eq!(choosen.len(), 2);
        assert_eq!(choosen[0].0, scid4);
        assert_eq!(choosen[0].1, 55000);
        assert_eq!(choosen[1].0, scid1);
        assert_eq!(choosen[1].1, 95000);
    }
}
