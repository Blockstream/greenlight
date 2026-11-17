//! Storage for signer metadata kept on the device, independently of the node.

use anyhow::Result;
use std::fmt::Debug;

#[cfg(feature = "file-storage")]
mod file;
#[cfg(feature = "file-storage")]
pub use file::FileDeviceStorage;

/// Device-local storage for opaque signer metadata.
///
/// Keys are logical identifiers, not filesystem paths. The signer namespaces
/// them by node ID, network and purpose. The store must preserve their bytes
/// unchanged; each caller owns its value encoding.
///
/// A successful [`save`](Self::save) must atomically replace the value and
/// survive a crash. An interrupted save may leave either the old or the new
/// value, never a partial write. Missing values must be distinguished from read
/// failures.
///
/// Device storage is trusted locally. Deleting it or restoring an older copy
/// can weaken the protections built on it.
pub trait DeviceStorage: Debug + Send + Sync {
    /// Load a value, returning `None` only when the key is absent.
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Atomically replace a value and make it durable before returning success.
    ///
    /// An error may leave either the old or the new value stored. Callers must
    /// not assume that a failed save left the value unchanged.
    fn save(&self, key: &str, value: &[u8]) -> Result<()>;
}

#[cfg(test)]
#[derive(Debug, Default)]
pub(super) struct MemoryDeviceStorage {
    values: std::sync::Mutex<std::collections::BTreeMap<String, Vec<u8>>>,
}

#[cfg(test)]
impl DeviceStorage for MemoryDeviceStorage {
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.values.lock().unwrap().get(key).cloned())
    }

    fn save(&self, key: &str, value: &[u8]) -> Result<()> {
        self.values
            .lock()
            .unwrap()
            .insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}
