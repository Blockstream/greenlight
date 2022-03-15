use anyhow::{Context, Result};
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

lazy_static! {
    static ref CA_RAW: Vec<u8> = include_str!("../tls/ca.pem").as_bytes().to_vec();
    static ref NOBODY_CRT: Vec<u8> = include_str!("../tls/users-nobody.pem").as_bytes().to_vec();
    static ref NOBODY_KEY: Vec<u8> = include_str!("../tls/users-nobody-key.pem")
        .as_bytes()
        .to_vec();
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

fn load_file_or_default(varname: &str, default: &Vec<u8>) -> Result<Vec<u8>> {
    match std::env::var(varname) {
        Ok(fname) => {
            debug!("Loading file {} for envvar {}", fname, varname);
            Ok(std::fs::read(fname.clone())
                .with_context(|| format!("could not read file {} for envvar {}", fname, varname))?)
        }
        Err(_) => Ok(default.clone()),
    }
}

impl TlsConfig {
    pub fn new() -> Result<Self> {
        // Allow overriding the defaults through the environment
        // variables, so we don't pollute the public interface with
        // stuff that is testing-related.
        let nobody_crt = load_file_or_default("GL_NOBODY_CRT", &NOBODY_CRT)?;
        let nobody_key = load_file_or_default("GL_NOBODY_KEY", &NOBODY_KEY)?;
        let ca_crt = load_file_or_default("GL_CA_CRT", &CA_RAW)?;

        let config = ClientTlsConfig::new()
            .domain_name("localhost")
            .ca_certificate(Certificate::from_pem(ca_crt.clone()))
            .identity(Identity::from_pem(nobody_crt, nobody_key));

        Ok(TlsConfig {
            inner: config,
            private_key: None,
            ca: ca_crt,
        })
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

    pub fn client_tls_config(&self) -> ClientTlsConfig {
        self.inner.clone()
    }
}
