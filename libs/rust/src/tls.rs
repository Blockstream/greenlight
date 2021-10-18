use tonic::transport::{Certificate, ClientTlsConfig, Identity};

lazy_static! {
    pub static ref CA_RAW: Vec<u8> = include_str!("../tls/ca.pem").as_bytes().to_vec();
    pub static ref CA: Certificate = Certificate::from_pem(include_str!("../tls/ca.pem"));
    pub static ref NOBODY: Identity = Identity::from_pem(
        include_str!("../tls/users-nobody.pem"),
        include_str!("../tls/users-nobody-key.pem")
    );
    pub static ref NOBODY_CONFIG: ClientTlsConfig = ClientTlsConfig::new()
        .domain_name("localhost")
        .ca_certificate(Certificate::from_pem(include_str!("../tls/ca.pem")))
        .identity(Identity::from_pem(
            include_str!("../tls/users-nobody.pem"),
            include_str!("../tls/users-nobody-key.pem")
        ));
}

/// In order to allow the clients to talk to the
/// [`crate::scheduler::Scheduler`] a default certificate and private
/// key is included in this crate. The only service endpoints that can
/// be contacted with this `NOBODY` identity are
/// [`Scheduler.register`] and [`Scheduler.recover`], as these are the
/// endpoints that are used to prove ownership of a node, and
/// returning valid certificates if that proof succeeds.
#[derive(Clone)]
pub struct TlsConfig {
    pub(crate) inner: ClientTlsConfig,

    /// Copy of the private key in the TLS identity. Stored here in
    /// order to be able to use it in the `AuthLayer`.
    pub(crate) private_key: Option<Vec<u8>>,

    pub ca: Vec<u8>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        TlsConfig {
            inner: NOBODY_CONFIG.clone(),
            private_key: None,
	    ca: CA_RAW.clone(),
        }
    }
}

impl TlsConfig {
    /// This function is used to upgrade the anonymous `NOBODY`
    /// configuration to a fully authenticated configuration.
    ///
    /// Only non-`NOBODY` configurations are able to talk to their
    /// nodes. If the `TlsConfig` is not upgraded, nodes will reply
    /// with handshake failures, and abort the connection attempt.
    pub fn identity(self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Self {
        TlsConfig {
            inner: self.inner.identity(Identity::from_pem(cert_pem, &key_pem)),
            private_key: Some(key_pem),
            ..self
        }
    }

    /// This function is mostly used to allow running integration
    /// tests against a local mock of the service. It should not be
    /// used in production, since the preconfigured CA ensures that
    /// only the greenlight production servers can complete a valid
    /// handshake.
    pub fn ca_certificate(self, ca: Vec<u8>) -> Self {
        TlsConfig {
            inner: self.inner.ca_certificate(Certificate::from_pem(&ca)),
	    ca: ca.clone(),
            ..self
        }
    }
}
