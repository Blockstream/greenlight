use crate::pb::scheduler::PairDeviceResponse;
use thiserror::Error;

pub mod new_device;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
    #[error(transparent)]
    X509Error(#[from] rcgen::RcgenError),
}

pub enum PairingSessionData {
    PairingResponse(PairDeviceResponse),
    PairingError(tonic::Status),
}
