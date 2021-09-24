use crate::runtime::exec;
use crate::tls::TlsConfig;
use anyhow::Error;
use gl_client as gl;
use gl_client::pb;
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::convert::TryInto;
use tonic::Status;

#[pyclass]
pub struct Node {
    inner: gl::node::Node,
    client: gl::node::Client,
    node_id: Vec<u8>,
}

#[pymethods]
impl Node {
    #[new]
    fn new(node_id: Vec<u8>, network: String, tls: TlsConfig, grpc_uri: String) -> PyResult<Self> {
        let network: gl::Network = match network.try_into() {
            Ok(v) => v,
            Err(_) => return Err(PyValueError::new_err("unknown network")),
        };

        let inner = gl::node::Node::new(node_id.clone(), network, tls.inner);
        let client = match exec(inner.clone().connect(grpc_uri)) {
            Ok(c) => c,
            Err(e) => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "could not connect to node: {}",
                    e
                )))
            }
        };
        Ok(Node {
            inner,
            client,
            node_id,
        })
    }

    fn get_info(&self) -> PyResult<Vec<u8>> {
        convert(exec(self.client.clone().get_info(pb::GetInfoRequest {})).map(|x| x.into_inner()))
    }

    fn stop(&self) -> PyResult<()> {
        let res = exec(self.client.clone().stop(pb::StopRequest {})).map(|x| x.into_inner());
        match res {
            Ok(_) => panic!("stop returned a response, when it really should just have stopped"),
            Err(_) => Ok(()),
        }
    }

    fn list_funds(&self) -> PyResult<Vec<u8>> {
        convert(
            exec(
                self.client
                    .clone()
                    .list_funds(pb::ListFundsRequest::default()),
            )
            .map(|x| x.into_inner()),
        )
    }

    fn list_invoices(&self) -> PyResult<Vec<u8>> {
        convert(
            exec(
                self.client
                    .clone()
                    .list_invoices(pb::ListInvoicesRequest::default()),
            )
            .map(|x| x.into_inner()),
        )
    }

    fn list_payments(&self) -> PyResult<Vec<u8>> {
        convert(
            exec(
                self.client
                    .clone()
                    .list_payments(pb::ListPaymentsRequest::default()),
            )
            .map(|x| x.into_inner()),
        )
    }

    fn list_peers(&self) -> PyResult<Vec<u8>> {
        convert(
            exec(
                self.client
                    .clone()
                    .list_peers(pb::ListPeersRequest::default()),
            )
            .map(|x| x.into_inner()),
        )
    }

    fn connect_peer(&self, node_id: String, addr: Option<String>) -> PyResult<Vec<u8>> {
	trace!("Connecting to node_id={} at addr={:?}", node_id, addr);
        convert(
            exec(self.client.clone().connect_peer(pb::ConnectRequest {
                node_id: node_id,
                addr: addr.unwrap_or_default(),
            }))
            .map(|x| x.into_inner()),
        )
    }
    fn disconnect_peer(&self, peer_id: String, force: Option<bool>) -> PyResult<Vec<u8>> {
	let force = force.unwrap_or(false);
	trace!("Disconnecting from peer_id={} at force={}", peer_id, force);
        convert(
            exec(self.client.clone().disconnect(pb::DisconnectRequest {
                node_id: peer_id,
                force: force,
            }))
            .map(|x| x.into_inner()),
        )
    }
}

pub fn convert<T: Message>(r: Result<T, Status>) -> PyResult<Vec<u8>> {
    let res = match r {
        Ok(v) => v,
        Err(e) => {
            return Err(PyValueError::new_err(format!(
                "error calling remote method: {}",
                e
            )))
        }
    };
    let mut buf = Vec::new();
    buf.reserve(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(buf)
}
