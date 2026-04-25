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

mod config;
mod credentials;
mod input;
mod node;
mod scheduler;
mod signer;
mod util;

pub use crate::{
    config::Config,
    credentials::{Credentials, DeveloperCert},
    node::{
        ChannelState, FundChannel, FundOutput, GetInfoResponse, Invoice, InvoicePaidEvent,
        InvoiceStatus, ListFundsResponse, ListIndex, ListInvoicesResponse,
        ListPaymentsRequest, ListPeerChannelsResponse, ListPaysResponse, ListPeersResponse,
        Node, NodeEvent, NodeEventStream, NodeState, OnchainReceiveResponse, OnchainSendResponse,
        OutputStatus, Pay, PayStatus, Payment, PaymentStatus, PaymentType, PaymentTypeFilter,
        Peer, PeerChannel, ReceiveResponse, SendResponse,
    },
    input::{InputType, ParsedInvoice},
    scheduler::Scheduler,
    signer::{Handle, Signer},
};

/// Which scheduler operation to perform.
enum SchedulerAction {
    Register { invite_code: Option<String> },
    Recover,
}

/// Shared implementation for register and recover flows.
fn schedule_node(
    seed: Vec<u8>,
    config: &config::Config,
    action: SchedulerAction,
) -> Result<std::sync::Arc<node::Node>, Error> {
    use std::sync::Arc;

    let network = config.network;
    let nobody = config.nobody();

    let seed_for_async = seed.clone();
    let credentials = util::exec(async move {
        let signer =
            gl_client::signer::Signer::new(seed_for_async, network, nobody.clone())
                .map_err(|e| Error::Other(e.to_string()))?;

        let scheduler = gl_client::scheduler::Scheduler::new(network, nobody)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

        let node_id_hex = hex::encode(signer.node_id());

        let creds_bytes = match action {
            SchedulerAction::Register { invite_code } => {
                scheduler
                    .register(&signer, invite_code)
                    .await
                    .map_err(|e| map_scheduler_error(e, &node_id_hex))?
                    .creds
            }
            SchedulerAction::Recover => {
                scheduler
                    .recover(&signer)
                    .await
                    .map_err(|e| map_scheduler_error(e, &node_id_hex))?
                    .creds
            }
        };

        credentials::Credentials::load(creds_bytes)
    })?;

    let authenticated_signer =
        gl_client::signer::Signer::new(seed, network, credentials.inner.clone())
            .map_err(|e| Error::Other(e.to_string()))?;

    let handle = signer::Handle::spawn(authenticated_signer);
    let node = node::Node::with_signer(credentials, handle)?;
    Ok(Arc::new(node))
}

/// Map scheduler errors to specific Error variants.
/// First tries tonic status codes, then falls back to error message matching.
fn map_scheduler_error(e: anyhow::Error, node_id_hex: &str) -> Error {
    // Walk the error chain looking for a tonic::Status with a specific code
    for cause in e.chain() {
        if let Some(status) = cause.downcast_ref::<tonic::Status>() {
            match status.code() {
                tonic::Code::AlreadyExists => {
                    return Error::DuplicateNode(node_id_hex.to_string())
                }
                tonic::Code::NotFound => return Error::NoSuchNode(node_id_hex.to_string()),
                // Don't return here — the tonic status might be a generic
                // wrapper (e.g. Internal/Unknown) around a more specific
                // error. Fall through to string matching.
                _ => {}
            }
        }
    }

    // Fall back to checking the full error message for known patterns.
    let msg = e.to_string();
    if msg.contains("NOT_FOUND")
        || msg.contains("no rows returned")
        || msg.contains("Recovery failed")
    {
        Error::NoSuchNode(node_id_hex.to_string())
    } else if msg.contains("ALREADY_EXISTS") {
        Error::DuplicateNode(node_id_hex.to_string())
    } else {
        Error::Other(msg)
    }
}

/// Parse a BIP39 mnemonic into a seed.
fn parse_mnemonic(mnemonic: &str) -> Result<Vec<u8>, Error> {
    use bip39::Mnemonic;
    use std::str::FromStr;
    let phrase = Mnemonic::from_str(mnemonic).map_err(|_e| Error::PhraseCorrupted())?;
    Ok(phrase.to_seed_normalized("").to_vec())
}

/// Register a new Greenlight node and return a connected Node with signer running.
///
/// The app should call `node.credentials()` to get the credential bytes
/// and persist them for future `connect()` calls.
#[uniffi::export]
pub fn register(
    mnemonic: String,
    invite_code: Option<String>,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    let seed = parse_mnemonic(&mnemonic)?;
    schedule_node(seed, config, SchedulerAction::Register { invite_code })
}

/// Recover credentials for an existing Greenlight node and return a connected Node.
///
/// The app should call `node.credentials()` to get the credential bytes
/// and persist them for future `connect()` calls.
#[uniffi::export]
pub fn recover(
    mnemonic: String,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    let seed = parse_mnemonic(&mnemonic)?;
    schedule_node(seed, config, SchedulerAction::Recover)
}

/// Connect to an existing Greenlight node using previously saved credentials.
#[uniffi::export]
pub fn connect(
    mnemonic: String,
    credentials: Vec<u8>,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    use std::sync::Arc;

    let seed = parse_mnemonic(&mnemonic)?;
    let network = config.network;
    let creds = credentials::Credentials::load(credentials)?;

    let authenticated_signer =
        gl_client::signer::Signer::new(seed, network, creds.inner.clone())
            .map_err(|e| Error::Other(e.to_string()))?;

    let handle = signer::Handle::spawn(authenticated_signer);
    let node = node::Node::with_signer(creds, handle)?;
    Ok(Arc::new(node))
}

/// Try to recover an existing node; if none exists, register a new one.
#[uniffi::export]
pub fn register_or_recover(
    mnemonic: String,
    invite_code: Option<String>,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    match recover(mnemonic.clone(), config) {
        Ok(node) => Ok(node),
        Err(Error::NoSuchNode(_)) => register(mnemonic, invite_code, config),
        Err(e) => Err(e),
    }
}

/// Parse a string and identify whether it's a BOLT11 invoice or a node ID.
///
/// Strips `lightning:` / `LIGHTNING:` prefixes automatically.
/// Works offline — no node connection needed.
#[uniffi::export]
pub fn parse_input(input: String) -> Result<input::InputType, Error> {
    input::parse_input(input)
}

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
