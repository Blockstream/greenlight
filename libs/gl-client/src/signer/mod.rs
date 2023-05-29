use crate::pb::scheduler::{scheduler_client::SchedulerClient, NodeInfoRequest, UpgradeRequest};
/// The core signer system. It runs in a dedicated thread or using the
/// caller thread, streaming incoming requests, verifying them,
/// signing if ok, and then shipping the response to the node.
use crate::pb::{node_client::NodeClient, Empty, HsmRequest, HsmRequestContext, HsmResponse};
use crate::tls::TlsConfig;
use crate::{node, node::Client};
use anyhow::{anyhow, Context, Result};
use bytes::{Buf, BufMut, Bytes};
use lightning_signer::bitcoin::Network;
use lightning_signer::node::NodeServices;
use log::{debug, info, trace, warn};
use std::convert::TryInto;
use std::sync::Arc;
use std::sync::Mutex;
use serde_bolt::Octets;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, Uri};
use tonic::Request;
use vls_protocol_signer::approver::{Approval, Approve, MemoApprover, PositiveApprover};
use vls_protocol_signer::handler;
use vls_protocol_signer::handler::Handler;

mod auth;
pub mod model;

const VERSION: &str = "v23.05";

#[derive(Clone)]
pub struct Signer {
    secret: [u8; 32],
    services: NodeServices,
    tls: TlsConfig,
    id: Vec<u8>,

    /// Cached version of the init response
    init: Vec<u8>,

    network: Network,
    state: Arc<Mutex<crate::persist::State>>,
}

impl Signer {
    pub fn new(secret: Vec<u8>, network: Network, tls: TlsConfig) -> Result<Signer> {
        use lightning_signer::policy::{
            filter::PolicyFilter, simple_validator::SimpleValidatorFactory,
        };
        use lightning_signer::signer::ClockStartingTimeFactory;
        use lightning_signer::util::clock::StandardClock;

        info!("Initializing signer for {VERSION} (VLS)");
        let mut sec: [u8; 32] = [0; 32];
        sec.copy_from_slice(&secret[0..32]);

        // The persister takes care of persisting metadata across
        // restarts
        let persister = Arc::new(crate::persist::MemoryPersister::new());
        let mut policy = lightning_signer::policy::simple_validator::make_simple_policy(network);

        #[cfg(feature = "permissive")]
        {
            policy.filter = PolicyFilter::new_permissive();
        }
        #[cfg(not(feature = "permissive"))]
        {
            policy.filter = PolicyFilter::default();
        }

        let validator_factory = Arc::new(SimpleValidatorFactory::new_with_policy(policy));
        let starting_time_factory = ClockStartingTimeFactory::new();
        let clock = Arc::new(StandardClock());

        let services = NodeServices {
            validator_factory,
            starting_time_factory,
            persister: persister.clone(),
            clock,
        };

        let handler =
            handler::RootHandlerBuilder::new(network, 0 as u64, services.clone(), sec).build();

        #[allow(deprecated)]
        let init = Signer::initmsg(&handler.0)?;
        let id = init[2..35].to_vec();

        trace!("Initialized signer for node_id={}", hex::encode(&id));
        Ok(Signer {
            secret: sec,
            services,
            tls,
            id,
            init,
            network,
            state: persister.state(),
        })
    }

    fn handler(&self) -> handler::RootHandler {
        handler::RootHandlerBuilder::new(self.network, 0 as u64, self.services.clone(), self.secret)
            .build()
            .0
    }

    fn handler_with_approver(&self, approver: Arc<dyn Approve>) -> handler::RootHandler {
        handler::RootHandlerBuilder::new(self.network, 0 as u64, self.services.clone(), self.secret)
            .approver(approver)
            .build()
            .0
    }

