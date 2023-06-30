use base64::Engine as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

fn generate_random_rpc_id() -> String {
    // TODO: verify that rand::random is a CSRNG

    // The specification requires an id using least 80 random bits of randomness
    let seed: [u8; 10] = rand::random();
    let result = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(seed);
    return result;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcMethod<'a, I, O, E> {
    method: &'a str,
    #[serde(skip_serializing)]
    request: std::marker::PhantomData<I>,
    #[serde(skip_serializing)]
    return_type: std::marker::PhantomData<O>,
    #[serde(skip_serializing)]
    error_type: std::marker::PhantomData<E>,
}

impl<'a, I, O, E> JsonRpcMethod<'a, I, O, E> {
    pub const fn new(method: &'a str) -> Self {
        return Self {
            method: method,
            request: std::marker::PhantomData,
            return_type: std::marker::PhantomData,
            error_type: std::marker::PhantomData,
        };
    }

    pub fn create_request(&self, params: I) -> JsonRpcRequest<I> {
        JsonRpcRequest::<I> {
            json_rpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: self.method.into(),
            params: params,
        }
    }
}

impl<O, E> JsonRpcMethod<'static, NoParams, O, E> {
    pub fn create_request_no_params(&self) -> JsonRpcRequest<NoParams> {
        self.create_request(NoParams::default())
    }
}

impl<'a, I, O, E> std::convert::From<&JsonRpcMethod<'a, I, O, E>> for String {
    fn from(value: &JsonRpcMethod<I, O, E>) -> Self {
        return value.method.clone().into();
    }
}

impl<'de, 'a, I, O, E> JsonRpcMethod<'a, I, O, E>
where
    O: Deserialize<'de>,
    E: Deserialize<'de>,
{
    pub fn parse_json_response(
        &self,
        json_str: &'de str,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error> {
        serde_json::from_str(&json_str)
    }
}

// We only intend to implement to implement an LSP-client and only intend on sending requests
// Therefore, we only implement the serialization of requests
//
// D is the data-type of the request-data
// R is the data-type of the result if the query is successful
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest<I> {
    json_rpc: String,
    id: String,
    method: String,
    params: I,
}

// LSPS0 specifies that the RPC-request must use a parameter-by-name structure.
//
// A JSONRpcRequest<(),()> will be serialized to a json where "params" : null
// A JsonRpcRequest<NoParams, ()> will be serialized to "params" : {} which is compliant
#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub struct NoParams {}

// Serde serializes NoParams to null by default
// LSPS0 requires an empty dictionary in this situation
impl Serialize for NoParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

impl<I> JsonRpcRequest<I> {
    pub fn new<O, E>(method: JsonRpcMethod<I, O, E>, params: I) -> Self {
        return Self {
            json_rpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: method.method.into(),
            params: params,
        };
    }
}

