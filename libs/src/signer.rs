/// The core signer system. It runs in a dedicated thread or using the
/// caller thread, streaming incoming requests, verifying them,
/// signing if ok, and then shipping the response to the node.
use crate::pb::{node_client::NodeClient, Empty, HsmRequest, HsmRequestContext, HsmResponse};
use crate::pb::{scheduler_client::SchedulerClient, NodeInfoRequest, UpgradeRequest};
use crate::tls::TlsConfig;
use crate::{node, node::Client};
use anyhow::{anyhow, Context, Result};
use bitcoin::Network;
use bytes::{Buf, Bytes};
use libhsmd_sys::{Capabilities, Capability, Hsmd};
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, Uri};
use tonic::Request;

static MAIN_CAPABILITIES: Capabilities =
    Capability::MASTER | Capability::SIGN_GOSSIP | Capability::ECDH;

#[derive(Clone)]
pub struct Signer {
    hsmd: Hsmd,
    tls: TlsConfig,
    id: Vec<u8>,

    /// Cached version of the init response
    init: Vec<u8>,

    network: Network,
}

impl Signer {
    pub fn new(secret: Vec<u8>, network: Network, tls: TlsConfig) -> Result<Signer> {
        let hsmd = Hsmd::new(secret, &network.to_string());
        let init = hsmd.init()?;
        let id = init[2..35].to_vec();

        trace!("Initialized signer for node_id={}", hex::encode(&id));
        Ok(Signer {
            hsmd,
            tls,
            id,
            init,
            network,
        })
    }

    /// Given the URI of the running node, connect to it and stream
    /// requests from it. The requests are then verified and processed
    /// using the `Hsmd`.
    pub async fn run_once(&self, node_uri: Uri) -> Result<()> {
        debug!("Connecting to node at {}", node_uri);
        let c = Channel::builder(node_uri)
            .tls_config(self.tls.inner.clone())?
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
        let mut b = Bytes::from(req.raw.clone());

        if b.get_u16() == 23 {
            warn!("Refusing to process sign-message request");
            return Err(anyhow!("Cannot process sign-message requests from node."));
        }

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

    pub fn node_id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn get_init(&self) -> Vec<u8> {
        self.init.clone()
    }

    pub fn bip32_ext_key(&self) -> Vec<u8> {
        self.init[35..].to_vec()
    }

    /// Connect to the scheduler given by the environment variable
    /// `GL_SCHEDULER_GRPC_URI` (of the default URI) and wait for the
    /// node to be scheduled. Once scheduled, connect to the node
    /// directly and start streaming and processing requests.
    pub async fn run_forever(&self) -> Result<()> {
        let scheduler_uri = std::env::var("GL_SCHEDULER_GRPC_URI")
            .unwrap_or_else(|_| "https://scheduler.gl.blckstrm.com:2601".to_string());

        debug!(
            "Contacting scheduler at {} to get the node address",
            scheduler_uri
        );

        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(self.tls.inner.clone())?
            .connect()
            .await?;
        let mut scheduler = SchedulerClient::new(channel);

        scheduler
            .maybe_upgrade(UpgradeRequest {
                initmsg: self.init.clone(),
                signer_version: self.version().to_owned(),
            })
            .await
            .context("Error asking scheduler to upgrade")?;

        loop {
            debug!("Calling scheduler.get_node_info");
            let node_info = match scheduler
                .get_node_info(NodeInfoRequest {
                    node_id: self.id.clone(),
                    wait: true,
                })
                .await
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

            match self
                .run_once(Uri::from_maybe_shared(node_info.grpc_uri)?)
                .await
            {
                Ok(()) => continue,
                Err(e) => warn!("Error running against node: {}", e),
            }
        }
    }

    pub fn sign_challenge(&self, challenge: Vec<u8>) -> Result<Vec<u8>> {
        if challenge.len() != 32 {
            return Err(anyhow!("challenge is not 32 bytes long"));
        }
        let client = self.hsmd.client(MAIN_CAPABILITIES);

        let mut req = vec![0_u8, 23, 00, 32];
        req.extend(challenge);

        let response = client.handle(req)?;
        if response[1] != 123 {
            return Err(anyhow!(
                "Expected response type to be 123, got {}",
                response[1]
            ));
        } else if response.len() != 2 + 64 + 1 {
            return Err(anyhow!(
                "Malformed response to sign_challenge, unexpected length {}",
                response.len()
            ));
        }
        Ok(response[2..66].to_vec())
    }

    /// Create a Node stub from this instance of the signer, configured to
    /// talk to the corresponding node.
    pub async fn node(&self) -> Result<Client> {
        node::Node::new(self.node_id(), self.network, self.tls.clone())
            .schedule()
            .await
    }

    pub fn version(&self) -> &'static str {
	libhsmd_sys::Hsmd::version()
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
        assert_eq!(signer.init.len(), 177);
        assert_eq!(
            signer.init,
            vec![
                0_u8, 111, 2, 5, 142, 139, 108, 42, 211, 99, 236, 89, 170, 19, 100, 41, 37, 109,
                116, 81, 100, 194, 189, 200, 127, 152, 240, 166, 134, 144, 236, 44, 92, 155, 11, 4,
                136, 178, 30, 2, 175, 86, 45, 251, 0, 0, 0, 0, 119, 232, 160, 181, 114, 16, 182,
                23, 70, 246, 204, 254, 122, 233, 131, 242, 174, 134, 193, 120, 104, 70, 176, 202,
                168, 243, 142, 127, 239, 60, 157, 212, 3, 162, 85, 18, 86, 240, 176, 177, 84, 94,
                241, 92, 64, 175, 69, 165, 146, 101, 79, 180, 195, 27, 117, 8, 66, 110, 100, 36,
                246, 115, 48, 193, 189, 247, 195, 58, 236, 143, 230, 177, 91, 217, 66, 67, 19, 204,
                22, 96, 65, 140, 86, 195, 109, 50, 228, 94, 193, 173, 103, 252, 196, 192, 173, 243,
                223, 127, 5, 118, 244, 107, 113, 69, 246, 232, 45, 169, 141, 60, 45, 217, 83, 168,
                194, 28, 130, 206, 68, 183, 248, 111, 74, 187, 5, 78, 201, 233, 42
            ]
        );
    }

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
                raw: msg
            })
            .await
            .is_err())
    }
}
