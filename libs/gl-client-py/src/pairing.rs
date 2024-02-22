use crate::runtime::exec;
use crate::tls::TlsConfig;
use bytes::BufMut;
use gl_client::pairing::{new_device, PairingSessionData};
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tokio::sync::mpsc;

#[pyclass]
pub struct NewDeviceClient {
    inner: new_device::Client<new_device::Connected>,
}

#[pymethods]
impl NewDeviceClient {
    #[new]
    fn new(tls: TlsConfig, uri: Option<String>) -> Result<Self> {
        let mut client = new_device::Client::new(tls.inner);

        if let Some(uri) = uri {
            client = client.with_uri(uri);
        }

        let inner = exec(client.connect())?;
        Ok(Self { inner })
    }

    fn pair_device(
        &self,
        name: &str,
        description: &str,
        restrictions: &str,
    ) -> Result<PyPairingChannelWrapper> {
        let inner = exec(self.inner.pair_device(name, description, restrictions))?;
        Ok(PyPairingChannelWrapper { inner })
    }
}

/// A wrapper class to return an iterable from a mpsc channel.
#[pyclass]
pub struct PyPairingChannelWrapper {
    inner: mpsc::Receiver<PairingSessionData>,
}

#[pymethods]
impl PyPairingChannelWrapper {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Vec<u8>> {
        slf.recv().ok()
    }

    fn recv(&mut self) -> PyResult<Vec<u8>> {
        let receiver = &mut self.inner;
        exec(async move {
            match receiver.recv().await {
                Some(data) => match data {
                    PairingSessionData::PairingResponse(d) => convert_pairing(d, 1),
                    PairingSessionData::PairingError(d) => {
                        debug!("pairing returned a PairingError {}", d);
                        Err(PyValueError::new_err(d.to_string()))
                    }
                },
                None => Err(PyValueError::new_err("channel error")),
            }
        })
    }
}

// Prepends a type to the message to identify the type in python.
pub fn convert_pairing<T: Message>(msg: T, typ: u8) -> PyResult<Vec<u8>> {
    let mut buf = Vec::with_capacity(msg.encoded_len() + 1);
    buf.put_u8(typ);
    msg.encode(&mut buf)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(buf)
}

type Result<T, E = ErrorWrapper> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum ErrorWrapper {
    #[error("{}", .0)]
    PairingError(#[from] gl_client::pairing::Error),
}

impl From<ErrorWrapper> for pyo3::PyErr {
    fn from(value: ErrorWrapper) -> Self {
        PyErr::new::<PyValueError, _>(value.to_string())
    }
}
