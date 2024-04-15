use anyhow::{anyhow, Result};
use gl_client::{
    credentials::{Device, Nobody, TlsConfigProvider},
    signer::Signer,
    util::get_node_id_from_tls_config,
};

pub trait CredentialExt {
    fn with_identity<V>(device_cert: V, device_key: V) -> Self
    where
        V: Into<Vec<u8>>;
}

impl CredentialExt for Nobody {
    fn with_identity<V>(device_cert: V, device_key: V) -> Self
    where
        V: Into<Vec<u8>>,
    {
        let mut creds = Nobody::default();
        creds.cert = device_cert.into();
        creds.key = device_key.into();
        creds
    }
}

impl CredentialExt for Device {
    fn with_identity<V>(device_cert: V, device_key: V) -> Self
    where
        V: Into<Vec<u8>>,
    {
        let mut creds = Device::default();
        creds.cert = device_cert.into();
        creds.key = device_key.into();
        creds
    }
}

pub trait DeviceExt {
    fn node_id(&self) -> Result<Vec<u8>>;
}

impl DeviceExt for Device {
    fn node_id(&self) -> Result<Vec<u8>> {
        get_node_id_from_tls_config(&self.tls_config())
    }
}

pub trait SignerExt {
    // I would name this create_default_rune but it might cause confusion
    // with the Default::default() used in the Device's default
    fn add_base_rune_to_device_credentials(&self, creds: Device) -> Result<Device>;
}

//TODO: Delete after Device::upgrade is made idempotent
impl SignerExt for Signer {
    fn add_base_rune_to_device_credentials(&self, mut creds: Device) -> Result<Device> {
        if creds.rune != String::default() {
            return Err(anyhow!("A rune has already been set for these credentials"));
        }

        let alt = runeauth::Alternative::new(
            "pubkey".to_string(),
            runeauth::Condition::Equal,
            hex::encode(self.node_id()),
            false,
        )
        .unwrap();
        creds.rune = self.create_rune(None, vec![vec![&alt.encode()]]).unwrap();
        Ok(creds)
    }
}
