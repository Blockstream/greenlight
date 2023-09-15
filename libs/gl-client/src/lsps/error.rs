use thiserror::Error;

#[derive(Error, Debug)]
pub enum LspsError {
    #[error("Unknown method")]
    MethodUnknown(String),
    #[error("Failed to parse json-request")]
    JsonParseRequestError(serde_json::Error),
    #[error("Failed to parse json-response")]
    JsonParseResponseError(serde_json::Error),
    #[error("Error while calling lightning grpc-method")]
    GrpcError(#[from] tonic::Status),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Timeout")]
    Timeout,
    #[error("Something unexpected happened")]
    Other(String),
}

impl From<std::io::Error> for LspsError {
    fn from(value: std::io::Error) -> Self {
        Self::Other(value.to_string())
    }
}

pub fn map_json_rpc_error_code_to_str(code: i64) -> &'static str {
    match code {
        -32700 => "parsing_error",
        -32600 => "invalid_request",
        -32601 => "method_not_found",
        -32602 => "invalid_params",
        -32603 => "internal_error",
        -32099..=-32000 => "implementation_defined_server_error",
        _ => "unknown_error_code",
    }
}

#[cfg(test)]
mod test {
    use crate::lsps::error::map_json_rpc_error_code_to_str;

    #[test]
    fn test_map_json_rpc_error_code_to_str() {
        assert_eq!(map_json_rpc_error_code_to_str(12), "unknown_error_code");
        assert_eq!(map_json_rpc_error_code_to_str(-32603), "internal_error");
    }
}
