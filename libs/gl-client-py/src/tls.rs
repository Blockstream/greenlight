use gl_client::tls;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct TlsConfig {
    pub(crate) inner: tls::TlsConfig,
}

#[pymethods]
impl TlsConfig {
    #[new]
    fn new() -> PyResult<TlsConfig> {
        let inner = tls::TlsConfig::new()
            .map_err(|e| PyValueError::new_err(format!("Error creating TlsConfig: {:?}", e)))?;
        Ok(Self { inner })
    }

    fn identity(&self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Self {
        Self {
            inner: self.inner.clone().identity(cert_pem, key_pem),
        }
    }

    fn with_ca_certificate(&self, ca: Vec<u8>) -> TlsConfig {
        Self {
            inner: self.inner.clone().ca_certificate(ca),
        }
    }

    fn ca_certificate(&self) -> Vec<u8> {
        self.inner.ca.clone()
    }
}
