use anyhow::{Context, Result};
use log::{debug, warn};
use rpc::LightningClient;
use serde_json::json;
use std::future::Future;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

pub mod config;
pub mod hsm;
pub mod messages;
pub mod node;
pub mod pb;
pub mod requests;
pub mod responses;
pub mod rpc;
pub mod stager;
#[cfg(unix)]
mod unix;

#[derive(Clone)]
pub struct GlPlugin {
    rpc: Arc<Mutex<LightningClient>>,
    _stage: Arc<stager::Stage>,
    events: broadcast::Sender<Event>,
}

/// A small wrapper around [`cln_plugin::Builder`] that allows us to
/// subscribe to events outside the plugin state itself, before
/// getting configured.
// TODO: Switch this out once the [`cln_plugin::Builder`] no longer
// pre-binds state
pub struct Builder {
    inner: cln_plugin::Builder<GlPlugin, tokio::io::Stdin, tokio::io::Stdout>,
    events: broadcast::Sender<Event>,
    state: GlPlugin,
}

impl Builder {
    pub fn subscribe_events(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }
    pub async fn start(self) -> Result<Option<Plugin>> {
        self.inner.start(self.state).await
    }

    pub fn hook<C, F>(self, hookname: &str, callback: C) -> Self
    where
        C: Send + Sync + 'static,
        C: Fn(cln_plugin::Plugin<GlPlugin>, serde_json::Value) -> F + 'static,
        F: Future<Output = Result<serde_json::Value, anyhow::Error>> + Send + Sync + 'static,
    {
        Builder {
            inner: self.inner.hook(hookname, callback),
            ..self
        }
    }
    pub fn subscribe<C, F>(self, hookname: &str, callback: C) -> Self
    where
        C: Send + Sync + 'static,
        C: Fn(cln_plugin::Plugin<GlPlugin>, serde_json::Value) -> F + 'static,
        F: Future<Output = Result<(), anyhow::Error>> + Send + Sync + 'static,
    {
        Builder {
            inner: self.inner.subscribe(hookname, callback),
            ..self
        }
    }
}

pub type Plugin = cln_plugin::Plugin<GlPlugin>;

impl GlPlugin {
    pub fn get_stage(&self) -> Arc<stager::Stage> {
        self._stage.clone()
    }
}

/// Initialize the plugin, but don't start it yet. Allows attaching
/// additional methods, hooks, and subscriptions.
pub fn init() -> Result<Builder> {
    let (events, _) = tokio::sync::broadcast::channel(16);
    let rpc = Arc::new(Mutex::new(LightningClient::new("lightning-rpc")));
    let stage = Arc::new(stager::Stage::new());
    let config = config::Config::new().unwrap();

    // We run this already at startup, not at configuration because if
    // the signerproxy doesn't find the socket on the FS it'll exit.
    let hsm_server = hsm::StagingHsmServer::new(
        PathBuf::from_str(&config.hsmd_sock_path).context("hsmd_sock_path is not a valid path")?,
        stage.clone(),
        config.node_info.clone(),
    );
    tokio::spawn(hsm_server.run());

    // We also run the grpc interface for clients already, since we
    // can wait on the RPC becoming reachable, and this way we don't
    // need to poll from the client.
    let node_server =
        crate::node::PluginNodeServer::new(stage.clone(), config.clone(), events.clone())?;
    tokio::spawn(node_server.run());

    let state = GlPlugin {
        events: events.clone(),
        rpc,
        _stage: stage,
    };

    let inner = cln_plugin::Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .hook("invoice_payment", on_invoice_payment);
    Ok(Builder { state, inner, events })
}

/// Notification handler that receives notifications on incoming
/// payments, then looks up the invoice in the DB, and forwards the
/// full information to the GRPC interface.
async fn on_invoice_payment(plugin: Plugin, v: serde_json::Value) -> Result<serde_json::Value> {
    debug!("Got an incoming payment via invoice_payment: {:?}", v);
    let state = plugin.state();
    let call: messages::InvoicePaymentCall = serde_json::from_value(v).unwrap();

    let rpc = state.rpc.lock().await.clone();
    let req = requests::ListInvoices {
        label: Some(call.payment.label.clone()),
        invstring: None,
        payment_hash: None,
    };

    let invoice = match rpc
        .call::<_, responses::ListInvoices>("listinvoices", req)
        .await
        .unwrap()
        .invoices
        .pop()
    {
        Some(i) => i,
        None => {
            warn!(
                "No invoice matching the notification label {} found",
                call.payment.label
            );
            return Ok(json!({"result": "continue"}));
        }
    };

    debug!(
        "Retrieved matching invoice for invoice_payment: {:?}",
        invoice
    );

    let amount: pb::Amount = call.payment.amount.try_into().unwrap();

    let mut tlvs = vec![];

    for t in call.payment.extratlvs.unwrap_or(vec![]) {
        tlvs.push(t.into());
    }
    use crate::pb::incoming_payment::Details;
    let details = pb::OffChainPayment {
        label: invoice.label,
        preimage: hex::decode(call.payment.preimage).unwrap(),
        amount: Some(amount.try_into().unwrap()),
        extratlvs: tlvs,
        bolt11: invoice.bolt11,
        payment_hash: hex::decode(invoice.payment_hash).unwrap(),
    };

    let p = pb::IncomingPayment {
        details: Some(Details::Offchain(details)),
    };

    match state.events.clone().send(Event::IncomingPayment(p)) {
        Ok(_) => {}
        Err(_) => warn!("No active listener for the incoming payment"),
    }
    Ok(json!({"result": "continue"}))
}

/// An event that we can observe during the operation of the plugin.
#[derive(Clone, Debug)]
pub enum Event {
    Stop(Arc<stager::Stage>),
    RpcCall,
    IncomingPayment(pb::IncomingPayment),
}
