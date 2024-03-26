use crate::credentials::{RuneProvider, TlsConfigProvider};
use crate::pb::scheduler::{scheduler_client::SchedulerClient, NodeInfoRequest, UpgradeRequest};
/// The core signer system. It runs in a dedicated thread or using the
/// caller thread, streaming incoming requests, verifying them,
/// signing if ok, and then shipping the response to the node.
use crate::pb::{node_client::NodeClient, Empty, HsmRequest, HsmRequestContext, HsmResponse};
use crate::signer::resolve::Resolver;
use crate::tls::TlsConfig;
use crate::{node, node::Client};
use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use bytes::BufMut;
use http::uri::InvalidUri;
use lightning_signer::bitcoin::hashes::Hash;
use lightning_signer::bitcoin::secp256k1::PublicKey;
use lightning_signer::bitcoin::Network;
use lightning_signer::node::NodeServices;
use lightning_signer::policy::filter::FilterRule;
use lightning_signer::util::crypto_utils;
use log::{debug, info, trace, warn, error};
use runeauth::{Condition, MapChecker, Restriction, Rune, RuneError};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tonic::transport::{Endpoint, Uri};
use tonic::{Code, Request};
use vls_protocol::msgs::{DeBolt, HsmdInitReplyV4};
use vls_protocol::serde_bolt::Octets;
use vls_protocol_signer::approver::{Approve, MemoApprover};
use vls_protocol_signer::handler;
use vls_protocol_signer::handler::Handler;

mod approver;
mod auth;
pub mod model;
mod report;
mod resolve;

const VERSION: &str = "v23.08";
const GITHASH: &str = env!("GIT_HASH");
const RUNE_VERSION: &str = "gl0";
// This is the same derivation key that is used by core lightning itself.
const RUNE_DERIVATION_SECRET: &str = "gl-commando";

#[derive(Clone)]
pub struct Signer {
    secret: [u8; 32],
    master_rune: Rune,
    services: NodeServices,
    tls: TlsConfig,
    id: Vec<u8>,

    /// Cached version of the init response
    init: Vec<u8>,

    network: Network,
    state: Arc<Mutex<crate::persist::State>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not connect to scheduler: ")]
    SchedulerConnection(),

    #[error("scheduler returned an error: {0}")]
    Scheduler(tonic::Status),

