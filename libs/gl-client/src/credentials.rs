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
use std::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
    path::Path,
};
use thiserror;

const CRED_VERSION: u32 = 1u32;

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

/// Credentials is a superset of sets that contain relevant credentials
/// like keys, certificates, etc. There are two possible variants:
/// - `Credentials::Nobody`: Contains no device specific credentials but
///                          is allowed to communicate with the backend.
///                          This variant can not communicate with a node.
/// - `Credentials::Device`: Contains device specific credentials that
///                          allow to communicate with the node that
///                          belongs to this device.
#[derive(Clone)]
pub enum Credentials {
    Nobody(Nobody),
    Device(Device),
}

impl Credentials {
    /// Creates and returns a `TlsConfig` that can be used to establish
    /// a connection to the services.
    pub fn tls_config(&self) -> Result<TlsConfig> {
        match self {
            Self::Nobody(c) => c.tls_config(),
            Self::Device(c) => c.tls_config(),
        }
    }

    /// Is a NoOp if the variant that the function is called on is
    /// `Credentials::Nobody`. Returns an error otherwise.
    pub fn is_nobody(&self) -> Result<()> {
        match self {
            Credentials::Nobody(_) => Ok(()),
            _ => Err(Error::IsIdentityError(self.to_string())),
        }
    }

    /// Is a NoOp if the variant that the function is called on is
    /// `Credentials::Device`. Returns an error otherwise.
    pub fn is_device(&self) -> Result<()> {
        match self {
            Credentials::Device(_) => Ok(()),
            _ => Err(Error::IsIdentityError(self.to_string())),
        }
    }

    /// Returns the byte encoded credentials when called on a Device.
    /// Returns an error otherwise.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Credentials::Nobody(_) => Err(Error::IsIdentityError(
                "can not return bytes for nobody".to_string(),
            )),
            Credentials::Device(d) => Ok(d.to_bytes()),
        }
    }
}

impl std::fmt::Display for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Credentials::Nobody(_) => "Nobody Credentials",
            Credentials::Device(_) => "Device Credentials",
        };
        f.write_str(str)
    }
}

impl TryInto<TlsConfig> for Credentials {
    type Error = Error;

    fn try_into(self) -> std::prelude::v1::Result<TlsConfig, Self::Error> {
        self.tls_config()
    }
}

/// A helper struct to combine the Tls certificate and the corresponding private
/// key.
#[derive(Clone, Debug, Default)]
struct Identity {
    cert: Vec<u8>,
    key: Vec<u8>,
}

/// `Builder<T>` is a flexible and generic structure for constructing different
/// types of `Credentials`, parameterized over `T`. It supports building
/// `Credentials` for the entities `Nobody` and `Device`, allowing for various
/// configurations through a fluent API.
///
/// # Examples
///
/// Creating `Nobody` credentials with default settings:
///
/// ```
/// use gl_client::credentials::Builder;
/// let nobody_credentials = Builder::as_nobody()
///     .with_default()
///     .expect("Failed to create default Nobody credentials")
///     .build()
///     .expect("Failed to build Nobody credentials");
/// ```
///
/// Creating `Device` credentials from a file:
///
/// ```no_run
/// use gl_client::credentials::Builder;
/// let device_credentials = Builder::as_device()
///     .from_path("path/to/credentials/file")
///     .expect("Failed to load credentials from file")
///     .build()
///     .expect("Failed to build Device credentials");
/// ```

#[derive(Clone, Debug, Default)]
pub struct Builder<T> {
    mark_type: PhantomData<T>,
    version: u32,
    identity: Option<Identity>,
    ca: Option<Vec<u8>>,
}

/// `Builder<Nobody>` implementation provides methods specific to constructing
/// `Credentials::Nobody`, which do not require a specific device configuration.
impl Builder<Nobody> {
    /// Creates a new `Builder<Nobody> instance with default configurations,
    /// is the entrypoint to create `Credentials::Nobody`.
    pub fn as_nobody() -> Builder<Nobody> {
        Builder::default()
    }

