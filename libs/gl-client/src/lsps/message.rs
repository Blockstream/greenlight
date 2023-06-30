use crate::lsps::json_rpc::{JsonRpcMethod, NoParams};
use serde::{Deserialize, Serialize};

// All rpc-methods defined in the LSPS standard
// The generics are <I,O,E> where
// - I represents the params
// - O represents the result data
// - E represents the error if present
pub const LSPS0_LISTPROTOCOLS: JsonRpcMethod<NoParams, ProtocolList, ()> =
    JsonRpcMethod::new("lsps0.listprotocols");

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolList {
    protocols: Vec<u32>,
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
