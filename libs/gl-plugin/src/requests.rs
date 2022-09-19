pub use clightningrpc::requests::*;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Outpoint {
    pub txid: Vec<u8>,
    pub outnum: u16,
}

#[derive(Debug, Clone)]
pub enum Amount {
    Millisatoshi(u64),
    Satoshi(u64),
    Bitcoin(u64),
    All,
    Any,
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Amount::Millisatoshi(a) => serializer.serialize_str(&format!("{}msat", a)),
            Amount::Satoshi(a) => serializer.serialize_str(&format!("{}sat", a)),
            Amount::Bitcoin(a) => serializer.serialize_str(&format!("{}btc", a)),
            Amount::All => serializer.serialize_str(&format!("all")),
            Amount::Any => serializer.serialize_str(&format!("any")),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Feerate {
    Normal,
    Slow,
    Urgent,
    PerKw(u64),
    PerKb(u64),
}

impl Serialize for Feerate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Feerate::Normal => serializer.serialize_str("normal"),
            Feerate::Slow => serializer.serialize_str("slow"),
            Feerate::Urgent => serializer.serialize_str("urgent"),
            Feerate::PerKb(n) => serializer.serialize_str(&format!("{}perkb", n)),
            Feerate::PerKw(n) => serializer.serialize_str(&format!("{}perkw", n)),
        }
    }
}
impl Serialize for Outpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", hex::encode(&self.txid), self.outnum))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Withdraw {
    pub destination: String,

    #[serde(rename = "satoshi")]
    pub amount: Amount,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub feerate: Option<Feerate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minconf: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub utxos: Option<Vec<Outpoint>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FundChannel {
    pub id: String,
    pub amount: Amount,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub feerate: Option<Feerate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub announce: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minconf: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CloseChannel {
    #[serde(rename = "id")]
    pub node_id: String,
    #[serde(rename = "unilateraltimeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Invoice {
    #[serde(rename = "msatoshi")]
    pub amount: Amount,
    pub label: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposeprivatechannels: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimage: Option<String>,

    #[serde(rename = "dev-routes", skip_serializing_if = "Option::is_none")]
    pub dev_routes: Option<Vec<Vec<RoutehintHopDev>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListFunds {}

#[derive(Debug, Clone, Serialize)]
pub struct Pay {
    pub bolt11: String,
    #[serde(rename = "msatoshi", skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_for: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListPays {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bolt11: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListInvoices {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invstring: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutehintHop {
    pub id: String,
    pub scid: String,
    pub feebase: u64,
    pub feeprop: u32,
    pub expirydelta: u16,
}

// This variant is used by dev-routes, using slightly different key names.
// TODO Remove once we have consolidated the routehint format.
#[derive(Debug, Clone, Serialize)]
pub struct RoutehintHopDev {
    pub id: String,
    pub short_channel_id: String,
    pub fee_base_msat: u64,
    pub fee_proportional_millionths: u32,
    pub cltv_expiry_delta: u16,
}

use std::collections::HashMap;
#[derive(Debug, Clone, Serialize)]
pub struct Keysend {
    pub destination: String,
    pub msatoshi: Amount,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxfeepercent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_for: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxdelay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exemptfee: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routehints: Option<Vec<Vec<RoutehintHop>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extratlvs: Option<HashMap<u64, String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListIncoming {}
