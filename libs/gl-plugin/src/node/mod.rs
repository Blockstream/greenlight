use crate::config::Config;
use crate::pb::{self, node_server::Node};
use crate::rpc::LightningClient;
use crate::stager;
use crate::storage::StateStore;
use crate::{messages, Event};
use anyhow::{Context, Error, Result};
use base64::{engine::general_purpose, Engine as _};
use bytes::BufMut;
use gl_client::persist::State;
use governor::{
    clock::MonotonicClock, state::direct::NotKeyed, state::InMemoryState, Quota, RateLimiter,
};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{broadcast, mpsc, Mutex, OnceCell};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::ServerTlsConfig, Code, Request, Response, Status};
mod wrapper;
pub use wrapper::WrappedNodeServer;
use gl_client::bitcoin;
use std::str::FromStr;


static LIMITER: OnceCell<RateLimiter<NotKeyed, InMemoryState, MonotonicClock>> =
    OnceCell::const_new();

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
    pub rpc: Arc<Mutex<LightningClient>>,
    rpc_path: PathBuf,
    events: tokio::sync::broadcast::Sender<super::Event>,
    signer_state: Arc<Mutex<State>>,
    grpc_binding: String,
    signer_state_store: Arc<Mutex<Box<dyn StateStore>>>,
    pub ctx: crate::context::Context,
}