    /// Create an `init` request that we can pass to the signer.
    fn initreq() -> vls_protocol::msgs::Message {
        vls_protocol::msgs::Message::HsmdInit(vls_protocol::msgs::HsmdInit {
            key_version: vls_protocol::model::Bip32KeyVersion {
                pubkey_version: 0,
                privkey_version: 0,
            },
            chain_params: vls_protocol::model::BlockId([0; 32]),
            encryption_key: None,
            dev_privkey: None,
            dev_bip32_seed: None,
            dev_channel_secrets: None,
            dev_channel_secrets_shaseed: None,
            hsm_wire_min_version: 1,
            hsm_wire_max_version: 2,
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

    fn initmsg(handler: &vls_protocol_signer::handler::RootHandler) -> Result<Vec<u8>> {
        Ok(handler.handle(Signer::initreq()).unwrap().0.as_vec())
    }

    /// Filter out any request that is not signed, such that the
    /// remainder is the minimal set to reconcile state changes
    /// against.
    ///
    /// Returns an error if a signature failed verification.
    fn check_request_auth(
        &self,
        requests: Vec<crate::pb::PendingRequest>,
    ) -> Vec<Result<crate::pb::PendingRequest>> {
        // Filter out requests lacking a required field. They are unverifiable anyway.
        use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};
        requests
            .into_iter()
            .filter(|r| r.pubkey.len() != 0 && r.signature.len() != 0)
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
                    .map(|_| r)
                    .map_err(|e| anyhow!("signature verification failed: {}", e))
            })
            .collect()
    }

