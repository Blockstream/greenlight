use crate::pb::cln::node_client as cln_client;
use crate::pb::node_client::NodeClient;
use crate::pb::scheduler::{scheduler_client::SchedulerClient, ScheduleRequest};
use crate::serialize;
use crate::tls::TlsConfig;
use crate::utils;
use anyhow::{anyhow, Result};
use lightning_signer::bitcoin::Network;
use log::{debug, info, trace};
use tonic::transport::{Channel, Uri};
use tower::ServiceBuilder;

/// A client to the remotely running node on the greenlight
/// infrastructure. It is configured to authenticate itself with the
/// device mTLS keypair and will sign outgoing requests with the same
/// mTLS keypair.
pub type Client = NodeClient<service::AuthService>;

pub type GClient = GenericClient<service::AuthService>;

pub type ClnClient = cln_client::NodeClient<service::AuthService>;

pub trait GrpcClient {
    fn new_with_inner(inner: service::AuthService) -> Self;
}

/// A builder to set up and configure a `Node`. The builder is mainly
/// there to override the `TlsConfig` or the `rune` before
/// constructing a `Node`.
pub struct NodeBuilder {
    node_id: Vec<u8>,
    network: Network,
    tls: TlsConfig,
    rune: String,
}

impl NodeBuilder {
    /// Returns a new builder from a `tlsConfig` and a `rune`.
    pub fn from_parts(node_id: Vec<u8>, network: Network, tls: TlsConfig, rune: &str) -> Self {
        let rune = rune.to_string();
        Self {
            node_id,
            network,
            tls,
            rune,
        }
    }

    /// Returns a new builder from an `auth` blob.
    pub fn from_auth(node_id: Vec<u8>, network: Network, auth: &[u8]) -> Result<Self> {
        let blob = serialize::AuthBlob::deserialize(&auth[..])?;
        let tls = TlsConfig::new()?.identity(blob.cert, blob.key);
        let rune: String = blob.rune;
        Ok(Self {
            node_id,
            network,
            tls,
            rune,
        })
    }

    /// Sets a `TlsConfig` for the `Node`. Overrides a `TlsConfig` created from
    /// the auth blob using `with_auth`.
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = tls;
        self
    }

    /// Sets a `futhark::Rune` for the `Node`. Overrides a `futhark::Rune` that
    /// is extracted from the auth blob via `with_auth`.
    pub fn with_rune(mut self, rune: &str) -> Self {
        self.rune = rune.to_string();
        self
    }

    /// Use the auth blob to create the `TlsConfig` and the `futhark::Rune` for
    /// the `Node`. Will be overridden by `with_rune()` and `with_tls`.
    pub fn with_auth(mut self, auth: &[u8]) -> Result<Self> {
        let blob = serialize::AuthBlob::deserialize(&auth[..])?;
        self.tls = TlsConfig::new()?.identity(blob.cert, blob.key);
        self.rune = blob.rune;
        Ok(self)
    }

    /// Build the `Node` from set parameters.
    pub fn build(self) -> Node {
        Node {
            node_id: self.node_id,
            network: self.network,
            tls: self.tls,
            rune: self.rune,
        }
    }
}

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
    rune: String,
}

impl GrpcClient for Client {
    fn new_with_inner(inner: service::AuthService) -> Self {
        Client::new(inner)
    }
}

impl GrpcClient for GClient {
    fn new_with_inner(inner: service::AuthService) -> Self {
        GenericClient::new(inner)
    }
}

impl GrpcClient for ClnClient {
    fn new_with_inner(inner: service::AuthService) -> Self {
        ClnClient::new(inner)
    }
}

impl Node {
    pub fn new(node_id: Vec<u8>, network: Network, tls: TlsConfig) -> Node {
        Node {
            node_id,
            network,
            tls,
            rune: "".to_string(),
        }
    }

    /// Returns a new `NodeBuilder` from a `TlsConfig` and a rune.
    pub fn builder_from_parts(
        node_id: Vec<u8>,
        network: Network,
        tls: TlsConfig,
        rune: &str,
    ) -> NodeBuilder {
        NodeBuilder::from_parts(node_id, network, tls, rune)
    }

    /// Returns a new `NodeBuilder` from an `auth` blob.
    pub fn builder_from_auth(
        node_id: Vec<u8>,
        network: Network,
        auth: &[u8],
    ) -> Result<NodeBuilder> {
        NodeBuilder::from_auth(node_id, network, auth)
    }

    pub async fn connect<C>(&self, node_uri: String) -> Result<C>
    where
        C: GrpcClient,
    {
        let node_uri = Uri::from_maybe_shared(node_uri)?;
        info!("Connecting to node at {}", node_uri);

        // If this is not yet a node-domain address we need to also
        // accept "localhost" as domain name from the certificate.
        let host = node_uri.host().unwrap();
        let tls = if host.starts_with("gl") {
            trace!(
                "Using real hostname {}, expecting the node to have a matching certificate",
                host
            );
            self.tls.clone()
        } else {
            trace!(
                "Overriding hostname, since this is not a gl node domain: {}",
                host
            );
            let mut tls = self.tls.clone();
            tls.inner = tls.inner.domain_name("localhost");
            tls
        };

        let layer = match tls.private_key {
            Some(k) => service::AuthLayer::new(k, self.rune.clone())?,
            None => {
                return Err(anyhow!(
                    "Cannot connect a node::Client without first configuring its identity"
                ))
            }
        };

        let chan = tonic::transport::Endpoint::from_shared(node_uri.to_string())?
            .tls_config(tls.inner)?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();
        let chan = ServiceBuilder::new().layer(layer).service(chan);

        Ok(C::new_with_inner(chan))
    }

    pub async fn schedule_with_uri<C>(self, scheduler_uri: String) -> Result<C>
    where
        C: GrpcClient,
    {
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

    pub async fn schedule<C>(self) -> Result<C>
    where
        C: GrpcClient,
    {
        let uri = utils::scheduler_uri();
        self.schedule_with_uri(uri).await
    }
}

mod generic;
mod service;
pub use generic::GenericClient;

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
