//! Various structs representing JSON-RPC responses
pub use clightningrpc::responses::*;

use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

/// A simple wrapper that generalizes bare amounts and amounts with
/// the `msat` suffix.
#[derive(Clone, Copy, Debug)]
pub struct MSat(pub u64);

struct MSatVisitor;
impl<'d> de::Visitor<'d> for MSatVisitor {
    type Value = MSat;

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !s.ends_with("msat") {
            return Err(E::custom("missing msat suffix"));
        }

        let numpart = s
            .get(0..(s.len() - 4))
            .ok_or_else(|| E::custom("missing msat suffix"))?;

        let res = u64::from_str(numpart).map_err(|_| E::custom("not a number"))?;
        Ok(MSat(res))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(MSat(v as u64))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a bare integer or a string ending with \"msat\"")
    }
}

impl<'d> Deserialize<'d> for MSat {
    fn deserialize<D>(deserializer: D) -> Result<MSat, D::Error>
    where
        D: Deserializer<'d>,
    {
        deserializer.deserialize_any(MSatVisitor)
    }
}

impl std::fmt::Display for MSat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}msat", self.0)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Withdraw {
    pub tx: String,
    pub txid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FundChannel {
    pub tx: String,
    pub txid: String,

    #[serde(rename = "outnum")]
    pub outpoint: u32,
    pub channel_id: String,
    pub close_to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetInfo {
    pub id: String,
    pub alias: String,
    pub color: String,
    pub num_peers: u64,
    pub num_pending_channels: u64,
    pub num_active_channels: u64,
    pub num_inactive_channels: u64,
    pub version: String,
    pub blockheight: u32,
    pub network: String,
    #[serde(rename = "lightning-dir")]
    pub ligthning_dir: String,
    pub warning_bitcoind_sync: Option<String>,
    pub warning_lightningd_sync: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloseChannel {
    #[serde(rename = "type")]
    pub close_type: String,
    pub tx: String,
    pub txid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Invoice {
    #[serde(rename = "expires_at")]
    pub expiry_time: u32,
    pub bolt11: String,
    pub payment_hash: String,
    pub payment_secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pay {
    pub destination: String,
    pub payment_hash: String,
    pub created_at: f64,
    pub completed_at: Option<u64>,
    pub parts: u32,
    pub msatoshi: u64,
    pub msatoshi_sent: u64,
    pub preimage: Option<String>,
    pub status: String,
    pub bolt11: Option<String>,
}

// Sadly the results of pay differ from the listpays elements, so we
// have to replicate this struct here, until we merge them correctly.
#[derive(Debug, Clone, Deserialize)]
pub struct ListPaysPay {
    pub bolt11: Option<String>,
    pub destination: String,
    pub payment_hash: String,
    pub created_at: f64,
    pub completed_at: Option<u64>,
    // parts is missing
    // msatoshi is renamed amount_msat
    pub amount_msat: Option<MSat>,
    pub amount_sent_msat: MSat,
    pub payment_preimage: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPays {
    pub pays: Vec<ListPaysPay>,
}

// Invoices returned as part of a listinvoices call
#[derive(Debug, Clone, Deserialize)]
pub struct ListInvoiceInvoice {
    pub label: String,
    pub description: String,
    pub payment_preimage: Option<String>,

    #[serde(rename = "amount_msat")]
    pub amount: Option<MSat>,

    #[serde(rename = "amount_received_msat")]
    pub received: Option<MSat>,

    #[serde(rename = "paid_at")]
    pub payment_time: Option<u32>,
    pub status: String,

    #[serde(rename = "expires_at")]
    pub expiry_time: u32,
    pub bolt11: String,
    pub payment_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListInvoices {
    pub invoices: Vec<ListInvoiceInvoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Keysend {
    pub destination: String,
    pub status: String,
    pub payment_preimage: Option<String>,
    pub payment_hash: String,
    pub msatoshi: Option<u64>,
    pub msatoshi_sent: Option<u64>,
    pub amount_sent_msat: Option<MSat>,
    pub amount_msat: Option<MSat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListIncoming {
    pub incoming: Vec<IncomingChannel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomingChannel {
    pub id: String,
    pub short_channel_id: String,
    pub fee_base_msat: String,
    pub fee_proportional_millionths: u32,
    pub cltv_expiry_delta: u32,
    pub incoming_capacity_msat: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetChainInfo {
    pub chain: String,
    pub headercount: u32,
    pub blockcount: u32,
    pub ibd: bool,
}

/// Sub-structure for 'getlog' and 'listpeers' item
#[derive(Debug, Clone, Deserialize)]
pub struct LogEntry {
    #[serde(rename = "type")]
    pub type_: String,
    pub num_skipped: Option<u64>,
    pub time: Option<String>,
    pub node_id: Option<String>,
    pub source: Option<String>,
    pub log: Option<String>,
    pub data: Option<String>,
}

/// 'getlog' command
#[derive(Debug, Clone, Deserialize)]
pub struct GetLog {
    pub created_at: String,
    pub bytes_used: u64,
    pub bytes_max: u64,
    pub log: Vec<LogEntry>,
}

/// Sub-structure for htlcs in 'listpeers'
#[derive(Debug, Clone, Deserialize)]
pub struct Htlc {
    pub direction: String,
    pub id: u64,
    pub amount_msat: MSat,
    pub expiry: u64,
    pub payment_hash: String,
    pub state: String,
    pub local_trimmed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    pub state: String,
    pub scratch_txid: Option<String>,
    pub owner: Option<String>,
    pub short_channel_id: Option<String>,
    pub alias: Option<Aliases>,
    pub direction: Option<u64>,
    pub channel_id: String,
    pub funding_txid: String,
    pub close_to_addr: Option<String>,
    pub close_to: Option<String>,
    pub private: bool,
    pub to_us_msat: MSat,
    pub min_to_us_msat: MSat,
    pub max_to_us_msat: MSat,
    pub total_msat: MSat,
    pub dust_limit_msat: MSat,
    pub max_total_htlc_in_msat: MSat,
    pub their_reserve_msat: MSat,
    pub our_reserve_msat: MSat,
    pub spendable_msat: MSat,
    pub receivable_msat: MSat,
    pub minimum_htlc_in_msat: MSat,
    pub their_to_self_delay: u64,
    pub our_to_self_delay: u64,
    pub max_accepted_htlcs: u64,
    pub status: Vec<String>,
    pub in_payments_offered: u64,
    pub in_offered_msat: MSat,
    pub in_payments_fulfilled: u64,
    pub in_fulfilled_msat: MSat,
    pub out_payments_offered: u64,
    pub out_offered_msat: MSat,
    pub out_payments_fulfilled: u64,
    pub out_fulfilled_msat: MSat,
    pub htlcs: Vec<Htlc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Aliases {
    pub local: Option<String>,
    pub remote: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Peer {
    pub id: String,
    pub connected: bool,
    pub netaddr: Option<Vec<String>>,
    pub features: Option<String>,
    pub channels: Vec<Channel>,
    pub log: Option<Vec<LogEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPeers {
    pub peers: Vec<Peer>,
}

/// Sub-structure for 'listfunds' output
#[derive(Debug, Clone, Deserialize)]
pub struct ListFundsOutput {
    pub txid: String,
    pub output: u64,
    pub amount_msat: MSat,
    pub address: String,
    pub status: String,
    pub reserved: bool,
    pub reserved_to_block: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListFundsChannel {
    pub peer_id: String,
    pub connected: bool,
    pub short_channel_id: Option<String>,
    pub our_amount_msat: MSat,
    pub amount_msat: MSat,
    pub funding_txid: String,
    pub funding_output: u64,
}

/// 'listfunds' command
#[derive(Debug, Clone, Deserialize)]
pub struct ListFunds {
    pub outputs: Vec<ListFundsOutput>,
    pub channels: Vec<ListFundsChannel>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_msat_parsing() {
        #[derive(Deserialize)]
        struct V {
            value: MSat,
        }

        struct T {
            input: &'static str,
            output: u64,
        }

        let tests: Vec<T> = vec![
            T {
                input: "{\"value\": \"1234msat\"}",
                output: 1234,
            },
            T {
                input: "{\"value\": 100000000000}",
                output: 100000000000,
            },
        ];

        for t in tests {
            let v: V = serde_json::from_str(t.input).unwrap();
            assert_eq!(v.value.0, t.output);
        }
    }
}
