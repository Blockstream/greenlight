use crate::credentials::{self, Credentials};
use crate::node::{self, GrpcClient};
use crate::pb::scheduler::scheduler_client::SchedulerClient;
use crate::tls::{self};
use crate::{pb, signer::Signer, utils};
use anyhow::{anyhow, Result};
use lightning_signer::bitcoin::Network;
use log::debug;
use runeauth;
use tonic::transport::Channel;

type Client = SchedulerClient<Channel>;

/// Represents a builder for creating a `Scheduler` instance.
///
/// This struct is used to configure and build a `Scheduler` with various
/// options, such as network settings, credentials, and a URI.
pub struct Builder {
    node_id: Vec<u8>,
    network: Network,
    creds: Credentials,
    uri: Option<String>,
}

impl Builder {
    /// Constructs a new `Builder` instance.
    ///
    /// # Arguments
    ///
    /// * `node_id` - A unique identifier for the node as a byte vector.
    /// * `network` - Network settings for the Scheduler.
    /// * `tls` - A value that can be converted into `TlsConfig`.
    ///
    /// # Returns
    ///
    /// A Result containing the new instance of `Builder` or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if `tls` fails to convert into `TlsConfig`.
    pub fn new(node_id: Vec<u8>, network: Network, creds: Credentials) -> Result<Self> {
        Ok(Self {
            node_id,
            network,
            uri: None,
            creds,
        })
    }

    /// Sets the URI for the Scheduler.
    ///
    /// # Arguments
    ///
    /// * `uri` - URI that the scheduler connects to.
    ///
    /// # Returns
    ///
    /// The updated Builder instance.
    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = Some(uri);
        self
    }

    /// Builds a Scheduler instance based on the configured parameters.
    ///
    /// This method finalizes the configuration and creates a new `Scheduler`
    /// instance with the provided settings.
    ///
    /// # Returns
    ///
    /// A Result containing the Scheduler or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the `Scheduler` creation fails.
    pub async fn connect(&self) -> Result<Scheduler> {
        let uri = self.uri.clone().unwrap_or_else(|| utils::scheduler_uri());
        let node_id = self.node_id.clone();
        Scheduler::new(node_id, self.network, uri, self.creds.clone()).await
    }
}

#[derive(Clone)]
pub struct Scheduler {
    /// Our local `node_id` used when talking to the scheduler to
    /// identify us.
    node_id: Vec<u8>,
    client: Client,
    network: Network,
    creds: Credentials,
}

impl Scheduler {
    pub fn builder(node_id: Vec<u8>, network: Network, creds: Credentials) -> Result<Builder> {
        Builder::new(node_id, network, creds)
    }

