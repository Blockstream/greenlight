use crate::config::Config;
use crate::pb::{self, node_server::Node};
use crate::storage::StateStore;
use crate::{messages, Event};
use crate::{stager, tramp};
use anyhow::{Context, Error, Result};
use base64::{engine::general_purpose, Engine as _};
use bytes::BufMut;
use cln_rpc::Notification;
use gl_client::persist::State;
use governor::{
    clock::MonotonicClock, state::direct::NotKeyed, state::InMemoryState, Quota, RateLimiter,
};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, Mutex, OnceCell};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::ServerTlsConfig, Code, Request, Response, Status};
mod wrapper;
use gl_client::bitcoin;
use std::str::FromStr;
pub use wrapper::WrappedNodeServer;

static LIMITER: OnceCell<RateLimiter<NotKeyed, InMemoryState, MonotonicClock>> =
    OnceCell::const_new();

static RPC_CLIENT: OnceCell<Arc<Mutex<cln_rpc::ClnRpc>>> = OnceCell::const_new();
static RPC_POLL_INTERVAL: Duration = Duration::from_millis(500);

#[allow(unused)]
const OPT_SUPPORTS_LSPS: usize = 729;

pub async fn get_rpc<P: AsRef<Path>>(path: P) -> Arc<Mutex<cln_rpc::ClnRpc>> {
    RPC_CLIENT
        .get_or_init(|| async {
            loop {
                match cln_rpc::ClnRpc::new(path.as_ref()).await {
                    Ok(client) => {
                        debug!("Connected to lightning-rpc.");
                        return Arc::new(Mutex::new(client));
                    }
                    Err(_) => {
                        debug!("Failed to connect to lightning-rpc. Retrying in {RPC_POLL_INTERVAL:?}...");
                        tokio::time::sleep(RPC_POLL_INTERVAL).await;
                        continue;
                    }
                }
            }
        })
        .await
        .clone()
}

lazy_static! {
    static ref HSM_ID_COUNT: AtomicUsize = AtomicUsize::new(0);

    /// The number of signers that are currently connected (best guess
    /// due to races). Allows us to determine whether we should
    /// initiate operations that might require signatures.
    static ref SIGNER_COUNT: AtomicUsize = AtomicUsize::new(0);
    static ref RPC_BCAST: broadcast::Sender<super::Event> = broadcast::channel(4).0;

    static ref SERIALIZED_CONFIGURE_REQUEST: Mutex<Option<String>> = Mutex::new(None);

    static ref RPC_READY: AtomicBool = AtomicBool::new(false);
}

/// The PluginNodeServer is the interface that is exposed to client devices
/// and is in charge of coordinating the various user-controlled
/// entities. This includes dispatching incoming RPC calls to the JSON-RPC
/// interface, as well as staging requests from the HSM so that they can be
/// streamed and replied to by devices that have access to the signing keys.
#[derive(Clone)]
pub struct PluginNodeServer {
    pub tls: ServerTlsConfig,
    pub stage: Arc<stager::Stage>,
    rpc_path: PathBuf,
    events: tokio::sync::broadcast::Sender<super::Event>,
    signer_state: Arc<Mutex<State>>,
    grpc_binding: String,
    signer_state_store: Arc<Mutex<Box<dyn StateStore>>>,
    pub ctx: crate::context::Context,
    notifications: tokio::sync::broadcast::Sender<Notification>,
}

