use crate::credentials::{self, RuneProvider, TlsConfigProvider};
use crate::node::{self, GrpcClient};
use crate::pb::scheduler::scheduler_client::SchedulerClient;
use crate::tls::{self};
use crate::utils::scheduler_uri;
use crate::{pb, signer::Signer};
use anyhow::{anyhow, Result};
use lightning_signer::bitcoin::Network;
use log::debug;
use runeauth;
use tonic::transport::Channel;

type Client = SchedulerClient<Channel>;

/// A scheduler client to interact with the scheduler service. It has
/// different implementations depending on the implementations
#[derive(Clone)]
pub struct Scheduler<Creds> {
    node_id: Vec<u8>,
    client: Client,
    network: Network,
    grpc_uri: String,
    creds: Creds,
    ca: Vec<u8>,
}

impl<Creds> Scheduler<Creds>
where
    Creds: TlsConfigProvider,
{
    /// Creates a new scheduler client with the provided parameters.
    /// A scheduler created this way is considered unauthenticated and
    /// limited in its scope.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::Nobody;
    /// # use gl_client::scheduler::Scheduler;
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Nobody::new();
    /// let scheduler = Scheduler::new(node_id, network, creds).await.unwrap();
    /// # }
    /// ```
    pub async fn new(node_id: Vec<u8>, network: Network, creds: Creds) -> Result<Scheduler<Creds>> {
        let grpc_uri = scheduler_uri();
        Self::with(node_id, network, creds, grpc_uri).await
    }

    /// Creates a new scheduler client with the provided parameters and
    /// custom URI.
    /// A scheduler created this way is considered unauthenticated and
    /// limited in its scope.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::Nobody;
    /// # use gl_client::scheduler::Scheduler;
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Nobody::new();
    /// let uri = "https://example.com".to_string();
    /// let scheduler = Scheduler::with(node_id, network, creds, uri).await.unwrap();
    /// # }
    /// ```
    pub async fn with(
        node_id: Vec<u8>,
        network: Network,
        creds: Creds,
        uri: impl Into<String>,
    ) -> Result<Scheduler<Creds>> {
        let uri = uri.into();
        debug!("Connecting to scheduler at {}", uri);
        let channel = tonic::transport::Endpoint::from_shared(uri.clone())?
            .tls_config(creds.tls_config().inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let client = SchedulerClient::new(channel);
        let ca = creds.tls_config().ca.clone();

        Ok(Scheduler {
            client,
            node_id,
            network,
            creds,
            grpc_uri: uri,
            ca,
        })
    }
}

impl<Creds> Scheduler<Creds> {
    /// Registers a new node with the scheduler service.
    ///
    /// # Arguments
    ///
    /// * `signer` - The signer instance bound to the node.
    /// * `invite_code` - Optional invite code to register the node.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::Nobody;
    /// # use gl_client::{scheduler::Scheduler, signer::Signer};
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Nobody::new();
    /// let scheduler = Scheduler::new(node_id.clone(), network, creds.clone()).await.unwrap();
    /// let secret = vec![0, 0, 0, 0];
    /// let signer = Signer::new(secret, network, creds).unwrap(); // Create or obtain a signer instance
    /// let registration_response = scheduler.register(&signer, None).await.unwrap();
    /// # }
    /// ```
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
        invite_code: impl Into<String>,
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
                invite_code: invite_code.into(),
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
            self.ca.clone(),
            res.rune.clone(),
        );
        res.creds = creds.to_bytes();

        Ok(res)
    }

    /// Recovers a previously registered node with the scheduler service.
    ///
    /// # Arguments
    ///
    /// * `signer` - The signer instance used to sign the recovery challenge.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::Nobody;
    /// # use gl_client::{scheduler::Scheduler, signer::Signer};
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Nobody::new();
    /// let scheduler = Scheduler::new(node_id.clone(), network, creds.clone()).await.unwrap();
    /// let secret = vec![0, 0, 0, 0];
    /// let signer = Signer::new(secret, network, creds).unwrap(); // Create or obtain a signer instance
    /// let recovery_response = scheduler.recover(&signer).await.unwrap();
    /// # }
    /// ```
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
            self.ca.clone(),
            res.rune.clone(),
        );
        res.creds = creds.to_bytes();

        Ok(res)
    }

    /// Elevates the scheduler client to an authenticated scheduler client
    /// that is able to schedule a node for example.
    ///
    /// # Arguments
    ///
    /// * `creds` - Credentials that carry a TlsConfig and a Rune. These
    /// are credentials returned during registration or recovery.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::{Device, Nobody};
    /// # use gl_client::{scheduler::Scheduler, signer::Signer};
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Nobody::new();
    /// let scheduler_unauthed = Scheduler::new(node_id.clone(), network, creds.clone()).await.unwrap();
    /// let secret = vec![0, 0, 0, 0];
    /// let signer = Signer::new(secret, network, creds).unwrap(); // Create or obtain a signer instance
    /// let registration_response = scheduler_unauthed.register(&signer, None).await.unwrap();
    /// let creds = Device::from_bytes(registration_response.creds);
    /// let scheduler_authed = scheduler_unauthed.authenticate(creds);
    /// # }
    /// ```
    pub async fn authenticate<Auth>(&self, creds: Auth) -> Result<Scheduler<Auth>>
    where
        Auth: TlsConfigProvider + RuneProvider,
    {
        debug!("Connecting to scheduler at {}", self.grpc_uri);
        let channel = tonic::transport::Endpoint::from_shared(self.grpc_uri.clone())?
            .tls_config(creds.tls_config().inner.clone())?
            .tcp_keepalive(Some(crate::TCP_KEEPALIVE))
            .http2_keep_alive_interval(crate::TCP_KEEPALIVE)
            .keep_alive_timeout(crate::TCP_KEEPALIVE_TIMEOUT)
            .keep_alive_while_idle(true)
            .connect_lazy();

        let client = SchedulerClient::new(channel);

        Ok(Scheduler {
            node_id: self.node_id.clone(),
            client,
            network: self.network,
            creds,
            grpc_uri: self.grpc_uri.clone(),
            ca: self.ca.clone(),
        })
    }
}

