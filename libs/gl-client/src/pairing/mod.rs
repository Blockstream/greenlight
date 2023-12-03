use crate::{credentials, pb::scheduler::PairDeviceResponse};
use thiserror::Error;

pub mod attestation_device;
pub mod new_device;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
    #[error(transparent)]
    X509Error(#[from] rcgen::RcgenError),
    #[error("could not build client: {0}")]
    BuildClientError(String),
    #[error(transparent)]
    GrpcError(#[from] tonic::Status),
    #[error(transparent)]
    CredentialsError(#[from] credentials::Error),
    #[error(transparent)]
    RuneError(#[from] runeauth::RuneError),
    #[error("could not approve pairing: {0}")]
    ApprovePairingError(String),
}

pub enum PairingSessionData {
    PairingResponse(PairDeviceResponse),
    PairingQr(String),
    PairingError(tonic::Status),
}

fn into_approve_pairing_error<T: ToString>(v: T) -> Error {
    Error::ApprovePairingError(v.to_string())
}
