// The json-rpc implementation used in json_rpc is strongly typed and heavily
// relies on Generics.
//
// When creating language bindings we must either
// - explicitly implement ffi for every time
// - use a trait that erases tye type. In this case we replace the type by serde_json::Value
//
// Note, that this
//
// If you are a rust-developer you probably want to use the json_rpc-module directly
// If you are building a foreign function interface you probably want to use the type-erased version
//
// The sonRpcMethodErased wraps the JsonRpcMethod into a type that works with serde_json::Value-objects.
// The JsonRpcMethodErased is object-safe and can be owned using a Box.
//
// The JsonRpcMethodErased method
// - does not do strict type-checking
// - comes at a small runtime cost (requires Box, serializes and deserializes some objects twice, unwrapping results)
// - comes at a small dev-cost for requiring a bit more error-handling

use crate::lsps::json_rpc::{
    ErrorData, JsonRpcMethod, JsonRpcRequest, JsonRpcResponse, JsonRpcResponseFailure,
    JsonRpcResponseSuccess,
};
use serde::Serialize;

pub type JsonRpcRequestErased = JsonRpcRequest<Vec<u8>>;
pub type JsonRpcResponseErased = JsonRpcResponse<Vec<u8>, Vec<u8>>;
pub type JsonRpcResponseSuccessErased = JsonRpcResponseSuccess<Vec<u8>>;
pub type JsonRpcResponseFailureErased = JsonRpcResponseFailure<Vec<u8>>;
pub type JsonRpcErrorDataErased = ErrorData<Vec<u8>>;

pub trait JsonRpcMethodErased {
    fn name<'a>(&'a self) -> &'a str;

    fn create_request(
        &self,
        params: Vec<u8>,
    ) -> Result<JsonRpcRequestErased, serde_json::Error>;

    fn parse_json_response(
        &self,
        json_str: &str,
    ) -> Result<JsonRpcResponseErased, serde_json::Error>;
}

impl<I, O, E> JsonRpcMethodErased for JsonRpcMethod< I, O, E>
where
    I: serde::de::DeserializeOwned + Serialize,
    O: serde::de::DeserializeOwned + Serialize,
    E: serde::de::DeserializeOwned + Serialize,
{
    fn name(&self) -> &str {
        &self.method
    }

    fn create_request(&self, params: Vec<u8>) -> Result<JsonRpcRequestErased, serde_json::Error> {
        let typed_params: I = serde_json::from_slice(&params)?;
        let typed_json_rpc_request: JsonRpcRequest<I> =
            JsonRpcMethod::create_request(&self, typed_params);
        typed_json_rpc_request.erase()
    }

    fn parse_json_response(
        &self,
        json_str: &str,
    ) -> Result<JsonRpcResponseErased, serde_json::Error> {
        // Check if the json-struct matches the expected type
        let result: JsonRpcResponse<O, E> =
            JsonRpcMethod::<I, O, E>::parse_json_response(&self, json_str)?;
        return result.erase();
    }
}


impl<I, O, E> JsonRpcMethod<I, O, E>
where
    I: serde::de::DeserializeOwned + Serialize + 'static,
    O: serde::de::DeserializeOwned + Serialize + 'static,
    E: serde::de::DeserializeOwned + Serialize + 'static,
{
    pub fn erase_box(self) -> Box<dyn JsonRpcMethodErased> {
        return Box::new(self);
    }

    pub fn ref_erase<'a>(&'a self) -> &'a dyn JsonRpcMethodErased {
        return self
    }

}



// The trait JsonRpcUnerased is only intended to be used by library developers
//
// The user of this library might want to use the strongly typed generic version
// or the fully type-erased version
//
// As a library developer, we don't want to implement the same functionality twice
// for the same RPC-call.
//
// That is why we introduce the JsonRpcUnerased trait.
// It fills in the serde_json::Value type wherever either I, O or E should be.
//
// By using this trait, functionality will work for both type of users

pub trait JsonRpcMethodUnerased<'a, I, O, E> {

    fn name(&self) -> &str;

    fn create_request(&self, params: I) -> Result<JsonRpcRequest<I>, serde_json::Error>;

    fn parse_json_response(
        &self,
        json_str: &str,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error>;
}

