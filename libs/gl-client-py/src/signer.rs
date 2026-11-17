use crate::credentials::Credentials;
use gl_client::bitcoin::Network;
use gl_client::signer::{DeviceStorage, FileDeviceStorage, SignerConfig, StateSignatureMode};
use log::warn;
use pyo3::{exceptions::PyValueError, prelude::*};
use std::sync::Arc;
use tokio::sync::mpsc;

#[pyclass]
#[derive(Clone)]
pub struct Signer {
    pub(crate) inner: gl_client::signer::Signer,
}

#[pymethods]
impl Signer {
    #[new]
    #[pyo3(signature = (secret, network, creds, *, state_signature_mode="hard", device_storage_path=None))]
    fn new(
        secret: Vec<u8>,
        network: String,
        creds: Credentials,
        state_signature_mode: &str,
        device_storage_path: Option<String>,
    ) -> PyResult<Signer> {
        let network: Network = match network.parse() {
            Ok(network) => network,
            Err(_) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown / unsupported network {}",
                    network
                )))
            }
        };

        let state_signature_mode = match state_signature_mode {
            "hard" => StateSignatureMode::Hard,
            "soft" => StateSignatureMode::Soft,
            "off" => StateSignatureMode::Off,
            other => {
                return Err(PyValueError::new_err(format!(
                    "Unknown state signature mode {other}: expected hard, soft or off"
                )))
            }
        };
        let device_storage: Option<Arc<dyn DeviceStorage>> = match state_signature_mode {
            StateSignatureMode::Hard | StateSignatureMode::Soft => {
                let path = device_storage_path
                    .as_deref()
                    .filter(|path| !path.is_empty())
                    .ok_or_else(|| {
                        PyValueError::new_err(
                            "Hard and Soft modes require a nonempty device_storage_path",
                        )
                    })?;
                Some(Arc::new(FileDeviceStorage::new(path).map_err(|e| {
                    PyValueError::new_err(format!("opening device storage: {e:#}"))
                })?))
            }
            _ => None,
        };
        let config = SignerConfig {
            state_signature_mode,
            device_storage,
            ..Default::default()
        };
        let inner = match gl_client::signer::Signer::new_with_config(
            secret,
            network,
            creds.inner,
            config,
        ) {
            Ok(v) => v,
            Err(e) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Error initializing Signer: {:#}",
                    e
                )))
            }
        };

        Ok(Signer { inner })
    }

    fn run_in_thread(&mut self) -> PyResult<SignerHandle> {
        trace!("Starting a new thread for signer");
        let inner = self.inner.clone();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let (tx, rx) = mpsc::channel(1);

        std::thread::spawn(move || {
            runtime.block_on(async move {
                if let Err(e) = inner.run_forever(rx).await {
                    log::error!("Error running signer in thread: {e}")
                }
            })
        });
        Ok(SignerHandle { signal: tx })
    }

    fn run_in_foreground(&self) -> PyResult<()> {
        trace!("Running signer in foreground thread");
        let (_tx, rx) = mpsc::channel(1);
        let res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { self.inner.run_forever(rx).await });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Error running Signer: {}",
                e
            ))),
        }
    }

    fn node_id(&self) -> Vec<u8> {
        self.inner.node_id()
    }

    fn init(&self) -> Vec<u8> {
        self.inner.get_init()
    }

    fn bip32_key(&self) -> Vec<u8> {
        self.inner.get_init()[35..].to_vec()
    }

    fn sign_challenge(&self, challenge: Vec<u8>) -> PyResult<Vec<u8>> {
        match self.inner.sign_challenge(challenge) {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyValueError::new_err(e.to_string())),
        }
    }

    fn version(&self) -> PyResult<&'static str> {
        Ok(self.inner.version())
    }

    fn create_rune(&self, restrictions: Vec<Vec<&str>>, rune: Option<&str>) -> PyResult<String> {
        self.inner
            .create_rune(rune, restrictions)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SignerHandle {
    pub(crate) signal: mpsc::Sender<()>,
}

#[pymethods]
impl SignerHandle {
    fn shutdown(&self) -> PyResult<()> {
        if let Err(e) = self.signal.try_send(()) {
            warn!("Failed to send shutdown signal, signer may already be stopped: {e}");
        }

        Ok(())
    }
}