impl PluginNodeServer {
    pub async fn new(
        stage: Arc<stager::Stage>,
        config: Config,
        events: tokio::sync::broadcast::Sender<super::Event>,
        notifications: tokio::sync::broadcast::Sender<Notification>,
        signer_state_store: Box<dyn StateStore>,
    ) -> Result<Self, Error> {
        let tls = ServerTlsConfig::new()
            .identity(config.identity.id)
            .client_ca_root(config.identity.ca);

        let mut rpc_path = std::env::current_dir().unwrap();
        rpc_path.push("lightning-rpc");
        info!("Connecting to lightning-rpc at {:?}", rpc_path);

        // Bridge the RPC_BCAST into the events queue
        let tx = events.clone();
        tokio::spawn(async move {
            let mut rx = RPC_BCAST.subscribe();
            loop {
                if let Ok(e) = rx.recv().await {
                    let _ = tx.send(e);
                }
            }
        });

        let signer_state = signer_state_store.read().await?;

        let ctx = crate::context::Context::new();

        let s = PluginNodeServer {
            ctx,
            tls,
            stage,
            events,
            rpc_path: rpc_path.clone(),
            signer_state: Arc::new(Mutex::new(signer_state)),
            signer_state_store: Arc::new(Mutex::new(signer_state_store)),
            grpc_binding: config.node_grpc_binding,
            notifications,
        };

        tokio::spawn(async move {
            let rpc_arc = get_rpc(&rpc_path).await.clone();
            let mut rpc = rpc_arc.lock().await;
            let list_datastore_req = cln_rpc::model::requests::ListdatastoreRequest {
                key: Some(vec!["glconf".to_string(), "request".to_string()]),
            };

            let res = rpc.call_typed(&list_datastore_req).await;

            match res {
                Ok(list_datastore_res) => {
                    if list_datastore_res.datastore.len() > 0 {
                        let serialized_configure_request =
                            list_datastore_res.datastore[0].string.clone();
                        match serialized_configure_request {
                            Some(serialized_configure_request) => {
                                let mut cached_serialized_configure_request =
                                    SERIALIZED_CONFIGURE_REQUEST.lock().await;
                                *cached_serialized_configure_request =
                                    Some(serialized_configure_request);
                            }
                            None => {}
                        }
                    }
                }
                Err(_) => {}
            }
        });

        Ok(s)
    }

    // Wait for the limiter to allow a new RPC call
    pub async fn limit(&self) {
        let limiter = LIMITER
            .get_or_init(|| async {
                let quota = Quota::per_minute(core::num::NonZeroU32::new(300).unwrap());
                RateLimiter::direct_with_clock(quota, &MonotonicClock::default())
            })
            .await;

        limiter.until_ready().await
    }
}

#[tonic::async_trait]
impl Node for PluginNodeServer {
    type StreamCustommsgStream = ReceiverStream<Result<pb::Custommsg, Status>>;
    type StreamHsmRequestsStream = ReceiverStream<Result<pb::HsmRequest, Status>>;
    type StreamLogStream = ReceiverStream<Result<pb::LogEntry, Status>>;

    async fn lsp_invoice(
        &self,
        req: Request<pb::LspInvoiceRequest>,
    ) -> Result<Response<pb::LspInvoiceResponse>, Status> {
        let req: pb::LspInvoiceRequest = req.into_inner();
        let rpc_arc = get_rpc(&self.rpc_path).await;

        let mut rpc = rpc_arc.lock().await;

        // Check if we have sufficient incoming capacity to skip JIT channel negotiation.
        // We require capacity + 5% buffer to account for fees and routing.
        // Only check for specific amounts (not "any" amount invoices).
        if req.amount_msat > 0 {
            let receivable = self
                .get_receivable_capacity(&mut rpc)
                .await
                .unwrap_or(0);

            // Add 5% buffer: capacity >= amount * 1.05
            // Equivalent to: capacity * 100 >= amount * 105
            let has_sufficient_capacity = req.amount_msat
                .saturating_mul(105)
                .checked_div(100)
                .map(|required| receivable >= required)
                .unwrap_or(false);

            if has_sufficient_capacity {
                log::info!(
                    "Sufficient incoming capacity ({} msat) for invoice amount ({} msat), creating regular invoice",
                    receivable,
                    req.amount_msat
                );

                // Create a regular invoice without JIT channel negotiation
                let invreq = crate::requests::Invoice {
                    amount_msat: cln_rpc::primitives::AmountOrAny::Amount(
                        cln_rpc::primitives::Amount::from_msat(req.amount_msat),
                    ),
                    description: req.description.clone(),
                    label: req.label.clone(),
                    expiry: None,
                    fallbacks: None,
                    preimage: None,
                    cltv: Some(144),
                    deschashonly: None,
                    exposeprivatechannels: None,
                    dev_routes: None,
                };

                let res: crate::responses::Invoice = rpc
                    .call_raw("invoice", &invreq)
                    .await
                    .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

                return Ok(Response::new(pb::LspInvoiceResponse {
                    bolt11: res.bolt11,
                    created_index: 0, // Not available in our Invoice response
                    expires_at: res.expiry_time,
                    payment_hash: hex::decode(&res.payment_hash)
                        .map_err(|e| Status::new(Code::Internal, format!("Invalid payment_hash: {}", e)))?,
                    payment_secret: res
                        .payment_secret
                        .map(|s| hex::decode(&s).unwrap_or_default())
                        .unwrap_or_default(),
                }));
            }

            log::info!(
                "Insufficient incoming capacity ({} msat) for invoice amount ({} msat), negotiating JIT channel",
                receivable,
                req.amount_msat
            );
        }

        // Get the CLN version to determine which RPC method to use
        let version = rpc
            .call_typed(&cln_rpc::model::requests::GetinfoRequest {})
            .await
            .map_err(|e| Status::new(Code::Internal, format!("Failed to get version: {}", e)))?
            .version;

        // In case the client did not specify an LSP to work with,
        // let's enumerate them, and select the best option ourselves.
        let lsps = self.get_lsps_offers(&mut rpc).await.map_err(|_e| {
            Status::not_found("Could not retrieve LSPS peers for invoice negotiation.")
        })?;

        if lsps.len() < 1 {
            return Err(Status::not_found(
                "Could not find an LSP peer to negotiate the LSPS2 channel for this invoice.",
            ));
        }

        let lsp = &lsps[0];
        log::info!("Selecting {:?} for invoice negotiation", lsp);

        // Use the new RPC method name for versions > v25.05gl1
        let res = if *version > *"v25.05gl1" {
            let mut invreq: crate::requests::LspInvoiceRequestV2 = req.into();
            invreq.lsp_id = lsp.node_id.to_owned();
            rpc.call_typed(&invreq)
                .await
                .map_err(|e| Status::new(Code::Internal, e.to_string()))?
        } else {
            let mut invreq: crate::requests::LspInvoiceRequest = req.into();
            invreq.lsp_id = lsp.node_id.to_owned();
            rpc.call_typed(&invreq)
                .await
                .map_err(|e| Status::new(Code::Internal, e.to_string()))?
        };

        Ok(Response::new(res.into()))
    }