    /// Configures the builder with a default identity and CA certificate,
    /// specifically for "Nobody" credentials.
    pub fn with_default(mut self) -> Result<Self> {
        let (cert, key) =
            tls::default_nobody_identity().map_err(Error::FetchDefaultNobodyCredentials)?;
        let ca = tls::default_ca().map_err(Error::FetchDefaultNobodyCredentials)?;
        self.identity = Some(Identity { cert, key });
        self.ca = Some(ca);
        Ok(self)
    }

    /// Finalizes the builder, attempting to construct a `Credentials::Nobody`
    /// instance. Requires that an identity and a ca have been set; otherwise,
    /// it returns an error.
    pub fn build(self) -> Result<Credentials> {
        debug!("Building nobody credentials from {:?}", &self);

        if let Some(identity) = self.identity.clone() {
            Ok(Credentials::Nobody(Nobody {
                cert: identity.cert,
                key: identity.key,
                ca: self.own_ca_or_default()?,
            }))
        } else {
            Err(Error::BuildCredentialsError(
                "missing: identity".to_string(),
            ))
        }
    }
}

/// `Builder<Device>` implementation provides methods specific to constructing
/// `Credentials::Device`, which include device-specific configurations like
/// certificates, private keys, and, CA certificates.
impl Builder<Device> {
    /// Constructs a new `Builder<Device>` instance with default configurations,
    /// is the entrypoint to create `Credentials::Device`.
    pub fn as_device() -> Builder<Device> {
        Builder::default()
    }

    /// Configures the builder with credentials loaded from a byte array.
    pub fn from_bytes(mut self, data: impl AsRef<[u8]>) -> Result<Self> {
        let data = model::Data::try_from(data.as_ref())?;
        self.version = data.version;
        if let (Some(cert), Some(key)) = (data.cert, data.key) {
            self.identity = Some(Identity { cert, key });
        }
        self.ca = data.ca;
        Ok(self)
    }

    /// Configures the builder with credentials loaded from a specified file
    /// path.
    pub fn from_path(self, path: impl AsRef<Path>) -> Result<Self> {
        let buf = std::fs::read(path)?;
        self.from_bytes(&buf)
    }

    /// Asynchronously upgrades the credentials using the provided scheduler and
    /// signer, potentially involving network operations or other async tasks.
    pub async fn upgrade(mut self, scheduler: &Scheduler, signer: &Signer) -> Result<Self> {
        use Error::*;

        // For now, upgrade is covered by recover
        let res = scheduler
            .recover(signer)
            .await
            .map_err(|e| UpgradeCredentialsError(e.to_string()))?;
        let mut data = model::Data::try_from(&res.creds[..])
            .map_err(|e| UpgradeCredentialsError(e.to_string()))?;
        data.version = CRED_VERSION;

        if let model::Data {
            version,
            cert: Some(cert),
            key: Some(key),
            ca: Some(ca),
        } = data.clone()
        {
            self.version = version;
            self.identity = Some(Identity { cert, key });
            self.ca = Some(ca);
            Ok(self)
        } else {
            let mut missing = String::new();
            if data.cert.is_none() {
                add_missing(&mut missing, "certificate");
            }
            if data.key.is_none() {
                add_missing(&mut missing, "private key");
            }
            if data.ca.is_none() {
                add_missing(&mut missing, "ca certificate");
            }
            Err(Error::UpgradeCredentialsError(format!(
                "missing: {}",
                missing
            )))
        }
    }

    /// Finalizes the builder, attempting to construct a `Credentials::Device`
    /// instance. Requires that an identity, and a ca have been set;
    /// otherwise, it returns an error.
    pub fn build(self) -> Result<Credentials> {
        debug!("Building device credentials from {:?}", &self);

        if let Some(identity) = self.identity.clone() {
            Ok(Credentials::Device(Device {
                cert: identity.cert,
                key: identity.key,
                ca: self.own_ca_or_default()?,
            }))
        } else {
            let mut missing = String::new();
            if self.identity.is_none() {
                add_missing(&mut missing, "identity");
            }
            Err(Error::BuildCredentialsError(format!(
                "missing: {}",
                missing
            )))
        }
    }
}

