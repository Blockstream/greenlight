use gl_client::{signer, tls::NOBODY_CONFIG, Network};
use pyo3::prelude::*;
use std::sync::Once;
use std::thread::spawn;
use tonic::transport::Identity;

#[macro_use]
extern crate log;

#[pyclass]
struct Signer {
    inner: signer::Signer,
}

/// Some initialization must run at most once, so here's a guard for
/// that.
static START: Once = Once::new();

#[pymethods]
impl Signer {
    #[new]
    fn new(secret: Vec<u8>, network: String) -> PyResult<Signer> {
        START.call_once(|| {
            env_logger::init();
        });

        let network = match network.as_str() {
            "bitcoin" => Network::BITCOIN,
            "testnet" => Network::TESTNET,
            "regtest" => Network::REGTEST,
            v => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown / unsupported network {}",
                    v
                )))
            }
        };

        let inner = match signer::Signer::new(secret, network, NOBODY_CONFIG.clone()) {
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

    fn with_identity(&self, device_cert: Vec<u8>, device_key: Vec<u8>) -> Self {
        let identity = Identity::from_pem(device_cert, device_key);
        Signer {
            inner: self.inner.clone().with_identity(identity),
        }
    }

    fn with_ca(&self, ca_cert: Vec<u8>) -> Self {
        Signer {
            inner: self.inner.clone().with_ca(ca_cert),
        }
    }

    fn run_in_thread(&mut self) -> PyResult<()> {
        trace!("Starting a new thread for signer");
        let inner = self.inner.clone();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        spawn(move || runtime.block_on(async move { inner.run_forever().await }));
        Ok(())
    }

    fn run_in_foreground(&self) -> PyResult<()> {
        trace!("Running signer in foreground thread");
        let res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { self.inner.run_forever().await });

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
}

/// A Python module implemented in Rust.
#[pymodule]
fn glclient(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Signer>()?;
    Ok(())
}