    pub async fn new(
        node_id: Vec<u8>,
        network: Network,
        uri: String,
        creds: Credentials,
    ) -> Result<Scheduler> {
        debug!("Connecting to scheduler at {}", uri);
        let channel = tonic::transport::Endpoint::from_shared(uri)?
            .tls_config(creds.tls_config().inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let client = SchedulerClient::new(channel);

        Ok(Scheduler {
            client,
            node_id,
            network,
            creds,
        })
    }

    pub async fn register(
        &self,
        signer: &Signer,
        invite_code: Option<String>,
    ) -> Result<pb::scheduler::RegistrationResponse> {
        let code = invite_code.unwrap_or_default();
        self.inner_register(signer, code).await
    }

    /// We split the register method into one with an invite code and one
    /// without an invite code in order to keep the api stable. We might want to
    /// remove the invite system in the future and so it does not make sense to
    /// change the signature of the register method.
    async fn inner_register(
        &self,
        signer: &Signer,
        invite_code: String,
    ) -> Result<pb::scheduler::RegistrationResponse> {
        log::debug!("Retrieving challenge for registration");
        let challenge = self
            .client
            .clone()
            .get_challenge(pb::scheduler::ChallengeRequest {
                scope: pb::scheduler::ChallengeScope::Register as i32,
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        log::trace!("Got a challenge: {}", hex::encode(&challenge.challenge));

        let signature = signer.sign_challenge(challenge.challenge.clone())?;
        let device_cert = tls::generate_self_signed_device_cert(
            &hex::encode(self.node_id.clone()),
            "default",
            vec!["localhost".into()],
        );
        let device_csr = device_cert.serialize_request_pem()?;
        debug!("Requesting registration with csr:\n{}", device_csr);

        let startupmsgs = signer
            .get_startup_messages()
            .into_iter()
            .map(|m| m.into())
            .collect();

        let mut res = self
            .client
            .clone()
            .register(pb::scheduler::RegistrationRequest {
                node_id: self.node_id.clone(),
                bip32_key: signer.bip32_ext_key(),
                network: self.network.to_string(),
                challenge: challenge.challenge,
                signer_proto: signer.version().to_owned(),
                init_msg: signer.get_init(),
                signature,
                csr: device_csr.into_bytes(),
                invite_code,
                startupmsgs,
            })
            .await?
            .into_inner();

        // This step ensures backwards compatibility with the backend. If we did
        // receive a device key, the backend did not sign the csr and we need to
        // return the response as it is. If the device key is empty, the csr was
        // signed and we return the client side generated private key.
        if res.device_key.is_empty() {
            debug!("Received signed certificate:\n{}", &res.device_cert);
            // We intercept the response and replace the private key with the
            // private key of the device_cert. This private key has been generated
            // on and has never left the client device.
            res.device_key = device_cert.serialize_private_key_pem();
        }

        let public_key = device_cert.get_key_pair().public_key_raw();
        debug!(
            "Asking signer to create a rune for public key {}",
            hex::encode(public_key)
        );

        // Create a new rune for the tls certs public key and append it to the
        // grpc response. Restricts the rune to the public key used for mTLS
        // authentication.
        let alt = runeauth::Alternative::new(
            "pubkey".to_string(),
            runeauth::Condition::Equal,
            hex::encode(public_key),
            false,
        )?;
        res.rune = signer.create_rune(None, vec![vec![&alt.encode()]])?;

        // Create a `credentials::Device` struct and serialize it into byte format to
        // return. This can than be stored on the device.
        let creds = credentials::Device::with(
            res.device_cert.clone().into_bytes(),
            res.device_key.clone().into_bytes(),
            self.creds.tls_config().ca.clone(),
            res.rune.clone(),
        );
        res.creds = creds.to_bytes()?;

        Ok(res)
    }

    pub async fn recover(&self, signer: &Signer) -> Result<pb::scheduler::RecoveryResponse> {
        let challenge = self
            .client
            .clone()
            .get_challenge(pb::scheduler::ChallengeRequest {
                scope: pb::scheduler::ChallengeScope::Recover as i32,
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let signature = signer.sign_challenge(challenge.challenge.clone())?;
        let name = format!("recovered-{}", hex::encode(&challenge.challenge[0..8]));
        let device_cert = tls::generate_self_signed_device_cert(
            &hex::encode(self.node_id.clone()),
            &name,
            vec!["localhost".into()],
        );
        let device_csr = device_cert.serialize_request_pem()?;
        debug!("Requesting recovery with csr:\n{}", device_csr);

        let mut res = self
            .client
            .clone()
            .recover(pb::scheduler::RecoveryRequest {
                node_id: self.node_id.clone(),
                challenge: challenge.challenge,
                signature,
                csr: device_csr.into_bytes(),
            })
            .await?
            .into_inner();

        // This step ensures backwards compatibility with the backend. If we did
        // receive a device key, the backend did not sign the csr and we need to
        // return the response as it is. If the device key is empty, the csr was
        // signed and we return the client side generated private key.
        if res.device_key.is_empty() {
            debug!("Received signed certificate:\n{}", &res.device_cert);
            // We intercept the response and replace the private key with the
            // private key of the device_cert. This private key has been generated
            // on and has never left the client device.
            res.device_key = device_cert.serialize_private_key_pem();
        }

        let public_key = device_cert.get_key_pair().public_key_raw();
        debug!(
            "Asking signer to create a rune for public key {}",
            hex::encode(public_key)
        );

        // Create a new rune for the tls certs public key and append it to the
        // grpc response. Restricts the rune to the public key used for mTLS
        // authentication.
        let alt = runeauth::Alternative::new(
            "pubkey".to_string(),
            runeauth::Condition::Equal,
            hex::encode(public_key),
            false,
        )?;
        res.rune = signer.create_rune(None, vec![vec![&alt.encode()]])?;

        // Create a `credentials::Device` struct and serialize it into byte format to
        // return. This can than be stored on the device.
        let creds = credentials::Device::with(
            res.device_cert.clone().into_bytes(),
            res.device_key.clone().into_bytes(),
            self.creds.tls_config().ca.clone(),
            res.rune.clone(),
        );
        res.creds = creds.to_bytes()?;

        Ok(res)
    }

    pub async fn schedule(&self) -> Result<pb::scheduler::NodeInfoResponse> {
        let res = self
            .client
            .clone()
            .schedule(pb::scheduler::ScheduleRequest {
                node_id: self.node_id.clone(),
            })
            .await?;
        Ok(res.into_inner())
    }

    pub async fn node<T>(&self) -> Result<T>
    where
        T: GrpcClient,
    {
        let cert_node_id = self.get_node_id_from_tls_config(&creds.tls_config())?;

        if cert_node_id != self.node_id {
            return Err(anyhow!("The node_id defined on the Credential's certificate does not match the node_id the scheduler was initialized with\nExpected {}, got {}", hex::encode(&self.node_id), hex::encode(&cert_node_id)));
        }

        let res = self.schedule().await?;
        node::Node::new(self.node_id.clone(), self.creds.clone())?
            .connect(res.grpc_uri)
            .await
    }

    pub async fn export_node(&self) -> Result<pb::scheduler::ExportNodeResponse> {
        Ok(self
            .client
            .clone()
            .export_node(pb::scheduler::ExportNodeRequest {})
            .await?
            .into_inner())
    }

    pub async fn get_invite_codes(&self) -> Result<pb::scheduler::ListInviteCodesResponse> {
        let res = self
            .client
            .clone()
            .list_invite_codes(pb::scheduler::ListInviteCodesRequest {})
            .await?;
        Ok(res.into_inner())
    }

    pub async fn add_outgoing_webhook(
        &self,
        outgoing_webhook_request: pb::scheduler::AddOutgoingWebhookRequest,
    ) -> Result<pb::scheduler::AddOutgoingWebhookResponse> {
        let res = self
            .client
            .clone()
            .add_outgoing_webhook(outgoing_webhook_request)
            .await?;
        Ok(res.into_inner())
    }

    pub async fn list_outgoing_webhooks(
        &self,
        list_outgoing_webhooks_request: pb::scheduler::ListOutgoingWebhooksRequest,
    ) -> Result<pb::scheduler::ListOutgoingWebhooksResponse> {
        let res = self
            .client
            .clone()
            .list_outgoing_webhooks(list_outgoing_webhooks_request)
            .await?;
        Ok(res.into_inner())
    }

    pub async fn delete_webhooks(
        &self,
        delete_webhooks_request: pb::scheduler::DeleteOutgoingWebhooksRequest,
    ) -> Result<pb::greenlight::Empty> {
        let res = self
            .client
            .clone()
            .delete_webhooks(delete_webhooks_request)
            .await?;
        Ok(res.into_inner())
    }

    pub async fn rotate_outgoing_webhook_secret(
        &self,
        rotate_outgoing_webhook_secret_request: pb::scheduler::RotateOutgoingWebhookSecretRequest,
    ) -> Result<pb::scheduler::WebhookSecretResponse> {
        let res = self
            .client
            .clone()
            .rotate_outgoing_webhook_secret(rotate_outgoing_webhook_secret_request)
            .await?;
        Ok(res.into_inner())
    }

    fn get_node_id_from_tls_config(&self, tls_config: &TlsConfig) -> Result<Vec<u8>> {
        let subject_common_name = match &tls_config.x509_cert {
            Some(x) => match x.subject_common_name() {
                Some(cn) => cn,
                None => {
                    return Err(anyhow!(
                        "Failed to parse the subject common name in the provided x509 certificate"
                    ))
                }
            },
            None => {
                return Err(anyhow!(
                    "The certificate could not be parsed in the x509 format"
                ))
            }
        };

        let split_subject_common_name = subject_common_name
            .split("/")
            .into_iter()
            .collect::<Vec<&str>>();

        assert!(split_subject_common_name[1] == "users");
        return Ok(hex::decode(split_subject_common_name[2])
            .expect("Failed to parse the node_id from the TlsConfig to bytes"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::{Credentials, Device};

    #[tokio::test]
    async fn test_node_returns_error_on_node_id_mismatch() {
        let cert_node_id: [u8; 32] = rand::random();
        let device_cert = tls::generate_self_signed_device_cert(
            &hex::encode(cert_node_id.clone()),
            "default".into(),
            vec!["localhost".into()],
        );

        let device_crt = device_cert.serialize_pem().unwrap();
        let device_key = device_cert.serialize_private_key_pem();

        let creds = Credentials::Device(Device {
            cert: device_crt.into(),
            key: device_key.into(),
            ..Default::default()
        });

        let scheduler_node_id: [u8; 32] = rand::random();
        let sched = Scheduler::new(scheduler_node_id.to_vec(), Network::Bitcoin)
            .await
            .unwrap();

        assert!(sched.node::<node::ClnClient>(creds).await.is_err_and(|e| e.to_string().contains("The node_id defined on the Credential's certificate does not match the node_id the scheduler was initialized with")));
    }
}