impl<T> Builder<T> {
    /// Sets the identity for the credentials being built, consisting of a
    /// certificate and a private key.
    pub fn with_identity(mut self, cert: impl Into<Vec<u8>>, key: impl Into<Vec<u8>>) -> Self {
        self.identity = Some(Identity {
            cert: cert.into(),
            key: key.into(),
        });
        self
    }

    /// Sets the CA certificate for the credentials being built.
    pub fn with_ca(mut self, ca: impl Into<Vec<u8>>) -> Self {
        self.ca = Some(ca.into());
        self
    }

    /// Attempts to use the builder's CA certificate if set; otherwise,
    /// loads the default CA certificate.
    fn own_ca_or_default(self) -> Result<Vec<u8>> {
        if let Some(ca) = self.ca {
            return Ok(ca);
        }
        debug!("loading default CA certificate");
        tls::default_ca().map_err(Error::FetchDefaultNobodyCredentials)
    }
}

/// The `Nobody` credentials struct.
#[derive(Clone, Debug, Default)]
pub struct Nobody {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub ca: Vec<u8>,
}

impl Nobody {
    /// Returns the nobody tls identity based on the environmental
    /// variables used by the `tls` crate.
    pub fn tls_config(&self) -> Result<TlsConfig> {
        let tls = tls::TlsConfig::with(&self.cert, &self.key, &self.ca)
            .map_err(Error::CreateTlsConfigError)?;
        Ok(tls)
    }
}

/// The `Device` credentials store the device's certificate, the device's
/// private key, and the certificate authority.
#[derive(Clone, Debug, Default)]
pub struct Device {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub ca: Vec<u8>,
}

impl Device {
    /// Returns the device's tls identity based on the device's
    /// credentials.
    pub fn tls_config(&self) -> Result<TlsConfig> {
        let tls = tls::TlsConfig::with(&self.cert, &self.key, &self.ca)
            .map_err(Error::CreateTlsConfigError)?;
        // .identity(self.cert.clone(), self.key.clone());
        Ok(tls)
    }

    /// Returns a byte encoded representation of the credentials. This
    /// can be used to store the credentials in one single file.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_owned().into()
    }
}

impl Into<Vec<u8>> for Device {
    fn into(self) -> Vec<u8> {
        let data: model::Data = self.into();
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
        }
    }
}

/// Helper that appends to a string and adds a `,` if not the first.
fn add_missing(missing: &mut String, add: &str) -> () {
    if !missing.is_empty() {
        missing.push_str(", ")
    }
    missing.push_str(add);
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
    }

    impl TryFrom<&[u8]> for Data {
        type Error = super::Error;

        fn try_from(buf: &[u8]) -> std::prelude::v1::Result<Self, Self::Error> {
            let data: Data = Data::decode(buf)?;
            Ok(data)
        }
    }

    impl Into<Vec<u8>> for Data {
        fn into(self) -> Vec<u8> {
            self.encode_to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials() {
        // Assert an error if data is not complete.
        let c = Builder::as_device().build();
        assert!(c.is_err_and(|x| x.to_string() == "could not build credentials missing: identity"));

        // Check that we can use all the ways to build the credentials;
        if let Credentials::Nobody(nobody) = Builder::as_nobody()
            .with_default()
            .unwrap()
            .build()
            .unwrap()
        {
            // Check that we can build nobody credentials.
            let _n = Builder::as_nobody()
                .with_ca(nobody.ca.clone())
                .with_identity(nobody.cert.clone(), nobody.key.clone())
                .build()
                .unwrap();
            // Check that we can build device credentials.
            let d = Builder::as_device()
                .with_ca(nobody.ca)
                .with_identity(nobody.cert, nobody.key)
                .build()
                .unwrap();
            // Check that we can build from a valid data blob.
            let _c = Builder::as_device()
                .from_bytes(d.to_bytes().unwrap())
                .unwrap()
                .build()
                .unwrap();
        }
    }

    #[test]
    fn test_encode() {
        let cert: Vec<u8> = vec![99, 98];
        let key = vec![97, 96];
        let ca = vec![95, 94];
        let data = model::Data {
            version: 1,
            cert: Some(cert.clone()),
            key: Some(key.clone()),
            ca: Some(ca.clone()),
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
    }
}
