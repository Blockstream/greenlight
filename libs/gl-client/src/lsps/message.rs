// TODO: Implement parsing of error types for lsps2.getinfo
use crate::lsps::error::LspsError;
pub use crate::lsps::json_rpc::{DefaultError, JsonRpcMethod, MapErrorCode, NoParams};
use crate::lsps::json_rpc_erased::{JsonRpcMethodErased, JsonRpcMethodUnerased};

use crate::lsps::lsps0::common_schemas::{IsoDatetime, MsatAmount};
use crate::lsps::lsps0::schema::ProtocolList;
use crate::lsps::lsps1::schema::{Lsps1InfoResponse, Lsps1GetOrderResponse, Lsps1GetOrderRequest};
use serde::de::Error as DeError;
use serde::{Deserialize, Serialize};

use super::error::map_json_rpc_error_code_to_str;


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
pub type Lsps0ListProtocols = JsonRpcMethod<NoParams, ProtocolList, DefaultError>;
pub type Lsps1Info = JsonRpcMethod<NoParams, Lsps1InfoResponse, DefaultError>;
pub type Lsps1Order = JsonRpcMethod<Lsps1GetOrderRequest, Lsps1GetOrderResponse, DefaultError>;
pub type Lsps2GetVersions = JsonRpcMethod<NoParams, Lsps2GetVersionsResponse, DefaultError>;
pub type Lsps2GetInfo = JsonRpcMethod<Lsps2GetInfoRequest, Lsp2GetInfoResponse, Lsp2GetInfoError>;
pub type Lsps2Buy = JsonRpcMethod<Lsps2BuyRequest, Lsps2BuyResponse, Lsps2BuyError>;

pub const LSPS0_LIST_PROTOCOLS: Lsps0ListProtocols =
    Lsps0ListProtocols::new("lsps0.list_protocols");

// LSPS1: Buy Channels
pub const LSPS1_GETINFO: Lsps1Info = Lsps1Info::new("lsps1.info");
pub const LSPS1_GETORDER: Lsps1Order = Lsps1Order::new("lsps1.order");

// LSPS2: JIT-channels
pub const LSPS2_GET_VERSIONS: Lsps2GetVersions = Lsps2GetVersions::new("lsps2.get_versions");
pub const LSPS2_GET_INFO: Lsps2GetInfo = Lsps2GetInfo::new("lsps2.get_info");
pub const LSPS2_BUY: Lsps2Buy = Lsps2Buy::new("lsps2.buy");

const MAX_PROMISE_LEN_BYTES: usize = 512;
#[derive(Debug)]
struct Promise {
    promise: String,
}

impl Promise {
    fn new(promise: String) -> Result<Self, String> {
        if promise.len() <= MAX_PROMISE_LEN_BYTES {
            Ok(Promise { promise: promise })
        } else {
            Err(String::from("Promise exceeds maximum length"))
        }
    }
}

impl Serialize for Promise {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return serializer.serialize_str(&self.promise);
    }
}

// We only accept promises with a max length of 512 to be compliant to the spec
// Note, that serde-json still forces us to parse the entire string fully.
// However, the client will not story the overly large json-file
impl<'de> Deserialize<'de> for Promise {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_repr = String::deserialize(deserializer)?;
        let promise = Promise::new(str_repr.clone());
        format!("{:?}", &promise);
        return Promise::new(str_repr.clone())
            .map_err(|_| D::Error::custom("promise exceeds max length"));
    }
}

pub enum JsonRpcMethodEnum {
    Lsps0ListProtocols(Lsps0ListProtocols),
    Lsps1Info(Lsps1Info),
    Lsps1Order(Lsps1Order),
    Lsp2GetVersions(Lsps2GetVersions),
    Lsps2GetInfo(Lsps2GetInfo),
    Lsps2Buy(Lsps2Buy),
}

impl JsonRpcMethodEnum {
    pub fn from_method_name(value: &str) -> Result<JsonRpcMethodEnum, LspsError> {
        match value {
            "lsps0.list_protocols" => Ok(Self::Lsps0ListProtocols(LSPS0_LIST_PROTOCOLS)),
            "lsps1.info" => Ok(Self::Lsps1Info(LSPS1_GETINFO)),
            "lsps1.order" => Ok(Self::Lsps1Order(LSPS1_GETORDER)),
            "lsps2.get_versions" => Ok(Self::Lsp2GetVersions(LSPS2_GET_VERSIONS)),
            "lsps2.get_info" => Ok(Self::Lsps2GetInfo(LSPS2_GET_INFO)),
            "lsps2.buy" => Ok(Self::Lsps2Buy(LSPS2_BUY)),
            default => Err(LspsError::MethodUnknown(String::from(default))),
        }
    }

