/// Various structs representing JSON-RPC responses
use serde::Deserialize;
use clightningrpc::common::MSat;
pub use clightningrpc::responses::*;


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
    pub fees_collected_msat: clightningrpc::common::MSat,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pay {
    pub destination: String,
    pub payment_hash: String,
    pub created_at: f64,
    pub parts: u32,
    pub msatoshi: u64,
    pub msatoshi_sent: u64,
    pub payment_preimage: Option<String>,
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
    // parts is missing
    // msatoshi is renamed amount_msat
    pub amount_msat: Option<String>,
    pub amount_sent_msat: String,
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
    pub amount: Option<String>,

    #[serde(rename = "amount_received_msat")]
    pub received: Option<String>,

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
    pub msatoshi: u64,
    pub msatoshi_sent: u64,
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
