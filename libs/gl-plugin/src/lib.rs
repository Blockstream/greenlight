use anyhow::Result;
use cln_rpc;
use log::{debug, warn};
use rpc::LightningClient;
use serde_json::json;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

pub mod config;
pub mod hsm;
mod lsp;
pub mod messages;
pub mod node;
pub mod pb;
pub mod requests;
pub mod responses;
pub mod rpc;
pub mod stager;
pub mod storage;
#[cfg(unix)]
mod unix;

mod context;

#[derive(Clone)]
pub struct GlPlugin {
    rpc: Arc<Mutex<LightningClient>>,
    stage: Arc<stager::Stage>,
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

    pub fn stage(&self) -> Arc<stager::Stage> {
        self.state.stage.clone()
    }
}

pub type Plugin = cln_plugin::Plugin<GlPlugin>;

impl GlPlugin {
    pub fn get_stage(&self) -> Arc<stager::Stage> {
        self.stage.clone()
    }
}

/// Initialize the plugin, but don't start it yet. Allows attaching
/// additional methods, hooks, and subscriptions.
pub async fn init(
    stage: Arc<stager::Stage>,
    events: tokio::sync::broadcast::Sender<Event>,
) -> Result<Builder> {
    let rpc = Arc::new(Mutex::new(LightningClient::new("lightning-rpc")));

    let state = GlPlugin {
        events: events.clone(),
        rpc,
        stage,
    };

    let inner = cln_plugin::Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .hook("htlc_accepted", lsp::on_htlc_accepted)
        .hook("invoice_payment", on_invoice_payment)
        .hook("peer_connected", on_peer_connected);
    Ok(Builder {
        state,
        inner,
        events,
    })
}

/// Notification handler that receives notifications on successful
/// peer connections, then stores them into the `datastore` for future
/// reference.
async fn on_peer_connected(plugin: Plugin, v: serde_json::Value) -> Result<serde_json::Value> {
    debug!("Got a successful peer connection: {:?}", v);
    let call = serde_json::from_value::<messages::PeerConnectedCall>(v.clone()).unwrap();
    let mut rpc = cln_rpc::ClnRpc::new(plugin.configuration().rpc_file).await?;
    let req = cln_rpc::model::requests::DatastoreRequest {
        key: vec![
            "greenlight".to_string(),
            "peerlist".to_string(),
            call.peer.id.clone(),
        ],
        string: Some(serde_json::to_string(&call.peer).unwrap()),
        hex: None,
        mode: None,
        generation: None,
    };

    // We ignore the response and continue anyways.
    let res = rpc.call_typed(req).await;
    debug!("Got datastore response: {:?}", res);
    Ok(json!({"result": "continue"}))
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

pub use cln_grpc as grpc;
