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
    pub fn load(raw: Buffer) -> Result<Credentials> {
        let inner = GlCredentials::load(raw.to_vec())
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Save credentials to raw bytes
    #[napi]
    pub fn save(&self) -> Result<Buffer> {
        let bytes = self.inner.save()
            .map_err(|e| Error::from_reason(e.to_string()))?;

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
        // Parse network string to GlNetwork enum
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
    pub fn register(&self, signer: &Signer, code: Option<String>) -> Result<Credentials> {
        let inner = self.inner
            .register(&signer.inner, code)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(Credentials { inner })
    }

    /// Recover node credentials
    /// 
    /// # Arguments
    /// * `signer` - The signer instance
    #[napi]
    pub fn recover(&self, signer: &Signer) -> Result<Credentials> {
        let inner = self.inner
            .recover(&signer.inner)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
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
        let inner = GlSigner::new(phrase)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(Self { inner })
    }

    /// Authenticate the signer with credentials
    /// 
    /// # Arguments
    /// * `credentials` - Device credentials from registration
    #[napi]
    pub fn authenticate(&self, credentials: &Credentials) -> Result<Signer> {
        let inner = self.inner.authenticate(&credentials.inner)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(Signer { inner })
    }

    /// Start the signer's background task
    /// Returns a handle to control the signer
    #[napi]
    pub fn start(&self) -> Result<Handle> {
        let inner = self.inner.start()
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Handle { inner })
    }

    /// Get the node ID for this signer
    #[napi]
    pub fn node_id(&self) -> Buffer {
        let node_id = self.inner.node_id();
        Buffer::from(node_id)
    }
}

#[napi]
impl Handle {
    /// Stop the signer's background task
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
        let inner = GlNode::new(&credentials.inner)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(Self { inner })
    }

    /// Stop the node if it is currently running
    #[napi]
    pub fn stop(&self) -> Result<()> {
        self.inner.stop()
            .map_err(|e| Error::from_reason(format!("Failed to stop node: {:?}", e)))
    }

    /// Receive a payment (generate invoice with JIT channel support)
    /// 
    /// # Arguments
    /// * `label` - Unique label for this invoice
    /// * `description` - Invoice description
    /// * `amount_msat` - Optional amount in millisatoshis
    #[napi]
    pub fn receive(
        &self,
        label: String,
        description: String,
        amount_msat: Option<i64>,
    ) -> Result<ReceiveResponse> {

        let amount = amount_msat.map(|a| a as u64);
        let response = self.inner.receive(label, description, amount)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
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
    pub fn send(&self, invoice: String, amount_msat: Option<i64>) -> Result<SendResponse> {
        let amount = amount_msat.map(|a| a as u64);
        let response = self.inner.send(invoice, amount)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
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
    pub fn onchain_send(
        &self,
        destination: String,
        amount_or_all: String,
    ) -> Result<OnchainSendResponse> {
        let response = self.inner.onchain_send(destination, amount_or_all)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(OnchainSendResponse {
            tx: Buffer::from(response.tx),
            txid: Buffer::from(response.txid),
            psbt: response.psbt,
        })
    }

    /// Generate a new on-chain address
    #[napi]
    pub fn onchain_receive(&self) -> Result<OnchainReceiveResponse> {
        let response = self.inner.onchain_receive()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        
        Ok(OnchainReceiveResponse {
            bech32: response.bech32,
            p2tr: response.p2tr,
        })
    }
}
