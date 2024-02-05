#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unhandled error: {0}")]
    Other(#[from] anyhow::Error),
    #[error("Invalid argument error: {0}")]
    InvalidArgument(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error while calling {method}: {error}")]
    Call {
        method: String,
        error: String,
    },
}