impl PluginNodeServer {
    pub async fn new(
        stage: Arc<stager::Stage>,
        config: Config,
        events: tokio::sync::broadcast::Sender<super::Event>,
        signer_state_store: Box<dyn StateStore>,
    ) -> Result<Self, Error> {
        let tls = ServerTlsConfig::new()
            .identity(config.identity.id)
            .client_ca_root(config.identity.ca);

        let mut rpc_path = std::env::current_dir().unwrap();
        rpc_path.push("lightning-rpc");
        info!("Connecting to lightning-rpc at {:?}", rpc_path);

        let rpc = Arc::new(Mutex::new(LightningClient::new(rpc_path.clone())));

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

        let rrpc = rpc.clone();

        let s = PluginNodeServer {
            ctx,
            tls,
            rpc,
            stage,
            events,
            rpc_path,
            signer_state: Arc::new(Mutex::new(signer_state)),
            signer_state_store: Arc::new(Mutex::new(signer_state_store)),
            grpc_binding: config.node_grpc_binding,
        };

        tokio::spawn(async move {
            debug!("Locking grpc interface until the JSON-RPC interface becomes available.");
            use tokio::time::{sleep, Duration};

            // Move the lock into the closure so we can release it later.
            let rpc = rrpc.lock().await;
            loop {
                let res: Result<crate::responses::GetInfo, crate::rpc::Error> =
                    rpc.call("getinfo", json!({})).await;
                match res {
                    Ok(_) => break,
                    Err(e) => {
                        warn!(
                            "JSON-RPC interface not yet available. Delaying 50ms. {:?}",
                            e
                        );
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            }

	    // Signal that the RPC is ready now.
	    RPC_READY.store(true, Ordering::SeqCst);

            let list_datastore_req = cln_rpc::model::requests::ListdatastoreRequest{
                key: Some(vec![
                    "glconf".to_string(),
                    "request".to_string()
                ])
            };

            let res: Result<cln_rpc::model::responses::ListdatastoreResponse, crate::rpc::Error> =
                rpc.call("listdatastore", list_datastore_req).await;

            match res {
                Ok(list_datastore_res) => {
                    if list_datastore_res.datastore.len() > 0 {
                        let serialized_configure_request = list_datastore_res.datastore[0].string.clone();
                        match serialized_configure_request {
                            Some(serialized_configure_request) => {
                                let mut cached_serialized_configure_request = SERIALIZED_CONFIGURE_REQUEST.lock().await;
                                *cached_serialized_configure_request = Some(serialized_configure_request);
                            }
                            None => {}
                        }
                    }
                }
                Err(_) => {}
            }
            
            drop(rpc);
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

    pub async fn get_rpc(&self) -> LightningClient {
        let rpc = self.rpc.lock().await;
        let r = rpc.clone();
        drop(rpc);
        r
    }
}

#[tonic::async_trait]
impl Node for PluginNodeServer {
    type StreamCustommsgStream = ReceiverStream<Result<pb::Custommsg, Status>>;
    type StreamHsmRequestsStream = ReceiverStream<Result<pb::HsmRequest, Status>>;
    type StreamLogStream = ReceiverStream<Result<pb::LogEntry, Status>>;

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
            let file = tokio::fs::File::open("/tmp/log").await?;
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

        {
            // Apply state changes to the in-memory state
            let mut state = self.signer_state.lock().await;
            state.merge(&new_state).map_err(|e| {
                Status::new(
                    Code::Internal,
                    format!("Error updating internal state: {e}"),
                )
            })?;

            // Send changes to the signer_state_store for persistence
            self.signer_state_store
                .lock()
                .await
                .write(state.clone())
                .await
                .map_err(|e| {
                    Status::new(
                        Code::Internal,
                        format!("error persisting state changes: {}", e),
                    )
                })?;
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

    async fn configure(&self, req: tonic::Request<pb::GlConfig>) -> Result<Response<pb::Empty>, Status>  {
        self.limit().await;
        let gl_config = req.into_inner();
        let rpc = self.get_rpc().await;

        let res: Result<crate::responses::GetInfo, crate::rpc::Error> =
            rpc.call("getinfo", json!({})).await;

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
                if address.network != network {
                    return Err(Status::new(
                        Code::Unknown,
                        format!(
                            "Network mismatch: \
                            Expected an address for {} but received an address for {}",
                            network,
                            address.network
                        ),
                    ));
                }
            }
            Err(e) => {
                return Err(Status::new(
                    Code::Unknown,
                    format!("The address {} is not valid: {}", gl_config.close_to_addr, e),
                ));
            }
        }

        let requests: Vec<crate::context::Request> = self.ctx.snapshot().await.into_iter().map(|r| r.into()).collect();
        let serialized_req = serde_json::to_string(&requests[0]).unwrap();
        let datastore_res: Result<crate::cln_rpc::model::responses::DatastoreResponse, crate::rpc::Error> =
            rpc.call("datastore", json!({
                "key": vec![
                    "glconf".to_string(),
                    "request".to_string(),
                ],
                "string": serialized_req,
            })).await;
        
        match datastore_res {
            Ok(_) => {
                let mut cached_gl_config = SERIALIZED_CONFIGURE_REQUEST.lock().await;
                *cached_gl_config = Some(serialized_req);

                Ok(Response::new(pb::Empty::default()))
            }
            Err(e) => {
                return Err(Status::new(
                    Code::Unknown,
                    format!("Failed to store the raw configure request in the datastore: {}", e),
                ))
            }
        }
    }
}

use cln_grpc::pb::node_server::NodeServer;

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

        log::info!("Reconnecting all peers");
        let mut rpc = cln_rpc::ClnRpc::new(self.rpc_path.clone()).await?;
        let peers = self.get_reconnect_peers().await?;

        for r in peers {
            trace!("Calling connect: {:?}", &r.id);
            let res = rpc.call_typed(r.clone()).await;
            trace!("Connect returned: {:?} -> {:?}", &r.id, res);

            match res {
                Ok(r) => info!("Connection to {} established: {:?}", &r.id, r),
                Err(e) => warn!("Could not connect to {}: {:?}", &r.id, e),
            }
        }
        return Ok(());
    }

    async fn get_reconnect_peers(
        &self,
    ) -> Result<Vec<cln_rpc::model::requests::ConnectRequest>, Error> {
        let rpc_path = self.rpc_path.clone();
        let mut rpc = cln_rpc::ClnRpc::new(rpc_path).await?;
        let peers = rpc
            .call_typed(cln_rpc::model::requests::ListpeersRequest {
                id: None,
                level: None,
            })
            .await?;

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
            .call_typed(cln_rpc::model::requests::ListdatastoreRequest {
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
                .map(|k| general_purpose::STANDARD_NO_PAD.decode(k).ok())
                .flatten();

            let sig = parts
                .headers
                .get("glauthsig")
                .map(|s| general_purpose::STANDARD_NO_PAD.decode(s).ok())
                .flatten();

            use bytes::Buf;
            let timestamp: Option<u64> = parts
                .headers
                .get("glts")
                .map(|s| general_purpose::STANDARD_NO_PAD.decode(s).ok())
                .flatten()
                .map(|s| s.as_slice().get_u64());

            if let (Some(pk), Some(sig)) = (pubkey, sig) {
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
                    "Got a request for {} with pubkey={}, sig={} and body size={:?}",
                    uri,
                    hex::encode(&pk),
                    hex::encode(&sig),
                    &buf.len(),
                );
                let req = crate::context::Request::new(
                    uri.to_string(),
                    <bytes::Bytes>::from(buf.clone()),
                    pk,
                    sig,
                    timestamp,
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
