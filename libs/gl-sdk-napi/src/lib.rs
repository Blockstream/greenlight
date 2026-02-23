#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

// Import from glsdk crate (gl-sdk library)
use glsdk::{
    Credentials as GlCredentials,
    Node as GlNode,
    Scheduler as GlScheduler,
    Signer as GlSigner,
    Handle as GlHandle,
    Network as GlNetwork,
    // Enum types for conversion
    ChannelState as GlChannelState,
    OutputStatus as GlOutputStatus,
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
// Struct Definitions (all structs must be defined before impl blocks)
// ============================================================================

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
    inner: GlNode,
}

// ============================================================================
// NAPI Implementations
// ============================================================================

#[napi]
impl Credentials {
    /// Load credentials from raw bytes
    #[napi(factory)]
    pub async fn load(raw: Buffer) -> Result<Credentials> {
        let bytes = raw.to_vec();
        let inner = tokio::task::spawn_blocking(move || {
            GlCredentials::load(bytes)
                .map_err(|e| Error::from_reason(e.to_string()))
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
            inner.save()
                .map_err(|e| Error::from_reason(e.to_string()))
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
            _ => return Err(Error::from_reason(format!(
                "Invalid network: {}. Must be 'bitcoin' or 'regtest'",
                network
            ))),
        };

        let inner = GlScheduler::new(gl_network)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self { inner })
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
        let inner = GlSigner::new(phrase)
            .map_err(|e| Error::from_reason(e.to_string()))?;

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
impl Node {
    /// Create a new node connection
    ///
    /// # Arguments
    /// * `credentials` - Device credentials
    #[napi(constructor)]
    pub fn new(credentials: &Credentials) -> Result<Self> {
        // Constructor stays sync — connection is established lazily
        let inner = GlNode::new(&credentials.inner)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self { inner })
    }

    /// Stop the node if it is currently running
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            inner.stop()
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
            peers: response.peers.into_iter().map(|p| Peer {
                id: Buffer::from(p.id),
                connected: p.connected,
                num_channels: p.num_channels,
                netaddr: p.netaddr,
                remote_addr: p.remote_addr,
                features: p.features.map(Buffer::from),
            }).collect(),
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
            channels: response.channels.into_iter().map(|c| PeerChannel {
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
            }).collect(),
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
            outputs: response.outputs.into_iter().map(|o| FundOutput {
                txid: Buffer::from(o.txid),
                output: o.output,
                amount_msat: o.amount_msat as i64,
                status: output_status_to_string(&o.status),
                address: o.address,
                blockheight: o.blockheight,
            }).collect(),
            channels: response.channels.into_iter().map(|c| FundChannel {
                peer_id: Buffer::from(c.peer_id),
                our_amount_msat: c.our_amount_msat as i64,
                amount_msat: c.amount_msat as i64,
                funding_txid: Buffer::from(c.funding_txid),
                funding_output: c.funding_output,
                connected: c.connected,
                state: channel_state_to_string(&c.state),
                short_channel_id: c.short_channel_id,
                channel_id: c.channel_id.map(Buffer::from),
            }).collect(),
        })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

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
