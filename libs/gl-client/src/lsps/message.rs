// TODO: Implement parsing of error types for lsps2.getinfo
use crate::lsps::error::LspsError;
pub use crate::lsps::json_rpc::{DefaultError, JsonRpcMethod, NoParams};
use crate::lsps::json_rpc_erased::{JsonRpcMethodErased, JsonRpcMethodUnerased};

use crate::lsps::lsps0::schema::ProtocolList;
use crate::lsps::lsps1::schema::{Lsps1GetOrderRequest, Lsps1GetOrderResponse, Lsps1InfoResponse};
use crate::lsps::lsps2::schema::{
    Lsps2BuyError, Lsps2BuyRequest, Lsps2BuyResponse, Lsps2GetInfoError, Lsps2GetInfoRequest,
    Lsps2GetInfoResponse, Lsps2GetVersionsResponse,
};

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
pub type Lsps2GetInfo = JsonRpcMethod<Lsps2GetInfoRequest, Lsps2GetInfoResponse, Lsps2GetInfoError>;
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
}
