use gl_client::pb;
use pyo3::prelude::*;
use crate::node::__pyo3_get_function_get_node_uri;

#[macro_use]
extern crate log;

mod node;
mod runtime;
mod scheduler;
mod signer;
mod tls;

pub use node::{Node, get_node_uri};
pub use scheduler::Scheduler;
pub use signer::Signer;
pub use tls::TlsConfig;

/// A Python module implemented in Rust.
#[pymodule]
fn glclient(_py: Python, m: &PyModule) -> PyResult<()> {
    env_logger::init();
    m.add_class::<Signer>()?;
    m.add_class::<Node>()?;
    m.add_class::<Scheduler>()?;
    m.add_class::<TlsConfig>()?;
    m.add_function(wrap_pyfunction!(get_node_uri, m)?)?;

    Ok(())
}
