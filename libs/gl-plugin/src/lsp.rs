//! LSP integrations and related code.

use crate::{lsp::tlv::ProtoBufMut, Plugin};
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

pub async fn on_htlc_accepted(plugin: Plugin, v: Value) -> Result<Value, anyhow::Error> {
    let req: HtlcAcceptedRequest = serde_json::from_value(v).unwrap();
    log::debug!("Decoded {:?}", &req);

    let htlc_amt = req.htlc.amount_msat;
    let onion_amt = req.onion.forward_msat;

    let res = if htlc_amt.msat() < onion_amt.msat() {
        log::info!(
            "Potential JIT LSP payment detected: htlc_amount={}msat < onion_amount={}msat",
            htlc_amt.msat(),
            onion_amt.msat()
        );

        let mut payload = req.onion.payload.clone();
        payload.set_tu64(TLV_FORWARD_AMT, htlc_amt.msat());
        let payment_secret = payload.get(TLV_PAYMENT_SECRET).unwrap();

        let mut rpc = cln_rpc::ClnRpc::new(plugin.configuration().rpc_file).await?;
        let res: cln_rpc::model::responses::ListinvoicesResponse = rpc
            .call_typed(cln_rpc::model::requests::ListinvoicesRequest {
                payment_hash: Some(hex::encode(&req.htlc.payment_hash)),
                label: None,
                offer_id: None,
                invstring: None,
            })
            .await?;
        if res.invoices.len() != 1 {
            log::warn!(
                "No invoice matching incoming HTLC payment_hash={} found, continuing",
                hex::encode(&req.htlc.payment_hash)
            );
            return Ok(serde_json::to_value(HtlcAcceptedResponse {
                result: "continue".to_string(),
                ..Default::default()
            })
            .unwrap());
        }

        let total_msat = res.invoices.iter().next().unwrap().amount_msat.unwrap();

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

// TODO(cdecker) This looks like a useful abrstraction to backport into cln-rpc?
pub mod tlv {
    use anyhow::anyhow;
    use bytes::{Buf, BufMut};
    use cln_rpc::primitives::TlvEntry;
    use serde::{Deserialize, Deserializer};

    /// A standalone type the represent a binary serialized
    /// TlvStream. This is distinct from TlvStream since that expects TLV
    /// streams to be encoded as maps in JSON.
    #[derive(Clone, Debug)]
    pub struct SerializedTlvStream {
        entries: Vec<TlvEntry>,
    }

    impl SerializedTlvStream {
        pub fn get(&self, typ: u64) -> Option<TlvEntry> {
            self.entries.iter().filter(|e| e.typ == typ).next().cloned()
        }

        pub fn insert(&mut self, e: TlvEntry) -> Result<(), anyhow::Error> {
            if let Some(old) = self.get(e.typ) {
                return Err(anyhow!(
                    "TlvStream contains entry of type={}, old={:?}, new={:?}",
                    e.typ,
                    old,
                    e
                ));
            }

            self.entries.push(e);
            self.entries
                .sort_by(|a, b| a.typ.partial_cmp(&b.typ).unwrap());

            Ok(())
        }

        pub fn set_bytes<T>(&mut self, typ: u64, val: T)
        where
            T: AsRef<[u8]>,
        {
            let pos = self.entries.iter().position(|e| e.typ == typ);
            match pos {
                Some(i) => self.entries[i].value = val.as_ref().to_vec(),
                None => self
                    .insert(TlvEntry {
                        typ,
                        value: val.as_ref().to_vec(),
                    })
                    .unwrap(),
            }
        }

        pub fn set_tu64(&mut self, typ: u64, val: TU64) {
            let mut b = bytes::BytesMut::new();
            b.put_tu64(val);
            self.set_bytes(typ, b)
        }
    }

    pub trait FromBytes: Sized {
        type Error;
        fn from_bytes<T>(s: T) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> + 'static;
    }

    impl FromBytes for SerializedTlvStream {
        type Error = anyhow::Error;
        fn from_bytes<T>(s: T) -> Result<Self, Self::Error>
        where
            T: AsRef<[u8]> + 'static,
        {
            let mut b = s.as_ref();
            //let mut b: bytes::Bytes = r.into();
            let mut entries: Vec<TlvEntry> = vec![];
            while b.remaining() >= 2 {
                let typ = b.get_compact_size() as u64;
                let len = b.get_compact_size() as usize;
                let value = b.copy_to_bytes(len).to_vec();
                entries.push(TlvEntry { typ, value });
            }

            Ok(SerializedTlvStream { entries })
        }
    }

    pub type CompactSize = u64;

    /// A variant of CompactSize that works on length-delimited
    /// buffers and therefore does not require a length prefix
    pub type TU64 = u64;

    /// Extensions on top of `Buf` to include LN proto primitives
    pub trait ProtoBuf: Buf {
        fn get_compact_size(&mut self) -> CompactSize {
            match self.get_u8() {
                253 => self.get_u16().into(),
                254 => self.get_u32().into(),
                255 => self.get_u64(),
                v => v.into(),
            }
            .into()
        }

        fn get_tu64(&mut self) -> TU64 {
            match self.remaining() {
                1 => self.get_u8() as u64,
                2 => self.get_u16() as u64,
                4 => self.get_u32() as u64,
                8 => self.get_u64() as u64,
                l => panic!("Unexpect TU64 length: {}", l),
            }
        }
    }

    impl ProtoBuf for bytes::Bytes {}
    impl ProtoBuf for &[u8] {}
    impl ProtoBuf for bytes::buf::Take<bytes::Bytes> {}

    pub trait ProtoBufMut: bytes::BufMut {
        fn put_compact_size(&mut self, cs: CompactSize) {
            match cs as u64 {
                0..=0xFC => self.put_u8(cs as u8),
                0xFD..=0xFFFF => {
                    self.put_u8(253);
                    self.put_u16(cs as u16);
                }
                0x10000..=0xFFFFFFFF => {
                    self.put_u8(254);
                    self.put_u32(cs as u32);
                }
                v => {
                    self.put_u8(255);
                    self.put_u64(v);
                }
            }
        }

        fn put_tu64(&mut self, u: TU64) {
            // Fixme: (nepet) We trim leading zero bytes here as they
            // cause some problems for the cln decoder - for now. Think
            // about an appropriate solution.
            let b: Vec<u8> = u
                .to_be_bytes()
                .iter()
                .map(|x| x.clone())
                .skip_while(|&x| x == 0)
                .collect();
            self.put_slice(&b);
        }
    }

    impl ProtoBufMut for bytes::BytesMut {}

    pub trait ToBytes: Sized {
        fn to_bytes(s: Self) -> Vec<u8>;
    }

    impl ToBytes for SerializedTlvStream {
        fn to_bytes(s: Self) -> Vec<u8> {
            let mut b = bytes::BytesMut::new();

            for e in s.entries.iter() {
                b.put_compact_size(e.typ);
                b.put_compact_size(e.value.len() as u64);
                b.put(&e.value[..]);
            }
            b.to_vec()
        }
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
            let l = b.get_compact_size();
            let b = b.take(l as usize); // Protect against overruns

            Self::from_bytes(b.into_inner()).map_err(|e| serde::de::Error::custom(e.to_string()))
        }
    }
    #[cfg(test)]
    mod test {}
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
