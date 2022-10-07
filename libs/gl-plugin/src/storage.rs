//! A backend to store the signer state in.

pub use gl_client::persist::State;
use log::debug;
use thiserror::Error;
use tonic::async_trait;

#[derive(Debug, Error)]
pub enum Error {
    /// underlying database error
    #[error("database error: {0}")]
    Sled(#[from] ::sled::Error),
    #[error("state corruption: {0}")]
    CorruptState(#[from] serde_json::Error),
    #[error("unhandled error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait]
pub trait StateStore: Send + Sync {
    async fn write(&self, state: State) -> Result<(), Error>;
    async fn read(&self) -> Result<State, Error>;
}

/// A StateStore that uses `sled` as its storage backend
pub struct SledStateStore {
    db: sled::Db,
}

impl SledStateStore {
    pub fn new(path: std::path::PathBuf) -> Result<SledStateStore, sled::Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
}

use sled::transaction::TransactionError;
impl From<TransactionError<Error>> for Error {
    fn from(e: TransactionError<Error>) -> Self {
        match e {
            TransactionError::Abort(e) => e,
            TransactionError::Storage(e) => Error::Sled(e),
        }
    }
}

const SLED_KEY: &str = "signer_state";

#[async_trait]
impl StateStore for SledStateStore {
    async fn read(&self) -> Result<State, Error> {
        match self.db.get(SLED_KEY)? {
            None => {
                debug!("Initializing a new signer state");
                Ok(State::new())
            }
            Some(v) => Ok(serde_json::from_slice(&v)?),
        }
    }

    async fn write(&self, state: State) -> Result<(), Error> {
        let raw = serde_json::to_vec(&state)?;
        self.db
            .insert(SLED_KEY, raw)
            .map(|_v| ())
            .map_err(|e| e.into())
    }
}
