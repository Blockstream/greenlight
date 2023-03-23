//! LSP integrations and related code.

use crate::Plugin;
use anyhow::Context;
use bytes::Buf;
use cln_rpc::primitives::{Amount, ShortChannelId, TlvEntry, TlvStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Onion {
    payload: tlv::SerializedTlvStream,
    short_channel_id: Option<ShortChannelId>,
    forward_msat: Amount,
    outgoing_cltv_value: u32,
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
    //amount_msat: u64,
    //cltv_expiry: u32,
    //cltv_expiry_relative: u16,
    #[serde(deserialize_with = "from_hex")]
    payment_hash: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct HtlcAcceptedRequest {
    onion: Onion,
    //htlc: Htlc,
    //forward_to: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct HtlcAcceptedResponse {
    result: String,

    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "to_hex")]
    payload: Option<Vec<u8>>,
}

pub async fn on_htlc_accepted(_p: Plugin, v: Value) -> Result<Value, anyhow::Error> {
    let req: HtlcAcceptedRequest = serde_json::from_value(dbg!(v)).unwrap();

    eprintln!("XXX {:?}", req);
    serde_json::to_value(HtlcAcceptedResponse {
        result: "continue".to_string(),
        payload: None,
    })
    .context("serializing response")
}

mod tlv {
    use bytes::Buf;
    use cln_rpc::primitives::TlvEntry;
    use serde::{Deserialize, Deserializer};

    /// A standalone type the represent a binary serialized
    /// TlvStream. This is distinct from TlvStream since that expects TLV
    /// streams to be encoded as maps in JSON.
    #[derive(Debug)]
    pub struct SerializedTlvStream {
        entries: Vec<TlvEntry>,
    }
    impl<'de> Deserialize<'de> for SerializedTlvStream {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Start by reading the hex-encoded string
            let s: String = Deserialize::deserialize(deserializer)?;
            let mut b: bytes::Bytes = hex::decode(s)
                .map_err(|e| serde::de::Error::custom(e.to_string()))?
                .into();

            // Skip the length prefix
            match b.get_u8() {
                253 => {
                    b.get_u16();
                }
                254 | 255 => return Err(serde::de::Error::custom("TLV stream longer than u16")),
                _ => {}
            }

            let mut entries: Vec<TlvEntry> = vec![];

            while b.remaining() >= 2 {
                let typ: u64 = match b.get_u8() {
                    253 => b.get_u16().into(),
                    254 => b.get_u32().into(),
                    255 => b.get_u64(),
                    v => v.into(),
                };
                let len: usize = match b.get_u8() {
                    253 => b.get_u16().into(),
                    254 | 255 => {
                        return Err(serde::de::Error::custom("TLV length larger than u16"));
                    }
                    v => v.into(),
                };
                let value = b.copy_to_bytes(len).to_vec();
                entries.push(TlvEntry { typ, value });
            }

            Ok(dbg!(SerializedTlvStream { entries }))
        }
    }
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
