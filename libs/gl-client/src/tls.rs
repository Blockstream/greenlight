use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use tonic::transport::{Certificate, ClientTlsConfig, Identity};
use x509_certificate::X509Certificate;

const CA_RAW: &[u8] = include_str!("../.resources/tls/ca.pem").as_bytes();
const NOBODY_CRT: &[u8] = include_str!(env!("GL_NOBODY_CRT")).as_bytes();
const NOBODY_KEY: &[u8] = include_str!(env!("GL_NOBODY_KEY")).as_bytes();

/// In order to allow the clients to talk to the
/// [`crate::scheduler::Scheduler`] a default certificate and private
/// key is included in this crate. The only service endpoints that can
/// be contacted with this `NOBODY` identity are
/// [`Scheduler.register`] and [`Scheduler.recover`], as these are the
/// endpoints that are used to prove ownership of a node, and
/// returning valid certificates if that proof succeeds.
#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub(crate) inner: ClientTlsConfig,

    /// Copy of the private key in the TLS identity. Stored here in
    /// order to be able to use it in the `AuthLayer`.
    pub(crate) private_key: Option<Vec<u8>>,

    pub ca: Vec<u8>,

    /// The device_crt parsed as an x509 certificate. Used to
    /// validate the common subject name against the node_id
    /// configured on the scheduler.
    pub x509_cert: Option<X509Certificate>,
}

/// Tries to load nobody credentials from a file that is passed by an envvar and
/// defaults to the nobody cert and key paths that have been set during build-
/// time.
fn load_file_or_default(varname: &str, default: &[u8]) -> Vec<u8> {
    match std::env::var(varname) {
        Ok(fname) => {
            debug!("Loading file {} for envvar {}", fname, varname);
            let f = std::fs::read(fname.clone());
            if f.is_err() {
                debug!(
                    "Could not find file {} for var {}, loading from default",
                    fname, varname
                );
                default.to_vec()
            } else {
                f.unwrap()
            }
        }
        Err(_) => default.to_vec(),
    }
}

impl TlsConfig {
    pub fn new() -> Self {
        debug!("Configuring TlsConfig with nobody identity");
        let nobody_crt = load_file_or_default("GL_NOBODY_CRT", NOBODY_CRT);
        let nobody_key = load_file_or_default("GL_NOBODY_KEY", NOBODY_KEY);
        let ca_crt = load_file_or_default("GL_CA_CRT", CA_RAW);
        // it is ok to panic here in case of a broken nobody certificate.
        // We can not do anything at all and should fail loudly!
        Self::with(nobody_crt, nobody_key, ca_crt)
    }

    pub fn with<V: AsRef<[u8]>>(crt: V, key: V, ca_crt: V) -> Self {
        let x509_cert = x509_certificate_from_pem_or_none(&crt);

        let config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(ca_crt.as_ref()))
            .identity(Identity::from_pem(crt, key.as_ref()));

        TlsConfig {
            inner: config,
            private_key: Some(key.as_ref().to_vec()),
            ca: ca_crt.as_ref().to_vec(),
            x509_cert,
        }
    }
}

