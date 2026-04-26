#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

// Import from glsdk crate (gl-sdk library)
use glsdk::{
    // Enum types for conversion
    ChannelState as GlChannelState,
    Credentials as GlCredentials,
    DeveloperCert as GlDeveloperCert,
    Handle as GlHandle,
    InputType as GlInputType,
    Network as GlNetwork,
    Node as GlNode,
    NodeEvent as GlNodeEvent,
    NodeEventStream as GlNodeEventStream,
    OutputStatus as GlOutputStatus,
    ParsedInvoice as GlParsedInvoice,
    Scheduler as GlScheduler,
    Signer as GlSigner,
};

// ============================================================================
// Response Types (must be defined first as they're used by other structs)
// ============================================================================

#[napi(object)]
pub struct ReceiveResponse {
    pub bolt11: String,
}

#[napi(object)]
pub struct SendResponse {
    pub status: u32,
    pub preimage: Buffer,
    pub payment_hash: Buffer,
    pub destination_pubkey: Option<Buffer>,
    /// Amount in millisatoshis (as i64 for JS compatibility)
    pub amount_msat: i64,
    /// Amount sent in millisatoshis (as i64 for JS compatibility)
    pub amount_sent_msat: i64,
    pub parts: u32,
}

#[napi(object)]
pub struct OnchainSendResponse {
    pub tx: Buffer,
    pub txid: Buffer,
    pub psbt: String,
}

#[napi(object)]
pub struct OnchainReceiveResponse {
    pub bech32: String,
    pub p2tr: String,
}

// ============================================================================
// Event Streaming Response Types
// ============================================================================

#[napi(object)]
pub struct InvoicePaidEvent {
    pub payment_hash: Buffer,
    pub bolt11: String,
    pub preimage: Buffer,
    pub label: String,
    /// Amount in millisatoshis (as i64 for JS compatibility)
    pub amount_msat: i64,
}

#[napi(object)]
pub struct NodeEvent {
    /// Discriminant: "invoice_paid" | "unknown"
    pub event_type: String,
    /// Present when event_type == "invoice_paid"
    pub invoice_paid: Option<InvoicePaidEvent>,
}

// ============================================================================
// GetInfo Response Types
// ============================================================================

#[napi(object)]
pub struct GetInfoResponse {
    pub id: Buffer,
    pub alias: Option<String>,
    pub color: Buffer,
    pub num_peers: u32,
    pub num_pending_channels: u32,
    pub num_active_channels: u32,
    pub num_inactive_channels: u32,
    pub version: String,
    pub lightning_dir: String,
    pub blockheight: u32,
    pub network: String,
    /// Fees collected in millisatoshis (as i64 for JS compatibility)
    pub fees_collected_msat: i64,
}

// ============================================================================
// ListPeers Response Types
// ============================================================================

#[napi(object)]
pub struct ListPeersResponse {
    pub peers: Vec<Peer>,
}

#[napi(object)]
pub struct Peer {
    pub id: Buffer,
    pub connected: bool,
    pub num_channels: Option<u32>,
    pub netaddr: Vec<String>,
    pub remote_addr: Option<String>,
    pub features: Option<Buffer>,
}

// ============================================================================
// ListPeerChannels Response Types
// ============================================================================

#[napi(object)]
pub struct ListPeerChannelsResponse {
    pub channels: Vec<PeerChannel>,
}

#[napi(object)]
pub struct PeerChannel {
    pub peer_id: Buffer,
    pub peer_connected: bool,
    /// Channel state as string (e.g., "CHANNELD_NORMAL", "OPENINGD")
    pub state: String,
    pub short_channel_id: Option<String>,
    pub channel_id: Option<Buffer>,
    pub funding_txid: Option<Buffer>,
    pub funding_outnum: Option<u32>,
    /// Balance to us in millisatoshis (as i64 for JS compatibility)
    pub to_us_msat: Option<i64>,
    /// Total channel capacity in millisatoshis (as i64 for JS compatibility)
    pub total_msat: Option<i64>,
    /// Spendable balance in millisatoshis (as i64 for JS compatibility)
    pub spendable_msat: Option<i64>,
    /// Receivable balance in millisatoshis (as i64 for JS compatibility)
    pub receivable_msat: Option<i64>,
}

// ============================================================================
// ListFunds Response Types
// ============================================================================

#[napi(object)]
pub struct ListFundsResponse {
    pub outputs: Vec<FundOutput>,
    pub channels: Vec<FundChannel>,
}

