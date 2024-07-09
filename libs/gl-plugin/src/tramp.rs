use crate::pb;
use anyhow::{anyhow, Result};
use cln_rpc::{
    primitives::{Amount, PublicKey, ShortChannelId},
    ClnRpc,
};
use futures::future::join_all;
use gl_client::bitcoin::hashes::hex::ToHex;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    let node_id = cln_rpc::primitives::PublicKey::from_slice(&req.trmp_node_id[..])?;

    log::debug!(
        "New trampoline payment via {}: {} ",
        hex::encode(req.trmp_node_id),
        req.bolt11.clone()
    );

    // Check if peer has signaled that they support forward trampoline pays:
    let mut rpc = ClnRpc::new(rpc_path.as_ref()).await?;
    let res = rpc
        .call_typed(&cln_rpc::model::requests::ListpeersRequest {
            id: Some(node_id),
            level: None,
        })
        .await?;

    if res.peers.is_empty() {
        return Err(anyhow!("node with id {} is unknown", node_id.to_hex()));
    }

    let features = hex::decode(
        res.peers[0]
            .features
            .as_ref()
            .ok_or_else(|| anyhow!("Could not extract features from peer object."))?,
    )?;

    feature_guard(features, TRAMPOLINE_FEATURE_BIT)?;

    // Extract the amount from the bolt11 or use the set amount field
    // Return an error if there is a mismatch.
    let decoded = rpc
        .call_typed(&cln_rpc::model::requests::DecodepayRequest {
            bolt11: req.bolt11.clone(),
            description: None,
        })
        .await?;

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
    let overpay = amount_msat as f64
        * (as_option(req.maxfeepercent).unwrap_or(DEFAULT_OVERPAY_PERCENT) as f64 / 100 as f64);
    let amount_msat = amount_msat + overpay as u64;

    debug!("overpay={}, total_amt={}", overpay, amount_msat);

    let res = rpc
        .call_typed(&cln_rpc::model::requests::ListpeerchannelsRequest { id: Some(node_id) })
        .await?;

    let channels = res.channels.unwrap_or_default();
    if channels.is_empty() {
        return Err(anyhow!("Has no channels with trampoline node"));
    }
    struct ChannelData {
        short_channel_id: cln_rpc::primitives::ShortChannelId,
        spendable_msat: u64,
    }

    let mut channels: Vec<ChannelData> = channels
        .into_iter()
        .filter_map(|ch| {
            let short_channel_id = match ch.short_channel_id {
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
            return Some(ChannelData {
                short_channel_id,
                spendable_msat,
            });
        })
        .collect::<Vec<ChannelData>>();

    channels.sort_by(|a, b| b.spendable_msat.cmp(&a.spendable_msat));

    let mut acc = 0;
    let mut choosen = vec![];
    while let Some(channel) = channels.pop() {
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

    // Check that we found enough spendables
    if acc < amount_msat {
        return Err(anyhow!("missing balance {}msat<{}msat", acc, amount_msat));
    }

    // FIXME should not be neccessary as we already check on the amount.
    let parts = choosen.len();
    if parts == 0 {
        return Err(anyhow!("no channels found to send"));
    }

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
    payload.set_tu64(TLV_AMT_MSAT, amount_msat);
    let payload_hex = hex::encode(SerializedTlvStream::to_bytes(payload));

    let mut part_id = if choosen.len() == 1 { 0 } else { 1 };
    let group_id = 1;
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
            timeout: Some(120),
            partid: Some(part_id),
            groupid: Some(group_id),
        })
        .await
    {
        Ok(r) => Ok(r),
        Err(e) => Err(anyhow!(e.to_string())),
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
