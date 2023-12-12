use std::time::{SystemTime, UNIX_EPOCH};

use crate::pb::scheduler::pairing_client::PairingClient;
use crate::pb::scheduler::{
    ApprovePairingRequest, Empty, GetPairingDataRequest, GetPairingDataResponse, PairDeviceRequest,
    PairDeviceResponse, PairingQr,
};
use crate::serialize::AuthBlob;
use crate::tls::{self, TlsConfig};
use anyhow::Result;
use bytes::BufMut;
use log::debug;
use picky::x509::Csr;
use picky_asn1_x509::{PublicKey, SubjectPublicKeyInfo};
use ring::{
    rand,
    signature::{self, EcdsaKeyPair},
};
use runeauth::Rune;
use rustls_pemfile as pemfile;
use thiserror::Error;
use tokio::sync::mpsc;
use tonic::transport::Channel;
use tonic::Status;

#[derive(Error, Debug)]
pub enum PairingError {
    #[error("grpc error {0}")]
    GrpcError(#[from] Status),
    #[error("could not create key pair {0}")]
    CreateKeyError(String),
    #[error("rcgen error {0}")]
    RcGenError(#[from] rcgen::RcgenError),
    #[error("could not approve pairing {0}")]
    ApproveParingError(String),
    #[error("pairing data is not valid {0}")]
    VerifyPairingDataError(String),
}

pub struct Builder {
    uri: String,
    tls: TlsConfig,
    rune: Option<Rune>,
    key: Option<Vec<u8>>,
}

impl Builder {
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = tls.clone();
        self.key = tls.private_key;
        self
    }

    pub fn with_rune(mut self, rune: Rune) -> Self {
        self.rune = Some(rune);
        self
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = uri;
        self
    }

    pub async fn build(self) -> Result<Pairing> {
        let channel = tonic::transport::Endpoint::from_shared(self.uri)?
            .tls_config(self.tls.inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let client = PairingClient::new(channel);

        Ok(Pairing {
            client,
            ca: self.tls.ca.clone(),
            rune: self.rune,
            key: self.key,
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            uri: crate::utils::scheduler_uri(),
            tls: TlsConfig::new().unwrap(),
            rune: Default::default(),
            key: Default::default(),
        }
    }
}

pub enum PairingSessionData {
    PairingResponse(PairDeviceResponse),
    PairingQr(PairingQr),
    PairingError(tonic::Status),
}

pub struct Pairing {
    client: PairingClient<Channel>,
    ca: Vec<u8>,
    rune: Option<Rune>,
    key: Option<Vec<u8>>,
}

impl Pairing {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Builder {
        Builder::default()
    }

    pub async fn pair_device(
        &self,
        name: &str,
        desc: &str,
        restrs: &str,
    ) -> Result<mpsc::Receiver<PairingSessionData>, PairingError> {
        debug!("start a new pairing request");

        let device_name = name.to_string();
        let desc = desc.to_string();
        let restrs = restrs.to_string();

        // Generate key pair.
        let pem = Some(tls::generate_ecdsa_key_pair().serialize_pem()).unwrap();

        // Generate csr.
        let device_cert = tls::generate_self_signed_device_cert_from_pem(
            &pem,
            &hex::encode("00"),
            name,
            vec!["localhost".into()],
        );
        let session_id = hex::encode(device_cert.get_key_pair().public_key_raw());
        let csr = device_cert.serialize_request_pem()?;

        let ca = self.ca.clone();

        // Create a channel to communicate beyond the bounds of this function
        let (tx, rx) = mpsc::channel(1);

        let mut client = self.client.clone();
        // The worker that handles the pairing. Communicate to the outside world
        // via the channel.
        tokio::spawn(async move {
            // Step 1 of the Pairing Protocol: Request pairing at the Greenlight
            // Backend.
            let request = client.pair_device(PairDeviceRequest {
                session_id: session_id.clone(),
                csr: csr.into_bytes(),
                device_name,
                desc,
                restrs,
            });

            // Step 2 of the Pairing Protocol: Return the PairingQR for the new
            // device to show it to an old device.
            let data = format!("gl-pairing:{}", session_id);
            tx.send(PairingSessionData::PairingQr(PairingQr { data }))
                .await
                .expect("could not pass qr data to the channel"); // We can unwrap here as there is no need to continue if the channel is broken.

            // Step 8 of the Pairing Protocol: Get back the response. We do fire
            // and forget here.
            let _ = match request.await {
                Ok(r) => {
                    let mut res = r.into_inner();
                    res.device_key = device_cert.serialize_private_key_pem();

                    let blob = AuthBlob {
                        cert: res.device_cert.clone().into_bytes(),
                        key: res.device_key.clone().into_bytes(),
                        ca,
                        rune: res.rune.clone(),
                    };

                    res.auth = blob.serialize();

                    tx.send(PairingSessionData::PairingResponse(res))
                }
                Err(e) => {
                    debug!("got an error during pairing process {}.", e);
                    tx.send(PairingSessionData::PairingError(e))
                }
            }
            .await;

            return;
        });

        Ok(rx)
    }

    pub async fn get_pairing_data(&self, session_id: &str) -> Result<GetPairingDataResponse> {
        Ok(self
            .client
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
    ) -> Result<Empty, PairingError> {
        if let Some(rune) = self.rune.clone() {
            if let Some(key) = &self.key {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| PairingError::ApproveParingError(e.to_string()))?
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
                    let mut key = std::io::Cursor::new(key);
                    pemfile::pkcs8_private_keys(&mut key)
                        .map_err(|e| PairingError::ApproveParingError(e.to_string()))?
                        .remove(0)
                };
                let kp = EcdsaKeyPair::from_pkcs8(
                    &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
                    key.as_ref(),
                )
                .map_err(|e| {
                    PairingError::ApproveParingError(format!(
                        "could not get keypair from PEM string: {}",
                        e
                    ))
                })?;
                let rng = rand::SystemRandom::new();
                let sig = kp
                    .sign(&rng, &buf)
                    .map_err(|e| PairingError::ApproveParingError(e.to_string()))?
                    .as_ref()
                    .to_vec();

                // Send approval.
                Ok(self
                    .client
                    .clone()
                    .approve_pairing(ApprovePairingRequest {
                        session_id: session_id.to_string(),
                        timestamp,
                        node_id: node_id.to_vec(),
                        device_name: device_name.to_string(),
                        restrs: restrs.to_string(),
                        sig: sig,
                        rune: rune.to_base64(),
                    })
                    .await?
                    .into_inner())
            } else {
                Err(PairingError::ApproveParingError(
                    "tls key missing".to_string(),
                ))
            }
        } else {
            Err(PairingError::ApproveParingError("rune missing".to_string()))
        }
    }

    pub fn verify_pairing_data(data: GetPairingDataResponse) -> Result<(), PairingError> {
        let mut crs = std::io::Cursor::new(&data.csr);
        let pem = picky::pem::Pem::read_from(&mut crs).map_err(to_verify_error)?;
        let csr = Csr::from_pem(&pem).map_err(to_verify_error)?;
        let sub_pk_der = csr.public_key().to_der().map_err(to_verify_error)?;
        let sub_pk_info: SubjectPublicKeyInfo =
            picky_asn1_der::from_bytes(&sub_pk_der).map_err(to_verify_error)?;

        if let PublicKey::Ec(bs) = sub_pk_info.subject_public_key {
            let pk = hex::encode(bs.0.payload_view());

            if pk == data.session_id {
                Ok(())
            } else {
                Err(PairingError::VerifyPairingDataError(format!(
                    "public key {} does not match pk {}",
                    data.session_id, pk
                )))
            }
        } else {
            Err(PairingError::VerifyPairingDataError(format!(
                "expected ecdsa pubkey"
            )))
        }
    }
}

fn to_verify_error<T: ToString>(e: T) -> PairingError {
    PairingError::VerifyPairingDataError(e.to_string())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_verify_pairing_data() {
        let pem = tls::generate_ecdsa_key_pair().serialize_pem();
        let device_cert = tls::generate_self_signed_device_cert_from_pem(
            &pem,
            &hex::encode("00"),
            "my-device",
            vec!["localhost".into()],
        );
        let csr = device_cert.serialize_request_pem().unwrap();
        let pk = hex::encode(device_cert.get_key_pair().public_key_raw());

        // Check with public key as session id.
        let pd = GetPairingDataResponse {
            session_id: pk,
            csr: csr.clone().into_bytes(),
            device_name: "my-device".to_string(),
            desc: "".to_string(),
            restrs: "".to_string(),
        };
        assert!(Pairing::verify_pairing_data(pd).is_ok());

        // Check with different public key as session id.
        let pd = GetPairingDataResponse {
            session_id: "00".to_string(),
            csr: csr.into_bytes(),
            device_name: "my-device".to_string(),
            desc: "".to_string(),
            restrs: "".to_string(),
        };
        assert!(Pairing::verify_pairing_data(pd).is_err());
    }
}