// Dummy implementation for when the user uses the generic api
impl<'a, I, O, E> JsonRpcMethodUnerased<'a, I, O, E> for JsonRpcMethod<I, O, E>
where
    O: serde::de::DeserializeOwned,
    E: serde::de::DeserializeOwned,
{
    fn name(&self) -> &str {
        JsonRpcMethod::name(self)
    }

    fn create_request(&self, params: I) -> Result<JsonRpcRequest<I>, serde_json::Error> {
        Ok(JsonRpcMethod::create_request(&self, params))
    }

    fn parse_json_response(
        &self,
        json_str: &str,
    ) -> Result<JsonRpcResponse<O, E>, serde_json::Error> {
        JsonRpcMethod::parse_json_response(&self, json_str)
    }
}

struct UneraseWrapper<'a> {
    inner : &'a dyn JsonRpcMethodErased
}

impl<'a> JsonRpcMethodUnerased<'a, Vec<u8>, Vec<u8>, Vec<u8>> for UneraseWrapper<'a> {

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn create_request(&self, params: Vec<u8>) -> Result<JsonRpcRequest<Vec<u8>>, serde_json::Error> {
        self.inner.create_request(params)
    }

    fn parse_json_response(
            &self,
            json_str: &str,
        ) -> Result<JsonRpcResponse<Vec<u8>, Vec<u8>>, serde_json::Error> {
       self.inner.parse_json_response(json_str)
    }

}

impl dyn JsonRpcMethodErased {

    // The impl promises here we return a concrete type
    // However, we'd rather keep the implementation details private in this module and don't want users messing with it
    pub fn unerase<'a>(&'a self) -> impl JsonRpcMethodUnerased<'a, Vec<u8>, Vec<u8>, Vec<u8>> {
        UneraseWrapper { inner: self }
    }
}

impl<I> JsonRpcRequest<I>
where
    I: Serialize,
{
    fn erase(self) -> Result<JsonRpcRequestErased, serde_json::Error> {
        let value = serde_json::to_vec(&self.params)?;
        Ok(JsonRpcRequest {
            json_rpc: self.json_rpc,
            id: self.id,
            method: self.method,
            params: value,
        })
    }
}

impl<O> JsonRpcResponseSuccess<O>
where
    O: Serialize,
{
    fn erase(self) -> Result<JsonRpcResponseSuccessErased, serde_json::Error> {
        Ok(JsonRpcResponseSuccessErased {
            id: self.id,
            result: serde_json::to_vec(&self.result)?,
            json_rpc: self.json_rpc,
        })
    }
}

impl<E> JsonRpcResponseFailure<E>
where
    E: Serialize,
{
    fn erase(self) -> Result<JsonRpcResponseFailureErased, serde_json::Error> {
        Ok(JsonRpcResponseFailureErased {
            id: self.id,
            error: self.error.erase()?,
            json_rpc: self.json_rpc,
        })
    }
}

impl<E> ErrorData<E>
where
    E: Serialize,
{
    fn erase(self) -> Result<JsonRpcErrorDataErased, serde_json::Error> {
        let error_data = if let Some(error) = &self.data {
            Some(serde_json::to_vec(error)?)
        } else {
            None
        };

        let x = JsonRpcErrorDataErased {
            code: self.code,
            data: error_data,
            message: self.message,
        };

        Ok(x)
    }
}

impl<O, E> JsonRpcResponse<O, E>
where
    O: Serialize,
    E: Serialize,
{
    fn erase(self) -> Result<JsonRpcResponseErased, serde_json::Error> {
        let result = match self {
            Self::Ok(ok) => JsonRpcResponseErased::Ok(ok.erase()?),
            Self::Error(err) => JsonRpcResponseErased::Error(err.erase()?),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lsps::json_rpc::JsonRpcMethod;

    #[derive(Serialize, serde::Deserialize)]
    struct TestStruct {
        test: String,
    }

    #[test]
    fn create_rpc_request_from_method_erased() {
        let rpc_method = JsonRpcMethod::<TestStruct, (), ()>::new("test.method");
        let rpc_method_erased = rpc_method.erase_box();

        // This rpc-request should work becasue the parameters match the schema
        let data = serde_json::json!({"test" : "This should work"});
        let rpc_request = rpc_method_erased.create_request(data).unwrap();
        assert_eq!(rpc_request.method, "test.method");
        assert_eq!(
            rpc_request.params.get("test").unwrap().as_str().unwrap(),
            "This should work"
        );
    }

    #[test]
    fn create_rpc_request_from_method_erased_checks_types() {
        let rpc_method = JsonRpcMethod::<TestStruct, (), ()>::new("test.method");
        let rpc_method_erased = rpc_method.erase_box();

        // This rpc-request should fail because the parameters do not match the schema
        // The test field is missing
        let rpc_request = rpc_method_erased.create_request(serde_json::json!({}));
        assert!(rpc_request.is_err())
    }

}
