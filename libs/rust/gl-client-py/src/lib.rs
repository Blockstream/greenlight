use gl_client::{pb, tls::NOBODY_CONFIG, Network};
use pyo3::prelude::*;
use tonic::transport::Identity;

#[macro_use]
extern crate log;

mod node;
mod runtime;
mod scheduler;
mod signer;

pub use node::Node;
pub use scheduler::Scheduler;
pub use signer::Signer;

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

#[pyfunction]
fn register<'p>(py: Python<'p>, s: Scheduler, signer: Signer) -> PyResult<&'p PyAny> {
    let secs = 10;
    pyo3_asyncio::tokio::future_into_py(py, async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
        Python::with_gil(|py| Ok(py.None()))
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn glclient(_py: Python, m: &PyModule) -> PyResult<()> {
    env_logger::init();
    m.add_class::<Signer>()?;
    m.add_class::<Node>()?;
    m.add_class::<Amount>()?;
    m.add_class::<Scheduler>()?;

    m.add_function(wrap_pyfunction!(register, m)?)?;
    Ok(())
}