    /// Given the URI of the running node, connect to it and stream
    /// requests from it. The requests are then verified and processed
    /// using the `Hsmd`.
    pub async fn run_once(&self, node_uri: Uri) -> Result<()> {
        debug!("Connecting to node at {}", node_uri);
        let c = Channel::builder(node_uri)
            .tls_config(self.tls.inner.clone().domain_name("localhost"))?
            .connect()
            .await?;

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
                .context("receiving the next request")?
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

            let ctxrequests: Vec<model::Request> = self
                .check_request_auth(req.requests.clone())
                .into_iter()
                .filter_map(|r| r.ok())
                .map(|r| decode_request(r))
                .collect::<Result<Vec<model::Request>>>()?;

            // TODO: Decode requests and reconcile them with the changes
            use auth::Authorizer;
            let auth = auth::DummyAuthorizer {};
            let approvals = auth.authorize(ctxrequests)?;
            // TODO: apply approval to approver / handler

            match self.process_request(req, approvals).await {
                Ok(response) => {
                    trace!("Sending response {}", hex::encode(&response.raw));
                    client
                        .respond_hsm_request(response)
                        .await
                        .context("sending response to hsm request")?;
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

    async fn process_request(&self, req: HsmRequest, approvals: Vec<Approval>) -> Result<HsmResponse> {
        let mut b = Bytes::from(req.raw.clone());
        let diff: crate::persist::State = req.signer_state.into();
        let prestate = {
            debug!("Updating local signer state with state from node");
            let mut state = self.state.lock().unwrap();
            trace!(
                "Applying diff between local and remote state: {:?}",
                state.diff(&diff)
            );

            state.merge(&diff).unwrap();
            trace!("Processing request {}", hex::encode(&req.raw),);
            state.clone()
        };

        if b.get_u16() == 23 {
            warn!("Refusing to process sign-message request");
            return Err(anyhow!("Cannot process sign-message requests from node."));
        }

        let msg = vls_protocol::msgs::from_vec(req.raw)?;

        log::trace!("Handling message {:?}", msg);

        let approver =
            Arc::new(MemoApprover::new(PositiveApprover()));
        approver.approve(approvals);
        let root_handler = self.handler_with_approver(approver);

        let handler: Box<dyn Handler> = match req.context {
            Some(HsmRequestContext { dbid: 0, .. }) | None => {
                // This is the main daemon talking to us.
                Box::new(root_handler)
            }
            Some(c) => {
                let pk: [u8; 33] = c.node_id.try_into().unwrap();
                let pk = vls_protocol::model::PubKey(pk);
                Box::new(root_handler.for_new_client(1 as u64, pk, c.dbid))
            }
        };

        let response = handler.handle(msg)
            .map_err(|e| anyhow!("processing request: {e:?}"))?;

        let signer_state: Vec<crate::pb::SignerStateEntry> = {
            debug!("Serializing state changes to report to node");
            let state = self.state.lock().unwrap();
            trace!(
                "Diff to state pre-request: {:?}",
                prestate.diff(&state).unwrap()
            );
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
        let requests = vec![
            // The classical init message; response will either be 111
            // or 113 depending on signer proto version.
            (11, Signer::initreq()),
            // v22.11 introduced an addiotiona startup message, the
            // bolt12 key generation
            (27, Signer::bolt12initreq()),
            // SCB needs a secret derived too
            (27, Signer::scbinitreq()),
	    // Commando needs a secret for its runes
            (27, Signer::commandoinitreq()),
        ];

        let serialized: Vec<Vec<u8>> = requests
            .iter()
            .map(|m| {
                let mut b = bytes::BytesMut::new();
                b.put_u16(m.0);
                b.put_slice(&serde_bolt::to_vec(&m.1).unwrap());

                b.to_vec()
            })
            .collect();
        let responses: Vec<Vec<u8>> = requests
            .into_iter()
            .map(|r| self.handler().handle(r.1).unwrap().0.as_vec())
            .collect();

        serialized
            .into_iter()
            .zip(responses)
            .map(|r| StartupMessage {
                request: r.0,
                response: r.1,
            })
            .collect()
    }

    pub fn bip32_ext_key(&self) -> Vec<u8> {
        self.init[35..].to_vec()
    }

    /// Connect to the scheduler given by the environment variable
    /// `GL_SCHEDULER_GRPC_URI` (of the default URI) and wait for the
    /// node to be scheduled. Once scheduled, connect to the node
    /// directly and start streaming and processing requests.
    pub async fn run_forever(&self, shutdown: mpsc::Receiver<()>) -> Result<()> {
        let scheduler_uri = crate::utils::scheduler_uri();
        Self::run_forever_with_uri(&self, shutdown, scheduler_uri).await
    }

    pub async fn run_forever_with_uri(
        &self,
        mut shutdown: mpsc::Receiver<()>,
        scheduler_uri: String,
    ) -> Result<()> {
        debug!(
            "Contacting scheduler at {} to get the node address",
            &scheduler_uri
        );

        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(self.tls.inner.clone())?
            .connect()
            .await?;
        let mut scheduler = SchedulerClient::new(channel);

        #[allow(deprecated)]
        scheduler
            .maybe_upgrade(UpgradeRequest {
                initmsg: self.init.clone(),
                signer_version: self.version().to_owned(),
                startupmsgs: self
                    .get_startup_messages()
                    .into_iter()
                    .map(|s| s.into())
                    .collect(),
            })
            .await
            .context("Error asking scheduler to upgrade")?;

        loop {
            debug!("Calling scheduler.get_node_info");
            let get_node = scheduler.get_node_info(NodeInfoRequest {
                node_id: self.id.clone(),

		// This `wait` parameter means that the scheduler will
		// not automatically schedule the node. Rather we are
		// telling the scheduler we want to be told as soon as
		// the node is being scheduled so we can re-attach to
		// that.
                wait: true,
            });
            tokio::select! {
                    info = get_node => {
                let node_info = match info
                                .map(|v| v.into_inner())
                            {
                                Ok(v) => {
                                    debug!("Got node_info from scheduler: {:?}", v);
                                    v
                                }
                                Err(e) => {
                                    trace!(
                                        "Got an error from the scheduler: {}. Sleeping before retrying",
                                        e
                                    );
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
                            .await {
                                warn!("Error running against node: {}", e);
                            }


                        },
                    _ = shutdown.recv() => {
                debug!("Received the signal to exit the signer loop");
                break;
            },
                };
        }
        info!("Exiting the signer loop");
        Ok(())
    }

    // TODO See comment on `sign_device_key`.
    pub fn sign_challenge(&self, challenge: Vec<u8>) -> Result<Vec<u8>> {
        if challenge.len() != 32 {
            return Err(anyhow!("challenge is not 32 bytes long"));
        }
        self.sign_message(challenge)
    }

    /// Signs the devices public key. This signature is meant to be appended
    /// to any payload signed by the device so that the signer can verify that
    /// it knows the device.
    ///
    // TODO Use a lower-level API that bypasses the LN message
    // prefix. This will allow us to re-expose the sign-message API to
    // node users.
    pub fn sign_device_key(&self, key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 65 {
            return Err(anyhow!("key is not 65 bytes long"));
        }
        self.sign_message(key.to_vec())
    }

    /// Signs a message with the hsmd client.
    fn sign_message(&self, msg: Vec<u8>) -> Result<Vec<u8>> {
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

        let req = vls_protocol::msgs::SignMessage { message: Octets(msg) };
        let response = self
            .handler()
            .handle(vls_protocol::msgs::Message::SignMessage(req))
            .unwrap();

        Ok(response.0.as_vec()[2..66].to_vec())
    }

    /// Signs an invoice.
    pub fn sign_invoice(&self, msg: Vec<u8>) -> Result<Vec<u8>> {
        if msg.len() > u16::MAX as usize {
            return Err(anyhow!("Message exceeds max len of {}", u16::MAX));
        }

        let sig = self
            .handler()
            .handle(vls_protocol::msgs::from_vec(msg.clone())?)
            .map_err(|_| anyhow!("Sign invoice failed"))?;
        Ok(sig.0.as_vec()[2..67].to_vec())
    }

    /// Create a Node stub from this instance of the signer, configured to
    /// talk to the corresponding node.
    pub async fn node(&self) -> Result<Client> {
        node::Node::new(self.node_id(), self.network, self.tls.clone())
            .schedule()
            .await
    }

    pub fn version(&self) -> &'static str {
        VERSION
    }
}

/// Used to decode incoming requests into their corresponding protobuf
/// message. This is used by the E2E verification to verify that
/// incoming requests match up with the user intent. User intent here
/// refers to there being a user-signed command that matches the
/// effects we are being asked to sign off.
fn decode_request(r: crate::pb::PendingRequest) -> Result<model::Request> {
    // Strip the compressions flag (1 byte) and the length prefix (4
    // bytes big endian) and we're left with just the payload. See
    // https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md#requests
    // for technical details.
    //
    // Notice that we assume that the compression flag is off.
    assert_eq!(r.request[0], 0u8);
    let payload = &r.request[5..];

    Ok(crate::signer::model::cln::decode_request(&r.uri, payload)
        .or_else(|_| crate::signer::model::greenlight::decode_request(&r.uri, payload))
        .unwrap())
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

    #[tokio::test]
    async fn test_init() {
        let signer = Signer::new(
            vec![0 as u8; 32],
            Network::Bitcoin,
            TlsConfig::new().unwrap(),
        )
        .unwrap();
        assert_eq!(signer.init.len(), 146);
    }

    /// We should not sign messages that we get from the node, since
    /// we're using the sign_message RPC message to create TLS
    /// certificate attestations. We can remove this limitation once
    /// we switch over to a salted hash when signing those
    /// attestations.
    #[tokio::test]
    async fn test_sign_message_rejection() {
        let signer = Signer::new(
            vec![0 as u8; 32],
            Network::Bitcoin,
            TlsConfig::new().unwrap(),
        )
        .unwrap();

        let msg = hex::decode("0017000B48656c6c6f20776f726c64").unwrap();
        assert!(signer
            .process_request(HsmRequest {
                            request_id: 0,
                            context: None,
                            raw: msg,
                            signer_state: vec![],
                            requests: Vec::new(),
                        },
                             vec![],
            )
            .await
            .is_err())
    }

    #[test]
    fn test_sign_message_max_size() {
        let signer =
            Signer::new(vec![0u8; 32], Network::Bitcoin, TlsConfig::new().unwrap()).unwrap();

        // We test if we reject a message that is too long.
        let msg = [0u8; u16::MAX as usize + 1];
        assert_eq!(
            signer.sign_message(msg.to_vec()).unwrap_err().to_string(),
            format!("Message exceeds max len of {}", u16::MAX)
        );
    }
}
