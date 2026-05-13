use std::collections::HashMap;

uniffi::setup_scaffolding!();

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum Error {
    #[error("{message}")]
    DuplicateNode {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },

    #[error("{message}")]
    NoSuchNode {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },

    #[error("{message}")]
    UnparseableCreds {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },

    #[error("{message}")]
    PhraseCorrupted {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },

    #[error("{message}")]
    Rpc {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },

    #[error("{message}")]
    Argument {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },

    #[error("{message}")]
    Other {
        code: i32,
        message: String,
        values: HashMap<String, String>,
    },
}

impl Error {
    pub fn duplicate_node(node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        Error::DuplicateNode {
            code: 1000,
            message: format!(
                "There is already a node for node_id={node_id}, maybe you want to recover?"
            ),
            values: HashMap::from([("node_id".into(), node_id)]),
        }
    }

    pub fn no_such_node(node_id: impl Into<String>) -> Self {
        let node_id = node_id.into();
        Error::NoSuchNode {
            code: 1001,
            message: format!(
                "There is no node with node_id={node_id}, maybe you need to register first?"
            ),
            values: HashMap::from([("node_id".into(), node_id)]),
        }
    }

    pub fn unparseable_creds() -> Self {
        Error::UnparseableCreds {
            code: 1100,
            message: "The provided credentials could not be parsed, please recover.".into(),
            values: HashMap::new(),
        }
    }

    pub fn phrase_corrupted() -> Self {
        Error::PhraseCorrupted {
            code: 1101,
            message: "The passphrase you provided fails the checksum".into(),
            values: HashMap::new(),
        }
    }

    pub fn rpc(detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Error::Rpc {
            code: 2000,
            message: format!("Error calling the rpc: {detail}"),
            values: HashMap::from([("detail".into(), detail)]),
        }
    }

    pub fn argument(arg_name: impl Into<String>, arg_value: impl Into<String>) -> Self {
        let arg_name = arg_name.into();
        let arg_value = arg_value.into();
        Error::Argument {
            code: 3000,
            message: format!("Invalid argument: {arg_name}={arg_value}"),
            values: HashMap::from([
                ("arg_name".into(), arg_name),
                ("arg_value".into(), arg_value),
            ]),
        }
    }

    pub fn other(detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Error::Other {
            code: 9000,
            message: format!("Generic error: {detail}"),
            values: HashMap::from([("detail".into(), detail)]),
        }
    }
}

mod config;
mod credentials;
mod input;
mod lnurl;
mod logging;
mod node;
mod node_builder;
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
        Node, NodeEvent, NodeEventListener, NodeEventStream, NodeState, OnchainReceiveResponse,
        OnchainSendResponse, OutputStatus, Pay, PayStatus, Payment, PaymentStatus, PaymentType,
        PaymentTypeFilter, Peer, PeerChannel, ReceiveResponse, SendResponse,
    },
    input::{ParsedInput, ParsedInvoice, ResolvedInput},
    logging::{LogEntry, LogLevel, LogListener},
    lnurl::{
        LnUrlErrorData, LnUrlPayRequest, LnUrlPayRequestData, LnUrlPayResult,
        LnUrlPaySuccessData, LnUrlWithdrawRequest, LnUrlWithdrawRequestData,
        LnUrlWithdrawResult, LnUrlWithdrawSuccessData, SuccessActionProcessed,
    },
    node_builder::NodeBuilder,
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
                .map_err(|e| Error::other(e.to_string()))?;

        let scheduler = gl_client::scheduler::Scheduler::new(network, nobody)
            .await
            .map_err(|e| Error::other(e.to_string()))?;

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
            .map_err(|e| Error::other(e.to_string()))?;

    let handle = signer::Handle::spawn(authenticated_signer);
    let node = node::Node::with_signer(credentials, handle, network)?;
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
                    return Error::duplicate_node(node_id_hex.to_string())
                }
                tonic::Code::NotFound => return Error::no_such_node(node_id_hex.to_string()),
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
        Error::no_such_node(node_id_hex.to_string())
    } else if msg.contains("ALREADY_EXISTS") {
        Error::duplicate_node(node_id_hex.to_string())
    } else {
        Error::other(msg)
    }
}

/// Parse a BIP39 mnemonic into a seed.
fn parse_mnemonic(mnemonic: &str) -> Result<Vec<u8>, Error> {
    use bip39::Mnemonic;
    use std::str::FromStr;
    let phrase = Mnemonic::from_str(mnemonic).map_err(|_e| Error::phrase_corrupted())?;
    Ok(phrase.to_seed_normalized("").to_vec())
}