impl<Creds> Scheduler<Creds>
where
    Creds: TlsConfigProvider + RuneProvider + Clone,
{
    /// Schedules a node at the scheduler service. Once a node is
    /// scheduled one can access it through the node client.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::Device;
    /// # use gl_client::{scheduler::Scheduler, node::{Node, Client}};
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Device::from_path("my/path/to/credentials.glc");
    /// let scheduler = Scheduler::new(node_id.clone(), network, creds.clone()).await.unwrap();
    /// let info = scheduler.schedule().await.unwrap();
    /// let node_client: Client  = Node::new(node_id, creds).unwrap().connect(info.grpc_uri).await.unwrap();
    /// # }
    /// ```
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

    /// Schedules a node at the scheduler service and returns a node
    /// client.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gl_client::credentials::Device;
    /// # use gl_client::scheduler::Scheduler;
    /// # use gl_client::node::Client;
    /// # use lightning_signer::bitcoin::Network;
    /// # async fn example() {
    /// let node_id = vec![0, 1, 2, 3];
    /// let network = Network::Regtest;
    /// let creds = Device::from_path("my/path/to/credentials.glc");
    /// let scheduler = Scheduler::new(node_id.clone(), network, creds.clone()).await.unwrap();
    /// let node_client: Client  = scheduler.node().await.unwrap();
    /// # }
    /// ```
    pub async fn node<T>(&self) -> Result<T>
    where
        T: GrpcClient,
    {
        let cert_node_id = self.get_node_id_from_tls_config()?;

        if cert_node_id != self.node_id {
            return Err(anyhow!("The node_id defined on the Credential's certificate does not match the node_id the scheduler was initialized with\nExpected {}, got {}", hex::encode(&self.node_id), hex::encode(&cert_node_id)));
        }

        let res = self.schedule().await?;
        node::Node::new(self.node_id.clone(), self.creds.clone())?
            .connect(res.grpc_uri)
            .await
    }

    pub async fn get_node_info(&self, wait: bool) -> Result<pb::scheduler::NodeInfoResponse> {
        Ok(self
            .client
            .clone()
            .get_node_info(pb::scheduler::NodeInfoRequest {
                node_id: self.node_id.clone(),
                wait: wait,
            })
            .await?
            .into_inner())
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
        uri: String,
    ) -> Result<pb::scheduler::AddOutgoingWebhookResponse> {
        let node_id = self.get_node_id_from_tls_config()?;
        let res = self
            .client
            .clone()
            .add_outgoing_webhook(pb::scheduler::AddOutgoingWebhookRequest { node_id, uri })
            .await?;
        Ok(res.into_inner())
    }

    pub async fn list_outgoing_webhooks(
        &self,
    ) -> Result<pb::scheduler::ListOutgoingWebhooksResponse> {
        let node_id = self.get_node_id_from_tls_config()?;
        let res = self
            .client
            .clone()
            .list_outgoing_webhooks(pb::scheduler::ListOutgoingWebhooksRequest { node_id })
            .await?;
        Ok(res.into_inner())
    }

    pub async fn delete_webhooks(&self, webhook_ids: Vec<i64>) -> Result<pb::greenlight::Empty> {
        let node_id = self.get_node_id_from_tls_config()?;
        let res = self
            .client
            .clone()
            .delete_webhooks(pb::scheduler::DeleteOutgoingWebhooksRequest {
                node_id,
                ids: webhook_ids,
            })
            .await?;
        Ok(res.into_inner())
    }

    pub async fn rotate_outgoing_webhook_secret(
        &self,
        webhook_id: i64,
    ) -> Result<pb::scheduler::WebhookSecretResponse> {
        let node_id = self.get_node_id_from_tls_config()?;
        let res = self
            .client
            .clone()
            .rotate_outgoing_webhook_secret(pb::scheduler::RotateOutgoingWebhookSecretRequest {
                node_id,
                webhook_id,
            })
            .await?;
        Ok(res.into_inner())
    }

    fn get_node_id_from_tls_config(&self) -> Result<Vec<u8>> {
        let tls_config = self.creds.tls_config();
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

        let split_subject_common_name = subject_common_name.split("/").collect::<Vec<&str>>();

        assert!(split_subject_common_name[1] == "users");
        Ok(hex::decode(split_subject_common_name[2])
            .expect("Failed to parse the node_id from the TlsConfig to bytes"))
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Device;

    use super::*;

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

        let creds = Device::with(device_crt, device_key, String::new(), "");

        let scheduler_node_id: [u8; 32] = rand::random();
        let sched = Scheduler::new(scheduler_node_id.to_vec(), Network::Bitcoin, creds)
            .await
            .unwrap();

        assert!(sched.node::<node::ClnClient>().await.is_err_and(|e| e.to_string().contains("The node_id defined on the Credential's certificate does not match the node_id the scheduler was initialized with")));
    }
}
