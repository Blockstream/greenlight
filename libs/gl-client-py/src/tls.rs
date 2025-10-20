use gl_client::tls;
use pyo3::exceptions::PyFileNotFoundError;
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
        let inner = tls::TlsConfig::new();
        Ok(Self { inner })
    }

    fn identity(&self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Self {
        Self {
            inner: self.inner.clone().identity(cert_pem, key_pem),
        }
    }

    fn identity_from_path(&self, path: &str) -> Result<Self, PyErr> {
        let result = Self {
            inner: self
                .inner
                .clone()
                .identity_from_path(path)
                .map_err(|_| PyFileNotFoundError::new_err(String::from(path)))?,
        };

        return Ok(result);
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
