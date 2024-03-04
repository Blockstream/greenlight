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
#[derive(Clone)]
pub struct Credentials {
    pub typ: CredentialType,
    pub inner: credentials::Credentials,
}

#[pymethods]
impl Credentials {
    #[staticmethod]
    pub fn nobody() -> Credentials {
        let inner = credentials::Nobody::new();
        Credentials {
            typ: CredentialType::Nobody,
            inner,
        }
    }

    #[staticmethod]
    pub fn nobody_with(cert: &[u8], key: &[u8], ca: &[u8]) -> Credentials {
        let inner = credentials::Nobody::with(cert, key, ca);
        Credentials {
            typ: CredentialType::Nobody,
            inner,
        }
    }

    #[staticmethod]
    pub fn from_path(path: &str) -> Credentials {
        let inner = credentials::Device::from_path(path);
        Credentials {
            typ: CredentialType::Device,
            inner,
        }
    }

    #[staticmethod]
    pub fn from_bytes(data: &[u8]) -> Credentials {
        let inner = credentials::Device::from_bytes(data);
        Credentials {
            typ: CredentialType::Device,
            inner,
        }
    }

    pub fn upgrade(&self, scheduler: &Scheduler, signer: &Signer) -> Result<Credentials> {
        let inner = exec(async move {
            self.inner
                .clone()
                .upgrade(&scheduler.inner, &signer.inner)
                .await
        })?;
        Ok(Self {
            typ: self.typ.clone(),
            inner,
        })
    }

    pub fn tls_config(&self) -> Result<TlsConfig> {
        let inner = self.inner.tls_config();
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
