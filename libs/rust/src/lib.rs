pub use libhsmd_sys::Hsmd;
use std::convert::TryFrom;
extern crate anyhow;
use anyhow::{anyhow, Result};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub mod node;
pub mod pb;
pub mod signer;
pub mod scheduler;
pub mod tls;

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
