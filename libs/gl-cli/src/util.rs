use dirs;
use gl_client::bitcoin::secp256k1::rand::{self, RngCore};
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

pub fn generate_seed() -> [u8; 32] {
    let mut seed = [0u8; 32];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut seed);
    seed
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
}

type Result<T, E = UtilsError> = core::result::Result<T, E>;
