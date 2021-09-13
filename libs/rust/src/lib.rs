pub use libhsmd_sys::Hsmd;

extern crate anyhow;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

pub mod pb;
pub mod signer;

pub enum Network {
    BITCOIN,
    TESTNET,
    REGTEST,
}

impl Into<&'static str> for Network {
    fn into(self: Network) -> &'static str {
        match self {
            Network::BITCOIN => "bitcoin",
            Network::TESTNET => "testnet",
            Network::REGTEST => "regtest",
        }
    }
}

/// Tools to interact with a node running on greenlight.
pub mod node {
    use super::{tls, Network};
    use crate::pb::node_client::NodeClient;
    use crate::pb::{scheduler_client::SchedulerClient, ScheduleRequest};
    use anyhow::{anyhow, Result};
    use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Uri};
    use tower::ServiceBuilder;

    type Client = NodeClient<service::AuthService>;

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
                Some(k) => service::AuthLayer::new(k),
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

            let chan = ServiceBuilder::new()
                .layer(layer)
                .service(chan);

            Ok(NodeClient::new(chan))
        }

        pub async fn schedule(self) -> Result<Client> {
            let scheduler_uri = std::env::var("GL_SCHEDULER_GRPC_URI")
                .unwrap_or("https://scheduler.gl.blckstrm.com:2601".to_string());

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

    mod service {

        use http::{Request, Response};
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use tonic::body::BoxBody;
        use tonic::transport::Body;
        use tonic::transport::Channel;
        use tower::{Layer, Service};

        pub struct AuthLayer {
            hmac_key: Vec<u8>,
        }

        impl AuthLayer {
            pub fn new(hmac_key: Vec<u8>) -> Self {
                AuthLayer { hmac_key: hmac_key }
            }
        }

        impl Layer<Channel> for AuthLayer {
            type Service = AuthService;

            fn layer(&self, inner: Channel) -> Self::Service {
                AuthService {
                    hmac_key: self.hmac_key.clone(),
                    inner,
                }
            }
        }

        pub struct AuthService {
            hmac_key: Vec<u8>,
            inner: Channel,
        }
        impl Service<Request<BoxBody>> for AuthService {
            type Response = Response<Body>;
            type Error = Box<dyn std::error::Error + Send + Sync>;
            #[allow(clippy::type_complexity)]
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                self.inner.poll_ready(cx).map_err(Into::into)
            }
            fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
                // This is necessary because tonic internally uses `tower::buffer::Buffer`.
                // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
                // for details on why this is necessary
                let clone = self.inner.clone();
                let mut inner = std::mem::replace(&mut self.inner, clone);
                let key = self.hmac_key.clone();
                Box::pin(async move {
                    use tonic::codegen::Body;
                    let (parts, mut body) = request.into_parts();

                    let data = dbg!(body.data().await.unwrap().unwrap());
                    trace!("Got request {:?}, signing with {:?}", data, key);

                    let body = crate::node::stasher::StashBody::new(data).into();
                    let request = Request::from_parts(parts, body);

                    // Do extra async work here...
                    let response = inner.call(request).await?;
                    trace!("Response!");
                    Ok(response)
                })
            }
        }
    }
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
}
pub mod tls {
    use tonic::transport::{Certificate, ClientTlsConfig, Identity};
    lazy_static! {
        pub static ref CA: Certificate = Certificate::from_pem(include_str!("../../../tls/ca.pem"));
        pub static ref NOBODY: Identity = Identity::from_pem(
            include_str!("../../../tls/users-nobody.pem"),
            include_str!("../../../tls/users-nobody-key.pem")
        );
        pub static ref NOBODY_CONFIG: ClientTlsConfig = ClientTlsConfig::new()
            .domain_name("localhost")
            .ca_certificate(Certificate::from_pem(include_str!("../../../tls/ca.pem")))
            .identity(Identity::from_pem(
                include_str!("../../../tls/users-nobody.pem"),
                include_str!("../../../tls/users-nobody-key.pem")
            ));
    }
}
