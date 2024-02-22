use super::PairingSessionData;
use crate::{
    credentials::Device,
    pb::scheduler::{pairing_client::PairingClient, PairDeviceRequest},
    tls::{self, TlsConfig},
};
use log::debug;
use tokio::sync::mpsc;
use tonic::transport::Channel;

type Result<T, E = super::Error> = core::result::Result<T, E>;

pub struct Unconnected();
pub struct Connected(PairingClient<Channel>);

pub struct Client<T> {
    inner: T,
    uri: String,
    tls: TlsConfig,
}

impl Client<Unconnected> {
    pub fn new(tls: TlsConfig) -> Client<Unconnected> {
        Client {
            inner: Unconnected(),
            uri: crate::utils::scheduler_uri(),
            tls,
        }
    }
}

impl Client<Unconnected> {
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
        Ok(Client {
            inner: Connected(PairingClient::new(channel)),
            uri: self.uri,
            tls: self.tls,
        })
    }
}

impl Client<Connected> {
    pub async fn pair_device(
        &self,
        name: &str,
        description: &str,
        restrictions: &str,
    ) -> Result<mpsc::Receiver<PairingSessionData>> {
        debug!("start a new pairing request");

        let device_name = name.to_string();
        let description = description.to_string();

        // Generate key pair.
        let kp = tls::generate_ecdsa_key_pair();

        // Generate csr.
        let device_cert = tls::generate_self_signed_device_cert(
            &hex::encode("00"), // We don't know the node id yet, this is to be filled out by the attestation device.
            name,
            vec!["localhost".into()],
            Some(kp),
        );
        let device_id = hex::encode(device_cert.get_key_pair().public_key_raw());
        let csr = device_cert.serialize_request_pem()?;

        // Restrictions should always contain the pubkey field to bind them to
        // the private key of the device.
        let mut restriction = format!("pubkey={}", device_id.clone());
        if !restrictions.is_empty() {
            // Append restrictions if set.
            restriction = format!("{}&{}", restriction, restrictions);
        }
        let restrictions = restriction;

        // Create a channel to communicate beyond the bounds of this function
        let (tx, rx) = mpsc::channel(1);

        let mut client = self.inner.0.clone();
        // The worker that handles the pairing. Communicate to the outside world
        // via the channel.
        tokio::spawn(async move {
            // Step 1 of the Pairing Protocol: Request pairing at the Greenlight
            // Backend.
            let request = client.pair_device(PairDeviceRequest {
                device_id: device_id.clone(),
                csr: csr.into_bytes(),
                device_name,
                description,
                restrictions,
            });

            // Step 8 of the Pairing Protocol: Get back the response. We do fire
            // and forget here.
            let _ = match request.await {
                Ok(r) => {
                    let mut res = r.into_inner();
                    res.device_key = device_cert.serialize_private_key_pem();
                    let creds = Device::with(
                        res.device_cert.clone().into_bytes(),
                        res.device_key.clone().into_bytes(),
                        res.rune.clone(),
                    );

                    res.creds = creds.into();
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
}
