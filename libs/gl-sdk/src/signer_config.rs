use crate::Error;
use gl_client::signer::{DeviceStorage, FileDeviceStorage};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Enum)]
pub enum StateSignatureMode {
    /// Disable signature checks for development and testing only.
    Off,
    /// Allow one successful migration, then use Hard mode.
    Soft,
    #[default]
    Hard,
}

impl From<StateSignatureMode> for gl_client::signer::StateSignatureMode {
    fn from(mode: StateSignatureMode) -> Self {
        match mode {
            StateSignatureMode::Off => Self::Off,
            StateSignatureMode::Soft => Self::Soft,
            StateSignatureMode::Hard => Self::Hard,
        }
    }
}

/// State verification and storage settings for SDK signers.
#[derive(Clone, Debug, Default, uniffi::Record)]
pub struct SignerConfig {
    pub state_signature_mode: StateSignatureMode,
    /// Persistent directory shared across restarts, containing signer metadata. Required in Hard and Soft modes.
    pub device_storage_path: Option<String>,
}

impl SignerConfig {
    pub(crate) fn to_client(&self) -> Result<gl_client::signer::SignerConfig, Error> {
        let device_storage: Option<Arc<dyn DeviceStorage>> = match self.state_signature_mode {
            StateSignatureMode::Hard | StateSignatureMode::Soft => {
                let path = self
                    .device_storage_path
                    .as_deref()
                    .filter(|path| !path.is_empty())
                    .ok_or_else(|| {
                        Error::argument("device_storage_path", "required for Hard and Soft modes")
                    })?;
                Some(Arc::new(FileDeviceStorage::new(path).map_err(|e| {
                    Error::other(format!("opening device storage: {e:#}"))
                })?))
            }
            StateSignatureMode::Off => None,
        };
        Ok(gl_client::signer::SignerConfig {
            state_signature_mode: self.state_signature_mode.into(),
            device_storage,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_opens_storage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("unused");
        let config = SignerConfig {
            device_storage_path: Some(path.to_str().unwrap().to_string()),
            ..Default::default()
        }
        .to_client()
        .unwrap();
        assert_eq!(
            config.state_signature_mode,
            gl_client::signer::StateSignatureMode::Hard
        );
        assert!(config.device_storage.is_some());
        assert!(path.is_dir());
    }

    #[test]
    fn hard_and_soft_require_a_nonempty_storage_path() {
        for mode in [StateSignatureMode::Hard, StateSignatureMode::Soft] {
            for path in [None, Some(String::new())] {
                let err = SignerConfig {
                    state_signature_mode: mode,
                    device_storage_path: path,
                }
                .to_client()
                .unwrap_err();
                assert!(matches!(err, Error::Argument { .. }));
            }
        }
    }

    #[test]
    fn separate_configs_reopen_the_same_persistent_storage() {
        let dir = tempfile::tempdir().unwrap();
        let config = SignerConfig {
            state_signature_mode: StateSignatureMode::Soft,
            device_storage_path: Some(dir.path().to_str().unwrap().to_string()),
        };
        {
            let first = config.to_client().unwrap();
            assert_eq!(
                first.state_signature_mode,
                gl_client::signer::StateSignatureMode::Soft
            );
            first
                .device_storage
                .unwrap()
                .save("marker", b"used")
                .unwrap();
        }
        let reopened = config.to_client().unwrap();
        assert_eq!(
            reopened
                .device_storage
                .unwrap()
                .load("marker")
                .unwrap()
                .unwrap(),
            b"used"
        );
    }
}