/// Crate-internal: connect using saved credentials. The builder
/// (`NodeBuilder::connect`) is the public entry point.
pub(crate) fn connect_internal(
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
            .map_err(|e| Error::other(e.to_string()))?;

    let handle = signer::Handle::spawn(authenticated_signer);
    let node = node::Node::with_signer(creds, handle, network)?;
    Ok(Arc::new(node))
}

/// Crate-internal: register a fresh node. The builder
/// (`NodeBuilder::register`) is the public entry point.
pub(crate) fn register_internal(
    mnemonic: String,
    invite_code: Option<String>,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    let seed = parse_mnemonic(&mnemonic)?;
    schedule_node(seed, config, SchedulerAction::Register { invite_code })
}

/// Crate-internal: recover an existing node. The builder
/// (`NodeBuilder::recover`) is the public entry point.
pub(crate) fn recover_internal(
    mnemonic: String,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    let seed = parse_mnemonic(&mnemonic)?;
    schedule_node(seed, config, SchedulerAction::Recover)
}

/// Crate-internal: register-or-recover. The builder
/// (`NodeBuilder::register_or_recover`) is the public entry point.
pub(crate) fn register_or_recover_internal(
    mnemonic: String,
    invite_code: Option<String>,
    config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    match recover_internal(mnemonic.clone(), config) {
        Ok(node) => Ok(node),
        Err(Error::NoSuchNode { .. }) => register_internal(mnemonic, invite_code, config),
        Err(e) => Err(e),
    }
}

/// Crate-internal: connect signerless — credentials only, no
/// SDK-side signer spawned. Used by `NodeBuilder::connect` when no
/// mnemonic is set.
///
/// Signing-required RPCs (`pay`, `receive` JIT-channel, etc.) rely
/// on a signer running elsewhere — typically the CLN node's local
/// signer or a paired device. This is the supported model for
/// signerless clients (browser extensions, paired devices, hardware
/// signers held outside the SDK process).
pub(crate) fn connect_signerless_internal(
    credentials: Vec<u8>,
    _config: &config::Config,
) -> Result<std::sync::Arc<node::Node>, Error> {
    use std::sync::Arc;
    let creds = credentials::Credentials::load(credentials)?;
    let node = node::Node::signerless(creds)?;
    Ok(Arc::new(node))
}

/// Synchronously classify the input. **No HTTP, no I/O.**
///
/// Recognises BOLT11 invoices, node IDs, LNURL bech32 strings, and
/// Lightning Addresses. Strips `lightning:` / `LIGHTNING:` prefixes
/// automatically. LNURL inputs are decoded to their underlying URL
/// but **not fetched** — the caller chooses whether to resolve
/// further (via `resolve_input`) or to surface the URL to the user
/// as-is.
///
/// Use this for offline operations like clipboard validation or
/// invoice sanity checks. Use `resolve_input` for the QR-scan flow
/// where you want the resolved pay/withdraw data in one call.
#[uniffi::export]
pub fn parse_input(input: String) -> Result<input::ParsedInput, Error> {
    input::parse_input(input)
}

/// Classify and resolve the input.
///
/// Internally calls `parse_input` for offline classification, then
/// for LNURL bech32 strings and Lightning Addresses performs the
/// HTTP GET to the LNURL endpoint and returns typed pay or withdraw
/// request data. For BOLT11 invoices and node IDs it returns
/// immediately without I/O.
///
/// Strips `lightning:` / `LIGHTNING:` prefixes automatically.
///
/// # Blocking
///
/// This function blocks the calling thread while any network I/O
/// completes. The SDK exposes a **synchronous-only** public API so
/// that every language binding (Python, Kotlin, Swift, Ruby, C++)
/// works without requiring an async runtime on the caller side.
/// Async work is executed internally on a shared Tokio runtime
/// managed by the SDK.
#[uniffi::export]
pub fn resolve_input(input: String) -> Result<input::ResolvedInput, Error> {
    util::exec(async { input::resolve_input(input).await })
}

/// Set up SDK logging. Call once before any other SDK function.
///
/// The listener receives all log messages from the SDK and the
/// underlying Greenlight client library. Call once, as early as
/// possible, so early logs are captured. Returns an error if a logger
/// has already been installed in this process. To change the filter
/// after installation, use `set_log_level`.
#[uniffi::export]
pub fn set_logger(
    level: logging::LogLevel,
    listener: Box<dyn logging::LogListener>,
) -> Result<(), Error> {
    logging::set_logger(level, listener)
}

/// Change the log filter at runtime without reinstalling the listener.
#[uniffi::export]
pub fn set_log_level(level: logging::LogLevel) {
    logging::set_log_level(level)
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
