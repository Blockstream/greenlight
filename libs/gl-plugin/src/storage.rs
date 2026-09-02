//! A backend to store the signer state in.

pub use gl_client::persist::State;
use log::debug;
use serde::{Deserialize, Serialize};
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

/// A structure that is used for storing JIT channel requests metadata that is
/// requested through [Node::lsp_invoice](pb::node_server::Node::lsp_invoice)
/// RPC call.
///
/// This structure is stored in CLN datastore. The reason of why do we need this
/// structure instead of querying invoices table is that we want to distinguish
/// incomming payments whether they were for JIT channel opening or just a simple
/// payment. Currently, CLN does not allow this, that's why this workaround
/// exists.
#[derive(Serialize, Deserialize)]
pub struct JitRequestMeta {
    /// A label of the requested invoice.
    pub label: String,
    /// Payment hash of the requested invoice.
    pub payment_hash: String,
    /// The requested amount of msats.
    pub requested_amount_msat: u64,
    /// The expected (reduced) amount if msats.
    ///
    /// Note that expected_amount_msat <= requested_amount_msat since
    /// expected_amount_msat = requested_amount_msat + lsp_fee.
    pub expected_amount_msat: u64,
    /// Original Bolt11 invoice which includes requested_amount_msat.
    pub bolt11: String,
    /// ID of the LSP through which the invoice was requested.
    pub lsp_id: String,
}
