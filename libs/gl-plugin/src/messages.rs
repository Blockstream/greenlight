use anyhow::{anyhow, Error};
use hex::{self, FromHex};
use serde::de::{self, Deserializer};
use serde::ser::{self, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum JsonRpcCall {
    //HtlcAccepted(HtlcAcceptedCall),
}

#[derive(Debug)]
pub struct ParserError {
    reason: String,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("ParserError {}", self.reason))
    }
}
impl std::error::Error for ParserError {}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    id: Option<Value>,
    jsonrpc: String,
    method: String,
    params: JsonRpcCall,
}

// "Inspired" by https://github.com/serde-rs/serde/issues/1028#issuecomment-325434041
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum MyRequests {
    HtlcAccepted(HtlcAcceptedCall),
    Getmanifest(GetManifestCall),
    Init(InitCall),
    InvoicePayment(InvoicePaymentCall),
    CommitmentRevocation(CommitmentRevocationCall),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct HtlcAcceptedCall {
    pub onion: HtlcAcceptedCallOnion,
    pub htlc: HtlcAcceptedCallHtlc,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InvoicePaymentCall {
    pub payment: InvoicePaymentCallPayment,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CommitmentRevocationCall {
    pub commitment_txid: String,
    pub penalty_tx: String,
    pub channel_id: Option<String>,
    pub commitnum: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InvoicePaymentCallPayment {
    pub label: String,
    pub preimage: String,
    #[serde(rename = "msat")]
    pub amount: String,
    pub extratlvs: Option<Vec<TlvField>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TlvField {
    #[serde(rename = "type")]
    pub typ: u64,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct GetManifestCall {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct GetManifestResult {
    pub subscriptions: Vec<String>,
    pub hooks: Vec<String>,
    pub dynamic: bool,
    pub options: Vec<PluginOption>,
    pub rpcmethods: Vec<PluginRpcMethod>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginOption {
    name: String,
    default: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PluginRpcMethod {
    name: String,
    usage: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct HtlcAcceptedCallOnion {
    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    pub payload: Vec<u8>,
    short_channel_id: Option<String>,
    forward_amount: String,
    outgoing_cltv_value: u64,

    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    next_onion: Vec<u8>,

    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    pub shared_secret: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct HtlcAcceptedCallHtlc {
    pub amount: String,
    cltv_expiry: u64,
    cltv_expiry_relative: u64,

    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    pub payment_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct HtlcAcceptedResponse {
    pub result: String,
    #[serde(serialize_with = "buffer_to_hex", deserialize_with = "hex_to_buffer")]
    pub payment_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitCall {
    pub options: Value,
    pub configuration: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum MyNotifications {
    Disconnect(DisconnectNotification),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisconnectNotification {
    pub id: String,
}

#[derive(Debug)]
pub enum JsonRpc<N, R> {
    Request(usize, R),
    Notification(N),
}

impl<N, R> Serialize for JsonRpc<N, R>
where
    N: Serialize,
    R: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JsonRpc::Request(id, ref r) => {
                let mut v = serde_json::to_value(r).map_err(ser::Error::custom)?;
                v["id"] = json!(id);
                v.serialize(serializer)
            }
            JsonRpc::Notification(ref n) => n.serialize(serializer),
        }
    }
}

impl<'de, N, R> Deserialize<'de> for JsonRpc<N, R>
where
    N: Deserialize<'de>,
    R: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct IdHelper {
            id: Option<usize>,
        }

        let v = Value::deserialize(deserializer)?;
        let helper = IdHelper::deserialize(&v).map_err(de::Error::custom)?;
        match helper.id {
            Some(id) => {
                let r = R::deserialize(v).map_err(de::Error::custom)?;
                Ok(JsonRpc::Request(id, r))
            }
            None => {
                let n = N::deserialize(v).map_err(de::Error::custom)?;
                Ok(JsonRpc::Notification(n))
            }
        }
    }
}
/// Serializes `buffer` to a lowercase hex string.
pub fn buffer_to_hex<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(buffer))
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
pub fn hex_to_buffer<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| Vec::from_hex(&string).map_err(|err| Error::custom(err.to_string())))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Amount {
    pub msatoshi: i64,
}

impl Amount {
    pub fn from_string(s: &str) -> Result<Amount, Error> {
        if !s.ends_with("msat") {
            return Err(anyhow!("Amount string does not end with msat."));
        }

        let amount_string: &str = s[0..s.len() - 4].into();

        let amount: i64 = match amount_string.parse::<i64>() {
            Ok(v) => v,
            Err(e) => return Err(anyhow!(e)),
        };

        Ok(Amount { msatoshi: amount })
    }
}

fn _string_to_amount<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        Amount::from_string(&string).map_err(|_| Error::custom("could not parse amount"))
    })
}

fn _amount_to_string<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}msat", amount.msatoshi);
    serializer.serialize_str(&s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_htlc_accepted_call() {
        let req = json!({"id": 1, "jsonrpc": "2.0", "method": "htlc_accepted", "params": {
            "onion": {
              "payload": "",
              "type": "legacy",
        "short_channel_id": "1x2x3",
              "forward_amount": "42msat",
        "outgoing_cltv_value": 500014,
        "shared_secret": "0000000000000000000000000000000000000000000000000000000000000000",
        "next_onion": "00DEADBEEF00",
            },
            "htlc": {
        "amount": "43msat",
        "cltv_expiry": 500028,
        "cltv_expiry_relative": 10,
        "payment_hash": "0000000000000000000000000000000000000000000000000000000000000000"
            }
        }
        });

        type T = JsonRpc<MyNotifications, MyRequests>;
        let req = serde_json::from_str::<T>(&req.to_string()).unwrap();
        match req {
            T::Request(id, c) => {
                assert_eq!(id, 1);
                match c {
                    MyRequests::HtlcAccepted(c) => {
                        //assert_eq!(c.onion.payload, "");
                        assert_eq!(c.onion.forward_amount, "42msat");
                        assert_eq!(c.onion.outgoing_cltv_value, 500014);
                        //assert_eq!(c.onion.next_onion, "[1365bytes of serialized onion]");
                        //assert_eq!(
                        //    c.onion.shared_secret,
                        //    "0000000000000000000000000000000000000000000000000000000000000000"
                        //);
                        //assert_eq!(
                        //    c.htlc.payment_hash,
                        //    "0000000000000000000000000000000000000000000000000000000000000000"
                        //);
                    }
                    _ => panic!("This was supposed to be an htlc_accepted call"),
                }
            }
            _ => panic!("This was supposed to be a request"),
        }
    }
}
