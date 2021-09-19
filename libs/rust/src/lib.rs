//! Greenlight client library to schedule nodes, interact with them
//! and sign off on signature requests.
//!

use std::convert::TryFrom;
extern crate anyhow;
use anyhow::{anyhow, Result};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

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

/// Register, recover and schedule your nodes on greenlight.
pub mod scheduler;

/// Your keys, your coins!
///
/// This module implements the logic to stream, verify and respond to
/// signature requests from the node. Without this the node cannot
/// move your funds.
pub mod signer;

/// Helpers to configure the mTLS connection authentication.
///
/// mTLS configuration for greenlight clients. Clients are
/// authenticated by presenting a valid mTLS certificate to the
/// node. Each node has its own CA. This CA is used to sign both the
/// device certificates, as well as the node certificate itself. This
/// ensures that only clients that are authorized can open a
/// connection to the node.
pub mod tls;

/// Which network are we running on?
#[derive(Copy, Clone)]
pub enum Network {
    BITCOIN,
    TESTNET,
    REGTEST,
}

impl Into<&'static str> for Network {
    fn into(self: Network) -> &'static str {
        match self {
            Network::BITCOIN => "bitcoin",
            Network::TESTNET => "testnet",
            Network::REGTEST => "regtest",
        }
    }
}

impl TryFrom<String> for Network {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Network> {
        let s = s.to_lowercase();
        match s.as_str() {
            "bitcoin" => Ok(Network::BITCOIN),
            "testnet" => Ok(Network::TESTNET),
            "regtest" => Ok(Network::REGTEST),
            o => Err(anyhow!("Could not parse network {}", o)),
        }
    }
}

/// Tools to interact with a node running on greenlight.
pub mod utils {

    pub fn scheduler_uri() -> String {
        std::env::var("GL_SCHEDULER_GRPC_URI")
            .unwrap_or("https://scheduler.gl.blckstrm.com:2601".to_string())
    }
}
