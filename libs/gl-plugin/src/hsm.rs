//! Service used to talk to the `hsmd` that is passing us the signer
//! requests.

use crate::config::NodeInfo;
use crate::pb::{hsm_server::Hsm, Empty, HsmRequest, HsmResponse, NodeConfig};
use crate::stager;
use anyhow::{Context, Result};
use futures::TryFutureExt;
use log::{debug, info, trace, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// The StagingHsmServer is used by the plugin to receive incoming requests
/// from the hsmproxy and stages the requests for clients of the Node
/// interface to stream and reply to.
#[derive(Clone)]
pub struct StagingHsmServer {
    stage: Arc<stager::Stage>,
    hsmd_sock_path: PathBuf,
    node_info: NodeInfo,
    node_config: NodeConfig,
}

impl StagingHsmServer {
    pub fn new(
        hsmd_sock_path: PathBuf,
        stage: Arc<stager::Stage>,
        node_info: NodeInfo,
        node_config: NodeConfig,
    ) -> StagingHsmServer {
        StagingHsmServer {
            stage,
            hsmd_sock_path,
            node_info,
            node_config,
        }
    }

    /// We have some canned responses from the signer, this gives us access.
    fn find_canned_response(&self, msg: &Vec<u8>) -> Option<Vec<u8>> {
        self.node_config
            .startupmsgs
            .iter()
            .find(|m| &m.request == msg)
            .map(|m| m.response.clone())
    }
}

#[tonic::async_trait]
impl Hsm for StagingHsmServer {
    async fn request(&self, request: Request<HsmRequest>) -> Result<Response<HsmResponse>, Status> {
        let req = request.into_inner();
        debug!("Received request from hsmproxy: {:?}", req);

        // Start by looking in the canned responses and return it if it is known
        if let Some(response) = self.find_canned_response(&req.raw) {
            debug!(
                "Returning canned response={:?} for request={:?}",
                response, req.raw
            );
            return Ok(Response::new(HsmResponse {
                request_id: req.request_id,
                raw: response,
                signer_state: Vec::new(),
            }));
        } else if req.get_type() == 11 {
            debug!("Returning stashed init msg: {:?}", self.node_info.initmsg);
            return Ok(Response::new(HsmResponse {
                request_id: req.request_id,
                raw: self.node_info.initmsg.clone(),
                signer_state: Vec::new(), // the signerproxy doesn't care about state
            }));
        } else if req.get_type() == 33 {
            debug!("Returning stashed dev-memleak response");
            return Ok(Response::new(HsmResponse {
                request_id: req.request_id,
                raw: vec![0, 133, 0],
                signer_state: Vec::new(), // the signerproxy doesn't care about state
            }));
        }

        let mut chan = match self.stage.send(req).await {
            Err(e) => {
                return Err(Status::unknown(format!(
                    "Error while queing request from node: {:?}",
                    e
                )))
            }
            Ok(c) => c,
        };

        let res = match chan.recv().await {
            None => {
                return Err(Status::unknown(format!(
                    "Channel closed while waiting for response",
                )))
            }
            Some(r) => r,
        };

        Ok(Response::new(res))
    }

    async fn ping(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        trace!("Got a ping");
        Ok(Response::new(Empty::default()))
    }
}

impl StagingHsmServer {
    pub async fn run(self) -> Result<()> {
        let mut path = std::path::PathBuf::new();
        path.push(std::env::current_dir().unwrap());
        path.push(&self.hsmd_sock_path);
        info!(
            "Configuring hsmd interface to listen on {}",
            path.to_str().unwrap()
        );
        std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap())?;

        if path.exists() {
            warn!(
                "Socket path {} already exists, deleting",
                path.to_string_lossy()
            );
            std::fs::remove_file(&path).context("removing stale hsmd socket")?;
        }
        let incoming = {
            let uds = tokio::net::UnixListener::bind(path)?;

            async_stream::stream! {
                loop {
            yield  uds.accept().map_ok(|(st, _)| crate::unix::UnixStream(st)).await;
                }
            }
        };

        info!("HSM server interface starting.");
        tonic::transport::Server::builder()
            .add_service(crate::pb::hsm_server::HsmServer::new(self))
            .serve_with_incoming(incoming)
            .await
            .context("serving HsmServer interface")
    }
}
