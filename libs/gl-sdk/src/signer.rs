use crate::{Credentials, Error, SignerConfig};
use bip39::Mnemonic;
use std::str::FromStr;
use tracing;

#[derive(uniffi::Object, Clone)]
pub struct Signer {
    pub(crate) inner: gl_client::signer::Signer,
}

#[uniffi::export]
impl Signer {
    #[uniffi::constructor()]
    pub fn new(phrase: String, device_storage_path: String) -> Result<Signer, Error> {
        Self::new_with_config(
            phrase,
            SignerConfig {
                device_storage_path: Some(device_storage_path),
                ..Default::default()
            },
        )
    }

    #[uniffi::constructor()]
    pub fn new_with_config(phrase: String, config: SignerConfig) -> Result<Signer, Error> {
        let phrase = Mnemonic::from_str(phrase.as_str()).map_err(|_e| Error::phrase_corrupted())?;
        let seed = phrase.to_seed_normalized(&"").to_vec();
        Self::new_from_seed_with_config(seed, config)
    }

    #[uniffi::constructor()]
    pub fn new_from_seed(seed: Vec<u8>, device_storage_path: String) -> Result<Signer, Error> {
        Self::new_from_seed_with_config(
            seed,
            SignerConfig {
                device_storage_path: Some(device_storage_path),
                ..Default::default()
            },
        )
    }

    #[uniffi::constructor()]
    pub fn new_from_seed_with_config(seed: Vec<u8>, config: SignerConfig) -> Result<Signer, Error> {
        let credentials = gl_client::credentials::Nobody::new();
        let config = config.to_client()?;

        let inner = gl_client::signer::Signer::new_with_config(
            seed,
            gl_client::bitcoin::Network::Bitcoin,
            credentials,
            config,
        )
        .map_err(|e| Error::other(format!("{e:#}")))?;
        Ok(Signer { inner })
    }

    pub fn authenticate(&self, creds: &Credentials) -> Result<Signer, Error> {
        Ok(Signer {
            inner: self.inner.with_credentials(creds.inner.clone()),
        })
    }

    pub fn start(&self) -> Result<Handle, Error> {
        Ok(Handle::spawn(self.inner.clone()))
    }

    pub fn node_id(&self) -> Vec<u8> {
        self.inner.node_id()
    }
}

// Not exported through uniffi, internal logic only.
impl Signer {
    async fn run(&self, signal: tokio::sync::mpsc::Receiver<()>) {
        self.inner
            .run_forever(signal)
            .await
            .expect("Error running signer loop");
    }
}

/// A handle to interact with a signer loop running and processing
/// requests in the background. Used primarily to stop the loop and
/// exiting the signer.
#[derive(uniffi::Object, Clone)]
pub struct Handle {
    chan: tokio::sync::mpsc::Sender<()>,
}

#[uniffi::export]
impl Handle {
    pub fn stop(&self) {
        self.chan.try_send(()).expect("sending shutdown signal");
    }
}

impl Handle {
    /// Spawns the signer on the shared signer runtime and returns a handle to stop it.
    pub(crate) fn spawn(signer: gl_client::signer::Signer) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        crate::util::get_signer_runtime().spawn(async move {
            if let Err(e) = signer.run_forever(rx).await {
                tracing::error!("Error running signer: {e}");
            }
        });
        Self { chan: tx }
    }

    /// Sends the shutdown signal, returning true if the signal was
    /// delivered and false if the signer has already stopped.
    pub(crate) fn try_stop(&self) -> bool {
        self.chan.try_send(()).is_ok()
    }
}
