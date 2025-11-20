uniffi::setup_scaffolding!();

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum Error {
    #[error("There is already a node for node_id={0}, maybe you want to recover?")]
    DuplicateNode(String),

    #[error("There is no node with node_id={0}, maybe you need to register first?")]
    NoSuchNode(String),

    #[error("The provided credentials could not be parsed, please recover.")]
    UnparseableCreds(),

    #[error("The passphrase you provided fails the checksum")]
    PhraseCorrupted(),

    #[error("Error calling the rpc: {0}")]
    Rpc(String),

    #[error("Invalid argument: {0}={1}")]
    Argument(String, String),
    
    #[error("Generic error: {0}")]
    Other(String),
}
mod credentials;
mod node;
mod scheduler;
mod signer;
mod util;

pub use crate::{
    credentials::Credentials,
    node::Node,
    scheduler::Scheduler,
    signer::{Handle, Signer},
};

#[derive(uniffi::Enum, Debug)]
pub enum Network {
    BITCOIN,
    REGTEST,
}

impl From<Network> for gl_client::bitcoin::Network {
    fn from(n: Network) -> gl_client::bitcoin::Network {
        match n {
            Network::BITCOIN => gl_client::bitcoin::Network::Bitcoin,
            Network::REGTEST => gl_client::bitcoin::Network::Regtest,
        }
    }
}
