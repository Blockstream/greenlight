use crate::{
    scheduler::Scheduler,
    signer::Signer,
    tls::{self, TlsConfig},
};
/// Credentials is a collection of all relevant keys and attestations
/// required to authenticate a device and authorize a command on the node.
/// They represent the identity of a device and can be encoded into a byte
/// format for easy storage.
use log::debug;
use std::{convert::TryFrom, path::Path};
use thiserror;

const CRED_VERSION: u32 = 1u32;
const CA_RAW: &[u8] = include_str!("../../tls/ca.pem").as_bytes();
const NOBODY_CRT: &[u8] = include_str!(env!("GL_NOBODY_CRT")).as_bytes();
const NOBODY_KEY: &[u8] = include_str!(env!("GL_NOBODY_KEY")).as_bytes();

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not get from identity: {}", .0)]
    GetFromIdentityError(String),
    #[error("identity mismatch: {}", .0)]
    IsIdentityError(String),
    #[error("could not decode into credentials")]
    DecodeCredentialsError(#[from] prost::DecodeError),
    #[error("could not encode credentials")]
    EncodeCredentialError(#[from] prost::EncodeError),
    #[error("could not upgrade credentials: {}", .0)]
    UpgradeCredentialsError(String),
    #[error("could not build credentials {}", .0)]
    BuildCredentialsError(String),
    #[error("could not create create credentials from data: {}", .0)]
    TransformDataIntoCredentialsError(String),
    #[error("could not create tls config {}", .0)]
    CreateTlsConfigError(#[source] anyhow::Error),
    #[error("could not read from file: {}", .0)]
    ReadFromFileError(#[from] std::io::Error),
    #[error("could not fetch default nobody credentials: {}", .0)]
    FetchDefaultNobodyCredentials(#[source] anyhow::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait TlsConfigProvider: Send + Sync {
    fn tls_config(&self) -> TlsConfig;
}
pub trait RuneProvider {
    fn rune(&self) -> String;
}

/// A helper struct to combine the Tls certificate and the corresponding private
/// key.
#[derive(Clone, Debug)]
struct Identity {
    cert: Vec<u8>,
    key: Vec<u8>,
}

impl Default for Identity {
    fn default() -> Self {
        let key = load_file_or_default("GL_NOBODY_KEY", NOBODY_KEY)
            .expect("Could not load file from GL_NOBODY_KEY");
        let cert = load_file_or_default("GL_NOBODY_CRT", NOBODY_CRT)
            .expect("Could not load file from GL_NOBODY_CRT");
        Self { cert, key }
    }
}

/// The `Nobody` credentials struct. This is an unauthenticated set of
/// credentials and can only be used for registration and recovery.
#[derive(Clone, Debug)]
pub struct Nobody {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub ca: Vec<u8>,
}

impl Nobody {
    /// Returns a new Nobody instance with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new Nobody instance with a custom set of parameters.
    pub fn with<V>(cert: V, key: V, ca: V) -> Self
    where
        V: Into<Vec<u8>>,
    {
        Self {
            cert: cert.into(),
            key: key.into(),
            ca: ca.into(),
        }
    }
}

impl TlsConfigProvider for Nobody {
    fn tls_config(&self) -> TlsConfig {
        tls::TlsConfig::with(&self.cert, &self.key, &self.ca)
    }
}

impl Default for Nobody {
    fn default() -> Self {
        let ca =
            load_file_or_default("GL_CA_CRT", CA_RAW).expect("Could not load file from GL_CA_CRT");
        let identity = Identity::default();

        Self {
            cert: identity.cert,
            key: identity.key,
            ca,
        }
    }
}

/// The `Device` credentials store the device's certificate, the device's
/// private key, the certificate authority and the device's rune.
#[derive(Clone, Debug)]
pub struct Device {
    pub version: u32,
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub ca: Vec<u8>,
    pub rune: String,
}

impl Device {
    /// Creates a new set of `Device` credentials from the given
    /// credentials data blob. It defaults to the nobody credentials set.
    pub fn from_bytes(data: impl AsRef<[u8]>) -> Self {
        let mut creds = Self::default();
        debug!("Build authenticated credentials from: {:?}", data.as_ref());
        if let Ok(data) = model::Data::try_from(data.as_ref()) {
            creds.version = data.version;
            if let Some(cert) = data.cert {
                creds.cert = cert
            }
            if let Some(key) = data.key {
                creds.key = key
            }
            if let Some(ca) = data.ca {
                creds.ca = ca
            }
            if let Some(rune) = data.rune {
                creds.rune = rune
            }
        }
        creds
    }

    /// Creates a new set of `Device` credentials from a path that
    /// contains a credentials data blob. Defaults to the nobody
    /// credentials set.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        debug!("Read credentials data from {:?}", path.as_ref());
        let data = std::fs::read(path).unwrap_or_default();
        Device::from_bytes(data)
    }

    /// Creates a new set of `Device` credentials from a complete set of
    /// credentials.
    pub fn with<V, S>(cert: V, key: V, ca: V, rune: S) -> Self
    where
        V: Into<Vec<u8>>,
        S: Into<String>,
    {
        Self {
            version: CRED_VERSION,
            cert: cert.into(),
            key: key.into(),
            ca: ca.into(),
            rune: rune.into(),
        }
    }

    /// Asynchronously upgrades the credentials using the provided scheduler and
    /// signer, potentially involving network operations or other async tasks.
    pub async fn upgrade<T>(mut self, scheduler: &Scheduler<T>, signer: &Signer) -> Result<Self>
    where
        T: TlsConfigProvider,
    {
        use Error::*;

        // For now, upgrade is covered by recover
        let res = scheduler
            .recover(signer)
            .await
            .map_err(|e| UpgradeCredentialsError(e.to_string()))?;
        let mut data = model::Data::try_from(&res.creds[..])
            .map_err(|e| UpgradeCredentialsError(e.to_string()))?;
        data.version = CRED_VERSION;
        if let Some(cert) = data.cert {
            self.cert = cert
        }
        if let Some(key) = data.key {
            self.key = key
        }
        if let Some(ca) = data.ca {
            self.ca = ca
        }
        if let Some(rune) = data.rune {
            self.rune = rune
        };
        Ok(self)
    }

    /// Returns a byte encoded representation of the credentials. This
    /// can be used to store the credentials in one single file.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_owned().into()
    }
}

impl TlsConfigProvider for Device {
    fn tls_config(&self) -> TlsConfig {
        tls::TlsConfig::with(&self.cert, &self.key, &self.ca)
    }
}

impl RuneProvider for Device {
    fn rune(&self) -> String {
        self.to_owned().rune
    }
}

impl From<Device> for Vec<u8> {
    fn from(value: Device) -> Self {
        let data: model::Data = value.into();
        data.into()
    }
}

impl From<Device> for model::Data {
    fn from(device: Device) -> Self {
        model::Data {
            version: CRED_VERSION,
            cert: Some(device.cert),
            key: Some(device.key),
            ca: Some(device.ca),
            rune: Some(device.rune),
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        let ca =
            load_file_or_default("GL_CA_CRT", CA_RAW).expect("Could not load file from GL_CA_CRT");
        let identity = Identity::default();
        Self {
            version: 0,
            cert: identity.cert,
            key: identity.key,
            ca,
            rune: Default::default(),
        }
    }
}

mod model {
    use prost::Message;
    use std::convert::TryFrom;

    /// The Data struct is used for encoding and decoding of credentials. It
    /// useses proto for byte encoding.
    #[derive(Message, Clone)]
    pub struct Data {
        #[prost(uint32, tag = "1")]
        pub version: u32,
        #[prost(bytes, optional, tag = "2")]
        pub cert: Option<Vec<u8>>,
        #[prost(bytes, optional, tag = "3")]
        pub key: Option<Vec<u8>>,
        #[prost(bytes, optional, tag = "4")]
        pub ca: Option<Vec<u8>>,
        #[prost(string, optional, tag = "5")]
        pub rune: Option<String>,
    }

    impl TryFrom<&[u8]> for Data {
        type Error = super::Error;

        fn try_from(buf: &[u8]) -> std::prelude::v1::Result<Self, Self::Error> {
            let data: Data = Data::decode(buf)?;
            Ok(data)
        }
    }

    impl From<Data> for Vec<u8> {
        fn from(value: Data) -> Self {
            value.encode_to_vec()
        }
    }
}

/// Tries to load nobody credentials from a file that is passed by an envvar and
/// defaults to the nobody cert and key paths that have been set during build-
/// time.
fn load_file_or_default(varname: &str, default: &[u8]) -> Result<Vec<u8>> {
    match std::env::var(varname) {
        Ok(fname) => {
            debug!("Loading file {} for envvar {}", fname, varname);
            let f = std::fs::read(fname.clone())?;
            Ok(f)
        }
        Err(_) => Ok(default.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let cert: Vec<u8> = vec![99, 98];
        let key = vec![97, 96];
        let ca = vec![95, 94];
        let rune = "non_functional_rune".to_string();
        let data = model::Data {
            version: 1,
            cert: Some(cert.clone()),
            key: Some(key.clone()),
            ca: Some(ca.clone()),
            rune: Some(rune.clone()),
        };
        let buf: Vec<u8> = data.into();
        print!("{:?}", buf);
        for n in cert {
            assert!(buf.contains(&n));
        }
        for n in key {
            assert!(buf.contains(&n));
        }
        for n in ca {
            assert!(buf.contains(&n));
        }
        for n in rune.as_bytes() {
            assert!(buf.contains(n));
        }
    }

    #[test]
    fn test_decode() {
        let data: Vec<u8> = vec![
            8, 1, 18, 2, 99, 98, 26, 2, 97, 96, 34, 2, 95, 94, 42, 19, 110, 111, 110, 95, 102, 117,
            110, 99, 116, 105, 111, 110, 97, 108, 95, 114, 117, 110, 101,
        ];
        let data = model::Data::try_from(&data[..]).unwrap();
        assert!(data.version == 1);
        assert!(data.cert.is_some_and(|d| d == vec![99, 98]));
        assert!(data.key.is_some_and(|d| d == vec![97, 96]));
        assert!(data.ca.is_some_and(|d| d == vec![95, 94]));
        assert!(data.rune.is_some_and(|d| d == *"non_functional_rune"));
    }
}
