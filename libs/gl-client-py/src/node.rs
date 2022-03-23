use crate::runtime::exec;
use crate::tls::TlsConfig;
use bitcoin::Network;
use gl_client as gl;
use gl_client::pb;
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tonic::{Code, Status};

#[pyclass]
pub struct Node {
    _inner: gl::node::Node,
    client: gl::node::Client,
    _node_id: Vec<u8>,
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
            _inner: inner,
            client,
            _node_id: node_id,
        })
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

    fn close(
        &self,
        peer_id: Vec<u8>,
        timeout: Option<u32>,
        address: Option<String>,
    ) -> PyResult<Vec<u8>> {
        convert(
            exec(self.client.clone().close_channel(pb::CloseChannelRequest {
                node_id: peer_id,
                unilateraltimeout: timeout.map(|s| pb::Timeout { seconds: s }),
                destination: address.map(|a| pb::BitcoinAddress { address: a }),
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

    fn new_address(&self, address_type: Option<&str>) -> PyResult<Vec<u8>> {
        let typ = match address_type {
            None => pb::BtcAddressType::Bech32,
            Some("bech32") => pb::BtcAddressType::P2shSegwit,
            Some("p2sh-segwit") => pb::BtcAddressType::P2shSegwit,
            Some(v) => {
                return Err(PyValueError::new_err(format!(
                    "Unknown address type {}, available types are bech32 and p2sh-segwit",
                    v
                )))
            }
        };

        convert(
            exec(self.client.clone().new_addr(pb::NewAddrRequest {
                address_type: typ as i32,
            }))
            .map(|x| x.into_inner()),
        )
    }

    fn withdraw(&self, args: &[u8]) -> PyResult<Vec<u8>> {
        let req = pb::WithdrawRequest::decode(args).map_err(error_decoding_request)?;

        convert(exec(self.client.clone().withdraw(req)).map(|x| x.into_inner()))
    }

    fn fund_channel(&self, args: &[u8]) -> PyResult<Vec<u8>> {
        let req = pb::FundChannelRequest::decode(args).map_err(error_decoding_request)?;

        convert(exec(self.client.clone().fund_channel(req)).map(|x| x.into_inner()))
    }

    fn create_invoice(&self, args: &[u8]) -> PyResult<Vec<u8>> {
        let req = pb::InvoiceRequest::decode(args).map_err(error_decoding_request)?;

        convert(exec(self.client.clone().create_invoice(req)).map(|x| x.into_inner()))
    }

    fn pay(&self, args: &[u8]) -> PyResult<Vec<u8>> {
        let req = pb::PayRequest::decode(args).map_err(error_decoding_request)?;

        convert(exec(self.client.clone().pay(req)).map(|x| x.into_inner()))
    }

    fn keysend(&self, args: &[u8]) -> PyResult<Vec<u8>> {
        let req = pb::KeysendRequest::decode(args).map_err(error_decoding_request)?;

        convert(exec(self.client.clone().keysend(req)).map(|x| x.into_inner()))
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

    fn call(&self, method: &str, req: &[u8]) -> PyResult<Vec<u8>> {
        let res = exec(self.dispatch(method, req));
        res
    }
}

impl Node {
    async fn dispatch(&self, method: &str, req: &[u8]) -> Result<Vec<u8>, PyErr> {
        let mut client = self.client.clone();
        match method {
            "GetInfo" => convert(
                client
                    .get_info(pb::GetInfoRequest::decode(req).unwrap())
                    .await
                    .map(|x| x.into_inner()),
            ),
            "ListInvoices" => convert(
                client
                    .list_invoices(pb::ListInvoicesRequest::decode(req).unwrap())
                    .await
                    .map(|x| x.into_inner()),
            ),
            m => Err(PyValueError::new_err(format!(
                "Unmapped method {}, please add it to node.rs",
                m
            ))),
        }
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

pub fn convert<T: Message>(r: Result<T, Status>) -> PyResult<Vec<u8>> {
    let res = r.map_err(error_calling_remote_method)?;
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(buf)
}
