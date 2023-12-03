use std::time::{SystemTime, UNIX_EPOCH};

use super::{into_approve_pairing_error, Error};
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
}
