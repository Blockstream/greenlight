use crate::Error;
use gl_client::credentials::Device as DeviceCredentials;

/// A developer certificate obtained from the Greenlight Developer
/// Console (GDC). When provided to a `Scheduler` via
/// `with_developer_cert()`, nodes registered through that scheduler
/// will be associated with the developer's account.
///
/// If no developer certificate is provided, the scheduler falls back
/// to the compiled-in default certificate, which may be sufficient
/// when using an invite code instead.
#[derive(uniffi::Object, Clone)]
pub struct DeveloperCert {
    pub(crate) inner: gl_client::credentials::Nobody,
}

#[uniffi::export]
impl DeveloperCert {
    /// Create a new `DeveloperCert` from the certificate and private
    /// key PEM bytes obtained from the Greenlight Developer Console.
    #[uniffi::constructor()]
    pub fn new(cert: Vec<u8>, key: Vec<u8>) -> Self {
        Self {
            inner: gl_client::credentials::Nobody::with(cert, key),
        }
    }
}

/// `Credentials` is a container for `node_id`, the mTLS client
/// certificate used to authenticate a client against a node, as well
/// as the seed secret if present. If no seed is present in the
/// credentials, then the `Client` will not start a signer in the
/// background.
#[derive(uniffi::Object, Clone)]
pub struct Credentials {
    pub(crate) inner: DeviceCredentials,
}

#[uniffi::export]
impl Credentials {
    #[uniffi::constructor()]
    pub fn load(raw: Vec<u8>) -> Result<Credentials, Error> {
        Ok(Self {
            inner: DeviceCredentials::from_bytes(raw),
        })
    }

    pub fn save(&self) -> Result<Vec<u8>, Error> {
        Ok(self.inner.to_bytes())
    }
}
