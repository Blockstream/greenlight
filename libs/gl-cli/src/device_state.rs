use anyhow::{Context, Result};
use gl_client::bitcoin::hashes::{sha256, Hash};
use gl_client::signer::DeviceStorage;
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

/// A file-backed device store using durable, atomic replacement.
#[derive(Debug)]
pub(crate) struct FileDeviceStorage {
    dir: PathBuf,
}

impl FileDeviceStorage {
    pub(crate) fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_owned();
        fs::create_dir_all(&dir)
            .with_context(|| format!("creating device state directory {}", dir.display()))?;
        Ok(Self { dir })
    }

    fn path(&self, key: &str) -> PathBuf {
        // Keys are opaque logical identifiers and must never become paths.
        self.dir
            .join(sha256::Hash::hash(key.as_bytes()).to_string())
    }
}

impl DeviceStorage for FileDeviceStorage {
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match fs::read(self.path(key)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).context("reading signer device state"),
        }
    }

    fn save(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut tmp = tempfile::NamedTempFile::new_in(&self.dir)
            .context("creating temporary device state file")?;
        tmp.write_all(value)
            .context("writing signer device state")?;
        tmp.as_file()
            .sync_all()
            .context("syncing signer device state")?;
        tmp.persist(self.path(key))
            .context("replacing signer device state")?;
        #[cfg(unix)]
        File::open(&self.dir)
            .and_then(|dir| dir.sync_all())
            .context("syncing device state directory")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_survive_reopening_and_replacement() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = FileDeviceStorage::new(dir.path()).unwrap();
            assert!(store.load("signer/regtest/node").unwrap().is_none());
            store.save("signer/regtest/node", b"old value").unwrap();
            store.save("signer/regtest/node", b"new").unwrap();
            store.save("signer/bitcoin/node", &[0, 255, 0]).unwrap();
        }
        let store = FileDeviceStorage::new(dir.path()).unwrap();
        assert_eq!(store.load("signer/regtest/node").unwrap().unwrap(), b"new");
        assert_eq!(
            store.load("signer/bitcoin/node").unwrap().unwrap(),
            [0, 255, 0]
        );
    }

    #[test]
    fn logical_keys_cannot_escape_the_storage_directory() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileDeviceStorage::new(dir.path()).unwrap();
        let key = "../../outside";
        store.save(key, b"value").unwrap();
        assert_eq!(store.path(key).parent(), Some(dir.path()));
        assert_eq!(store.load(key).unwrap().unwrap(), b"value");
    }

    #[test]
    fn read_errors_are_not_treated_as_missing_state() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileDeviceStorage::new(dir.path()).unwrap();
        fs::create_dir(store.path("key")).unwrap();
        assert!(store.load("key").is_err());
    }
}