    #[error("could not connect to node: {0}")]
    NodeConnection(#[from] tonic::transport::Error),

    #[error("connection to node lost: {0}")]
    NodeDisconnect(#[from] tonic::Status),

    #[error("authentication error: {0}")]
    Auth(crate::Error),

    #[error("scheduler returned faulty URI: {0}")]
    InvalidUri(#[from] InvalidUri),

    #[error("resolver error: request {0:?}, context: {1:?}")]
    Resolver(Vec<u8>, Vec<crate::signer::model::Request>),

    #[error("error asking node to be upgraded: {0}")]
    Upgrade(tonic::Status),

    #[error("protocol error: {0}")]
    Protocol(#[from] vls_protocol::Error),

    #[error("other: {0}")]
    Other(anyhow::Error),
}

impl Signer {
    pub fn new<T>(secret: Vec<u8>, network: Network, creds: T) -> Result<Signer, anyhow::Error>
    where
        T: TlsConfigProvider,
    {
        use lightning_signer::policy::{
            filter::PolicyFilter, simple_validator::SimpleValidatorFactory,
        };
        use lightning_signer::signer::ClockStartingTimeFactory;
        use lightning_signer::util::clock::StandardClock;

        info!("Initializing signer for {VERSION} ({GITHASH}) (VLS)");
        let mut sec: [u8; 32] = [0; 32];
        sec.copy_from_slice(&secret[0..32]);

        // The persister takes care of persisting metadata across
        // restarts
        let persister = Arc::new(crate::persist::MemoryPersister::new());
        let mut policy = lightning_signer::policy::simple_validator::make_simple_policy(network);

        policy.filter = PolicyFilter::default();
        policy.filter.merge(PolicyFilter {
            // TODO: Remove once we have fully switched over to zero-fee anchors
            rules: vec![
                FilterRule::new_warn("policy-channel-safe-type-anchors"),
                FilterRule::new_warn("policy-routing-balanced"),
            ],
        });

        policy.filter.merge(PolicyFilter {
            // TODO: Remove once we have implemented zero invoice support
            rules: vec![
                FilterRule::new_warn("policy-routing-balanced"),
                FilterRule::new_warn("policy-htlc-fee-range"),
            ],
        });

        // Increase the invoices limit. Results in a larger state, but
        // bumping into this is rather annoying.
        policy.max_invoices = 10_000usize;

        // Relaxed max_routing_fee since we no longer have the
        // presplitter which was causing the HTLCs to be smaller.
        policy.max_routing_fee_msat = 1_000_000;

        let validator_factory = Arc::new(SimpleValidatorFactory::new_with_policy(policy));
        let starting_time_factory = ClockStartingTimeFactory::new();
        let clock = Arc::new(StandardClock());

        let services = NodeServices {
            validator_factory,
            starting_time_factory,
            persister: persister.clone(),
            clock,
        };

        let mut handler = handler::HandlerBuilder::new(network, 0 as u64, services.clone(), sec)
            .build()
            .map_err(|e| anyhow!("building root_handler: {:?}", e))?
            .0;

        // Calling init on the `InitHandler` from above puts it into a
        // state that it can be upgraded into the `RootHandler` that
        // we need for the rest of the run.
        let init = Signer::initmsg(&mut handler)?;

        let init = HsmdInitReplyV4::from_vec(init).unwrap();

        let id = init.node_id.0.to_vec();
        use vls_protocol::msgs::SerBolt;
        let init = init.as_vec();

        // Init master rune. We create the rune seed from the nodes
        // seed by deriving a hardened key tagged with "rune secret".
        let rune_secret = crypto_utils::hkdf_sha256(&sec, RUNE_DERIVATION_SECRET.as_bytes(), &[]);
        let mr = Rune::new_master_rune(&rune_secret, vec![], None, Some(RUNE_VERSION.to_string()))?;

        trace!("Initialized signer for node_id={}", hex::encode(&id));
        Ok(Signer {
            secret: sec,
            master_rune: mr,
            services,
            tls: creds.tls_config(),
            id,
            init,
            network,
            state: persister.state(),
        })
    }

    fn init_handler(&self) -> Result<handler::InitHandler, anyhow::Error> {
        let h = handler::HandlerBuilder::new(
            self.network,
            0 as u64,
            self.services.clone(),
            self.secret,
        )
        .build()
        .map_err(|e| anyhow!("building root_handler: {:?}", e))?
        .0;

        Ok(h)
    }

    fn handler(&self) -> Result<handler::RootHandler, anyhow::Error> {
        let mut h = self.init_handler()?;
        h.handle(Signer::initreq())
            .expect("handling the hsmd_init message");
        Ok(h.into_root_handler())
    }

    fn handler_with_approver(
        &self,
        approver: Arc<dyn Approve>,
    ) -> Result<handler::RootHandler, Error> {
        let mut h = handler::HandlerBuilder::new(
            self.network,
            0 as u64,
            self.services.clone(),
            self.secret,
        )
        .approver(approver)
        .build()
        .map_err(|e| crate::signer::Error::Other(anyhow!("Could not create handler: {:?}", e)))?
        .0;
        h.handle(Signer::initreq())
            .expect("handling the hsmd_init message");
        Ok(h.into_root_handler())
    }

    /// Create an `init` request that we can pass to the signer.
    fn initreq() -> vls_protocol::msgs::Message {
        vls_protocol::msgs::Message::HsmdInit(vls_protocol::msgs::HsmdInit {
            key_version: vls_protocol::model::Bip32KeyVersion {
                pubkey_version: 0,
                privkey_version: 0,
            },
            chain_params: lightning_signer::bitcoin::BlockHash::all_zeros(),
            encryption_key: None,
            dev_privkey: None,
            dev_bip32_seed: None,
            dev_channel_secrets: None,
            dev_channel_secrets_shaseed: None,
            hsm_wire_min_version: 3,
            hsm_wire_max_version: 4,
        })
    }

    fn bolt12initreq() -> vls_protocol::msgs::Message {
        vls_protocol::msgs::Message::DeriveSecret(vls_protocol::msgs::DeriveSecret {
            info: Octets("bolt12-invoice-base".as_bytes().to_vec()),
        })
    }

    fn scbinitreq() -> vls_protocol::msgs::Message {
        vls_protocol::msgs::Message::DeriveSecret(vls_protocol::msgs::DeriveSecret {
            info: Octets("scb secret".as_bytes().to_vec()),
        })
    }

    fn commandoinitreq() -> vls_protocol::msgs::Message {
        vls_protocol::msgs::Message::DeriveSecret(vls_protocol::msgs::DeriveSecret {
            info: Octets("commando".as_bytes().to_vec()),
        })
    }

    fn initmsg(handler: &mut vls_protocol_signer::handler::InitHandler) -> Result<Vec<u8>, Error> {
        Ok(handler.handle(Signer::initreq()).unwrap().1.as_vec())
    }

    /// Filter out any request that is not signed, such that the
    /// remainder is the minimal set to reconcile state changes
    /// against.
    ///
    /// Returns an error if a signature failed verification or if the
    /// rune verification failed.
    fn check_request_auth(
        &self,
        requests: Vec<crate::pb::PendingRequest>,
    ) -> Vec<Result<crate::pb::PendingRequest, anyhow::Error>> {
        // Filter out requests lacking a required field. They are unverifiable anyway.
        use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};
        // Todo: partition results to provide more detailed errors.
        requests
            .into_iter()
            .filter(|r| !r.pubkey.is_empty() && !r.signature.is_empty() && !r.rune.is_empty())
            .map(|r| {
                let pk = UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, &r.pubkey);
                let mut data = r.request.clone();

                // If we have a timestamp associated we must add it to
                // the payload being checked. Same thing happens on
                // the client too.
                if r.timestamp != 0 {
                    data.put_u64(r.timestamp);
                }

                pk.verify(&data, &r.signature)
                    .map_err(|e| anyhow!("signature verification failed: {}", e))?;

                self.verify_rune(r.clone())
                    .map(|_| r)
                    .map_err(|e| anyhow!("rune verification failed: {}", e))
            })
            .collect()
    }

    /// Verifies that the public key of the request and the signers rune version
    /// match the corresponding restrictions of the rune.
    fn verify_rune(&self, request: crate::pb::PendingRequest) -> Result<(), anyhow::Error> {
        let rune64 = general_purpose::URL_SAFE.encode(request.rune);
        let rune = Rune::from_base64(&rune64)?;

        // A valid gl-rune must contain a pubkey field as this  is bound to the
        // signer. Against the rules of runes we do not accept a rune that has
        // no restriction on a public key.
        if !rune.to_string().contains("pubkey=") {
            return Err(anyhow!("rune is missing pubkey field"));
        }

        let mut checks: HashMap<String, String> = HashMap::new();
        checks.insert("pubkey".to_string(), hex::encode(request.pubkey));

        // Runes only check on the version if the unique id field is set. The id
        // and the version are part of the empty field.
        if let Some(device_id) = rune.get_id() {
            checks.insert("".to_string(), format!("{}-{}", device_id, RUNE_VERSION));
        }

        // Check that the request points to `cln.Node`.
        let mut parts = request.uri.split('/');
        parts.next();
        match parts.next() {
            Some(service) => {
                if service != "cln.Node" && service != "greenlight.Node" {
                    debug!("request from unknown service {}.", service);
                    return Err(anyhow!("service {} is not valid", service));
                }
            }
            None => {
                debug!("could not extract service from the uri while verifying rune.");
                return Err(anyhow!("can not extract service from uri"));
            }
        };

        // Extract the method from the request uri: eg. `/cln.Node/CreateInvoice`
        // becomes `createinvoice`.
        let method = match parts.next() {
            Some(m) => m.to_lowercase(),
            None => {
                debug!("could not extract method from uri while verifying rune.");
                return Err(anyhow!("can not extract uri form request"));
            }
        };
        checks.insert("method".to_string(), method.to_string());

        match self
            .master_rune
            .check_with_reason(&rune64, MapChecker { map: checks })
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Given the URI of the running node, connect to it and stream
    /// requests from it. The requests are then verified and processed
    /// using the `Hsmd`.
    pub async fn run_once(&self, node_uri: Uri) -> Result<(), Error> {
        debug!("Connecting to node at {}", node_uri);
        let c = Endpoint::from_shared(node_uri.to_string())?
            .tls_config(self.tls.inner.clone().domain_name("localhost"))?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let mut client = NodeClient::new(c);

        let mut stream = client
            .stream_hsm_requests(Request::new(Empty::default()))
            .await?
            .into_inner();

        debug!("Starting to stream signer requests");
        loop {
            let req = match stream
                .message()
                .await
                .map_err(|e| Error::NodeDisconnect(e))?
            {
                Some(r) => r,
                None => {
                    warn!("Signer request stream ended, the node shouldn't do this.");
                    return Ok(());
                }
            };
            let hex_req = hex::encode(&req.raw);
            let signer_state = req.signer_state.clone();
            trace!("Received request {}", hex_req);

            match self.process_request(req).await {
                Ok(response) => {
                    trace!("Sending response {}", hex::encode(&response.raw));
                    client
                        .respond_hsm_request(response)
                        .await
                        .map_err(|e| Error::NodeDisconnect(e))?;
                }
                Err(e) => {
                    warn!(
                        "Ignoring error {} for request {} with state {:?}",
                        e, hex_req, signer_state,
                    )
                }
            };
        }
    }

    fn authenticate_request(
        &self,
        msg: &vls_protocol::msgs::Message,
        reqs: &Vec<model::Request>,
    ) -> Result<(), Error> {
        log::trace!(
            "Resolving signature request against pending grpc commands: {:?}",
            reqs
        );

        // Quick path out of here: we can't find a resolution for a
        // request, then abort!
        Resolver::try_resolve(msg, &reqs)?;

        Ok(())
    }

    async fn process_request(&self, req: HsmRequest) -> Result<HsmResponse, Error> {
        let diff: crate::persist::State = req.signer_state.clone().into();

        let prestate = {
            debug!("Updating local signer state with state from node");
            let mut state = self.state.lock().unwrap();
            state.merge(&diff).unwrap();
            trace!("Processing request {}", hex::encode(&req.raw));
            state.clone()
        };

        // The first two bytes represent the message type. Check that
        // it is not a `sign-message` request (type 23).
        if let &[h, l, ..] = req.raw.as_slice() {
            let typ = ((h as u16) << 8) | (l as u16);
            if typ == 23 {
                warn!("Refusing to process sign-message request");
                return Err(Error::Other(anyhow!(
                    "Cannot process sign-message requests from node."
                )));
            }
        }

        let ctxrequests: Vec<model::Request> = self
            .check_request_auth(req.requests.clone())
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|r| decode_request(r))
            .filter_map(|r| match r {
                Ok(r) => Some(r),
                Err(e) => {
                    log::error!("Unable to decode request in context: {}", e);
                    None
                }
            })
            .collect::<Vec<model::Request>>();

        let msg = vls_protocol::msgs::from_vec(req.raw.clone()).map_err(|e| Error::Protocol(e))?;
        log::debug!("Handling message {:?}", msg);
        log::trace!("Signer state {}", serde_json::to_string(&prestate).unwrap());

        if let Err(e) = self.authenticate_request(&msg, &ctxrequests) {
            report::Reporter::report(crate::pb::scheduler::SignerRejection {
                msg: e.to_string(),
                request: Some(req.clone()),
                git_version: GITHASH.to_string(),
            })
            .await;
            #[cfg(not(feature = "permissive"))]
            return Err(Error::Resolver(req.raw, ctxrequests));
        };

        // If present, add the close_to_addr to the allowlist
        for parsed_request in ctxrequests.iter() {
            match parsed_request {
                model::Request::GlConfig(gl_config) => {
                    let pubkey = PublicKey::from_slice(&self.id);
                    match pubkey {
                        Ok(p) => {
                            let _ = self
                                .services
                                .persister
                                .update_node_allowlist(&p, vec![gl_config.close_to_addr.clone()]);
                        }
                        Err(e) => debug!("Could not parse public key {:?}: {:?}", self.id, e),
                    }
                }
                _ => {}
            }
        }

        use auth::Authorizer;
        let auth = auth::GreenlightAuthorizer {};
        let approvals = auth.authorize(&ctxrequests).map_err(|e| Error::Auth(e))?;
        debug!("Current approvals: {:?}", approvals);

        let approver = Arc::new(MemoApprover::new(approver::ReportingApprover::new(
            #[cfg(feature = "permissive")]
            vls_protocol_signer::approver::PositiveApprover(),
            #[cfg(not(feature = "permissive"))]
            vls_protocol_signer::approver::NegativeApprover(),
        )));
        approver.approve(approvals);
        let root_handler = self.handler_with_approver(approver)?;

        log::trace!("Updating state from context");
        update_state_from_context(&ctxrequests, &root_handler)
            .expect("Updating state from context requests");
        log::trace!("State updated");

        // Match over root and client handler.
        let response = match req.context {
            Some(HsmRequestContext { dbid: 0, .. }) | None => {
                // This is the main daemon talking to us.
                root_handler.handle(msg)
            }
            Some(c) => {
                let pk: [u8; 33] = c.node_id.try_into().unwrap();
                let pk = vls_protocol::model::PubKey(pk);
                root_handler
                    .for_new_client(1 as u64, pk, c.dbid)
                    .handle(msg)
            }
        }
        .map_err(|e| Error::Other(anyhow!("processing request: {e:?}")))?;

        let signer_state: Vec<crate::pb::SignerStateEntry> = {
            debug!("Serializing state changes to report to node");
            let state = self.state.lock().unwrap();
            state.clone().into()
        };

        Ok(HsmResponse {
            raw: response.0.as_vec(),
            request_id: req.request_id,
            signer_state,
        })
    }

    pub fn node_id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn get_init(&self) -> Vec<u8> {
        self.init.clone()
    }

    /// Retrieve the messages we know `lightningd` will ask when
    /// starting. Since we can't be attached during startup, or on
    /// background sync runs, we need to stash them at the `scheduler`
    /// so we can start without a signer present.
    pub fn get_startup_messages(&self) -> Vec<StartupMessage> {
        let mut init_handler = self.init_handler().unwrap();

        let init = StartupMessage {
            request: Signer::initreq().inner().as_vec(),
            response: init_handler.handle(Signer::initreq()).unwrap().1.as_vec(),
        };

        let requests = vec![
            // v22.11 introduced an addiotiona startup message, the
            // bolt12 key generation
            Signer::bolt12initreq(),
            // SCB needs a secret derived too
            Signer::scbinitreq(),
            // Commando needs a secret for its runes
            Signer::commandoinitreq(),
        ];

        let serialized: Vec<Vec<u8>> = requests.iter().map(|m| m.inner().as_vec()).collect();
        let responses: Vec<Vec<u8>> = requests
            .into_iter()
            .map(|r| self.handler().unwrap().handle(r).unwrap().0.as_vec())
            .collect();

        let mut msgs: Vec<StartupMessage> = serialized
            .into_iter()
            .zip(responses)
            .map(|r| {
                log::debug!("Storing canned request-response: {:?} -> {:?}", r.0, r.1);

                StartupMessage {
                    request: r.0,
                    response: r.1,
                }
            })
            .collect();

        msgs.insert(0, init);

        msgs
    }

    pub fn bip32_ext_key(&self) -> Vec<u8> {
        use vls_protocol::{msgs, msgs::Message};
        let initmsg = msgs::from_vec(self.init.clone()).expect("unparseable init message");

        match initmsg {
            Message::HsmdInit2Reply(m) => m.bip32.0.to_vec(),
            Message::HsmdInitReplyV4(m) => m.bip32.0.to_vec(),
            Message::HsmdInitReplyV2(m) => m.bip32.0.to_vec(),
            m => panic!("Unknown initmsg {:?}, cannot extract bip32 key", m),
        }
    }

    pub fn legacy_bip32_ext_key(&self) -> Vec<u8> {
        let mut handler = self.init_handler().expect("retrieving the handler");
        let req = vls_protocol::msgs::Message::HsmdInit(vls_protocol::msgs::HsmdInit {
            key_version: vls_protocol::model::Bip32KeyVersion {
                pubkey_version: 0,
                privkey_version: 0,
            },
            chain_params: lightning_signer::bitcoin::BlockHash::all_zeros(),
            encryption_key: None,
            dev_privkey: None,
            dev_bip32_seed: None,
            dev_channel_secrets: None,
            dev_channel_secrets_shaseed: None,
            hsm_wire_min_version: 1,
            hsm_wire_max_version: 2,
        });

        let initmsg = handler
            .handle(req)
            .expect("handling legacy init message")
            .1
            .as_vec();
        initmsg[35..].to_vec()
    }

    /// Connect to the scheduler given by the environment variable
    /// `GL_SCHEDULER_GRPC_URI` (of the default URI) and wait for the
    /// node to be scheduled. Once scheduled, connect to the node
    /// directly and start streaming and processing requests.
    pub async fn run_forever(&self, shutdown: mpsc::Receiver<()>) -> Result<(), anyhow::Error> {
        let scheduler_uri = crate::utils::scheduler_uri();
        Self::run_forever_with_uri(&self, shutdown, scheduler_uri).await
    }

    /// Create and, if necessary, upgrade the scheduler
    async fn init_scheduler(
        &self,
        scheduler_uri: String,
    ) -> Result<SchedulerClient<tonic::transport::channel::Channel>> {
        debug!("Connecting to scheduler at {scheduler_uri}");

        let channel = Endpoint::from_shared(scheduler_uri)?
            .tls_config(self.tls.inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();
        let mut scheduler = SchedulerClient::new(channel);

        // Upgrade node if necessary.
        // If it fails due to connection error, sleep and retry. Re-throw all other errors.
        loop {
            #[allow(deprecated)]
            let maybe_upgrade_res = scheduler
                .maybe_upgrade(UpgradeRequest {
                    initmsg: self.init.clone(),
                    signer_version: self.version().to_owned(),
                    startupmsgs: self
                        .get_startup_messages()
                        .into_iter()
                        .map(|s| s.into())
                        .collect(),
                })
                .await;

            if let Err(err_status) = maybe_upgrade_res {
                match err_status.code() {
                    Code::Unavailable => {
                        debug!("Cannot connect to scheduler, sleeping and retrying");
                        sleep(Duration::from_secs(3)).await;
                        continue;
                    }
                    _ => {
                        return Err(Error::Upgrade(err_status))?;
                    }
                }
            }

            break;
        }
        Ok(scheduler)
    }

    /// The core signer loop. Connects to the signer and keeps the connection alive.
    ///
    /// Used as inner loop for `run_forever_with_uri`.
    async fn run_forever_inner(
        &self,
        mut scheduler: SchedulerClient<tonic::transport::channel::Channel>,
    ) -> Result<(), anyhow::Error> {
        loop {
            debug!("Calling scheduler.get_node_info");
            let node_info_res = scheduler
                .get_node_info(NodeInfoRequest {
                    node_id: self.id.clone(),

                    // This `wait` parameter means that the scheduler will
                    // not automatically schedule the node. Rather we are
                    // telling the scheduler we want to be told as soon as
                    // the node is being scheduled so we can re-attach to
                    // that.
                    wait: true,
                })
                .await;

            let node_info = match node_info_res.map(|v| v.into_inner()) {
                Ok(v) => {
                    debug!("Got node_info from scheduler: {:?}", v);
                    v
                }
                Err(e) => {
                    trace!("Got an error from the scheduler: {e}. Sleeping before retrying");
                    sleep(Duration::from_millis(1000)).await;
                    continue;
                }
            };

            if node_info.grpc_uri.is_empty() {
                trace!("Got an empty GRPC URI, node is not scheduled, sleeping and retrying");
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            if let Err(e) = self
                .run_once(Uri::from_maybe_shared(node_info.grpc_uri)?)
                .await
            {
                warn!("Error running against node: {e}");
            }
        }
    }

    pub async fn run_forever_with_uri(
        &self,
        mut shutdown: mpsc::Receiver<()>,
        scheduler_uri: String,
    ) -> Result<(), anyhow::Error> {
        let scheduler = self.init_scheduler(scheduler_uri).await?;
        tokio::select! {
            run_forever_inner_res = self.run_forever_inner(scheduler) => {
                error!("Inner signer loop exited unexpectedly: {run_forever_inner_res:?}");
            },
            _ = shutdown.recv() => debug!("Received the signal to exit the signer loop")
        };

        info!("Exiting the signer loop");
        Ok(())
    }

    // TODO See comment on `sign_device_key`.
    pub fn sign_challenge(&self, challenge: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
        if challenge.len() != 32 {
            return Err(anyhow!("challenge is not 32 bytes long"));
        }
        let (sig, _) = self.sign_message(challenge)?;
        Ok(sig)
    }

    /// Signs the devices public key. This signature is meant to be appended
    /// to any payload signed by the device so that the signer can verify that
    /// it knows the device.
    ///
    // TODO Use a lower-level API that bypasses the LN message
    // prefix. This will allow us to re-expose the sign-message API to
    // node users.
    pub fn sign_device_key(&self, key: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        if key.len() != 65 {
            return Err(anyhow!("key is not 65 bytes long"));
        }
        let (sig, _) = self.sign_message(key.to_vec())?;
        Ok(sig)
    }

    /// Signs a message with the hsmd client. Returns a tuple with the signature
    /// and the unmodified recovery id.
    pub fn sign_message(&self, msg: Vec<u8>) -> Result<(Vec<u8>, u8), anyhow::Error> {
        if msg.len() > u16::MAX as usize {
            return Err(anyhow!("Message exceeds max len of {}", u16::MAX));
        }

        let len = u16::to_be_bytes(msg.len() as u16);
        if len.len() != 2 {
            return Err(anyhow!(
                "Message to be signed has unexpected len {}",
                len.len()
            ));
        }

        let req = vls_protocol::msgs::SignMessage {
            message: Octets(msg),
        };
        let response = self
            .handler()?
            .handle(vls_protocol::msgs::Message::SignMessage(req))
            .unwrap();

        // The signature returned by VLS consists of the signature with the
        // recovery id appended.
        let complete_sig = response.0.as_vec();
        let sig = complete_sig[2..66].to_vec();
        let recovery_id = complete_sig[66];
        Ok((sig, recovery_id))
    }

    /// Signs an invoice.
    pub fn sign_invoice(&self, msg: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
        if msg.len() > u16::MAX as usize {
            return Err(anyhow!("Message exceeds max len of {}", u16::MAX));
        }

        let sig = self
            .handler()?
            .handle(vls_protocol::msgs::from_vec(msg.clone())?)
            .map_err(|_| anyhow!("Sign invoice failed"))?;
        Ok(sig.0.as_vec()[2..67].to_vec())
    }

    /// Create a Node stub from this instance of the signer, configured to
    /// talk to the corresponding node.
    pub async fn node<Creds>(&self, creds: Creds) -> Result<Client, anyhow::Error>
    where
        Creds: TlsConfigProvider + RuneProvider,
    {
        node::Node::new(self.node_id(), creds)?
            .schedule()
            .await
    }

    pub fn version(&self) -> &'static str {
        VERSION
    }

    /// Creates a base64 string called a rune which is used to authorize
    /// commands on the node and to issue signatures from the signer. Each new
    /// rune must contain a `pubkey` field that equals the public key that is
    /// used to sign-off signature requests. Nobody can remove restrictions from
    /// a rune.
    ///
    /// If a `rune` is supplied the restrictions are added to this rune. This
    /// way one can invoke a rune that only allows for a subset of commands.
    ///
    /// `restrictions` is a vector of restrictions where each restriction itself
    /// is a vector of one ore more alternatives.
    ///
    /// - =: passes if equal ie. identical. e.g. method=withdraw
    /// - /: not equals, e.g. method/withdraw
    /// - ^: starts with, e.g. id^024b9a1fa8e006f1e3937f
    /// - $: ends with, e.g. id$381df1cc449605.
    /// - ~: contains, e.g. id~006f1e3937f65f66c40.
    /// - <: is a decimal integer, and is less than. e.g. time<1656759180
    /// - \>: is a decimal integer, and is greater than. e.g. time>1656759180
    /// - {: preceeds in alphabetical order (or matches but is shorter), e.g. id{02ff.
    /// - }: follows in alphabetical order (or matches but is longer), e.g. id}02ff.
    /// - #: a comment, ignored, e.g. dumb example#.
    /// - !: only passes if the name does not exist. e.g. something!.  Every other operator except # fails if name does not exist!
    ///
    /// # Examples
    /// This creates a fresh rune that is only restricted to a pubkey:
    ///
    /// `create_rune(None, vec![vec!["pubkey=000000"]])`
    ///
    /// "wjEjvKoFJToMLBv4QVbJpSbMoGFlnYVxs8yy40PIBgs9MC1nbDAmcHVia2V5PTAwMDAwMA"
    ///
    /// This adds a restriction to the rune, in this case a restriction that only
    /// allows to call methods that start with "list" or "get", basically a
    /// read-only rune:
    ///
    /// `create_rune("wjEjvKoFJToMLBv4QVbJpSbMoGFlnYVxs8yy40PIBgs9MC1nbDAmcHVia2V5PTAwMDAwMA", vec![vec!["method^list", "method^get"]])`
    ///
    pub fn create_rune(
        &self,
        rune: Option<&str>,
        restrictions: Vec<Vec<&str>>,
    ) -> Result<String, anyhow::Error> {
        if let Some(rune) = rune {
            // We got a rune, add restrictions to it!
            let mut rune: Rune = Rune::from_base64(rune)?;
            restrictions.into_iter().for_each(|alts| {
                let joined = alts.join("|");
                _ = rune.add_restriction(joined.as_str())
            });
            return Ok(rune.to_base64());
        } else {
            let res: Vec<Restriction> = restrictions
                .into_iter()
                .map(|alts| {
                    let joined = alts.join("|");
                    Restriction::try_from(joined.as_str())
                })
                .collect::<Result<Vec<Restriction>, RuneError>>()?;

            // New rune, we need a unique id.
            // FIXME: Add a counter that persists in SSS.
            let unique_id = 0;

            // Check that at least one restriction has a `pubkey` field set.
            let has_pubkey_field = res.iter().any(|r: &Restriction| {
                r.alternatives
                    .iter()
                    .any(|a| a.get_field() == *"pubkey" && a.get_condition() == Condition::Equal)
            });
            if !has_pubkey_field {
                return Err(anyhow!("Missing a restriction on the pubkey"));
            }

            let rune = Rune::new(
                self.master_rune.authcode(),
                res,
                Some(unique_id.to_string()),
                Some(RUNE_VERSION.to_string()),
            )?;
            Ok(rune.to_base64())
        }
    }
}

/// Look through the context requests and update the state
/// accordingly. This is useful to modify allowlists and invoice lists
/// extracted from the authenticated requests.
fn update_state_from_context(
    requests: &Vec<model::Request>,
    handler: &handler::RootHandler,
) -> Result<(), Error> {
    log::debug!("Updating state from {} context request", requests.len());
    let node = handler.node();

    requests
        .iter()
        .for_each(|r| update_state_from_request(r, &node).unwrap());
    Ok(())
}

fn update_state_from_request(
    request: &model::Request,
    node: &lightning_signer::node::Node,
) -> Result<(), Error> {
    use lightning_signer::invoice::Invoice;
    use std::str::FromStr;
    match request {
        model::Request::SendPay(model::cln::SendpayRequest {
            bolt11: Some(inv), ..
        }) => {
            let invoice = Invoice::from_str(inv).unwrap();
            log::debug!(
                "Adding invoice {:?} as side-effect of this sendpay {:?}",
                invoice,
                request
            );
            node.add_invoice(invoice).unwrap();
        }
        _ => {}
    }

    Ok(())
}

/// Used to decode incoming requests into their corresponding protobuf
/// message. This is used by the E2E verification to verify that
/// incoming requests match up with the user intent. User intent here
/// refers to there being a user-signed command that matches the
/// effects we are being asked to sign off.
fn decode_request(r: crate::pb::PendingRequest) -> Result<model::Request, anyhow::Error> {
    // Strip the compressions flag (1 byte) and the length prefix (4
    // bytes big endian) and we're left with just the payload. See
    // https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md#requests
    // for technical details.
    //
    // Notice that we assume that the compression flag is off.
    assert_eq!(r.request[0], 0u8);
    let payload = &r.request[5..];

    crate::signer::model::cln::decode_request(&r.uri, payload)
        .or_else(|_| crate::signer::model::greenlight::decode_request(&r.uri, payload))
}

/// A `(request, response)`-tuple passed to the scheduler to allow
/// signerless startups.
pub struct StartupMessage {
    request: Vec<u8>,
    response: Vec<u8>,
}

impl From<StartupMessage> for crate::pb::scheduler::StartupMessage {
    fn from(r: StartupMessage) -> Self {
        Self {
            request: r.request,
            response: r.response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials;
    use crate::pb;

    /// We should not sign messages that we get from the node, since
    /// we're using the sign_message RPC message to create TLS
    /// certificate attestations. We can remove this limitation once
    /// we switch over to a salted hash when signing those
    /// attestations.
    #[tokio::test]
    async fn test_sign_message_rejection() {
        let signer = Signer::new(vec![0 as u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();

        let msg = hex::decode("0017000B48656c6c6f20776f726c64").unwrap();
        assert!(signer
            .process_request(HsmRequest {
                request_id: 0,
                context: None,
                raw: msg,
                signer_state: vec![],
                requests: Vec::new(),
            },)
            .await
            .is_err());
    }

    /// We should reject a signing request with an empty message.
    #[tokio::test]
    async fn test_empty_message() {
        let signer = Signer::new(vec![0 as u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();

        assert_eq!(
            signer
                .process_request(HsmRequest {
                    request_id: 0,
                    context: None,
                    raw: vec![],
                    signer_state: vec![],
                    requests: Vec::new(),
                },)
                .await
                .unwrap_err()
                .to_string(),
            *"protocol error: ShortRead"
        )
    }

    #[test]
    fn test_sign_message_max_size() {
        let signer = Signer::new(vec![0u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();

        // We test if we reject a message that is too long.
        let msg = [0u8; u16::MAX as usize + 1];
        assert_eq!(
            signer.sign_message(msg.to_vec()).unwrap_err().to_string(),
            format!("Message exceeds max len of {}", u16::MAX)
        );
    }

    /// Some users were relying on the broken behavior of
    /// `bip32_ext_key`. We need to ensure that the behavior remains
    /// stable for the time being until we have ensured no users of it
    /// remain.
    #[test]
    fn test_legacy_bip32_key() {
        let signer = Signer::new(vec![0u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();

        let bip32 = signer.legacy_bip32_ext_key();
        let expected: Vec<u8> = vec![
            4, 136, 178, 30, 2, 175, 86, 45, 251, 0, 0, 0, 0, 119, 232, 160, 181, 114, 16, 182, 23,
            70, 246, 204, 254, 122, 233, 131, 242, 174, 134, 193, 120, 104, 70, 176, 202, 168, 243,
            142, 127, 239, 60, 157, 212, 3, 162, 85, 18, 86, 240, 176, 177, 84, 94, 241, 92, 64,
            175, 69, 165, 146, 101, 79, 180, 195, 27, 117, 8, 66, 110, 100, 36, 246, 115, 48, 193,
            189, 3, 247, 195, 58, 236, 143, 230, 177, 91, 217, 66, 67, 19, 204, 22, 96, 65, 140,
            86, 195, 109, 50, 228, 94, 193, 173, 103, 252, 196, 192, 173, 243, 223,
        ];

        assert_eq!(bip32, expected);
    }

    /// We want to ensure that we can not generate a rune that is unrestricted
    /// on the public key "pubkey=<public-key-of-devices-tls-cert>".
    #[test]
    fn test_rune_expects_pubkey() {
        let signer = Signer::new(vec![0u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();

        let alt = "pubkey=112233";
        let wrong_alt = "pubkey^112233";

        // Check empty restrictions.
        assert!(signer.create_rune(None, vec![]).is_err());

        // Check wrong restriction.
        assert!(signer.create_rune(None, vec![vec![wrong_alt]]).is_err());

        // Check good restriction.
        assert!(signer.create_rune(None, vec![vec![alt]]).is_ok());

        // Check at least one alternative in one restriction.
        assert!(signer
            .create_rune(None, vec![vec![wrong_alt], vec![wrong_alt, alt]])
            .is_ok());
    }

    #[test]
    fn test_rune_expansion() {
        let signer = Signer::new(vec![0u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();
        let rune = "wjEjvKoFJToMLBv4QVbJpSbMoGFlnYVxs8yy40PIBgs9MC1nbDAmcHVia2V5PTAwMDAwMA==";

        let new_rune = signer
            .create_rune(Some(rune), vec![vec!["method^get"]])
            .unwrap();
        let rs = Rune::from_base64(&new_rune).unwrap().to_string();
        assert!(rs.contains("0-gl0&pubkey=000000&method^get"))
    }

    #[test]
    fn test_rune_checks_method() {
        let signer = Signer::new(vec![0u8; 32], Network::Bitcoin, credentials::Nobody::default()).unwrap();

        // This is just a placeholder public key, could also be a different one;
        let pubkey = signer.node_id();
        let pubkey_rest = format!("pubkey={}", hex::encode(&pubkey));

        // Create a rune that allows methods that start with `create`.
        let rune = signer
            .create_rune(None, vec![vec![&pubkey_rest], vec!["method^create"]])
            .unwrap();

        // A method/uri that starts with `create` is ok.
        let uri = "/cln.Node/CreateInvoice".to_string();
        let r = pb::PendingRequest {
            request: vec![],
            uri,
            signature: vec![],
            pubkey: pubkey.clone(),
            timestamp: 0,
            rune: general_purpose::URL_SAFE.decode(&rune).unwrap(),
        };
        assert!(signer.verify_rune(r).is_ok());

        // method/uri `Pay` is not allowed by the rune.
        let uri = "/cln.Node/Pay".to_string();
        let r = pb::PendingRequest {
            request: vec![],
            uri,
            signature: vec![],
            pubkey: pubkey.clone(),
            timestamp: 0,
            rune: general_purpose::URL_SAFE.decode(&rune).unwrap(),
        };
        assert!(signer.verify_rune(r).is_err());

        // The `greenlight.Node` service also needs to be accepted for
        // setting the `close_to_addr`.
        let uri = "/greenlight.Node/CreateInvoice".to_string();
        let r = pb::PendingRequest {
            request: vec![],
            uri,
            signature: vec![],
            pubkey: pubkey.clone(),
            timestamp: 0,
            rune: general_purpose::URL_SAFE.decode(&rune).unwrap(),
        };
        assert!(signer.verify_rune(r).is_ok());

        // A service other than `cln.Node` and `greenlight.Node` is
        // not allowed.
        let uri = "/wrong.Service/CreateInvoice".to_string();
        let r = pb::PendingRequest {
            request: vec![],
            uri,
            signature: vec![],
            pubkey: pubkey.clone(),
            timestamp: 0,
            rune: general_purpose::URL_SAFE.decode(&rune).unwrap(),
        };
        assert!(signer.verify_rune(r).is_err());
    }
}