    // Useful for language bindings.
    // The python code can
    pub fn ref_erase(&self) -> &dyn JsonRpcMethodErased {
        match self {
            Self::Lsps0ListProtocols(list_protocol) => list_protocol.ref_erase(),
            Self::Lsps1Info(info) => info.ref_erase(),
            Self::Lsps1Order(order) => order.ref_erase(),
            Self::Lsp2GetVersions(order) => order.ref_erase(),
            Self::Lsps2GetInfo(order) => order.ref_erase(),
            Self::Lsps2Buy(buy) => buy.ref_erase(),
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
pub struct Lsps2GetVersionsResponse {
    versions: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2GetInfoRequest {
    version: i64,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsp2GetInfoResponse {
    opening_fee_params_menu: Vec<OpeningFeeParamsMenuItem>,
    min_payment_size_msat: String,
    max_payment_size_msat: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsp2GetInfoError {}

impl MapErrorCode for Lsp2GetInfoError {
    fn get_code_str(code: i64) -> &'static str {
        match code {
            1 => "unsupported_version",
            2 => "unrecognized_or_stale_token",
            _ => map_json_rpc_error_code_to_str(code),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningFeeParamsMenuItem {
    min_fee_msat: MsatAmount,
    proportional: u64,
    _valid_until: IsoDatetime,
    min_lifetime: u64,
    max_client_to_self_delay: u64,
    promise: Promise,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2BuyRequest {
    version: String,
    opening_fee_params: OpeningFeeParamsMenuItem,
    payment_size_msat: MsatAmount,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2BuyResponse {
    jit_channel_scid: String,
    lsp_cltv_expiry_delta: u64,
    #[serde(default)]
    client_trusts_lsp: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsps2BuyError {}

impl MapErrorCode for Lsps2BuyError {
    fn get_code_str(code: i64) -> &'static str {
        match code {
            1 => "unsupported_version",
            2 => "invalid_opening_fee_params",
            3 => "payment_size_too_small",
            4 => "payment_size_too_large",
            _ => map_json_rpc_error_code_to_str(code),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::lsps::json_rpc::generate_random_rpc_id;

    use super::*;
    use serde_json::{from_str, to_string, Value};

    #[test]
    fn serialize_request_with_no_params() {
        let method = LSPS0_LIST_PROTOCOLS;
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
            "lsps0.list_protocols"
        );
        assert!(v.get("params").unwrap().as_object().unwrap().is_empty())
    }


    #[test]
    fn parsing_error_when_opening_fee_menu_has_extra_fields() {
        // LSPS2 mentions
        // Clients MUST fail and abort the flow if a opening_fee_params object has unrecognized fields.
        let fee_menu_item = serde_json::json!(
            {
                "min_fee_msat": "546000",
                "proportional": 1200,
                "valid_until": "2023-02-23T08:47:30.511Z",
                "min_lifetime": 1008,
                "max_client_to_self_delay": 2016,
                "promise": "abcdefghijklmnopqrstuvwxyz",
                "extra_field" : "This shouldn't be their"
            }
        );

        let parsed_opening_fee_menu_item: Result<OpeningFeeParamsMenuItem, _> =
            serde_json::from_value(fee_menu_item);
        assert!(
            parsed_opening_fee_menu_item.is_err_and(|x| format!("{}", x).contains("extra_field"))
        )
    }


    #[test]
    fn parse_valid_promise() {
        let promise_json = "\"abcdefghijklmnopqrstuvwxyz\"";
        let promise = serde_json::from_str::<Promise>(promise_json).expect("Can parse promise");
        assert_eq!(promise.promise, "abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn parse_too_long_promise_fails() {
        // Each a char correspond to 1 byte
        // We refuse to parse the promise if it is too long
        // LSPS2 requires us to ignore Promise that are too long
        // so the client cannot be burdened with unneeded storage requirements
        let a_513_chars = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";
        let a_512_chars = "\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"";

        serde_json::from_str::<Promise>(a_512_chars)
            .expect("Should fail because 512 bytes is the max length");
        serde_json::from_str::<Promise>(a_513_chars).expect_err("Should fail to parse promise ");
    }

    #[test]
    fn client_trust_lsp_defaults_to_false() {
        let data = serde_json::json!({
            "jit_channel_scid" : "#scid#",
            "lsp_cltv_expiry_delta" : 144
        });

        let buy_response =
            serde_json::from_value::<Lsps2BuyResponse>(data).expect("The response can be parsed");

        assert!(
            !buy_response.client_trusts_lsp,
            "If the field is absent it assumed the client should not trust the LSP"
        )
    }
}
