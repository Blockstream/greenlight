//! Greenlight client library to schedule nodes, interact with them
//! and sign off on signature requests.
//!

/// Interact with a node running on greenlight.
///
/// The node must be scheduled using [`crate::scheduler::Scheduler`]:
///
///
pub mod node;

/// Generated protobuf messages and client stubs.
///
/// Since the client needs to be configured correctly, don't use
/// [`pb::node_client::NodeClient`] directly, rather use
/// [`node::Node`] to create a correctly configured client.
pub mod pb;

use std::time::Duration;

/// Register, recover and schedule your nodes on greenlight.
pub mod scheduler;

/// Your keys, your coins!
///
/// This module implements the logic to stream, verify and respond to
/// signature requests from the node. Without this the node cannot
/// move your funds.
pub mod signer;

pub mod persist;

pub mod lnurl;

/// The pairing service that pairs signer-less clients with existing
/// signers.
pub mod pairing;
/// Helpers to configure the mTLS connection authentication.
///
/// mTLS configuration for greenlight clients. Clients are
/// authenticated by presenting a valid mTLS certificate to the
/// node. Each node has its own CA. This CA is used to sign both the
/// device certificates, as well as the node certificate itself. This
/// ensures that only clients that are authorized can open a
/// connection to the node.
pub mod tls;

#[cfg(feature = "export")]
pub mod export;

/// Tools to interact with a node running on greenlight.
pub mod utils;

pub mod credentials;

pub mod util;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The signature request does not match any authorized RPC calls")]
    MissingAuthorization,
}

pub use lightning_signer::bitcoin;
pub use lightning_signer::lightning;
pub use lightning_signer::lightning_invoice;

pub(crate) const TCP_KEEPALIVE: Duration = Duration::from_secs(1);
pub(crate) const TCP_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(5);

pub mod runes;
