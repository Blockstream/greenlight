use std::time::{SystemTime, UNIX_EPOCH};

use super::{into_approve_pairing_error, into_verify_pairing_data_error, Error};
use crate::{
    credentials::{RuneProvider, TlsConfigProvider},
    pb::{
        self,
        scheduler::{
            pairing_client::PairingClient, ApprovePairingRequest, GetPairingDataRequest,
            GetPairingDataResponse,
        },
    },
    tls::TlsConfig,
};
use bytes::BufMut as _;
use picky::{pem::Pem, x509::Csr};
use picky_asn1_x509::{PublicKey, SubjectPublicKeyInfo};
use ring::{
    rand,
    signature::{self, EcdsaKeyPair, KeyPair},
};

use rustls_pemfile as pemfile;
use tonic::transport::Channel;

type Result<T, E = super::Error> = core::result::Result<T, E>;

pub struct Connected(PairingClient<Channel>);
pub struct Unconnected();

pub struct Client<T> {
    inner: T,
    tls: TlsConfig,
    uri: String,
    rune: String,
    key: Vec<u8>,
}

impl Client<Unconnected> {
    pub fn new<C>(creds: C) -> Result<Client<Unconnected>>
    where
        C: TlsConfigProvider + RuneProvider,
    {
        let tls = creds.tls_config();
        let rune = creds.rune();
        let key = tls
            .clone()
            .private_key
            .ok_or(Error::BuildClientError("empty tls private key".to_string()))?;
        Ok(Self {
            inner: Unconnected(),
            tls,
            uri: crate::utils::scheduler_uri(),
            rune,
            key,
        })
    }

    pub fn with_uri(mut self, uri: String) -> Client<Unconnected> {
        self.uri = uri;
        self
    }

    pub async fn connect(self) -> Result<Client<Connected>> {
        let channel = tonic::transport::Endpoint::from_shared(self.uri.clone())?
            .tls_config(self.tls.inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let inner = PairingClient::new(channel);

        Ok(Client {
            inner: Connected(inner),
            tls: self.tls,
            uri: self.uri,
            rune: self.rune,
            key: self.key,
        })
    }
}

impl Client<Connected> {
    pub async fn get_pairing_data(&self, session_id: &str) -> Result<GetPairingDataResponse> {
        Ok(self
            .inner
            .0
            .clone()
            .get_pairing_data(GetPairingDataRequest {
                session_id: session_id.to_string(),
            })
            .await?
            .into_inner())
    }

    pub async fn approve_pairing(
        &self,
        session_id: &str,
        node_id: &[u8],
        device_name: &str,
        restrs: &str,
    ) -> Result<pb::greenlight::Empty> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(into_approve_pairing_error)?
            .as_secs();

        // Gather data to sign over.
        let mut buf = vec![];
        buf.put(session_id.as_bytes());
        buf.put_u64(timestamp);
        buf.put(&node_id[..]);
        buf.put(device_name.as_bytes());
        buf.put(restrs.as_bytes());

        // Sign data.
        let key = {
            let mut key = std::io::Cursor::new(&self.key);
            pemfile::pkcs8_private_keys(&mut key)
                .map_err(into_approve_pairing_error)?
                .remove(0)
        };
        let kp =
            EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, key.as_ref())
                .map_err(into_approve_pairing_error)?;
        let rng = rand::SystemRandom::new();
        let sig = kp
            .sign(&rng, &buf)
            .map_err(into_approve_pairing_error)?
            .as_ref()
            .to_vec();

