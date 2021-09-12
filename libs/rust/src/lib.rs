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
    use service::AuthService;
    use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Uri};
    use tower::ServiceBuilder;

    type Client = NodeClient<service::AuthSvc>;

    /// A builder is a synchronous mechanism to configure a NodeStub
    /// before actually connecting to it asynchronously.
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
            let chan: tonic::transport::Channel = Channel::builder(node_uri)
                .tls_config(self.client_tls)?
                .connect()
                .await?;

            let chan = ServiceBuilder::new()
                .layer_fn(service::AuthSvc::new)
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

        pub struct AuthService {
            hmac_key: Vec<u8>,
            inner: Channel,
        }
        pub struct AuthSvc {
            inner: Channel,
        }

        impl AuthSvc {
            pub fn new(inner: Channel) -> Self {
                AuthSvc { inner }
            }
        }

        impl Service<Request<BoxBody>> for AuthSvc {
            type Response = Response<Body>;
            type Error = Box<dyn std::error::Error + Send + Sync>;
            #[allow(clippy::type_complexity)]
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                self.inner.poll_ready(cx).map_err(Into::into)
            }

            fn call(&mut self, req: Request<BoxBody>) -> Self::Future {
                // This is necessary because tonic internally uses `tower::buffer::Buffer`.
                // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
                // for details on why this is necessary
                let clone = self.inner.clone();
                let mut inner = std::mem::replace(&mut self.inner, clone);
                Box::pin(async move {
                    use tonic::codegen::Body;
		    let mut request = req;

                    let (parts, mut body) = request.into_parts();
                    //let mut data = body.data().await.unwrap().unwrap();
                    //trace!("Got request {:?}", &data);

                    //futures_util::pin_mut!(body);
                    //let body = tonic::body::BoxBody::new(body);

                    let request = Request::from_parts(parts, body);

                    // Do extra async work here...
                    let response = inner.call(request).await?;
                    trace!("Response!");
                    Ok(response)
                })
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
