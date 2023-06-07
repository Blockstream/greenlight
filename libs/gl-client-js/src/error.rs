use prost::DecodeError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error calling {meth}: {err}")]
    Call { meth: String, err: String },
    #[error("error converting: {0}")]
    Convert(String),

    #[error("error decoding response: {source}")]
    Decode {
        #[from]
        source: DecodeError,
    },
    #[error("error performing a call: {source}")]
    Status {
        #[from]
        source: tonic::Status,
    },
    #[error("no method {0}")]
    NoSuchMethod(String),
}
