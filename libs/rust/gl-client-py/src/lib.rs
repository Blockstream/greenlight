use gl_client::pb;
use pyo3::prelude::*;

#[macro_use]
extern crate log;

mod node;
mod runtime;
mod scheduler;
mod signer;
mod tls;

pub use node::Node;
pub use scheduler::Scheduler;
pub use signer::Signer;
pub use tls::TlsConfig;

/// Simple wrapper around the protobuf message type to allow handling
/// rust-native amounts in python.
#[pyclass]
struct Amount {
    inner: pb::Amount,
}

#[pymethods]
impl Amount {
    #[staticmethod]
    fn satoshis(v: u64) -> Self {
        Amount {
            inner: pb::Amount {
                unit: Some(pb::amount::Unit::Satoshi(v)),
            },
        }
    }

    #[staticmethod]
    fn millisatoshis(v: u64) -> Self {
        Amount {
            inner: pb::Amount {
                unit: Some(pb::amount::Unit::Millisatoshi(v)),
            },
        }
    }

    #[staticmethod]
    fn bitcoins(v: u64) -> Self {
        Amount {
            inner: pb::Amount {
                unit: Some(pb::amount::Unit::Bitcoin(v)),
            },
        }
    }

    #[staticmethod]
    fn all() -> Self {
        Amount {
            inner: pb::Amount {
                unit: Some(pb::amount::Unit::All(true)),
            },
        }
    }

    #[staticmethod]
    fn any() -> Self {
        Amount {
            inner: pb::Amount {
                unit: Some(pb::amount::Unit::Any(true)),
            },
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn glclient(_py: Python, m: &PyModule) -> PyResult<()> {
    env_logger::init();
    m.add_class::<Signer>()?;
    m.add_class::<Node>()?;
    m.add_class::<Amount>()?;
    m.add_class::<Scheduler>()?;
    m.add_class::<TlsConfig>()?;

    Ok(())
}