        // Send approval.
        Ok(self
            .inner
            .0
            .clone()
            .approve_pairing(ApprovePairingRequest {
                session_id: session_id.to_string(),
                timestamp,
                device_name: device_name.to_string(),
                restrictions: restrs.to_string(),
                sig: sig,
                rune: self.rune.clone(),
                pubkey: kp.public_key().as_ref().to_vec(),
            })
            .await?
            .into_inner())
    }

    pub fn verify_pairing_data(data: GetPairingDataResponse) -> Result<()> {
        let mut crs = std::io::Cursor::new(&data.csr);
        let pem = Pem::read_from(&mut crs).map_err(into_verify_pairing_data_error)?;
        let csr = Csr::from_pem(&pem).map_err(into_verify_pairing_data_error)?;
        let sub_pk_der = csr
            .public_key()
            .to_der()
            .map_err(into_verify_pairing_data_error)?;
        let sub_pk_info: SubjectPublicKeyInfo =
            picky_asn1_der::from_bytes(&sub_pk_der).map_err(into_verify_pairing_data_error)?;

        if let PublicKey::Ec(bs) = sub_pk_info.subject_public_key {
            let pk = hex::encode(bs.0.payload_view());

            if pk == data.session_id
                && Self::restriction_contains_pubkey_exactly_once(
                    &data.restrictions,
                    &data.session_id,
                )
            {
                Ok(())
            } else {
                Err(Error::VerifyPairingDataError(format!(
                    "public key {} does not match pk {}",
                    data.session_id, pk
                )))
            }
        } else {
            Err(Error::VerifyPairingDataError(format!(
                "public key is not ecdsa"
            )))
        }
    }

    /// Checks that a restriction string only contains a pubkey field exactly
    /// once that is not preceded or followed by a '|' to ensure that it is
    /// not part of an alternative but a restriction by itself.
    fn restriction_contains_pubkey_exactly_once(s: &str, pubkey: &str) -> bool {
        let search_field = format!("pubkey={}", pubkey);
        match s.find(&search_field) {
            Some(index) => {
                // Check if 'pubkey=<pubkey>' is not preceded by '|'
                if index > 0 && s.chars().nth(index - 1) == Some('|') {
                    return false;
                }

                // Check if 'pubkey=<pubkey>' is not followed by '|'
                let end_index = index + search_field.len();
                if end_index < s.len() && s.chars().nth(end_index) == Some('|') {
                    return false;
                }

                // Check if 'pubkey=<pubkey>' appears exactly once
                s.matches(&search_field).count() == 1
            }
            None => false,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tls;

    #[test]
    fn test_verify_pairing_data() {
        let kp = tls::generate_ecdsa_key_pair();
        let device_cert = tls::generate_self_signed_device_cert(
            &hex::encode("00"),
            "my-device",
            vec!["localhost".into()],
            Some(kp),
        );
        let csr = device_cert.serialize_request_pem().unwrap();
        let pk = hex::encode(device_cert.get_key_pair().public_key_raw());

        // Check with public key as session id.
        let pd = GetPairingDataResponse {
            session_id: pk.clone(),
            csr: csr.clone().into_bytes(),
            device_name: "my-device".to_string(),
            description: "".to_string(),
            restrictions: format!("pubkey={}", pk.clone()),
        };
        assert!(Client::verify_pairing_data(pd).is_ok());

        // Check with different "pubkey" restriction than session id.
        let pd = GetPairingDataResponse {
            session_id: pk.clone(),
            csr: csr.clone().into_bytes(),
            device_name: "my-device".to_string(),
            description: "".to_string(),
            restrictions: format!("pubkey={}", "02000000"),
        };
        assert!(Client::verify_pairing_data(pd).is_err());

        // Check with second "pubkey" in same alternative.
        let pd = GetPairingDataResponse {
            session_id: pk.clone(),
            csr: csr.clone().into_bytes(),
            device_name: "my-device".to_string(),
            description: "".to_string(),
            restrictions: format!("pubkey={}|pubkey=02000000", pk),
        };
        assert!(Client::verify_pairing_data(pd).is_err());

        // Check with different public key as session id.
        let pd = GetPairingDataResponse {
            session_id: "00".to_string(),
            csr: csr.into_bytes(),
            device_name: "my-device".to_string(),
            description: "".to_string(),
            restrictions: format!("pubkey={}", pk.clone()),
        };
        assert!(Client::verify_pairing_data(pd).is_err());
    }
}
