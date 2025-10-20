use crate::runtime::exec;
use gl_client::lsps::client::LspClient as LspClientInner;
use gl_client::lsps::error::LspsError;
use gl_client::lsps::json_rpc::{generate_random_rpc_id, JsonRpcResponse};
use gl_client::lsps::message as lsps_message;
use gl_client::node::{Client, ClnClient};
use pyo3::exceptions::{PyBaseException, PyConnectionError, PyTimeoutError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::PyErr;

use hex::ToHex;
#[pyclass]
pub struct LspClient {
    lsp_client: LspClientInner,
}

impl LspClient {
    pub fn new(client: Client, cln_client: ClnClient) -> Self {
        LspClient {
            lsp_client: LspClientInner::new(client, cln_client),
        }
    }
}

fn lsps_err_to_py_err(err: &LspsError) -> PyErr {
    match err {
        LspsError::MethodUnknown(method_name) => {
            PyValueError::new_err(format!("Unknown method {:?}", method_name))
        }
        LspsError::ConnectionClosed => PyConnectionError::new_err("Failed to connect"),
        LspsError::GrpcError(status) => PyConnectionError::new_err(String::from(status.message())),
        LspsError::Timeout => PyTimeoutError::new_err("Did not receive a response from the LSPS"),
        LspsError::JsonParseRequestError(error) => {
            PyValueError::new_err(format!("Failed to parse json-request, {:}", error))
        }
        LspsError::JsonParseResponseError(error) => {
            PyValueError::new_err(format!("Failed to parse json-response, {:}", error))
        }
        LspsError::Other(error_message) => PyBaseException::new_err(String::from(error_message)),
    }
}

#[pymethods]
impl LspClient {
    // When doing ffi with python we'de like to keep the interface as small as possible.
    //
    // We already have JSON-serialization and deserialization working because the underlying protocol uses JSON-rpc
    //
    // When one of the JSON-rpc method is called from python the user can just specify the peer-id and the serialized parameter they want to send
    // The serialized result will be returned
    pub fn rpc_call(
        &mut self,
        py: Python,
        peer_id: &[u8],
        method_name: &str,
        value: &[u8],
    ) -> PyResult<PyObject> {
        let json_rpc_id = generate_random_rpc_id();
        self.rpc_call_with_json_rpc_id(py, peer_id, method_name, value, json_rpc_id)
    }

    pub fn rpc_call_with_json_rpc_id(
        &mut self,
        py: Python,
        peer_id: &[u8],
        method_name: &str,
        value: &[u8],
        json_rpc_id: String,
    ) -> PyResult<PyObject> {
        // Parse the method-name and call the rpc-request
        let rpc_response: JsonRpcResponse<Vec<u8>, Vec<u8>> =
            lsps_message::JsonRpcMethodEnum::from_method_name(method_name)
                .and_then(|method| {
                    exec(self.lsp_client.request_with_json_rpc_id(
                        peer_id,
                        &method,
                        value.to_vec(),
                        json_rpc_id,
                    ))
                })
                .map_err(|err| lsps_err_to_py_err(&err))?;

        match rpc_response {
            JsonRpcResponse::Ok(ok) => {
                let response = ok.result; // response as byte-array
                let py_object: PyObject = PyBytes::new(py, &response).into();
                return Ok(py_object);
            }
            JsonRpcResponse::Error(err) => {
                // We should be able to put the error-data in here
                // Replace this by a custom exception type
                return Err(PyBaseException::new_err(format!(
                    "{:?} - {:?}",
                    err.error.code, err.error.message
                )));
            }
        }
    }

    pub fn list_lsp_servers(&mut self) -> PyResult<Vec<String>> {
        let result = exec(self.lsp_client.list_lsp_servers());

        match result {
            Ok(result) => Ok(result.iter().map(|x| x.encode_hex()).collect()),
            Err(err) => Err(lsps_err_to_py_err(&err)),
        }
    }
}
