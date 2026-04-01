// SDK configuration for Greenlight node operations.
// Holds network selection and optional developer certificate.

use std::sync::Arc;
use crate::credentials::DeveloperCert;
use crate::Network;

#[derive(uniffi::Object, Clone)]
pub struct Config {
    pub(crate) network: gl_client::bitcoin::Network,
    pub(crate) developer_cert: Option<gl_client::credentials::Nobody>,
}

impl Config {
    /// Resolve the credentials to use for unauthenticated scheduler
    /// calls (register, recover). Uses the developer certificate if
    /// one was provided, otherwise falls back to the compiled-in default.
    pub(crate) fn nobody(&self) -> gl_client::credentials::Nobody {
        self.developer_cert
            .clone()
            .unwrap_or_else(gl_client::credentials::Nobody::new)
    }
}

#[uniffi::export]
impl Config {
    /// Create a Config with default settings: BITCOIN network, no developer certificate.
    #[uniffi::constructor()]
    pub fn new() -> Self {
        Self {
            network: gl_client::bitcoin::Network::Bitcoin,
            developer_cert: None,
        }
    }

    /// Return a new Config with the given developer certificate.
    /// Nodes registered through this config will be associated with the developer's account.
    pub fn with_developer_cert(&self, cert: &DeveloperCert) -> Arc<Config> {
        Arc::new(Config {
            developer_cert: Some(cert.inner.clone()),
            ..self.clone()
        })
    }

    /// Return a new Config with the given network.
    pub fn with_network(&self, network: Network) -> Arc<Config> {
        Arc::new(Config {
            network: network.into(),
            ..self.clone()
        })
    }
}
