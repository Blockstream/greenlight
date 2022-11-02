//! Greenlight client library to schedule nodes, interact with them
//! and sign off on signature requests.
//!

extern crate anyhow;

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

pub mod persist;

/// Helpers to configure the mTLS connection authentication.
///
/// mTLS configuration for greenlight clients. Clients are
/// authenticated by presenting a valid mTLS certificate to the
/// node. Each node has its own CA. This CA is used to sign both the
/// device certificates, as well as the node certificate itself. This
/// ensures that only clients that are authorized can open a
/// connection to the node.
pub mod tls;

/// Tools to interact with a node running on greenlight.
pub mod utils {
    use bech32::{self, ToBase32};
    use anyhow::Result;


    pub fn scheduler_uri() -> String {
        std::env::var("GL_SCHEDULER_GRPC_URI")
            .unwrap_or_else(|_| "https://scheduler.gl.blckstrm.com:2601".to_string())
    }

    /// Used to get the uri of the node. The nodes uri is of the form
    /// `https://[node_id: bech32].node.gl.blckstrm.com`. The node id is bech32
    /// formatted as a DNS label has a max length of 63 octets. We use the hrp
    /// `gl` to indicate that this "address" is meant to be used with
    ///  greenlight.
    pub fn get_node_uri(node_id: Vec<u8>) -> Result<String> {
        // Override the uri is mainly used for test purposes.
        match std::env::var("GL_NODE_URI") {
            Ok(uri) => return Ok(uri),
            Err(_) => (),
        };

        // Encode to bech32 to match the DNS label limit of 63 octets
        // per label.
        let label = bech32::encode("gl",node_id.to_base32(), bech32::Variant::Bech32)?;
        Ok(format!("https://{}.nodes.gl.blckstrm.com", label))
    }
}

#[cfg(test)]
mod tests {
    use super::{utils};
    use anyhow::Result;
    use hex::FromHex;
    
    #[test]
    fn test_get_node_uri() -> Result<()> {
        let node_id = Vec::from_hex("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")?;
        let uri = utils::get_node_uri(node_id)?;
        assert!(uri == "https://gl1qfumuen7l8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqtesc2mj04.nodes.gl.blckstrm.com");
        Ok(())
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The signature request does not match any authorized RPC calls")]
    MissingAuthorization,
}
