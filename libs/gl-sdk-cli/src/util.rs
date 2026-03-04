use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const PHRASE_FILE_NAME: &str = "hsm_secret";
pub const CREDENTIALS_FILE_NAME: &str = "credentials.gfs";
const DEFAULT_GREENLIGHT_DIR: &str = "greenlight";

pub struct DataDir(pub PathBuf);

impl Default for DataDir {
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

/// The secret stored in `hsm_secret` — either a BIP39 mnemonic
/// phrase (text) or raw seed bytes (gl-cli legacy format).
pub enum Secret {
    Phrase(String),
    Seed(Vec<u8>),
}

pub fn read_secret(data_dir: &DataDir) -> Result<Secret> {
    let path = data_dir.0.join(PHRASE_FILE_NAME);
    let raw = fs::read(&path).map_err(|_| {
        Error::PhraseNotFound(format!("could not read from {}", path.display()))
    })?;

    // Try UTF-8 mnemonic first (glsdk format)
    if let Ok(text) = std::str::from_utf8(&raw) {
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.contains(' ') {
            return Ok(Secret::Phrase(trimmed.to_string()));
        }
    }

    // Raw seed bytes (gl-cli legacy format)
    Ok(Secret::Seed(raw))
}

pub fn signer_from_secret(secret: Secret) -> Result<glsdk::Signer> {
    match secret {
        Secret::Phrase(phrase) => glsdk::Signer::new(phrase).map_err(Error::Sdk),
        Secret::Seed(seed) => glsdk::Signer::new_from_seed(seed).map_err(Error::Sdk),
    }
}

pub fn make_signer(data_dir: &DataDir) -> Result<glsdk::Signer> {
    signer_from_secret(read_secret(data_dir)?)
}

pub fn write_phrase(data_dir: &DataDir, phrase: &str) -> Result<()> {
    fs::create_dir_all(&data_dir.0)?;
    let path = data_dir.0.join(PHRASE_FILE_NAME);
    fs::write(&path, phrase)?;
    Ok(())
}

pub fn read_credentials(data_dir: &DataDir) -> Result<glsdk::Credentials> {
    let path = data_dir.0.join(CREDENTIALS_FILE_NAME);
    let raw = fs::read(&path).map_err(|_| {
        Error::CredentialsNotFound(format!("could not read from {}", path.display()))
    })?;
    glsdk::Credentials::load(raw).map_err(Error::Sdk)
}

pub fn write_credentials(data_dir: &DataDir, creds: &glsdk::Credentials) -> Result<()> {
    fs::create_dir_all(&data_dir.0)?;
    let path = data_dir.0.join(CREDENTIALS_FILE_NAME);
    let raw = creds.save()?;
    fs::write(&path, &raw)?;
    Ok(())
}

pub fn parse_network(s: &str) -> Result<glsdk::Network> {
    match s {
        "bitcoin" => Ok(glsdk::Network::BITCOIN),
        "regtest" => Ok(glsdk::Network::REGTEST),
        _ => Err(Error::Other(format!(
            "unsupported network: {s} (expected bitcoin or regtest)"
        ))),
    }
}
