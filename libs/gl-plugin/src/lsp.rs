//! LSP integrations and related code.

use crate::{
    tlv::{self, ProtoBufMut},
    Plugin,
};
use anyhow::Context;
use bytes::BufMut;
use cln_rpc::primitives::{Amount, ShortChannelId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Onion {
    payload: tlv::SerializedTlvStream,
    short_channel_id: Option<ShortChannelId>,
    forward_msat: Option<Amount>,
    outgoing_cltv_value: Option<u32>,
    #[serde(deserialize_with = "from_hex")]
    shared_secret: Vec<u8>,
    #[serde(deserialize_with = "from_hex")]
    next_onion: Vec<u8>,
    total_msat: Option<Amount>,
}
#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Htlc {
    short_channel_id: ShortChannelId,
    //id: u64,
    amount_msat: Amount,
    //cltv_expiry: u32,
    //cltv_expiry_relative: u16,
    #[serde(deserialize_with = "from_hex")]
    payment_hash: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct HtlcAcceptedRequest {
    onion: Onion,
    htlc: Htlc,
    //forward_to: Vec<u8>,
}

#[derive(Debug, Serialize, Default)]
struct HtlcAcceptedResponse {
    result: String,

    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "to_hex")]
    payload: Option<Vec<u8>>,
}

const TLV_FORWARD_AMT: u64 = 2;
const TLV_PAYMENT_SECRET: u64 = 8;

/// A macro to break out of the current hook flow and return a `continue`
/// signal to core-lightning. This is to be used when we don't know how to
/// handle a given payload or as a shortcut in case we could identify that the
/// incoming htlc is not part of a LSP jit channel opening.
macro_rules! unwrap_or_continue {
    ($res:expr) => {
        match $res {
            Ok(x) => x,
            Err(e) => {
                log::debug!("Lsp-plugin continue, reason: {}", e.to_string());
                return Ok(serde_json::to_value(HtlcAcceptedResponse {
                    result: "continue".to_string(),
                    ..Default::default()
                })
                .expect("Could not serialize json value"));
            }
        }
    };
}

pub async fn on_htlc_accepted(plugin: Plugin, v: Value) -> Result<Value, anyhow::Error> {
    let req: HtlcAcceptedRequest = unwrap_or_continue!(serde_json::from_value(v));
    log::debug!("Decoded {:?}", &req);

    let htlc_amt = req.htlc.amount_msat;
    let onion_amt = unwrap_or_continue!(req.onion.forward_msat.ok_or(format!(
        "payload={} is missing forward_msat",
        &req.onion.payload
    )));

    let res = if htlc_amt.msat() < onion_amt.msat() {
        log::info!(
            "Potential JIT LSP payment detected: htlc_amount={}msat < onion_amount={}msat",
            htlc_amt.msat(),
            onion_amt.msat()
        );

        let mut payload = req.onion.payload.clone();
        payload.set_tu64(TLV_FORWARD_AMT, htlc_amt.msat());
        let payment_secret = unwrap_or_continue!(payload.get(TLV_PAYMENT_SECRET).ok_or(format!(
            "payload={} is missing payment_secret",
            &payload.to_string()
        )));

        let mut rpc = cln_rpc::ClnRpc::new(plugin.configuration().rpc_file).await?;
        let res: cln_rpc::model::responses::ListinvoicesResponse = rpc
            .call_typed(&cln_rpc::model::requests::ListinvoicesRequest {
                payment_hash: Some(hex::encode(&req.htlc.payment_hash)),
                label: None,
                offer_id: None,
                invstring: None,
                start: None,
                index: None,
                limit: None,
            })
            .await?;

        let invoice = unwrap_or_continue!(res.invoices.first().ok_or(format!(
            "no invoice matching incoming HTLC payment_hash={} found",
            hex::encode(&req.htlc.payment_hash),
        )));
        let total_msat = unwrap_or_continue!(invoice
            .amount_msat
            .ok_or("invoice has no total amount msat"));

        let mut ps = bytes::BytesMut::new();
        ps.put(&payment_secret.value[0..32]);
        ps.put_tu64(total_msat.msat());
        payload.set_bytes(TLV_PAYMENT_SECRET, ps);

        log::info!(
            "Amended onion payload with forward_amt={}msat and total_msat={}msat (from invoice)",
            htlc_amt.msat(),
            total_msat.msat(),
        );

        let payload = tlv::SerializedTlvStream::to_bytes(payload);
        log::debug!("Serialized payload: {}", hex::encode(&payload));

        use tlv::ToBytes;
        HtlcAcceptedResponse {
            result: "continue".to_string(),
            payload: Some(payload),
        }
    } else {
        log::info!("HTLC amount matches onion payload amount, deferring to lightningd");

        HtlcAcceptedResponse {
            result: "continue".to_string(),
            ..Default::default()
        }
    };

    serde_json::to_value(res).context("serialize result")
}

use hex::FromHex;
use serde::{Deserializer, Serializer};

/// Serializes `buffer` to a lowercase hex string.
pub fn to_hex<T, S>(buffer: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    match buffer {
        None => serializer.serialize_none(),
        Some(buffer) => serializer.serialize_str(&hex::encode(&buffer.as_ref())),
    }
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
pub fn from_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| Vec::from_hex(&string).map_err(|err| Error::custom(err.to_string())))
}
