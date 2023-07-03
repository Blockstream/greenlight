use crate::lsps::json_rpc::{JsonRpcMethod, NoParams};
use serde::{Deserialize, Serialize};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

type SatAmount = u64;
type OnchainFeeRate = u64;

// All rpc-methods defined in the LSPS standard
// The generics are <I,O,E> where
// - I represents the params
// - O represents the result data
// - E represents the error if present
pub const LSPS0_LISTPROTOCOLS: JsonRpcMethod<NoParams, ProtocolList, ()> =
    JsonRpcMethod::new("lsps0.listprotocols");

pub const LSPS1_GETINFO: JsonRpcMethod<NoParams, Lsps1InfoResponse, ()> =
    JsonRpcMethod::new("lsps1.info");
pub const LSPS1_GETORDER: JsonRpcMethod<Lsps1GetOrderRequest, Lsps1GetOrderResponse, ()> =
    JsonRpcMethod::new("lsps1.order");

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolList {
    protocols: Vec<u32>,
}

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
    api_version: u16,
    lsp_balance_sat: SatAmount,
    client_balance_sat: SatAmount,
    confirms_within_blocks: u8,
    channel_expiry_blocks: u32,
    token: Option<String>,
    refund_onchain_address: Option<String>,
    #[serde(rename = "announceChannel")]
    announce_channel: String,
}

use serde_with::serde_as;

#[serde_as]
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
    #[serde_as(as = "Rfc3339")]
    created_at: time::OffsetDateTime,
    #[serde_as(as = "Rfc3339")]
    expires_at: time::OffsetDateTime,
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

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::{from_str, to_string, Value};

    #[test]
    fn serialize_request_with_no_params() {
        let method = LSPS0_LISTPROTOCOLS;
        let rpc_request = method.create_request_no_params();
        let json_str = to_string(&rpc_request).unwrap();

        // Test that params is an empty dict
        //
        // LSPS-0 spec demands that a parameter-by-name scheme is always followed
        let v: Value = from_str(&json_str).unwrap();
        assert_eq!(v.get("json_rpc").unwrap(), "2.0");
        assert_eq!(v.get("method").unwrap(), "lsps0.listprotocols");
        assert!(v.get("params").unwrap().as_object().unwrap().is_empty())
    }
}
