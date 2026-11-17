use super::DeviceStorage;
use crate::bitcoin::hashes::{sha256, Hash};
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

/// A file-backed device store using durable, atomic replacement.
#[derive(Debug)]
pub struct FileDeviceStorage {
    dir: PathBuf,
}

impl FileDeviceStorage {
    /// Open directory-backed storage, creating the directory when needed.
    pub fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_owned();
        create_storage_directory(&dir)
            .with_context(|| format!("creating device state directory {}", dir.display()))?;
        Ok(Self { dir })
    }

    fn path(&self, key: &str) -> PathBuf {
        // Keys are opaque logical identifiers and must never become paths.
        self.dir
            .join(sha256::Hash::hash(key.as_bytes()).to_string())
    }
}

fn create_storage_directory(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    create_storage_directory(parent)?;
    match fs::create_dir(path) {
        Ok(()) => (),
        Err(error) if error.kind() == ErrorKind::AlreadyExists && path.is_dir() => (),
        Err(error) => return Err(error),
    }
    // A file sync alone cannot make the newly created directory durable.
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
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
