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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_seed1() {
        let (_, mnemonic) = generate_seed(None).unwrap();
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn gen_seed2() {
        let sentence = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string();
        let expected_seed = [
            0x5e, 0xb0, 0x0b, 0xbd, 0xdc, 0xf0, 0x69, 0x08, 0x48, 0x89, 0xa8, 0xab, 0x91, 0x55,
            0x56, 0x81, 0x65, 0xf5, 0xc4, 0x53, 0xcc, 0xb8, 0x5e, 0x70, 0x81, 0x1a, 0xae, 0xd6,
            0xf6, 0xda, 0x5f, 0xc1,
        ];
        let (seed, mnemonic) = generate_seed(Some(sentence.clone())).unwrap();
        assert_eq!(seed, expected_seed);
        assert_eq!(mnemonic.to_string(), sentence);
    }

    #[test]
    fn gen_seed3() {
        // 0 words, invalid mnemonic
        let result = generate_seed(Some("".to_string()));
        assert!(result.is_err_and(|e| e.to_string().contains("invalid word count: 0")));

        // 11 words, invalid mnemonic
        let result = generate_seed(Some("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon".to_string()));
        assert!(result.is_err_and(|e| e.to_string().contains("invalid word count: 11")));

        // 15 words, valid mnemonic but we want 12 words
        let result = generate_seed(Some("birth danger dismiss bounce ostrich museum model glory depth seed clip pitch skull carpet myself".to_string()));
        assert!(result.is_err_and(|e| e
            .to_string()
            .contains("contains 15 words, but 12 were expected")));

        // 12 words, but invalid word at the end
        let result = generate_seed(Some("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon pizzzzza".to_string()));
        assert!(result.is_err_and(|e| e.to_string().contains("unknown word (word 11)")));

        // 12 words, but invalid checksum
        let result = generate_seed(Some("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon pizza".to_string()));
        assert!(result.is_err_and(|e| e.to_string().contains("invalid checksum")));
    }
}
