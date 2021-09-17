use super::{tls, Network};
use crate::pb::node_client::NodeClient;
use crate::pb::{scheduler_client::SchedulerClient, ScheduleRequest};
use anyhow::{anyhow, Result};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Uri};
use tower::ServiceBuilder;
use crate::utils;

pub type Client = NodeClient<service::AuthService>;

/// A builder is a synchronous mechanism to configure a NodeStub
/// before actually connecting to it asynchronously.
#[allow(dead_code)]
pub struct Builder {
    node_id: Vec<u8>,
    network: Network,
    client_tls: ClientTlsConfig,
    identity: Identity,

    // A copy of the private key allowing us to sign outgoing requests.
    private_key: Option<Vec<u8>>,
}

impl Builder {
    pub fn new(node_id: Vec<u8>, network: Network, client_tls: ClientTlsConfig) -> Builder {
        Builder {
            node_id,
            network,
            client_tls,
            identity: tls::NOBODY.clone(),
            private_key: None,
        }
    }

    pub fn ca_certificate(self, cert: Certificate) -> Self {
        Builder {
            client_tls: self.client_tls.ca_certificate(cert),
            ..self
        }
    }

    pub fn identity(self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Self {
        Builder {
            identity: Identity::from_pem(cert_pem, key_pem.clone()),
            private_key: Some(key_pem),
            ..self
        }
    }

    pub async fn connect(self, node_uri: String) -> Result<Client> {
        let node_uri = Uri::from_maybe_shared(node_uri)?;

        let layer = match self.private_key {
            Some(k) => service::AuthLayer::new(k)?,
            None => {
                return Err(anyhow!(
                    "Cannot connect a node::Client without first configuring its identity"
                ))
            }
        };

        let chan: tonic::transport::Channel = Channel::builder(node_uri)
            .tls_config(self.client_tls)?
            .connect()
            .await?;

        let chan = ServiceBuilder::new().layer(layer).service(chan);

        Ok(NodeClient::new(chan))
    }

    pub async fn schedule(self) -> Result<Client> {
        let scheduler_uri = utils::scheduler_uri();
        debug!(
            "Contacting scheduler at {} to get the node address",
            scheduler_uri
        );

        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(self.client_tls.clone())?
            .connect()
            .await?;
        let mut scheduler = SchedulerClient::new(channel);

        let node_info = scheduler
            .schedule(ScheduleRequest {
                node_id: self.node_id.clone(),
            })
            .await
            .map(|v| v.into_inner())?;

        debug!("Node scheduled at {}", node_info.grpc_uri);

        self.connect(node_info.grpc_uri).await
    }
}

mod service;

pub mod stasher {
    use bytes::Bytes;
    use http::HeaderMap;
    use http_body::Body;
    use pin_project::pin_project;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    use tonic::body::BoxBody;
    use tonic::Status;

    #[pin_project]
    #[derive(Debug)]
    pub(crate) struct StashBody {
        value: Option<Bytes>,
    }

    impl StashBody {
        pub(crate) fn new(val: Bytes) -> Self {
            Self { value: Some(val) }
        }
    }

    impl Body for StashBody {
        type Data = Bytes;
        type Error = Status;

        fn is_end_stream(&self) -> bool {
            self.value.is_none()
        }

        fn poll_data(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
            Poll::Ready(self.project().value.take().map(|v| Ok(v)))
        }

        fn poll_trailers(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<Option<HeaderMap>, Status>> {
            Poll::Ready(Ok(None))
        }
    }

    impl From<StashBody> for BoxBody {
        fn from(v: StashBody) -> BoxBody {
            BoxBody::new(v)
        }
    }
}
