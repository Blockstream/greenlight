use crate::tls::TlsConfig;
use bitcoin::Network;
use log::warn;
use pyo3::prelude::*;
use tokio::sync::mpsc;

#[pyclass]
#[derive(Clone)]
pub struct Signer {
    pub(crate) inner: gl_client::signer::Signer,
}

#[pymethods]
impl Signer {
    #[new]
    fn new(secret: Vec<u8>, network: String, tls: TlsConfig) -> PyResult<Signer> {
        let network: Network = match network.parse() {
            Ok(network) => network,
            Err(_) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown / unsupported network {}",
                    network
                )))
            }
        };

        let inner = match gl_client::signer::Signer::new(secret, network, tls.inner) {
            Ok(v) => v,
            Err(e) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Error initializing Signer: {}",
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

        std::thread::spawn(move || runtime.block_on(async move { inner.run_forever(rx).await }));
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
