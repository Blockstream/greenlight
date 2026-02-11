use dirs;
use gl_client::credentials;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use thiserror;

pub const SEED_FILE_NAME: &str = "hsm_secret";
pub const CREDENTIALS_FILE_NAME: &str = "credentials.gfs";
pub const DEFAULT_GREENLIGHT_DIR: &str = "greenlight";

// -- Seed section

pub fn generate_seed(words: Option<String>) -> Result<([u8; 32], bip39::Mnemonic)> {
    let mnemonic = match words {
        Some(sentence) => bip39::Mnemonic::parse(sentence)?,
        None => bip39::Mnemonic::generate(12)?,
    };
    let n = mnemonic.word_count();
    if n != 12 {
        return Err(UtilsError::custom(format!(
            "Mnemonic contains {n} words, but 12 were expected."
        )));
    }
    let seed: [u8; 32] = mnemonic.to_seed("")[0..32].try_into()?;
    Ok((seed, mnemonic))
}

pub fn read_seed(file_path: impl AsRef<Path>) -> Option<Vec<u8>> {
    fs::read(file_path).ok()
}

pub fn write_seed(file_path: impl AsRef<Path>, seed: impl AsRef<[u8]>) -> Result<()> {
    let file_path = file_path.as_ref();
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(file_path)?;
    file.write_all(seed.as_ref())?;
    file.sync_all()?;

    Ok(())
}

// -- Credentials section

pub fn write_credentials(file_path: impl AsRef<Path>, creds: impl AsRef<[u8]>) -> Result<()> {
    let mut file = File::create(&file_path)?;
    file.write_all(creds.as_ref())?;
    file.sync_all()?;

    Ok(())
}

pub fn read_credentials(file_path: impl AsRef<Path>) -> Option<credentials::Device> {
    let cred_data = fs::read(file_path).ok();
    if let Some(data) = cred_data {
        let creds = credentials::Device::from_bytes(data);
        return Some(creds);
    }
    None
}

// -- Misc

pub struct DataDir(pub PathBuf);

impl core::default::Default for DataDir {
    fn default() -> Self {
        let data_dir = dirs::data_dir().unwrap().join(DEFAULT_GREENLIGHT_DIR);
        Self(data_dir)
    }
}

impl AsRef<Path> for DataDir {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

// -- Error implementations

#[derive(thiserror::Error, core::fmt::Debug)]
pub enum UtilsError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    MnemonicError(#[from] bip39::Error),
    #[error(transparent)]
    DataError(#[from] std::array::TryFromSliceError),
    #[error("{0}")]
    Custom(String),
}

impl UtilsError {
    pub fn custom(e: impl std::fmt::Display) -> UtilsError {
        UtilsError::Custom(e.to_string())
    }
}

type Result<T, E = UtilsError> = core::result::Result<T, E>;
