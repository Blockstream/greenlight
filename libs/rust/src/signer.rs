/// The core signer system. It runs in a dedicated thread or using the
/// caller thread, streaming incoming requests, verifying them,
/// signing if ok, and then shipping the response to the node.
use crate::pb::{node_client::NodeClient, Empty, HsmRequest, HsmRequestContext, HsmResponse};
use crate::pb::{scheduler_client::SchedulerClient, NodeInfoRequest};
use crate::Network;
use anyhow::{Context, Result};
use libhsmd_sys::Hsmd;
use libhsmd_sys::{Capabilities, Capability};
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, ClientTlsConfig, Identity, Uri};
use tonic::Request;

static MAIN_CAPABILITIES: Capabilities =
    Capability::MASTER | Capability::SIGN_GOSSIP | Capability::ECDH;

#[derive(Clone)]
pub struct Signer {
    hsmd: Hsmd,
    tls: ClientTlsConfig,
    id: Vec<u8>,
}

impl Signer {
    pub fn new(secret: Vec<u8>, network: Network, client_tls: ClientTlsConfig) -> Result<Signer> {
        let hsmd = Hsmd::new(secret, network.into());
        let init = hsmd.init()?;
        let id = init[2..35].to_vec();
        trace!("Initialized signer for node_id={}", hex::encode(&id));
        Ok(Signer {
            hsmd,
            tls: client_tls,
            id,
        })
    }

    pub fn with_identity(self, identity: Identity) -> Signer {
        Signer {
            tls: self.tls.identity(identity),
            ..self
        }
    }

    /// Given the URI of the running node, connect to it and stream
    /// requests from it. The requests are then verified and processed
    /// using the `Hsmd`.
    pub async fn run_once(&self, node_uri: Uri) -> Result<()> {
        debug!("Connecting to node at {}", node_uri);
        let c = Channel::builder(node_uri)
            .tls_config(self.tls.clone())?
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
            trace!("Received request {}", hex::encode(&req.raw));
            let response = self.process_request(req).await?;
            trace!("Sending response {}", hex::encode(&response.raw));
            client
                .respond_hsm_request(response)
                .await
                .context("sending response to hsm request")?;
        }
    }

    async fn process_request(&self, req: HsmRequest) -> Result<HsmResponse> {
        let client = match req.context {
            Some(HsmRequestContext {
                dbid: 0,
                capabilities,
                ..
            }) => self.hsmd.client(capabilities),
            Some(c) => self
                .hsmd
                .client_with_context(c.capabilities, c.dbid, c.node_id),
            None => self.hsmd.client(MAIN_CAPABILITIES),
        };

        Ok(HsmResponse {
            raw: client.handle(req.raw)?,
            request_id: req.request_id,
        })
    }

    /// Connect to the scheduler given by the environment variable
    /// `GL_SCHEDULER_GRPC_URI` (of the default URI) and wait for the
    /// node to be scheduled. Once scheduled, connect to the node
    /// directly and start streaming and processing requests.
    pub async fn run_forever(&self) -> Result<()> {
        let scheduler_uri = std::env::var("GL_SCHEDULER_GRPC_URI")
            .unwrap_or("https://scheduler.gl.blckstrm.com:2601".to_string());

        debug!(
            "Contacting scheduler at {} to get the node address",
            scheduler_uri
        );

        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(self.tls.clone())?
            .connect()
            .await?;
        let mut scheduler = SchedulerClient::new(channel);
        loop {
            trace!("Calling scheduler.get_node_info");
            let node_info = match scheduler
                .get_node_info(NodeInfoRequest {
                    node_id: self.id.clone(),
                    wait: true,
                })
                .await.map(|v| v.into_inner())
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

            if node_info.grpc_uri == "" {
                trace!("Got an empty GRPC URI, node is not scheduled, sleeping and retrying");
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

            match self
                .run_once(Uri::from_maybe_shared(node_info.grpc_uri)?)
                .await
            {
                Ok(()) => continue,
                Err(e) => warn!("Error running against node: {}", e),
            }
        }
    }
}

#[derive(Debug)]
struct InitInfo {
    node_id: Vec<u8>,
    bip32_ext_key: Vec<u8>,
}
