use crate::lsps::lsps0::common_schemas::{IsoDatetime, SatAmount};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type OnchainFeeRate = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1InfoResponse {
    supported_versions: Vec<u16>,
    website: Option<String>,
    options: Lsps1Options,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1Options {
    minimum_channel_confirmations: Option<u8>,
    minimum_onchain_payment_confirmations: Option<u8>,
    supports_zero_channel_reserve: bool,
    min_onchain_payment_size_sat: Option<u64>,
    max_channel_expiry_blocks: Option<u32>,
    min_initial_client_balance_sat: Option<u64>,
    min_initial_lsp_balance_sat: Option<u64>,
    max_initial_client_balance_sat: Option<u64>,
    min_channel_balance_sat: Option<u64>,
    max_channel_balance_sat: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1GetOrderRequest {
    pub api_version: u16,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    pub refund_onchain_address: Option<String>,
    #[serde(rename = "announceChannel")]
    pub announce_channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps1GetOrderResponse {
    uuid: Uuid,
    api_version: u16,
    lsp_balance_sat: SatAmount,
    client_balance_sat: SatAmount,
    confirms_within_blocks: u8,
    channel_expiry_blocks: u8,
    token: String,
    #[serde(rename = "announceChannel")]
    announce_channel: bool,
    created_at: IsoDatetime,
    expires_at: IsoDatetime,
    order_state: OrderState,
    payment: Payment,
}

#[derive(Debug, Serialize, Deserialize)]
enum OrderState {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
enum PaymentState {
    #[serde(rename = "EXPECT_PAYMENT")]
    ExpectPayment,
    #[serde(rename = "HOLD")]
    Hold,
    #[serde(rename = "STATE")]
    State,
    #[serde(rename = "REFUNDED")]
    Refunded,
}

#[derive(Debug, Serialize, Deserialize)]
struct OnchainPayment {
    outpoint: String,
    sat: SatAmount,
    confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Payment {
    state: PaymentState,
    fee_total_sat: SatAmount,
    order_total_sat: SatAmount,

    bolt11_invoice: String,
    onchain_address: String,
    required_onchain_block_confirmations: u8,

    minimum_fee_for_0conf: OnchainFeeRate,
    on_chain_payments: Vec<OnchainPayment>,
}
