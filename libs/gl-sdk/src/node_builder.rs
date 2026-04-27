// Builder-style Node creation.
//
// `NodeBuilder` is the sole public entry point for Node construction
// across all foreign bindings. The shape is "request the action,
// optionally with modifiers":
//
//     // Signerless connect — caller does not have the mnemonic in
//     // this process. The SDK runs no signer; signing happens
//     // elsewhere (paired device, CLN node's local signer, hardware
//     // signer). This is the supported model for keyless clients.
//     let node = NodeBuilder::new(&config).connect(credentials, None)?;
//
//     // Signed connect — caller hands the mnemonic per build call,
//     // SDK spawns a signer.
//     let node = NodeBuilder::new(&config).connect(credentials, Some(mnemonic))?;
//
//     // Register / recover require a mnemonic by definition (the
//     // signer must sign the registration/recovery challenge).
//     let node = NodeBuilder::new(&config)
//         .with_event_listener(listener)
//         .register(mnemonic, invite_code)?;
//
// The mnemonic is the security-sensitive input: hand it in only when
// you actually want the SDK to act as a signer for that call.
//
// Adding a new modifier later is a new `with_*` setter — additive,
// never breaks existing callers.

use std::sync::Arc;

use crate::{
    config::Config,
    node::{Node, NodeEventListener},
    Error,
};

/// Configurable Node construction. See module docs.
///
/// All fields are immutable after construction. Each `with_*` setter
/// returns a fresh `Arc<NodeBuilder>` that shares ownership of any
/// previously-installed modifiers via `Arc<dyn …>`. No interior
/// mutability, no locks — the builder is a value, not a state
/// machine.
#[derive(uniffi::Object)]
pub struct NodeBuilder {
    config: Arc<Config>,
    event_listener: Option<Arc<dyn NodeEventListener>>,
}

#[uniffi::export]
impl NodeBuilder {
    /// Create a builder for a Node with `config`. No I/O happens
    /// until you call `connect` / `register` / `recover` /
    /// `register_or_recover`.
    #[uniffi::constructor]
    pub fn new(config: &Config) -> Arc<Self> {
        Arc::new(Self {
            config: Arc::new(config.clone()),
            event_listener: None,
        })
    }

    /// Install a node event listener. Events fire from the moment the
    /// gRPC stream is established by the build call (`register` /
    /// `recover` / `connect` / …), so attach the listener via the
    /// builder rather than after the fact to capture events from the
    /// very first moment.
    ///
    /// Returns a new builder that shares the rest of the
    /// configuration. Build calls on the returned builder will
    /// install the listener; the original builder is unchanged.
    pub fn with_event_listener(
        self: Arc<Self>,
        listener: Box<dyn NodeEventListener>,
    ) -> Arc<Self> {
        // UniFFI's callback-interface lowering hands us a
        // `Box<dyn Trait>`. We re-wrap it as `Arc<dyn Trait>` because
        // the builder is reusable across multiple build calls — each
        // build clones the Arc into the resulting Node, and `Box`
        // can't be cloned. This is a one-time cost paid per setter
        // call.
        Arc::new(Self {
            config: Arc::clone(&self.config),
            event_listener: Some(Arc::from(listener)),
        })
    }

    /// Register a new Greenlight node and return a connected Node
    /// with the SDK signer running and any configured modifiers
    /// applied.
    ///
    /// `mnemonic` is required — registration drives the signer to
    /// sign the registration challenge, so the SDK must hold the
    /// seed for this call.
    pub fn register(
        &self,
        mnemonic: String,
        invite_code: Option<String>,
    ) -> Result<Arc<Node>, Error> {
        let node = crate::register_internal(mnemonic, invite_code, &self.config)?;
        self.attach_observers(&node)?;
        Ok(node)
    }

    /// Recover credentials for an existing node and return a
    /// connected Node with any configured modifiers applied.
    ///
    /// `mnemonic` is required — recovery drives the signer to
    /// authenticate.
    pub fn recover(&self, mnemonic: String) -> Result<Arc<Node>, Error> {
        let node = crate::recover_internal(mnemonic, &self.config)?;
        self.attach_observers(&node)?;
        Ok(node)
    }

    /// Connect to an existing node using saved credentials and return
    /// a connected Node with any configured modifiers applied.
    ///
    /// If `mnemonic` is `Some(...)`, the SDK spawns a signer for the
    /// connected Node. If `None`, the Node is signerless and signing
    /// happens elsewhere (paired device, CLN node's local signer,
    /// hardware signer).
    pub fn connect(
        &self,
        credentials: Vec<u8>,
        mnemonic: Option<String>,
    ) -> Result<Arc<Node>, Error> {
        let node = match mnemonic {
            Some(mnemonic) => crate::connect_internal(mnemonic, credentials, &self.config)?,
            None => crate::connect_signerless_internal(credentials, &self.config)?,
        };
        self.attach_observers(&node)?;
        Ok(node)
    }

    /// Try to recover; if the node doesn't exist, register a new one.
    ///
    /// `mnemonic` is required — both recover and register drive the
    /// signer.
    pub fn register_or_recover(
        &self,
        mnemonic: String,
        invite_code: Option<String>,
    ) -> Result<Arc<Node>, Error> {
        let node =
            crate::register_or_recover_internal(mnemonic, invite_code, &self.config)?;
        self.attach_observers(&node)?;
        Ok(node)
    }
}

impl NodeBuilder {
    /// Attach all configured modifiers to a freshly-built Node.
    /// Modifiers are shared (not consumed) — the same builder can
    /// drive multiple builds and they all get the same listener.
    fn attach_observers(&self, node: &Arc<Node>) -> Result<(), Error> {
        if let Some(listener) = self.event_listener.as_ref() {
            node.set_event_listener(Arc::clone(listener))?;
        }
        Ok(())
    }
}
