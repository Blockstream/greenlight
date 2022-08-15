use crate::pb::node_client::NodeClient;
use crate::pb::{scheduler_client::SchedulerClient, ScheduleRequest};
use crate::tls::TlsConfig;
use crate::utils;
use anyhow::{anyhow, Result};
use bitcoin::Network;
use tonic::transport::{Channel, Uri};
use tower::ServiceBuilder;

/// A client to the remotely running node on the greenlight
/// infrastructure. It is configured to authenticate itself with the
/// device mTLS keypair and will sign outgoing requests with the same
/// mTLS keypair.
pub type Client = NodeClient<service::AuthService>;

/// A builder to configure a [`Client`] that can either connect to a
/// node directly, assuming you have the `grpc_uri` that the node is
/// listening on, or it can talk to the
/// [`crate::scheduler::Scheduler`] to schedule the node and configure
/// the [`Client`] accordingly.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Node {
    node_id: Vec<u8>,
    network: Network,
    tls: TlsConfig,
}

impl Node {
    pub fn new(node_id: Vec<u8>, network: Network, tls: TlsConfig) -> Node {
        Node {
            node_id,
            network,
            tls,
        }
    }

    pub async fn connect(self, node_uri: String) -> Result<Client> {
        let node_uri = Uri::from_maybe_shared(node_uri)?;
        info!("Connecting to node at {}", node_uri);

        let layer = match self.tls.private_key {
            Some(k) => service::AuthLayer::new(k)?,
            None => {
                return Err(anyhow!(
                    "Cannot connect a node::Client without first configuring its identity"
                ))
            }
        };

        let chan: tonic::transport::Channel = Channel::builder(node_uri)
            .tls_config(self.tls.inner)?
            .connect()
            .await?;

        let chan = ServiceBuilder::new().layer(layer).service(chan);

        Ok(NodeClient::new(chan))
    }

    pub async fn schedule_with_uri(self, scheduler_uri: String) -> Result<Client> {
        debug!(
            "Contacting scheduler at {} to get the node address",
            scheduler_uri
        );

        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(self.tls.inner.clone())?
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

    pub async fn schedule(self) -> Result<Client> {
        let uri = utils::scheduler_uri();
        self.schedule_with_uri(uri).await
    }
}

mod service;

mod stasher {
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
            Poll::Ready(self.project().value.take().map(Ok))
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