impl TlsConfig {
    /// This function is used to upgrade the anonymous `NOBODY`
    /// configuration to a fully authenticated configuration.
    ///
    /// Only non-`NOBODY` configurations are able to talk to their
    /// nodes. If the `TlsConfig` is not upgraded, nodes will reply
    /// with handshake failures, and abort the connection attempt.
    pub fn identity(self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Self {
        let x509_cert = x509_certificate_from_pem_or_none(&cert_pem);

        TlsConfig {
            inner: self.inner.identity(Identity::from_pem(&cert_pem, &key_pem)),
            private_key: Some(key_pem),
            x509_cert,
            ..self
        }
    }

    /// Upgrades the connection using an identity based on a certificate
    /// and key from a path.
    ///
    /// The path is a directory that contains a `client.crt` and
    /// a `client-key.pem`-file which contain respectively the certificate
    /// and private key.
    pub fn identity_from_path<P: AsRef<Path>>(self, path: P) -> Result<Self> {
        let cert_path = path.as_ref().join("client.crt");
        let key_path = path.as_ref().join("client-key.pem");

        let cert_pem = std::fs::read(cert_path.clone())
            .with_context(|| format!("Failed to read '{}'", cert_path.display()))?;
        let key_pem = std::fs::read(key_path.clone())
            .with_context(|| format!("Failed to read '{}", key_path.display()))?;

        Ok(self.identity(cert_pem, key_pem))
    }

    /// This function is mostly used to allow running integration
    /// tests against a local mock of the service. It should not be
    /// used in production, since the preconfigured CA ensures that
    /// only the greenlight production servers can complete a valid
    /// handshake.
    pub fn ca_certificate(self, ca: Vec<u8>) -> Self {
        TlsConfig {
            inner: self.inner.ca_certificate(Certificate::from_pem(&ca)),
            ca,
            ..self
        }
    }

    pub fn client_tls_config(&self) -> ClientTlsConfig {
        self.inner.clone()
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper that returns an Option that contains a `X509Certificate`
/// if it could be parsed from the given `pem` data or None if it could
/// not be parsed. Logs a failed attempt.
fn x509_certificate_from_pem_or_none(pem: impl AsRef<[u8]>) -> Option<X509Certificate> {
    X509Certificate::from_pem(pem)
        .map_err(|e| debug!("Failed to parse x509 certificate: {}", e))
        .ok()
}

/// Generate a new device certificate from a fresh set of keys. The path in the
/// common name (CN) field is "/users/{node_id}/{device}". This certificate is
/// self signed and needs to be signed off by the users certificate authority to
/// be valid. This certificate can not act as a ca and sign sub certificates.
/// It can take an optional key pair to create the certificate from instead of
/// generating a key pair from scratch.
pub fn generate_self_signed_device_cert(
    node_id: &str,
    device: &str,
    subject_alt_names: Vec<String>,
    key_pair: Option<rcgen::KeyPair>,
) -> DeviceCert {
    // Start from a fresh key pair unless one was handed to us.
    let key_pair =
        key_pair.unwrap_or_else(|| rcgen::KeyPair::generate().expect("could not generate keypair"));

    // Configure the certificate.
    let mut params = cert_params_from_template(subject_alt_names);

    // Is a leaf certificate only so it is not allowed to sign child
    // certificates.
    params.is_ca = rcgen::IsCa::ExplicitNoCa;
    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        format!("/users/{}/{}", node_id, device),
    );

    // self_signed consumes the params, but we keep them around to be
    // able to generate the registration CSR later.
    let cert = params
        .clone()
        .self_signed(&key_pair)
        .expect("could not self-sign the device certificate");

    // A CSR cannot carry the CA flag at all, and rcgen refuses to
    // serialize one with an explicit is_ca setting. The leaf-ness is
    // enforced by the signing profile on the scheduler side.
    params.is_ca = rcgen::IsCa::NoCa;

    DeviceCert {
        params,
        key_pair,
        cert,
    }
}

/// A self-signed device certificate bundled with the key pair it was
/// created for. rcgen 0.13 no longer keeps the key pair inside the
/// certificate, so we carry both plus the parameters that are needed
/// to produce the registration CSR.
pub struct DeviceCert {
    params: rcgen::CertificateParams,
    key_pair: rcgen::KeyPair,
    cert: rcgen::Certificate,
}

impl DeviceCert {
    pub fn get_key_pair(&self) -> &rcgen::KeyPair {
        &self.key_pair
    }

    /// PEM of the PKCS#10 certificate signing request, signed with the
    /// device key, as sent to the scheduler during registration.
    pub fn serialize_request_pem(&self) -> Result<String, rcgen::Error> {
        Ok(self.params.serialize_request(&self.key_pair)?.pem()?)
    }

    pub fn serialize_pem(&self) -> Result<String, rcgen::Error> {
        Ok(self.cert.pem())
    }

    pub fn serialize_private_key_pem(&self) -> String {
        self.key_pair.serialize_pem()
    }
}

fn cert_params_from_template(subject_alt_names: Vec<String>) -> rcgen::CertificateParams {
    let mut params = rcgen::CertificateParams::new(subject_alt_names)
        .expect("could not create certificate parameters");

    // Certificate can be used to issue unlimited sub certificates for devices.
    params
        .distinguished_name
        .push(rcgen::DnType::CountryName, "US");
    params
        .distinguished_name
        .push(rcgen::DnType::LocalityName, "SAN FRANCISCO");
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationName, "Blockstream");
    params
        .distinguished_name
        .push(rcgen::DnType::StateOrProvinceName, "CALIFORNIA");
    params.distinguished_name.push(
        rcgen::DnType::OrganizationalUnitName,
        "CertificateAuthority",
    );

    params
}

pub fn generate_ecdsa_key_pair() -> rcgen::KeyPair {
    rcgen::KeyPair::generate().unwrap()
}

#[cfg(test)]
pub mod tests {
    use rcgen::KeyPair;

    use super::*;

    #[test]
    fn test_generate_self_signed_device_cert() {
        let device_cert =
            generate_self_signed_device_cert("mynodeid", "device", vec!["localhost".into()], None);
        assert!(device_cert
            .serialize_pem()
            .unwrap()
            .starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(device_cert
            .serialize_private_key_pem()
            .starts_with("-----BEGIN PRIVATE KEY-----"));
    }

    #[test]
    fn test_generate_self_signed_device_cert_from_pem() {
        let kp = generate_ecdsa_key_pair();
        let keys = KeyPair::from_pem(&kp.serialize_pem()).unwrap();
        let cert = generate_self_signed_device_cert(
            "mynodeid",
            "device",
            vec!["localhost".into()],
            Some(keys),
        );
        assert!(kp.serialize_pem() == cert.get_key_pair().serialize_pem());
    }
}
