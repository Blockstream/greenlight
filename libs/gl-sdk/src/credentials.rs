use crate::Error;
use gl_client::credentials::Device as DeviceCredentials;

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