#[napi(object)]
pub struct FundOutput {
    pub txid: Buffer,
    pub output: u32,
    /// Amount in millisatoshis (as i64 for JS compatibility)
    pub amount_msat: i64,
    /// Output status (e.g., "unconfirmed", "confirmed", "spent", "immature")
    pub status: String,
    pub address: Option<String>,
    pub blockheight: Option<u32>,
}

#[napi(object)]
pub struct FundChannel {
    pub peer_id: Buffer,
    /// Our amount in millisatoshis (as i64 for JS compatibility)
    pub our_amount_msat: i64,
    /// Total amount in millisatoshis (as i64 for JS compatibility)
    pub amount_msat: i64,
    pub funding_txid: Buffer,
    pub funding_output: u32,
    pub connected: bool,
    /// Channel state as string (e.g., "CHANNELD_NORMAL", "OPENINGD")
    pub state: String,
    pub short_channel_id: Option<String>,
    pub channel_id: Option<Buffer>,
}

// ============================================================================
// Input Parsing Types
// ============================================================================

#[napi(object)]
pub struct ParsedInvoice {
    pub bolt11: String,
    pub payee_pubkey: Option<Buffer>,
    pub payment_hash: Buffer,
    pub description: Option<String>,
    /// Amount in millisatoshis (i64 for JS), `None` for any-amount invoices.
    pub amount_msat: Option<i64>,
    /// Seconds from creation until the invoice expires.
    pub expiry: i64,
    /// Unix timestamp (seconds) when the invoice was created.
    pub timestamp: i64,
}

/// Parsed input. Discriminated by `type` field. Exactly one of the
/// variant fields (`bolt11`, `node_id`, `lnurl_pay`, `lnurl_withdraw`)
/// is populated based on the discriminant.
#[napi(object)]
pub struct InputType {
    /// "bolt11" | "node_id" | "lnurl_pay" | "lnurl_withdraw"
    pub r#type: String,
    /// Present when type == "bolt11"
    pub bolt11: Option<ParsedInvoice>,
    /// Present when type == "node_id"
    pub node_id: Option<String>,
    /// Present when type == "lnurl_pay"
    pub lnurl_pay: Option<LnUrlPayRequestData>,
    /// Present when type == "lnurl_withdraw"
    pub lnurl_withdraw: Option<LnUrlWithdrawRequestData>,
}

// ============================================================================
// LNURL Types
// ============================================================================

#[napi(object)]
pub struct LnUrlPayRequestData {
    pub callback: String,
    /// Minimum amount in millisatoshis (i64 for JS)
    pub min_sendable: i64,
    /// Maximum amount in millisatoshis (i64 for JS)
    pub max_sendable: i64,
    pub metadata: String,
    pub comment_allowed: i64,
    pub description: String,
    pub lnurl: String,
}

#[napi(object)]
pub struct LnUrlWithdrawRequestData {
    pub callback: String,
    pub k1: String,
    pub default_description: String,
    /// Minimum withdrawable in millisatoshis (i64 for JS)
    pub min_withdrawable: i64,
    /// Maximum withdrawable in millisatoshis (i64 for JS)
    pub max_withdrawable: i64,
    pub lnurl: String,
}

#[napi(object)]
pub struct LnUrlPayRequest {
    pub data: LnUrlPayRequestData,
    /// Amount in millisatoshis (i64 for JS)
    pub amount_msat: i64,
    pub comment: Option<String>,
    /// When true (the default), a URL success action is rejected if its
    /// domain differs from the callback's domain.
    pub validate_success_action_url: Option<bool>,
}

#[napi(object)]
pub struct LnUrlWithdrawRequest {
    pub data: LnUrlWithdrawRequestData,
    /// Amount in millisatoshis (i64 for JS)
    pub amount_msat: i64,
    pub description: Option<String>,
}

#[napi(object)]
pub struct LnUrlPaySuccessData {
    pub payment_preimage: String,
    pub success_action: Option<SuccessActionProcessed>,
}

#[napi(object)]
pub struct LnUrlErrorData {
    pub reason: String,
}

#[napi(object)]
pub struct LnUrlPayErrorData {
    pub payment_hash: String,
    pub reason: String,
}

