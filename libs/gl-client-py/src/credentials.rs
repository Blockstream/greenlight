use crate::runtime::exec;
use crate::scheduler::Scheduler;
use crate::signer::Signer;
use crate::TlsConfig;
use gl_client::credentials;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyclass]
#[derive(Clone)]
pub enum CredentialType {
    Nobody = 0,
    Device = 1,
}

#[pyclass]
pub struct DeviceBuilder {
    inner: credentials::Builder<credentials::Device>,
}

#[pymethods]
impl DeviceBuilder {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: credentials::Builder::as_device(),
        }
    }

    pub fn from_path(&mut self, path: &str) -> Result<Self> {
        let inner = self.inner.clone().from_path(path)?;
        Ok(Self { inner })
    }

    pub fn from_bytes(&self, data: &[u8]) -> Result<Self> {
        let inner = self.inner.clone().from_bytes(data)?;
        Ok(Self { inner })
    }

    pub fn with_identity(&self, cert: &[u8], key: &[u8]) -> Result<Self> {
        let inner = self.inner.clone().with_identity(cert, key);
        Ok(Self { inner })
    }

    pub fn with_ca(&self, ca: &[u8]) -> Result<Self> {
        let inner = self.inner.clone().with_ca(ca);
        Ok(Self { inner })
    }

    pub fn upgrade(&self, scheduler: &Scheduler, signer: &Signer) -> Result<Self> {
        let inner = exec(async move {
            self.inner
                .clone()
                .upgrade(&scheduler.inner, &signer.inner)
                .await
        })?;
        Ok(Self { inner })
    }

    pub fn build(&self) -> Result<Credentials> {
        let inner = self.inner.clone().build()?;
        Ok(Credentials {
            typ: CredentialType::Device,
            inner,
        })
    }
}

#[pyclass]
pub struct NobodyBuilder {
    inner: credentials::Builder<credentials::Nobody>,
}

#[pymethods]
impl NobodyBuilder {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: credentials::Builder::as_nobody(),
        }
    }

    pub fn with_default(&self) -> Result<Self> {
        let inner = self.inner.clone().with_default()?;
        Ok(Self { inner })
    }

    pub fn with_identity(&self, cert: &[u8], key: &[u8]) -> Result<Self> {
        let inner = self.inner.clone().with_identity(cert, key);
        Ok(Self { inner })
    }

    pub fn with_ca(&self, ca: &[u8]) -> Result<Self> {
        let inner = self.inner.clone().with_ca(ca);
        Ok(Self { inner })
    }

    pub fn build(&self) -> Result<Credentials> {
        let inner = self.inner.clone().build()?;
        Ok(Credentials {
            typ: CredentialType::Nobody,
            inner,
        })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Credentials {
    pub typ: CredentialType,
    pub inner: credentials::Credentials,
}

#[pymethods]
impl Credentials {
    #[staticmethod]
    pub fn as_device() -> DeviceBuilder {
        DeviceBuilder::new()
    }

    #[staticmethod]
    pub fn as_nobody() -> NobodyBuilder {
        NobodyBuilder::new()
    }

    pub fn tls_config(&self) -> Result<TlsConfig> {
        let inner = self.inner.tls_config()?;
        Ok(TlsConfig { inner })
    }

    pub fn to_bytes<'a>(&self, py: Python<'a>) -> Result<&'a PyBytes> {
        match &self.inner {
            credentials::Credentials::Nobody(_) => Err(credentials::Error::IsIdentityError(
                "can not convert nobody into bytes".to_string(),
            ))?,
            credentials::Credentials::Device(c) => Ok(PyBytes::new(py, &c.to_bytes()[..])),
        }
    }
}

type Result<T, E = ErrorWrapper> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum ErrorWrapper {
    #[error("{}", .0)]
    CredentialsError(#[from] credentials::Error),
}

impl From<ErrorWrapper> for pyo3::PyErr {
    fn from(value: ErrorWrapper) -> Self {
        PyErr::new::<PyValueError, _>(value.to_string())
    }
}
