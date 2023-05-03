use crate::runtime::exec;
use crate::tls::TlsConfig;
use gl_client::bitcoin::Network;
use gl_client as gl;
use gl_client::pb;
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tonic::{Code, Status};

#[pyclass]
pub struct Node {
    client: gl::node::Client,
    gclient: gl::node::GClient,
}

#[pymethods]
impl Node {
    #[new]
    fn new(node_id: Vec<u8>, network: String, tls: TlsConfig, grpc_uri: String) -> PyResult<Self> {
        let network: Network = match network.parse() {
            Ok(v) => v,
            Err(_) => return Err(PyValueError::new_err("unknown network")),
        };

        let inner = gl::node::Node::new(node_id.clone(), network, tls.inner);

        // Connect to both interfaces in parallel to avoid doubling the startup time:

	// TODO: Could be massively simplified by using a scoped task
	// from tokio_scoped to a
        let (client, gclient) = exec(async {
            let i = inner.clone();
            let u = grpc_uri.clone();
            let h1 = tokio::spawn(async move { i.connect(u).await });
            let i = inner.clone();
            let u = grpc_uri.clone();
            let h2 = tokio::spawn(async move { i.connect(u).await });

            Ok::<(gl::node::Client, gl::node::GClient), anyhow::Error>((h1.await??, h2.await??))
        })
        .map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("could not connect to node: {}", e))
        })?;

        Ok(Node { client, gclient })
    }

    fn call(&self, method: &str, payload: Vec<u8>) -> PyResult<Vec<u8>> {
        exec(self.gclient.clone().call(method, payload))
            .map(|x| x.into_inner().to_vec())
            .map_err(|s| PyValueError::new_err(format!("Error calling {}: {}", method, s)))
    }

    fn stream_log(&self, args: &[u8]) -> PyResult<LogStream> {
        let req = pb::StreamLogRequest::decode(args).map_err(error_decoding_request)?;

        let stream = exec(self.client.clone().stream_log(req))
            .map(|x| x.into_inner())
            .map_err(error_starting_stream)?;
        Ok(LogStream { inner: stream })
    }

    fn stream_incoming(&self, args: &[u8]) -> PyResult<IncomingStream> {
        let req = pb::StreamIncomingFilter::decode(args).map_err(error_decoding_request)?;

        let stream = exec(self.client.clone().stream_incoming(req))
            .map(|x| x.into_inner())
            .map_err(error_starting_stream)?;
        Ok(IncomingStream { inner: stream })
    }
}

fn error_decoding_request<D: core::fmt::Display>(e: D) -> PyErr {
    PyValueError::new_err(format!("error decoding request: {}", e))
}

pub fn error_calling_remote_method<D: core::fmt::Display>(e: D) -> PyErr {
    PyValueError::new_err(format!("error calling remote method: {}", e))
}

fn error_starting_stream<D: core::fmt::Display>(e: D) -> PyErr {
    PyValueError::new_err(format!("Error starting stream: {}", e))
}

#[pyclass]
struct LogStream {
    inner: tonic::codec::Streaming<pb::LogEntry>,
}

#[pymethods]
impl LogStream {
    fn next(&mut self) -> PyResult<Option<Vec<u8>>> {
        convert_stream_entry(exec(async { self.inner.message().await }))
    }
}

#[pyclass]
struct IncomingStream {
    inner: tonic::codec::Streaming<pb::IncomingPayment>,
}

#[pymethods]
impl IncomingStream {
    fn next(&mut self) -> PyResult<Option<Vec<u8>>> {
        convert_stream_entry(exec(async { self.inner.message().await }))
    }
}

fn convert_stream_entry<T: Message>(r: Result<Option<T>, Status>) -> PyResult<Option<Vec<u8>>> {
    let res = match r {
        Ok(Some(v)) => v,
        Ok(None) => return Ok(None),
        Err(e) => match e.code() {
            Code::Unknown => {
                // Unknown most likely just means we lost the
                // connection. This is due to a shutdown and shouldn't
                // be as noisy as other errors.
                return Ok(None);
            }
            _ => {
                log::warn!("ERROR {:?}", e);
                return Err(error_calling_remote_method(e));
            }
        },
    };
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(Some(buf))
}
