use crate::pb::node_client::NodeClient;
use crate::pb::{scheduler_client::SchedulerClient, ScheduleRequest};
use crate::tls::TlsConfig;
use crate::utils;
use anyhow::{anyhow, Result};
use lightning_signer::bitcoin::Network;
use tonic::transport::{Channel, Uri};
use tower::ServiceBuilder;

/// A client to the remotely running node on the greenlight
/// infrastructure. It is configured to authenticate itself with the
/// device mTLS keypair and will sign outgoing requests with the same
/// mTLS keypair.
pub type Client = NodeClient<service::AuthService>;

pub type GClient = GenericClient<service::AuthService>;

pub trait GrpcClient {
    fn new_with_inner(inner: service::AuthService) -> Self;
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

impl Node {
    pub fn new(node_id: Vec<u8>, network: Network, tls: TlsConfig) -> Node {
        Node {
            node_id,
            network,
            tls,
        }
    }

    pub async fn connect<C>(self, node_uri: String) -> Result<C>
    where
        C: GrpcClient,
    {
        let node_uri = Uri::from_maybe_shared(node_uri)?;
        info!("Connecting to node at {}", node_uri);

        let hostname = node_uri
            .clone()
            .host()
            .ok_or_else(|| anyhow!("No hostname in node uri {}", node_uri.to_string()))?
            .to_string();

        debug!("Setting tls host_name {}", &hostname);

        let layer = match self.tls.private_key {
            Some(k) => service::AuthLayer::new(k)?,
            None => {
                return Err(anyhow!(
                    "Cannot connect a node::Client without first configuring its identity"
                ))
            }
        };

        // We strip off the scheme as we are only interested in the hostname and
        // the port for domain name validation.
        let mut host = String::from(node_uri.host().unwrap_or(""));
        match node_uri.port() {
            Some(p) => {
                host.push_str(":");
                host.push_str(p.as_str());
            }
            None => {}
        }

        // Tls does not like things other than a domain name in the server name
        // record. As we do a canary rollout, node_id can be either an ip or a
        // domain name. To accomplish for this choice we need to parse the
        // node_id. If the node_uri is a valid domain name we assume that the
        // proxy should be used and set the server name tls extension.
        if domain_parser::parse(host.as_str()) {
            // node_uri is a valid domain name, set tls extension server name.
            debug!("Setting tls extension server name {}", host.to_string());
            // Connect the channel and add auth middleware.
            let chan = Channel::builder(node_uri)
                .tls_config(self.tls.inner.domain_name(host))?
                .connect()
                .await?;

            let chan = ServiceBuilder::new().layer(layer).service(chan);

            Ok(C::new_with_inner(chan))
        } else {
            // node_uri is not a valid domain name, skip tls extension server
            // name.
            debug!(
                "Skipping tls extension, got node_uri {}",
                node_uri.to_string()
            );
            let chan = Channel::builder(node_uri)
                .tls_config(self.tls.inner)?
                .connect()
                .await?;

            let chan = ServiceBuilder::new().layer(layer).service(chan);

            Ok(C::new_with_inner(chan))
        }
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

mod domain_parser {
    use addr::{parser::DomainName, psl::List};

    // parse returns true if the target is a valid domain name, false otherwise.
    pub fn parse(target: &str) -> bool {
        List.parse_domain_name(target).is_ok()
    }
}

#[cfg(test)]
mod tests {
    const MAX_LABEL_LEN: usize = 63;

    #[test]
    fn domain_name_too_long() {
        let _tmp = [b'1'; MAX_LABEL_LEN + 1];
        let label = std::str::from_utf8(&_tmp).unwrap();

        let valid = crate::node::domain_parser::parse(label);
        assert!(!valid)
    }

    #[test]
    fn domain_name_has_port() {
        let input = "blckstrm.com:1111";
        let valid = crate::node::domain_parser::parse(input);
        assert!(!valid)
    }

    #[test]
    fn domain_name_is_ip() {
        let input = "127.0.0.1";
        let valid = crate::node::domain_parser::parse(input);
        assert!(!valid)
    }

    #[test]
    fn domain_name() {
        let input = "mynode.gl.blckstrm.com";
        let valid = crate::node::domain_parser::parse(input);
        assert!(valid)
    }
}
