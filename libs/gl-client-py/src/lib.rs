use gl_client::{bitcoin, export::decrypt_with_seed};
use pyo3::prelude::*;

#[macro_use]
extern crate log;

mod credentials;
mod lsps;
mod node;
mod runtime;
mod scheduler;
mod signer;
mod tls;

pub use lsps::LspClient;
pub use node::Node;
pub use scheduler::Scheduler;
pub use signer::{Signer, SignerHandle};
pub use tls::TlsConfig;

#[pyfunction]
pub fn backup_decrypt_with_seed(encrypted: Vec<u8>, seed: Vec<u8>) -> PyResult<Vec<u8>> {
    use pyo3::exceptions::PyValueError;
    let mut bytes = bytes::BytesMut::zeroed(encrypted.len());
    bytes.clone_from_slice(&encrypted);
    let seed = bitcoin::secp256k1::SecretKey::from_slice(&seed)
        .map_err(|e| PyValueError::new_err(format!("error decoding secret: {}", e)))?;
    let res = decrypt_with_seed(bytes, &seed)
        .map_err(|e| PyValueError::new_err(format!("error decrypting: {}", e)))?;

    Ok(res[..].into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn glclient(_py: Python, m: &PyModule) -> PyResult<()> {
    env_logger::init();
    m.add_class::<Signer>()?;
    m.add_class::<SignerHandle>()?;
    m.add_class::<Node>()?;
    m.add_class::<Scheduler>()?;
    m.add_class::<TlsConfig>()?;
    m.add_class::<LspClient>()?;
    m.add_class::<credentials::Credentials>()?;

    m.add_function(wrap_pyfunction!(backup_decrypt_with_seed, m)?)?;

    Ok(())
}