    async fn stream_custommsg(
        &self,
        _: Request<pb::StreamCustommsgRequest>,
    ) -> Result<Response<Self::StreamCustommsgStream>, Status> {
        log::debug!("Added a new listener for custommsg");
        let (tx, rx) = mpsc::channel(1);
        let mut stream = self.events.subscribe();
        // TODO: We can do better by returning the broadcast receiver
        // directly. Well really we should be filtering the events by
        // type, so maybe a `.map()` on the stream can work?
        tokio::spawn(async move {
            while let Ok(msg) = stream.recv().await {
                if let Event::CustomMsg(m) = msg {
                    log::trace!("Forwarding custommsg {:?} to listener", m);
                    if let Err(e) = tx.send(Ok(m)).await {
                        log::warn!("Unable to send custmmsg to listener: {:?}", e);
                        break;
                    }
                }
            }
            panic!("stream.recv loop exited...");
        });
        return Ok(Response::new(ReceiverStream::new(rx)));
    }

    async fn stream_log(
        &self,
        _: Request<pb::StreamLogRequest>,
    ) -> Result<Response<Self::StreamLogStream>, Status> {
        match async {
            let (tx, rx) = mpsc::channel(1);
            let mut lines = linemux::MuxedLines::new()?;
            lines.add_file("/tmp/log").await?;

            // TODO: Yes, this may produce duplicate lines, when new
            // log entries are produced while we're streaming the
            // backlog out, but do we care?
            use tokio::io::{AsyncBufReadExt, BufReader};
            // The nodelet uses its CWD, but CLN creates a network
            // subdirectory
            let file = tokio::fs::File::open("../log").await?;
            let mut file = BufReader::new(file).lines();

            tokio::spawn(async move {
                match async {
                    while let Some(line) = file.next_line().await? {
                        tx.send(Ok(pb::LogEntry {
                            line: line.trim().to_owned(),
                        }))
                        .await?
                    }

                    while let Ok(Some(line)) = lines.next_line().await {
                        tx.send(Ok(pb::LogEntry {
                            line: line.line().trim().to_string(),
                        }))
                        .await?;
                    }
                    Ok(())
                }
                .await as Result<(), anyhow::Error>
                {
                    Ok(()) => {}
                    Err(e) => {
                        warn!("error streaming logs to client: {}", e);
                    }
                }
            });
            Ok(ReceiverStream::new(rx))
        }
        .await as Result<Self::StreamLogStream, anyhow::Error>
        {
            Ok(v) => Ok(Response::new(v)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn stream_hsm_requests(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<Self::StreamHsmRequestsStream>, Status> {
        let hsm_id = HSM_ID_COUNT.fetch_add(1, Ordering::SeqCst);
        SIGNER_COUNT.fetch_add(1, Ordering::SeqCst);
        info!(
            "New signer with hsm_id={} attached, streaming requests",
            hsm_id
        );

        let (tx, rx) = mpsc::channel(10);
        let mut stream = self.stage.mystream().await;
        let signer_state = self.signer_state.clone();
        let ctx = self.ctx.clone();

        tokio::spawn(async move {
            trace!("hsmd hsm_id={} request processor started", hsm_id);

            {
                // We start by immediately injecting a
                // vls_protocol::Message::GetHeartbeat. This serves two
                // purposes: already send the initial snapshot of the
                // signer state to the signer as early as possible, and
                // triggering a pruning on the signer, if enabled. In
                // incremental mode this ensures that any subsequent,
                // presumably time-critical messages, do not have to carry
                // the large state with them.

                let state = signer_state.lock().await.clone();
                let state: Vec<gl_client::pb::SignerStateEntry> = state.into();
                let state: Vec<pb::SignerStateEntry> = state
                    .into_iter()
                    .map(|s| pb::SignerStateEntry {
                        key: s.key,
                        version: s.version,
                        value: s.value,
                    })
                    .collect();

                let msg = vls_protocol::msgs::GetHeartbeat {};
                use vls_protocol::msgs::SerBolt;
                let req = crate::pb::HsmRequest {
                    // Notice that the request_counter starts at 1000, to
                    // avoid collisions.
                    request_id: 0,
                    signer_state: state,
                    raw: msg.as_vec(),
                    requests: vec![], // No pending requests yet, nothing to authorize.
                    context: None,
                };

                if let Err(e) = tx.send(Ok(req)).await {
                    log::warn!("Failed to send heartbeat message to signer: {}", e);
                }
            }

            loop {
                let mut req = match stream.next().await {
                    Err(e) => {
                        error!(
                            "Could not get next request from stage: {:?} for hsm_id={}",
                            e, hsm_id
                        );
                        break;
                    }
                    Ok(r) => r,
                };
                trace!(
                    "Sending request={} to hsm_id={}",
                    req.request.request_id,
                    hsm_id
                );

                let state = signer_state.lock().await.clone();
                let state: Vec<gl_client::pb::SignerStateEntry> = state.into();

                // TODO Consolidate protos in `gl-client` and `gl-plugin`, then remove this map.
                let state: Vec<pb::SignerStateEntry> = state
                    .into_iter()
                    .map(|s| pb::SignerStateEntry {
                        key: s.key,
                        version: s.version,
                        value: s.value,
                    })
                    .collect();

                req.request.signer_state = state.into();
                req.request.requests = ctx.snapshot().await.into_iter().map(|r| r.into()).collect();

                let serialized_configure_request = SERIALIZED_CONFIGURE_REQUEST.lock().await;

                match &(*serialized_configure_request) {
                    Some(serialized_configure_request) => {
                        let configure_request = serde_json::from_str::<crate::context::Request>(
                            serialized_configure_request,
                        )
                        .unwrap();
                        req.request.requests.push(configure_request.into());
                    }
                    None => {}
                }

                debug!(
                    "Sending signer requests with {} requests and {} state entries",
                    req.request.requests.len(),
                    req.request.signer_state.len()
                );

                eprintln!("WIRE: plugin -> signer: {:?}", req);
                if let Err(e) = tx.send(Ok(req.request)).await {
                    warn!("Error streaming request {:?} to hsm_id={}", e, hsm_id);
                    break;
                }
            }
            info!("Signer hsm_id={} exited", hsm_id);
            SIGNER_COUNT.fetch_sub(1, Ordering::SeqCst);
        });

        trace!("Returning stream_hsm_request channel");
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn respond_hsm_request(
        &self,
        request: Request<pb::HsmResponse>,
    ) -> Result<Response<pb::Empty>, Status> {
        let req = request.into_inner();

        if req.error != "" {
            log::error!("Signer reports an error: {}", req.error);
            log::warn!("The above error was returned instead of a response.");
            return Ok(Response::new(pb::Empty::default()));
        }
        eprintln!("WIRE: signer -> plugin: {:?}", req);

        // Create a state from the key-value-version tuples. Need to
        // convert here, since `pb` is duplicated in the two different
        // crates.
        let signer_state: Vec<gl_client::pb::SignerStateEntry> = req
            .signer_state
            .iter()
            .map(|i| gl_client::pb::SignerStateEntry {
                key: i.key.to_owned(),
                value: i.value.to_owned(),
                version: i.version,
            })
            .collect();
        let new_state: gl_client::persist::State = signer_state.into();

        // Apply state changes to the in-memory state
        let mut state = self.signer_state.lock().await;
        state.merge(&new_state).map_err(|e| {
            Status::new(
                Code::Internal,
                format!("Error updating internal state: {e}"),
            )
        })?;

        // Send changes to the signer_state_store for persistence
        let store = self.signer_state_store.lock().await;
        if let Err(e) = store.write(state.clone()).await {
            log::warn!(
                "The returned state could not be stored. Ignoring response for request_id={}, error={:?}",
                req.request_id, e
            );
            /* Exit here so we don't end up committing the changes
             * to CLN, but not to the state store. That'd cause
             * drifts in states that are very hard to debug, and
             * harder to correct. */
            return Ok(Response::new(pb::Empty::default()));
        }

        if let Err(e) = self.stage.respond(req).await {
            warn!("Suppressing error: {:?}", e);
        }
        Ok(Response::new(pb::Empty::default()))
    }

    type StreamIncomingStream = ReceiverStream<Result<pb::IncomingPayment, Status>>;

    async fn stream_incoming(
        &self,
        _req: tonic::Request<pb::StreamIncomingFilter>,
    ) -> Result<Response<Self::StreamIncomingStream>, Status> {
        // TODO See if we can just return the broadcast::Receiver
        // instead of pulling off broadcast and into an mpsc.
        let (tx, rx) = mpsc::channel(1);
        let mut bcast = self.events.subscribe();
        tokio::spawn(async move {
            while let Ok(p) = bcast.recv().await {
                match p {
                    super::Event::IncomingPayment(p) => {
                        let _ = tx.send(Ok(p)).await;
                    }
                    _ => {}
                }
            }
        });

        return Ok(Response::new(ReceiverStream::new(rx)));
    }

    async fn configure(
        &self,
        req: tonic::Request<pb::GlConfig>,
    ) -> Result<Response<pb::Empty>, Status> {
        self.limit().await;
        let gl_config = req.into_inner();
        let rpc_arc = get_rpc(&self.rpc_path).await;
        let mut rpc = rpc_arc.lock().await;

        let res = rpc
            .call_typed(&cln_rpc::model::requests::GetinfoRequest {})
            .await;

        let network = match res {
            Ok(get_info_response) => match get_info_response.network.parse() {
                Ok(v) => v,
                Err(_) => Err(Status::new(
                    Code::Unknown,
                    format!("Failed to parse 'network' from 'getinfo' response"),
                ))?,
            },
            Err(e) => {
                return Err(Status::new(
                        Code::Unknown,
                        format!("Failed to retrieve a response from 'getinfo' while setting the node's configuration: {}", e),
                    ));
            }
        };

        match bitcoin::Address::from_str(&gl_config.close_to_addr) {
            Ok(address) => {
                if let Err(e) = address.require_network(network) {
                    return Err(Status::new(
                        Code::Unknown,
                        format!(
                            "Network validation failed: {}",
                            e
                        ),
                    ));
                }
            }
            Err(e) => {
                return Err(Status::new(
                    Code::Unknown,
                    format!(
                        "The address {} is not valid: {}",
                        gl_config.close_to_addr, e
                    ),
                ));
            }
        }

        let requests: Vec<crate::context::Request> = self
            .ctx
            .snapshot()
            .await
            .into_iter()
            .map(|r| r.into())
            .collect();
        let serialized_req = serde_json::to_string(&requests[0]).unwrap();
        let datastore_res = rpc
            .call_typed(&cln_rpc::model::requests::DatastoreRequest {
                key: vec!["glconf".to_string(), "request".to_string()],
                string: Some(serialized_req.clone()),
                hex: None,
                mode: None,
                generation: None,
            })
            .await;

        match datastore_res {
            Ok(_) => {
                let mut cached_gl_config = SERIALIZED_CONFIGURE_REQUEST.lock().await;
                *cached_gl_config = Some(serialized_req);

                Ok(Response::new(pb::Empty::default()))
            }
            Err(e) => {
                return Err(Status::new(
                    Code::Unknown,
                    format!(
                        "Failed to store the raw configure request in the datastore: {}",
                        e
                    ),
                ))
            }
        }
    }

    async fn trampoline_pay(
        &self,
        r: tonic::Request<pb::TrampolinePayRequest>,
    ) -> Result<tonic::Response<pb::TrampolinePayResponse>, Status> {
        tramp::trampolinepay(r.into_inner(), self.rpc_path.clone())
            .await
            .map(cln_rpc::model::responses::PayResponse::into)
            .map(|res: cln_grpc::pb::PayResponse| {
                tonic::Response::new(pb::TrampolinePayResponse {
                    payment_preimage: res.payment_preimage,
                    payment_hash: res.payment_hash,
                    created_at: res.created_at,
                    parts: res.parts,
                    amount_msat: res.amount_msat.unwrap_or_default().msat,
                    amount_sent_msat: res.amount_sent_msat.unwrap_or_default().msat,
                    destination: res.destination.unwrap_or_default(),
                })
            })
            .map_err(|err| {
                debug!("Trampoline payment failed: {}", err);
                err.into()
            })
    }
}

use cln_grpc::pb::node_server::NodeServer;

#[derive(Clone, Debug)]
struct Lsps2Offer {
    node_id: String,
    #[allow(unused)]
    params: Vec<crate::responses::OpeningFeeParams>,
}

impl PluginNodeServer {
    pub async fn run(self) -> Result<()> {
        let addr = self.grpc_binding.parse().unwrap();

        let cln_node = NodeServer::new(
            WrappedNodeServer::new(self.clone())
                .await
                .context("creating NodeServer instance")?,
        );

        let router = tonic::transport::Server::builder()
            .max_frame_size(4 * 1024 * 1024) // 4MB max request size
            .tcp_keepalive(Some(tokio::time::Duration::from_secs(1)))
            .tls_config(self.tls.clone())?
            .layer(SignatureContextLayer {
                ctx: self.ctx.clone(),
            })
            .add_service(RpcWaitService::new(cln_node, self.rpc_path.clone()))
            .add_service(crate::pb::node_server::NodeServer::new(self.clone()));

        router
            .serve(addr)
            .await
            .context("grpc interface exited with error")
    }

    /// Reconnect all peers with whom we have a channel or previously
    /// connected explicitly to.
    pub async fn reconnect_peers(&self) -> Result<(), Error> {
        if SIGNER_COUNT.load(Ordering::SeqCst) < 1 {
            use anyhow::anyhow;
            return Err(anyhow!(
                "Cannot reconnect peers, no signer to complete the handshake"
            ));
        }

        log::info!("Reconnecting all peers (plugin)");
        let peers = self.get_reconnect_peers().await?;
        log::info!(
            "Found {} peers to reconnect: {:?} (plugin)",
            peers.len(),
            peers.iter().map(|p| p.id.clone())
        );

        let rpc_arc = get_rpc(&self.rpc_path).await;
        let mut rpc = rpc_arc.lock().await;

        for r in peers {
            trace!("Calling connect: {:?} (plugin)", &r.id);
            let res = rpc.call_typed(&r).await;
            trace!("Connect returned: {:?} -> {:?} (plugin)", &r.id, res);

            match res {
                Ok(r) => info!("Connection to {} established: {:?} (plugin)", &r.id, r),
                Err(e) => warn!("Could not connect to {}: {:?} (plugin)", &r.id, e),
            }
        }
        return Ok(());
    }

    async fn list_peers(
        &self,
        rpc: &mut cln_rpc::ClnRpc,
    ) -> Result<cln_rpc::model::responses::ListpeersResponse, Error> {
        rpc.call_typed(&cln_rpc::model::requests::ListpeersRequest {
            id: None,
            level: None,
        })
        .await
        .map_err(|e| e.into())
    }

    async fn get_lsps_offers(&self, rpc: &mut cln_rpc::ClnRpc) -> Result<Vec<Lsps2Offer>, Error> {
        // Collect peers offering LSP functionality
        let lpeers = self.list_peers(rpc).await?;

        // Filter out the ones that do not announce the LSPs features.
        // TODO: Re-enable the filtering once the cln-lsps-service plugin announces the features.
        let _lsps: Vec<cln_rpc::model::responses::ListpeersPeers> = lpeers
            .peers
            .into_iter()
            //.filter(|p| has_feature(
            //    hex::decode(p.features.clone().unwrap_or_default()).expect("featurebits are hex"),
            //    OPT_SUPPORTS_LSPS
            //))
            .collect();

        // Query all peers for their LSPS offers, but with a brief
        // timeout so the invoice creation isn't help up too long.
        let futs: Vec<
            tokio::task::JoinHandle<(
                String,
                Result<
                    Result<crate::responses::LspGetinfoResponse, cln_rpc::RpcError>,
                    tokio::time::error::Elapsed,
                >,
            )>,
        > = _lsps
            .into_iter()
            .map(|peer| {
                let rpc_path = self.rpc_path.clone();
                tokio::spawn(async move {
                    let peer_id = format!("{:x}", peer.id);
                    let mut rpc = cln_rpc::ClnRpc::new(rpc_path.clone()).await.unwrap();
                    let req = crate::requests::LspGetinfoRequest {
                        lsp_id: peer_id.clone(),
                        token: None,
                    };

                    (
                        peer_id,
                        tokio::time::timeout(
                            tokio::time::Duration::from_secs(2),
                            rpc.call_typed(&req),
                        )
                        .await,
                    )
                })
            })
            .collect();

        let mut res = vec![];
        for f in futs {
            match f.await {
                //TODO We need to drag the node_id along.
                Ok((node_id, Ok(Ok(r)))) => res.push(Lsps2Offer {
                    node_id: node_id,
                    params: r.opening_fee_params_menu,
                }),
                Ok((node_id, Err(e))) => warn!(
                    "Error fetching LSPS menu items from peer_id={}: {:?}",
                    node_id, e
                ),
                Ok((node_id, Ok(Err(e)))) => warn!(
                    "Error fetching LSPS menu items from peer_id={}: {:?}",
                    node_id, e
                ),
                Err(_) => warn!("Timeout fetching LSPS menu items"),
            }
        }

        log::info!("Gathered {} LSP menus", res.len());
        log::trace!("LSP menus: {:?}", &res);

        Ok(res)
    }

    /// Get the total receivable capacity across all active channels.
    ///
    /// Returns the sum of `receivable_msat` for all channels in
    /// `CHANNELD_NORMAL` state with a connected peer.
    async fn get_receivable_capacity(&self, rpc: &mut cln_rpc::ClnRpc) -> Result<u64, Error> {
        use cln_rpc::primitives::ChannelState;

        let res = rpc
            .call_typed(&cln_rpc::model::requests::ListpeerchannelsRequest { id: None })
            .await?;

        let total: u64 = res
            .channels
            .into_iter()
            .filter(|c| c.peer_connected && c.state == ChannelState::CHANNELD_NORMAL)
            .filter_map(|c| c.receivable_msat)
            .map(|a| a.msat())
            .sum();

        log::debug!("Total receivable capacity: {} msat", total);
        Ok(total)
    }

    async fn get_reconnect_peers(
        &self,
    ) -> Result<Vec<cln_rpc::model::requests::ConnectRequest>, Error> {
        let rpc_arc = get_rpc(&self.rpc_path).await;
        let mut rpc = rpc_arc.lock().await;
        let peers = self.list_peers(&mut rpc).await?;

        let mut requests: Vec<cln_rpc::model::requests::ConnectRequest> = peers
            .peers
            .iter()
            .filter(|&p| p.connected)
            .map(|p| cln_rpc::model::requests::ConnectRequest {
                id: p.id.to_string(),
                host: None,
                port: None,
            })
            .collect();

        let mut dspeers: Vec<cln_rpc::model::requests::ConnectRequest> = rpc
            .call_typed(&cln_rpc::model::requests::ListdatastoreRequest {
                key: Some(vec!["greenlight".to_string(), "peerlist".to_string()]),
            })
            .await?
            .datastore
            .iter()
            .map(|x| {
                // We need to replace unnecessary escape characters that
                // have been added by the datastore, as serde is a bit
                // picky on that.
                let mut s = x.string.clone().unwrap();
                s = s.replace('\\', "");
                serde_json::from_str::<messages::Peer>(&s).unwrap()
            })
            .map(|x| cln_rpc::model::requests::ConnectRequest {
                id: x.id,
                host: Some(x.addr),
                port: None,
            })
            .collect();

        // Merge the two peer lists;
        requests.append(&mut dspeers);
        requests.sort_by(|a, b| a.id.cmp(&b.id));
        requests.dedup_by(|a, b| a.id.eq(&b.id));

        Ok(requests)
    }
}

use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct SignatureContextLayer {
    ctx: crate::context::Context,
}

impl SignatureContextLayer {
    pub fn new(context: crate::context::Context) -> Self {
        SignatureContextLayer { ctx: context }
    }
}

impl<S> Layer<S> for SignatureContextLayer {
    type Service = SignatureContextService<S>;

    fn layer(&self, service: S) -> Self::Service {
        SignatureContextService {
            inner: service,
            ctx: self.ctx.clone(),
        }
    }
}

// Is the maximum message size we allow to buffer up on requests.
const MAX_MESSAGE_SIZE: usize = 4000000;

#[derive(Debug, Clone)]
pub struct SignatureContextService<S> {
    inner: S,
    ctx: crate::context::Context,
}

impl<S> Service<hyper::Request<hyper::Body>> for SignatureContextService<S>
where
    S: Service<hyper::Request<hyper::Body>, Response = hyper::Response<tonic::body::BoxBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: hyper::Request<hyper::Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let reqctx = self.ctx.clone();

        Box::pin(async move {
            use tonic::codegen::Body;
            let (parts, mut body) = request.into_parts();

            let uri = parts.uri.path_and_query().unwrap();
            let _ = RPC_BCAST
                .clone()
                .send(super::Event::RpcCall(uri.to_string()));

            let pubkey = parts
                .headers
                .get("glauthpubkey")
                .and_then(|k| general_purpose::STANDARD_NO_PAD.decode(k).ok());

            let sig = parts
                .headers
                .get("glauthsig")
                .and_then(|k| general_purpose::STANDARD_NO_PAD.decode(k).ok());

            use bytes::Buf;
            let timestamp: Option<u64> = parts
                .headers
                .get("glts")
                .and_then(|k| general_purpose::STANDARD_NO_PAD.decode(k).ok())
                .map(|s| s.as_slice().get_u64());

            let rune = parts
                .headers
                .get("glrune")
                .and_then(|k| general_purpose::URL_SAFE.decode(k).ok());

            if let (Some(pk), Some(sig), Some(rune)) = (pubkey, sig, rune) {
                // Now that we know we'll be adding this to the
                // context we can start buffering the request.
                let mut buf = Vec::new();
                while let Some(chunk) = body.data().await {
                    let chunk = chunk.unwrap();
                    // We check on the MAX_MESSAGE_SIZE to avoid an unlimited sized
                    // message buffer
                    if buf.len() + chunk.len() > MAX_MESSAGE_SIZE {
                        debug!("Message {} exceeds size limit", uri.path().to_string());
                        return Ok(tonic::Status::new(
                            tonic::Code::InvalidArgument,
                            format!("payload too large"),
                        )
                        .to_http());
                    }
                    buf.put(chunk);
                }

                trace!(
                    "Got a request for {} with pubkey={}, sig={}, rune={} and body size={:?}",
                    uri,
                    hex::encode(&pk),
                    hex::encode(&sig),
                    hex::encode(&rune),
                    &buf.len(),
                );
                let req = crate::context::Request::new(
                    uri.to_string(),
                    <bytes::Bytes>::from(buf.clone()),
                    pk,
                    sig,
                    timestamp,
                    rune,
                );

                reqctx.add_request(req.clone()).await;

                let body: hyper::Body = buf.into();
                let request = hyper::Request::from_parts(parts, body);
                let res = inner.call(request).await;

                // Defer cleanup into a separate task, otherwise we'd
                // need `res` to be `Send` which we can't
                // guarantee. This is needed since adding an await
                // point splits the state machine at that point.
                tokio::spawn(async move {
                    reqctx.remove_request(req).await;
                });
                res.map_err(Into::into)
            } else {
                // No point in buffering the request, we're not going
                // to add it to the `HsmRequestContext`
                let request = hyper::Request::from_parts(parts, body);
                inner.call(request).await.map_err(Into::into)
            }
        })
    }
}

mod rpcwait;
pub use rpcwait::RpcWaitService;