/// Result of an LNURL-pay operation. Discriminated by `type` field.
#[napi(object)]
pub struct LnUrlPayResult {
    /// "success", "error", or "pay_error"
    pub r#type: String,
    /// Present when type == "success"
    pub success: Option<LnUrlPaySuccessData>,
    /// Present when type == "error" (LNURL service rejected the request)
    pub error: Option<LnUrlErrorData>,
    /// Present when type == "pay_error" (invoice fetched but paying it failed)
    pub pay_error: Option<LnUrlPayErrorData>,
}

#[napi(object)]
pub struct LnUrlWithdrawSuccessData {
    pub invoice: String,
}

/// Result of an LNURL-withdraw operation. Discriminated by `type` field.
#[napi(object)]
pub struct LnUrlWithdrawResult {
    /// "ok" or "error"
    pub r#type: String,
    /// Present when type == "ok"
    pub ok: Option<LnUrlWithdrawSuccessData>,
    /// Present when type == "error"
    pub error: Option<LnUrlErrorData>,
}

/// Processed success action. Discriminated by `type` field.
#[napi(object)]
pub struct SuccessActionProcessed {
    /// "message", "url", or "aes"
    pub r#type: String,
    /// Present for "message" type
    pub message: Option<String>,
    /// Present for "url" type
    pub description: Option<String>,
    /// Present for "url" type
    pub url: Option<String>,
    /// Present for "aes" type (decrypted plaintext)
    pub plaintext: Option<String>,
}

// ============================================================================
// Struct Definitions (all structs must be defined before impl blocks)
// ============================================================================

#[napi]
pub struct DeveloperCert {
    inner: GlDeveloperCert,
}

#[napi]
pub struct Credentials {
    inner: GlCredentials,
}

#[napi]
pub struct Scheduler {
    inner: GlScheduler,
}

#[napi]
pub struct Signer {
    inner: GlSigner,
}

#[napi]
pub struct Handle {
    inner: GlHandle,
}

#[napi]
pub struct Node {
    inner: std::sync::Arc<GlNode>,
}

#[napi]
pub struct NodeEventStream {
    inner: std::sync::Arc<GlNodeEventStream>,
}

// ============================================================================
// NAPI Implementations
// ============================================================================

#[napi]
impl DeveloperCert {
    /// Create a new developer certificate from cert and key PEM bytes
    /// obtained from the Greenlight Developer Console.
    ///
    /// # Arguments
    /// * `cert` - Certificate PEM bytes
    /// * `key` - Private key PEM bytes
    #[napi(constructor)]
    pub fn new(cert: Buffer, key: Buffer) -> Self {
        let inner = GlDeveloperCert::new(cert.to_vec(), key.to_vec());
        Self { inner }
    }
}

