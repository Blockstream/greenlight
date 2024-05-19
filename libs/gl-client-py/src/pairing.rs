use crate::credentials::Credentials;
use crate::runtime::exec;
use bytes::BufMut;
use gl_client::pairing::{attestation_device, new_device, PairingSessionData};
use gl_client::pb::scheduler::GetPairingDataResponse;
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
    fn new(creds: Credentials, uri: Option<String>) -> Result<Self> {
        let mut client = new_device::Client::new(creds.inner);

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

#[pyclass]
pub struct AttestationDeviceClient {
    inner: attestation_device::Client<attestation_device::Connected>,
}

#[pymethods]
impl AttestationDeviceClient {
    #[new]
    fn new(creds: Credentials, uri: Option<String>) -> Result<Self> {
        let mut client = attestation_device::Client::new(creds.inner)?;
        if let Some(uri) = uri {
            client = client.with_uri(uri);
        }
        let inner = exec(client.connect())?;
        Ok(AttestationDeviceClient { inner })
    }

    fn get_pairing_data(&self, session_id: &str) -> Result<Vec<u8>> {
        Ok(convert(exec(async move {
            self.inner.get_pairing_data(session_id).await
        }))?)
    }

    fn approve_pairing(
        &self,
        session_id: &str,
        node_id: &[u8],
        device_name: &str,
        restrs: &str,
    ) -> Result<Vec<u8>> {
        Ok(convert(exec(async move {
            self.inner
                .approve_pairing(session_id, node_id, device_name, restrs)
                .await
        }))?)
    }

    fn verify_pairing_data(&self, data: Vec<u8>) -> Result<()> {
        let pd = GetPairingDataResponse::decode(&data[..])?;
        Ok(attestation_device::Client::verify_pairing_data(pd)?)
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
                    PairingSessionData::PairingQr(d) => convert_pairing(d, 2),
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

// Converts a the message into bytes
fn convert<T: Message, E>(r: Result<T, E>) -> Result<Vec<u8>, E> {
    let res = r?;
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(buf)
}

type Result<T, E = ErrorWrapper> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum ErrorWrapper {
    #[error("{}", .0)]
    PairingError(#[from] gl_client::pairing::Error),
    #[error("{}", .0)]
    ProtoError(#[from] prost::DecodeError),
}

impl From<ErrorWrapper> for pyo3::PyErr {
    fn from(value: ErrorWrapper) -> Self {
        PyErr::new::<PyValueError, _>(value.to_string())
    }
}
