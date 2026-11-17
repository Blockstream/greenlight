// SDK configuration for Greenlight node operations.
// Holds network selection, signer settings and optional developer certificate.

use crate::credentials::DeveloperCert;
use crate::{Error, Network, SignerConfig};
use std::sync::Arc;

#[derive(uniffi::Object, Clone)]
pub struct Config {
    pub(crate) network: gl_client::bitcoin::Network,
    pub(crate) developer_cert: Option<gl_client::credentials::Nobody>,
    pub(crate) signer_config: SignerConfig,
}

impl Config {
    pub(crate) fn create_signer<T: gl_client::credentials::TlsConfigProvider>(
        &self,
        seed: Vec<u8>,
        credentials: T,
    ) -> Result<gl_client::signer::Signer, Error> {
        gl_client::signer::Signer::new_with_config(
            seed,
            self.network,
            credentials,
            self.signer_config.to_client()?,
        )
        .map_err(|e| Error::other(format!("{e:#}")))
    }

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
            signer_config: SignerConfig::default(),
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

    /// Configure all signers started by NodeBuilder with this Config.
    pub fn with_signer_config(&self, signer_config: SignerConfig) -> Arc<Config> {
        Arc::new(Config {
            signer_config,
            ..self.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StateSignatureMode;

    #[test]
    fn configured_signers_use_migration_settings_without_changing_the_original_config() {
        let base = Config::new().with_network(Network::REGTEST);
        let migration = base.with_signer_config(SignerConfig {
            state_signature_mode: StateSignatureMode::Soft,
            device_storage_path: None,
        });
        assert_eq!(migration.network, base.network);
        let creds = gl_client::credentials::Nobody::new();
        let result = migration.create_signer(vec![0; 32], creds.clone());
        assert!(matches!(result, Err(Error::Argument { .. })));
        assert!(base.create_signer(vec![0; 32], creds).is_ok());
    }
}
