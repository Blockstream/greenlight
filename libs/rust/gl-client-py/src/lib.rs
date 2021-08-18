use gl_client::{signer, tls::NOBODY_CONFIG, Network};
use pyo3::prelude::*;
use std::thread::spawn;
use tonic::transport::Identity;

#[pyclass]
struct Signer {
    inner: signer::Signer,
}

#[pymethods]
impl Signer {
    #[new]
    fn new(
        secret: Vec<u8>,
        network: String,
        device_cert: Vec<u8>,
        device_key: Vec<u8>,
    ) -> PyResult<Signer> {
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

        let identity = Identity::from_pem(device_cert, device_key);
        let inner = match signer::Signer::new(secret, network, NOBODY_CONFIG.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Error initializing Signer: {}",
                    e
                )))
            }
        }
        .with_identity(identity);
        Ok(Signer { inner })
    }

    fn run_in_thread(&mut self) -> PyResult<()> {
        let inner = self.inner.clone();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        spawn(move || runtime.block_on(async move { inner.run_forever().await }));
        Ok(())
    }

    fn run_in_foreground(&self) -> PyResult<()> {
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
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyglclient(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Signer>()?;
    Ok(())
}
