use pyo3::prelude::*;
use gl_client as gl;

#[pyclass]
pub struct Node {
    _inner: gl::node::Client,
}

#[pymethods]
impl Node {
    #[new]
    fn new(_node_id: Vec<u8>, _network: String, _tls_cert: Vec<u8>, _tls_key: Vec<u8>) -> Self {
        unimplemented!()
    }
}