impl JsonRpcRequest<NoParams> {
    pub fn new_no_params<O, E>(method: JsonRpcMethod<NoParams, O, E>) -> Self {
        return Self {
            json_rpc: String::from("2.0"),
            id: generate_random_rpc_id(),
            method: method.method.into(),
            params: NoParams::default(),
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponseSuccess<O> {
    id: String,
    result: O,
    json_rpc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponseFailure<E> {
    id: String,
    error: ErrorData<E>,
    json_rpc: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse<O, E> {
    Error(JsonRpcResponseFailure<E>),
    Ok(JsonRpcResponseSuccess<O>),
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorData<E> {
    code: i64,
    message: String,
    data: Option<E>,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn serialize_no_params() {
        let no_params = NoParams::default();
        let json_str = serde_json::to_string(&no_params).unwrap();

        assert_eq!(json_str, "{}")
    }

    #[test]
    fn deserialize_no_params() {
        let _: NoParams = serde_json::from_str("{}").unwrap();
    }

    #[test]
    fn serialize_json_rpc_request() {
        let rpc_request = JsonRpcRequest {
            id: "abcefg".into(),
            json_rpc: "2.0".into(),
            params: NoParams::default(),
            method: "test.method".into(),
        };

        let json_str = serde_json::to_string(&rpc_request).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value.get("json_rpc").unwrap(), "2.0");
        assert_eq!(value.get("id").unwrap(), &rpc_request.id);
        assert_eq!(value.get("method").unwrap(), "test.method");
        assert!(value.get("params").unwrap().as_object().unwrap().is_empty())
    }

    #[test]
    fn serialize_json_rpc_response_success() {
        let rpc_response_ok: JsonRpcResponseSuccess<String> = JsonRpcResponseSuccess {
            id: String::from("abc"),
            result: String::from("result_data"),
            json_rpc: String::from("2.0"),
        };

        let rpc_response: JsonRpcResponse<String, ()> = JsonRpcResponse::Ok(rpc_response_ok);

        let json_str: String = serde_json::to_string(&rpc_response).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value.get("json_rpc").unwrap(), "2.0");
        assert_eq!(value.get("id").unwrap(), "abc");
        assert_eq!(value.get("result").unwrap(), "result_data")
    }

    #[test]
    fn serialize_json_rpc_response_error() {
        let rpc_response: JsonRpcResponse<String, ()> =
            JsonRpcResponse::Error(JsonRpcResponseFailure {
                json_rpc: String::from("2.0"),
                id: String::from("abc"),
                error: ErrorData {
                    code: -32700,
                    message: String::from("Failed to parse data"),
                    data: None,
                },
            });

        let json_str: String = serde_json::to_string(&rpc_response).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value.get("json_rpc").unwrap(), "2.0");
        assert_eq!(value.get("id").unwrap(), "abc");
        assert_eq!(value.get("error").unwrap().get("code").unwrap(), -32700);
        assert_eq!(
            value.get("error").unwrap().get("message").unwrap(),
            "Failed to parse data"
        );
    }

    #[test]
    fn create_rpc_request_from_call() {
        let rpc_method = JsonRpcMethod::<NoParams, (), ()>::new("test.method");
        let rpc_request = rpc_method.create_request_no_params();

        assert_eq!(rpc_request.method, "test.method");
        assert_eq!(rpc_request.json_rpc, "2.0");
        assert_eq!(rpc_request.params, NoParams::default());
    }

    #[test]
    fn parse_rpc_response_success_from_call() {
        let rpc_method = JsonRpcMethod::<NoParams, String, ()>::new("test.return_string");

        let json_value = serde_json::json!({
            "json_rpc" : "2.0",
            "result" : "result_data",
            "id" : "request_id"
        });

        let json_str = serde_json::to_string(&json_value).unwrap();

        let result = rpc_method.parse_json_response(&json_str).unwrap();

        match result {
            JsonRpcResponse::Error(_) => panic!("Deserialized a good response but got panic"),
            JsonRpcResponse::Ok(ok) => {
                assert_eq!(ok.json_rpc, "2.0");
                assert_eq!(ok.id, "request_id");
                assert_eq!(ok.result, "result_data")
            }
        }
    }

    #[test]
    fn parse_rpc_response_failure_from_call() {
        let rpc_method = JsonRpcMethod::<NoParams, String, ()>::new("test.return_string");

        let json_value = serde_json::json!({
            "json_rpc" : "2.0",
            "error" : { "code" : -32700, "message" : "Failed to parse response"},
            "id" : "request_id"
        });

        let json_str = serde_json::to_string(&json_value).unwrap();

        let result = rpc_method.parse_json_response(&json_str).unwrap();

        match result {
            JsonRpcResponse::Error(err) => {
                assert_eq!(err.json_rpc, "2.0");

                assert_eq!(err.error.code, -32700);
                assert_eq!(err.error.message, "Failed to parse response");

                assert_eq!(err.id, "request_id")
            }
            JsonRpcResponse::Ok(_ok) => {
                panic!("Failure deserialized as Ok")
            }
        }
    }
}