#[napi]
impl Credentials {
    /// Load credentials from raw bytes
    #[napi(factory)]
    pub async fn load(raw: Buffer) -> Result<Credentials> {
        let bytes = raw.to_vec();
        let inner = tokio::task::spawn_blocking(move || {
            GlCredentials::load(bytes).map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(Self { inner })
    }

    /// Save credentials to raw bytes
    #[napi]
    pub async fn save(&self) -> Result<Buffer> {
        let inner = self.inner.clone();
        let bytes = tokio::task::spawn_blocking(move || {
            inner.save().map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(Buffer::from(bytes))
    }
}

#[napi]
impl Scheduler {
    /// Create a new scheduler client
    ///
    /// # Arguments
    /// * `network` - Network name ("bitcoin" or "regtest")
    #[napi(constructor)]
    pub fn new(network: String) -> Result<Self> {
        // Constructor stays sync — it's just parsing a string and initialising a struct
        let gl_network = match network.to_lowercase().as_str() {
            "bitcoin" => GlNetwork::BITCOIN,
            "regtest" => GlNetwork::REGTEST,
            _ => {
                return Err(Error::from_reason(format!(
                    "Invalid network: {}. Must be 'bitcoin' or 'regtest'",
                    network
                )))
            }
        };

        let inner = GlScheduler::new(gl_network).map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self { inner })
    }

    /// Configure a developer certificate obtained from the Greenlight
    /// Developer Console. Nodes registered through this scheduler
    /// will be associated with the developer's account.
    ///
    /// Returns a new Scheduler instance with the developer certificate
    /// configured.
    ///
    /// # Arguments
    /// * `cert` - Developer certificate from the Greenlight Developer Console
    #[napi]
    pub fn with_developer_cert(&self, cert: &DeveloperCert) -> Scheduler {
        let inner = self.inner.with_developer_cert(&cert.inner);
        Scheduler { inner }
    }

    /// Register a new node with the scheduler
    ///
    /// # Arguments
    /// * `signer` - The signer instance
    /// * `code` - Optional invite code
    #[napi]
    pub async fn register(&self, signer: &Signer, code: Option<String>) -> Result<Credentials> {
        let inner_scheduler = self.inner.clone();
        let inner_signer = signer.inner.clone();
        let inner = tokio::task::spawn_blocking(move || {
            inner_scheduler
                .register(&inner_signer, code)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(Credentials { inner })
    }

    /// Recover node credentials
    ///
    /// # Arguments
    /// * `signer` - The signer instance
    #[napi]
    pub async fn recover(&self, signer: &Signer) -> Result<Credentials> {
        let inner_scheduler = self.inner.clone();
        let inner_signer = signer.inner.clone();
        let inner = tokio::task::spawn_blocking(move || {
            inner_scheduler
                .recover(&inner_signer)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(Credentials { inner })
    }
}

#[napi]
impl Signer {
    /// Create a new signer from a BIP39 mnemonic phrase
    ///
    /// # Arguments
    /// * `phrase` - BIP39 mnemonic phrase (12 or 24 words)
    #[napi(constructor)]
    pub fn new(phrase: String) -> Result<Self> {
        // Constructor stays sync — pure key derivation, no I/O
        let inner = GlSigner::new(phrase).map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self { inner })
    }

    /// Authenticate the signer with credentials
    ///
    /// # Arguments
    /// * `credentials` - Device credentials from registration
    #[napi]
    pub async fn authenticate(&self, credentials: &Credentials) -> Result<Signer> {
        let inner_signer = self.inner.clone();
        let inner_creds = credentials.inner.clone();
        let inner = tokio::task::spawn_blocking(move || {
            inner_signer
                .authenticate(&inner_creds)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(Signer { inner })
    }

    /// Start the signer's background task
    /// Returns a handle to control the signer
    #[napi]
    pub async fn start(&self) -> Result<Handle> {
        let inner_signer = self.inner.clone();
        let inner = tokio::task::spawn_blocking(move || {
            inner_signer
                .start()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(Handle { inner })
    }

    /// Get the node ID for this signer
    /// (stays sync — pure in-memory computation, no I/O)
    #[napi]
    pub fn node_id(&self) -> Buffer {
        Buffer::from(self.inner.node_id())
    }
}

#[napi]
impl Handle {
    /// Stop the signer's background task
    /// (stays sync — just sends a stop signal)
    #[napi]
    pub fn stop(&self) {
        self.inner.stop();
    }
}

#[napi]
impl NodeEventStream {
    /// Get the next event from the stream.
    ///
    /// Blocks the calling thread (but not the JS event loop) until an
    /// event is available. Returns `null` when the stream ends or the
    /// connection is lost.
    #[napi]
    pub async fn next(&self) -> Result<Option<NodeEvent>> {
        let stream = std::sync::Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || {
            stream
                .next()
                .map_err(|e| Error::from_reason(e.to_string()))
                .map(|opt| opt.map(napi_node_event_from_gl))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?
    }
}

#[napi]
impl Node {
    /// Create a new node connection
    ///
    /// # Arguments
    /// * `credentials` - Device credentials
    #[napi(constructor)]
    pub fn new(credentials: &Credentials) -> Result<Self> {
        // Constructor stays sync — connection is established lazily
        let inner =
            GlNode::new(&credentials.inner).map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self { inner: std::sync::Arc::new(inner) })
    }

    /// Stop the node if it is currently running
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            inner
                .stop()
                .map_err(|e| Error::from_reason(format!("Failed to stop node: {:?}", e)))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?
    }

    /// Receive a payment (generate invoice with JIT channel support)
    ///
    /// # Arguments
    /// * `label` - Unique label for this invoice
    /// * `description` - Invoice description
    /// * `amount_msat` - Optional amount in millisatoshis
    #[napi]
    pub async fn receive(
        &self,
        label: String,
        description: String,
        amount_msat: Option<i64>,
    ) -> Result<ReceiveResponse> {
        let inner = self.inner.clone();
        let amount = amount_msat.map(|a| a as u64);
        let response = tokio::task::spawn_blocking(move || {
            inner
                .receive(label, description, amount)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(ReceiveResponse {
            bolt11: response.bolt11,
        })
    }

    /// Send a payment
    ///
    /// # Arguments
    /// * `invoice` - BOLT11 invoice string
    /// * `amount_msat` - Optional amount for zero-amount invoices
    #[napi]
    pub async fn send(&self, invoice: String, amount_msat: Option<i64>) -> Result<SendResponse> {
        let inner = self.inner.clone();
        let amount = amount_msat.map(|a| a as u64);
        let response = tokio::task::spawn_blocking(move || {
            inner
                .send(invoice, amount)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(SendResponse {
            status: response.status as u32,
            preimage: Buffer::from(response.preimage),
            payment_hash: Buffer::from(response.payment_hash),
            destination_pubkey: response.destination_pubkey.map(Buffer::from),
            amount_msat: response.amount_msat as i64,
            amount_sent_msat: response.amount_sent_msat as i64,
            parts: response.parts,
        })
    }

    /// Send an on-chain transaction
    ///
    /// # Arguments
    /// * `destination` - Bitcoin address
    /// * `amount_or_all` - Amount (e.g., "10000sat", "1000msat") or "all"
    #[napi]
    pub async fn onchain_send(
        &self,
        destination: String,
        amount_or_all: String,
    ) -> Result<OnchainSendResponse> {
        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || {
            inner
                .onchain_send(destination, amount_or_all)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(OnchainSendResponse {
            tx: Buffer::from(response.tx),
            txid: Buffer::from(response.txid),
            psbt: response.psbt,
        })
    }

    /// Generate a new on-chain address
    #[napi]
    pub async fn onchain_receive(&self) -> Result<OnchainReceiveResponse> {
        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || {
            inner
                .onchain_receive()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(OnchainReceiveResponse {
            bech32: response.bech32,
            p2tr: response.p2tr,
        })
    }

    /// Stream real-time events from the node.
    ///
    /// Returns a `NodeEventStream`. Call `.next()` on it repeatedly to
    /// receive events (e.g., invoice payments) as they occur.
    ///
    /// Returns `Unimplemented` if the connected node build does not yet
    /// support `StreamNodeEvents`.
    #[napi]
    pub async fn stream_node_events(&self) -> Result<NodeEventStream> {
        let inner = self.inner.clone();
        let gl_stream = tokio::task::spawn_blocking(move || {
            inner
                .stream_node_events()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        // gl-sdk returns Arc<GlNodeEventStream> — store it directly since
        // GlNodeEventStream wraps a Mutex<Streaming<...>> and is not Clone.
        Ok(NodeEventStream { inner: gl_stream })
    }

    /// Get information about the node
    ///
    /// Returns basic information about the node including its ID,
    /// alias, network, and channel counts.
    #[napi]
    pub async fn get_info(&self) -> Result<GetInfoResponse> {
        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || {
            inner
                .get_info()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(GetInfoResponse {
            id: Buffer::from(response.id),
            alias: response.alias,
            color: Buffer::from(response.color),
            num_peers: response.num_peers,
            num_pending_channels: response.num_pending_channels,
            num_active_channels: response.num_active_channels,
            num_inactive_channels: response.num_inactive_channels,
            version: response.version,
            lightning_dir: response.lightning_dir,
            blockheight: response.blockheight,
            network: response.network,
            fees_collected_msat: response.fees_collected_msat as i64,
        })
    }

    /// List all peers connected to this node
    ///
    /// Returns information about all peers including their connection status.
    #[napi]
    pub async fn list_peers(&self) -> Result<ListPeersResponse> {
        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || {
            inner
                .list_peers()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(ListPeersResponse {
            peers: response
                .peers
                .into_iter()
                .map(|p| Peer {
                    id: Buffer::from(p.id),
                    connected: p.connected,
                    num_channels: p.num_channels,
                    netaddr: p.netaddr,
                    remote_addr: p.remote_addr,
                    features: p.features.map(Buffer::from),
                })
                .collect(),
        })
    }

    /// List all channels with peers
    ///
    /// Returns detailed information about all channels including their
    /// state, capacity, and balances.
    #[napi]
    pub async fn list_peer_channels(&self) -> Result<ListPeerChannelsResponse> {
        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || {
            inner
                .list_peer_channels()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(ListPeerChannelsResponse {
            channels: response
                .channels
                .into_iter()
                .map(|c| PeerChannel {
                    peer_id: Buffer::from(c.peer_id),
                    peer_connected: c.peer_connected,
                    state: channel_state_to_string(&c.state),
                    short_channel_id: c.short_channel_id,
                    channel_id: c.channel_id.map(Buffer::from),
                    funding_txid: c.funding_txid.map(Buffer::from),
                    funding_outnum: c.funding_outnum,
                    to_us_msat: c.to_us_msat.map(|v| v as i64),
                    total_msat: c.total_msat.map(|v| v as i64),
                    spendable_msat: c.spendable_msat.map(|v| v as i64),
                    receivable_msat: c.receivable_msat.map(|v| v as i64),
                })
                .collect(),
        })
    }

    /// List all funds available to the node
    ///
    /// Returns information about on-chain outputs and channel funds
    /// that are available or pending.
    #[napi]
    pub async fn list_funds(&self) -> Result<ListFundsResponse> {
        let inner = self.inner.clone();
        let response = tokio::task::spawn_blocking(move || {
            inner
                .list_funds()
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(ListFundsResponse {
            outputs: response
                .outputs
                .into_iter()
                .map(|o| FundOutput {
                    txid: Buffer::from(o.txid),
                    output: o.output,
                    amount_msat: o.amount_msat as i64,
                    status: output_status_to_string(&o.status),
                    address: o.address,
                    blockheight: o.blockheight,
                })
                .collect(),
            channels: response
                .channels
                .into_iter()
                .map(|c| FundChannel {
                    peer_id: Buffer::from(c.peer_id),
                    our_amount_msat: c.our_amount_msat as i64,
                    amount_msat: c.amount_msat as i64,
                    funding_txid: Buffer::from(c.funding_txid),
                    funding_output: c.funding_output,
                    connected: c.connected,
                    state: channel_state_to_string(&c.state),
                    short_channel_id: c.short_channel_id,
                    channel_id: c.channel_id.map(Buffer::from),
                })
                .collect(),
        })
    }

    // ── LNURL methods ───────────────────────────────────────────

    /// Execute an LNURL-pay flow.
    ///
    /// Build the request from `LnUrlPayRequestData` (obtained out of
    /// band) and a chosen amount.
    #[napi]
    pub async fn lnurl_pay(&self, request: LnUrlPayRequest) -> Result<LnUrlPayResult> {
        let inner = self.inner.clone();
        let gl_request = gl_lnurl_pay_request_from_napi(request);
        let result = tokio::task::spawn_blocking(move || {
            inner
                .lnurl_pay(gl_request)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(napi_lnurl_pay_result_from_gl(result))
    }

    /// Execute an LNURL-withdraw flow.
    ///
    /// Build the request from `LnUrlWithdrawRequestData` (obtained out
    /// of band) and a chosen amount.
    #[napi]
    pub async fn lnurl_withdraw(&self, request: LnUrlWithdrawRequest) -> Result<LnUrlWithdrawResult> {
        let inner = self.inner.clone();
        let gl_request = gl_lnurl_withdraw_request_from_napi(request);
        let result = tokio::task::spawn_blocking(move || {
            inner
                .lnurl_withdraw(gl_request)
                .map_err(|e| Error::from_reason(e.to_string()))
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))??;

        Ok(napi_lnurl_withdraw_result_from_gl(result))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a gl-sdk `NodeEvent` to the NAPI flat discriminated-union object.
fn napi_node_event_from_gl(event: GlNodeEvent) -> NodeEvent {
    match event {
        GlNodeEvent::InvoicePaid { details } => NodeEvent {
            event_type: "invoice_paid".to_string(),
            invoice_paid: Some(InvoicePaidEvent {
                payment_hash: Buffer::from(details.payment_hash),
                bolt11: details.bolt11,
                preimage: Buffer::from(details.preimage),
                label: details.label,
                amount_msat: details.amount_msat as i64,
            }),
        },
        GlNodeEvent::Unknown => NodeEvent {
            event_type: "unknown".to_string(),
            invoice_paid: None,
        },
    }
}

fn channel_state_to_string(state: &GlChannelState) -> String {
    match state {
        GlChannelState::Openingd => "OPENINGD".to_string(),
        GlChannelState::ChanneldAwaitingLockin => "CHANNELD_AWAITING_LOCKIN".to_string(),
        GlChannelState::ChanneldNormal => "CHANNELD_NORMAL".to_string(),
        GlChannelState::ChanneldShuttingDown => "CHANNELD_SHUTTING_DOWN".to_string(),
        GlChannelState::ClosingdSigexchange => "CLOSINGD_SIGEXCHANGE".to_string(),
        GlChannelState::ClosingdComplete => "CLOSINGD_COMPLETE".to_string(),
        GlChannelState::AwaitingUnilateral => "AWAITING_UNILATERAL".to_string(),
        GlChannelState::FundingSpendSeen => "FUNDING_SPEND_SEEN".to_string(),
        GlChannelState::Onchain => "ONCHAIN".to_string(),
        GlChannelState::DualopendOpenInit => "DUALOPEND_OPEN_INIT".to_string(),
        GlChannelState::DualopendAwaitingLockin => "DUALOPEND_AWAITING_LOCKIN".to_string(),
        GlChannelState::DualopendOpenCommitted => "DUALOPEND_OPEN_COMMITTED".to_string(),
        GlChannelState::DualopendOpenCommitReady => "DUALOPEND_OPEN_COMMIT_READY".to_string(),
    }
}

fn output_status_to_string(status: &GlOutputStatus) -> String {
    match status {
        GlOutputStatus::Unconfirmed => "unconfirmed".to_string(),
        GlOutputStatus::Confirmed => "confirmed".to_string(),
        GlOutputStatus::Spent => "spent".to_string(),
        GlOutputStatus::Immature => "immature".to_string(),
    }
}

// ============================================================================
// Input Parsing Conversion Helpers
// ============================================================================

fn napi_parsed_invoice_from_gl(invoice: GlParsedInvoice) -> ParsedInvoice {
    ParsedInvoice {
        bolt11: invoice.bolt11,
        payee_pubkey: invoice.payee_pubkey.map(Buffer::from),
        payment_hash: Buffer::from(invoice.payment_hash),
        description: invoice.description,
        amount_msat: invoice.amount_msat.map(|v| v as i64),
        expiry: invoice.expiry as i64,
        timestamp: invoice.timestamp as i64,
    }
}

fn napi_pay_request_data_from_gl(data: glsdk::LnUrlPayRequestData) -> LnUrlPayRequestData {
    LnUrlPayRequestData {
        callback: data.callback,
        min_sendable: data.min_sendable as i64,
        max_sendable: data.max_sendable as i64,
        metadata: data.metadata,
        comment_allowed: data.comment_allowed as i64,
        description: data.description,
        lnurl: data.lnurl,
    }
}

fn napi_withdraw_request_data_from_gl(
    data: glsdk::LnUrlWithdrawRequestData,
) -> LnUrlWithdrawRequestData {
    LnUrlWithdrawRequestData {
        callback: data.callback,
        k1: data.k1,
        default_description: data.default_description,
        min_withdrawable: data.min_withdrawable as i64,
        max_withdrawable: data.max_withdrawable as i64,
        lnurl: data.lnurl,
    }
}

fn napi_input_type_from_gl(input: GlInputType) -> InputType {
    match input {
        GlInputType::Bolt11 { invoice } => InputType {
            r#type: "bolt11".to_string(),
            bolt11: Some(napi_parsed_invoice_from_gl(invoice)),
            node_id: None,
            lnurl_pay: None,
            lnurl_withdraw: None,
        },
        GlInputType::NodeId { node_id } => InputType {
            r#type: "node_id".to_string(),
            bolt11: None,
            node_id: Some(node_id),
            lnurl_pay: None,
            lnurl_withdraw: None,
        },
        GlInputType::LnUrlPay { data } => InputType {
            r#type: "lnurl_pay".to_string(),
            bolt11: None,
            node_id: None,
            lnurl_pay: Some(napi_pay_request_data_from_gl(data)),
            lnurl_withdraw: None,
        },
        GlInputType::LnUrlWithdraw { data } => InputType {
            r#type: "lnurl_withdraw".to_string(),
            bolt11: None,
            node_id: None,
            lnurl_pay: None,
            lnurl_withdraw: Some(napi_withdraw_request_data_from_gl(data)),
        },
    }
}

/// Parse and resolve any supported input.
///
/// For LNURL bech32 strings and Lightning Addresses this performs the
/// HTTP GET to the LNURL endpoint and returns typed pay or withdraw
/// request data. For BOLT11 invoices and node IDs it returns
/// immediately without I/O.
///
/// Strips `lightning:` / `LIGHTNING:` prefixes automatically.
#[napi]
pub async fn parse_input(input: String) -> Result<InputType> {
    let resolved = glsdk::parse_input(input)
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(napi_input_type_from_gl(resolved))
}

// ============================================================================
// LNURL Conversion Helpers
// ============================================================================

fn gl_pay_request_data_from_napi(data: LnUrlPayRequestData) -> glsdk::LnUrlPayRequestData {
    glsdk::LnUrlPayRequestData {
        callback: data.callback,
        min_sendable: data.min_sendable as u64,
        max_sendable: data.max_sendable as u64,
        metadata: data.metadata,
        comment_allowed: data.comment_allowed as u64,
        description: data.description,
        lnurl: data.lnurl,
    }
}

fn gl_withdraw_request_data_from_napi(
    data: LnUrlWithdrawRequestData,
) -> glsdk::LnUrlWithdrawRequestData {
    glsdk::LnUrlWithdrawRequestData {
        callback: data.callback,
        k1: data.k1,
        default_description: data.default_description,
        min_withdrawable: data.min_withdrawable as u64,
        max_withdrawable: data.max_withdrawable as u64,
        lnurl: data.lnurl,
    }
}

fn gl_lnurl_pay_request_from_napi(req: LnUrlPayRequest) -> glsdk::LnUrlPayRequest {
    glsdk::LnUrlPayRequest {
        data: gl_pay_request_data_from_napi(req.data),
        amount_msat: req.amount_msat as u64,
        comment: req.comment,
        validate_success_action_url: req.validate_success_action_url,
    }
}

fn gl_lnurl_withdraw_request_from_napi(req: LnUrlWithdrawRequest) -> glsdk::LnUrlWithdrawRequest {
    glsdk::LnUrlWithdrawRequest {
        data: gl_withdraw_request_data_from_napi(req.data),
        amount_msat: req.amount_msat as u64,
        description: req.description,
    }
}

fn napi_success_action_from_gl(action: glsdk::SuccessActionProcessed) -> SuccessActionProcessed {
    match action {
        glsdk::SuccessActionProcessed::Message { message } => SuccessActionProcessed {
            r#type: "message".to_string(),
            message: Some(message),
            description: None,
            url: None,
            plaintext: None,
        },
        glsdk::SuccessActionProcessed::Url { description, url } => SuccessActionProcessed {
            r#type: "url".to_string(),
            message: None,
            description: Some(description),
            url: Some(url),
            plaintext: None,
        },
        glsdk::SuccessActionProcessed::Aes {
            description,
            plaintext,
        } => SuccessActionProcessed {
            r#type: "aes".to_string(),
            message: None,
            description: Some(description),
            url: None,
            plaintext: Some(plaintext),
        },
    }
}

fn napi_lnurl_pay_result_from_gl(result: glsdk::LnUrlPayResult) -> LnUrlPayResult {
    match result {
        glsdk::LnUrlPayResult::EndpointSuccess { data } => LnUrlPayResult {
            r#type: "success".to_string(),
            success: Some(LnUrlPaySuccessData {
                payment_preimage: data.payment_preimage,
                success_action: data.success_action.map(napi_success_action_from_gl),
            }),
            error: None,
            pay_error: None,
        },
        glsdk::LnUrlPayResult::EndpointError { data } => LnUrlPayResult {
            r#type: "error".to_string(),
            success: None,
            error: Some(LnUrlErrorData {
                reason: data.reason,
            }),
            pay_error: None,
        },
        glsdk::LnUrlPayResult::PayError { data } => LnUrlPayResult {
            r#type: "pay_error".to_string(),
            success: None,
            error: None,
            pay_error: Some(LnUrlPayErrorData {
                payment_hash: data.payment_hash,
                reason: data.reason,
            }),
        },
    }
}

fn napi_lnurl_withdraw_result_from_gl(result: glsdk::LnUrlWithdrawResult) -> LnUrlWithdrawResult {
    match result {
        glsdk::LnUrlWithdrawResult::Ok { data } => LnUrlWithdrawResult {
            r#type: "ok".to_string(),
            ok: Some(LnUrlWithdrawSuccessData {
                invoice: data.invoice,
            }),
            error: None,
        },
        glsdk::LnUrlWithdrawResult::ErrorStatus { data } => LnUrlWithdrawResult {
            r#type: "error".to_string(),
            ok: None,
            error: Some(LnUrlErrorData {
                reason: data.reason,
            }),
        },
    }
}
