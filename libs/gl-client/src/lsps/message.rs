use crate::lsps::error::LspsError;
pub use crate::lsps::json_rpc::{JsonRpcMethod, NoParams};
use crate::lsps::json_rpc_erased::{JsonRpcMethodErased, JsonRpcMethodUnerased};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

type SatAmount = u64;
type OnchainFeeRate = u64;

// All rpc-methods defined in the LSPS standard
// The generics are <I,O,E> where
// - I represents the params
// - O represents the result data
// - E represents the error if present
//
// To create language bindings for a new rpc-call you must
// 1. Add it to the JsonRpcMethodEnum
// 2. Add it to the from_method_name function
// 3. Add it to the ref_erase function
pub type Lsps0ListProtocols = JsonRpcMethod<NoParams, ProtocolList, ()>;
pub type Lsps1Info = JsonRpcMethod<NoParams, Lsps1InfoResponse, ()>;
pub type Lsps1Order = JsonRpcMethod<Lsps1GetOrderRequest, Lsps1GetOrderResponse, ()>;

pub const LSPS0_LISTPROTOCOLS: Lsps0ListProtocols = Lsps0ListProtocols::new("lsps0.listprotocols");
pub const LSPS1_GETINFO: Lsps1Info = Lsps1Info::new("lsps1.info");
pub const LSPS1_GETORDER: Lsps1Order = Lsps1Order::new("lsps1.order");

pub enum JsonRpcMethodEnum {
    Lsps0ListProtocols(Lsps0ListProtocols),
    Lsps1Info(Lsps1Info),
    Lsps1Order(Lsps1Order),
}

impl JsonRpcMethodEnum {
    pub fn from_method_name(value: &str) -> Result<JsonRpcMethodEnum, LspsError> {
        match value {
            "lsps0.listprotocols" => Ok(Self::Lsps0ListProtocols(LSPS0_LISTPROTOCOLS)),
            "lsps1.info" => Ok(Self::Lsps1Info(LSPS1_GETINFO)),
            "lsps1.order" => Ok(Self::Lsps1Order(LSPS1_GETORDER)),
            default => Err(LspsError::MethodUnknown(String::from(default))),
        }
    }

    // Useful for language bindings.
    // The python code can
    pub fn ref_erase<'a>(&'a self) -> &'a dyn JsonRpcMethodErased {
        match self {
            Self::Lsps0ListProtocols(list_protocol) => list_protocol.ref_erase(),
            Self::Lsps1Info(info) => info.ref_erase(),
            Self::Lsps1Order(order) => order.ref_erase(),
        }
    }
}

impl<'a> JsonRpcMethodUnerased<'a, Vec<u8>, Vec<u8>, Vec<u8>> for JsonRpcMethodEnum {
    fn name(&self) -> &str {
        self.ref_erase().name()
    }

    fn create_request(
        &self,
        params: Vec<u8>,
        json_rpc_id: String,
    ) -> Result<super::json_rpc::JsonRpcRequest<Vec<u8>>, serde_json::Error> {
        self.ref_erase().create_request(params, json_rpc_id)
    }

    fn parse_json_response_str(
        &self,
        json_str: &str,
    ) -> Result<super::json_rpc::JsonRpcResponse<Vec<u8>, Vec<u8>>, serde_json::Error> {
        self.ref_erase().parse_json_response_str(json_str)
    }

    fn parse_json_response_value(
        &self,
        json_value: serde_json::Value,
    ) -> Result<super::json_rpc::JsonRpcResponse<Vec<u8>, Vec<u8>>, serde_json::Error> {
        self.ref_erase().parse_json_response_value(json_value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolList {
    pub protocols: Vec<u32>,
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

    use crate::lsps::json_rpc::generate_random_rpc_id;

    use super::*;
    use serde_json::{from_str, to_string, Value};

    #[test]
    fn serialize_request_with_no_params() {
        let method = LSPS0_LISTPROTOCOLS;
        let json_rpc_id = generate_random_rpc_id();
        let rpc_request = method.create_request_no_params(json_rpc_id);
        let json_str = to_string(&rpc_request).unwrap();

        // Test that params is an empty dict
        //
        // LSPS-0 spec demands that a parameter-by-name scheme is always followed
        let v: Value = from_str(&json_str).unwrap();
        assert_eq!(v.get("jsonrpc").unwrap(), "2.0");
        assert_eq!(
            v.get("method").unwrap().as_str().unwrap(),
            "lsps0.listprotocols"
        );
        assert!(v.get("params").unwrap().as_object().unwrap().is_empty())
    }

    #[test]
    fn serialize_protocol_list() {
        let protocols = ProtocolList {
            protocols: vec![1, 3],
        };

        let json_str = serde_json::to_string(&protocols).unwrap();
        assert_eq!(json_str, "{\"protocols\":[1,3]}")
    }
}
