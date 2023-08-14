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
        return Self::Other(value.to_string());
    }
}
