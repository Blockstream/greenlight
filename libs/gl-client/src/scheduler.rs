use crate::pb::scheduler::scheduler_client::SchedulerClient;
use crate::tls::{self, TlsConfig};

use crate::node::GrpcClient;
use crate::{node, pb, signer::Signer, utils};
use anyhow::{anyhow, Result};
use lightning_signer::bitcoin::Network;
use tonic::transport::Channel;

type Client = SchedulerClient<Channel>;

/// Builds [`Scheduler`] with custom configuration values and creates the grpc
/// client for the `Scheduler`.
/// Methods can be chained in order to set configuration values.
/// The [`Scheduler`] is constructed and connected to the backend by calling
/// `connect`.
/// A new `SchedulerBuilder` instance is obtained via
/// [`SchedulerBuilder::new()`]
/// or by the `Scheduler` via
/// [`Scheduler::builder()`].
///
/// `node_id` and `network` are mandatory configuration parameters.
///
/// # Examples
///
/// ```
/// use lightning_signer::bitcoin::Network;
/// let scheduler = SchedulerBuilder::new(vec![0u8; 32], Network::Testnet)
///     .uri("https://my.gl-scheduler.uri.com")?
///     .connect()
///     .await?;
/// ```
pub struct SchedulerBuilder {
    node_id: Vec<u8>,
    network: Network,
    tls: Option<TlsConfig>,
    uri: Option<String>,
}

impl SchedulerBuilder {
    /// Creates a new SchedulerBuilder
    pub fn new(node_id: Vec<u8>, network: Network) -> SchedulerBuilder {
        SchedulerBuilder {
            node_id,
            network,
            tls: None,
            uri: None,
        }
    }

    /// Sets the `TlsConfig` that the `Scheduler` uses to connect to the
    /// greenlight backend. If unset defaults to the default `TlsConfig`.
    pub fn tls_config<'a>(&'a mut self, tls: &TlsConfig) -> Result<&'a mut SchedulerBuilder> {
        if self.tls.is_some() {
            return Err(anyhow!("tls config already set"));
        }
        self.tls = Some(tls.clone());
        Ok(self)
    }

    /// Sets the uri that the `Scheduler` uses to connect to the greenlight
    /// backend. If unset defaults to the builtin in default uri.
    pub fn uri<'a>(&'a mut self, uri: String) -> Result<&'a mut SchedulerBuilder> {
        if self.uri.is_some() {
            return Err(anyhow!("uri already set"));
        }
        self.uri = Some(uri);
        Ok(self)
    }

    /// Builds the `Scheduler` ready with the grpc-`Client` connected to the
    /// greenlight backend.
    pub async fn connect(&self) -> Result<Scheduler> {
        let uri = self.uri.clone().unwrap_or_else(|| utils::scheduler_uri());
        let tls = match self.tls.clone() {
            Some(tls) => tls,
            None => crate::tls::TlsConfig::new()?,
        };
        debug!("Connection to scheduler at {}", uri);
        let channel = Channel::from_shared(uri)?
            .tls_config(tls.inner.clone())?
            .connect()
            .await?;

        let client = SchedulerClient::new(channel);
        Ok(Scheduler {
            node_id: self.node_id.clone(),
            client,
            network: self.network,
        })
    }
}

#[derive(Clone)]
pub struct Scheduler {
    node_id: Vec<u8>,
    client: Client,
    network: Network,
}

impl Scheduler {
    /// Returns the builder for a `Scheduler`. See [`SchedulerBuilder`] for
    /// details on how to use the builder.
    pub fn builder(node_id: Vec<u8>, network: Network) -> SchedulerBuilder {
        SchedulerBuilder::new(node_id, network)
    }

    #[deprecated = "users should instead use `Scheduler::builder(node_id: Vec<u8>, network: Network)`"]
    pub async fn with(
        node_id: Vec<u8>,
        network: Network,
        uri: String,
        tls: &TlsConfig,
    ) -> Result<Scheduler> {
        let scheduler = Self::builder(node_id, network)
            .uri(uri)?
            .tls_config(tls)?
            .connect()
            .await?;
        Ok(scheduler)
    }

    #[deprecated = "users should instead use `Scheduler::builder(node_id: Vec<u8>, network: Network)`"]
    pub async fn new(node_id: Vec<u8>, network: Network) -> Result<Scheduler> {
        let scheduler = Self::builder(node_id, network).connect().await?;
        Ok(scheduler)
    }

    pub async fn register(
        &self,
        signer: &Signer,
        invite_code: Option<String>,
    ) -> Result<pb::scheduler::RegistrationResponse> {
        let code = invite_code.unwrap_or_default();
        return self.inner_register(signer, code).await;
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
        let challenge = self
            .client
            .clone()
            .get_challenge(pb::scheduler::ChallengeRequest {
                scope: pb::scheduler::ChallengeScope::Register as i32,
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let signature = signer.sign_challenge(challenge.challenge.clone())?;
        let device_cert = tls::generate_self_signed_device_cert(
            &hex::encode(self.node_id.clone()),
            "default".into(),
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

        // We ask the signer for a signature of the public key to append the
        // public key to any payload that is sent to a node.
        let public_key = device_cert.get_key_pair().public_key_raw();
        debug!(
            "Asking singer to sign public key {}",
            hex::encode(public_key)
        );
        let r = signer.sign_device_key(public_key)?;
        debug!("Got signature: {}", hex::encode(r));

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

        // We ask the signer for a signature of the public key to append the
        // public key to any payload that is sent to a node.
        let public_key = device_cert.get_key_pair().public_key_raw();
        debug!(
            "Asking singer to sign public key {}",
            hex::encode(public_key)
        );
        let r = signer.sign_device_key(public_key)?;
        debug!("Got signature: {}", hex::encode(r));

        Ok(res)
    }

    pub async fn schedule<T>(&self, tls: TlsConfig) -> Result<T>
    where
        T: GrpcClient,
    {
        let sched = self
            .client
            .clone()
            .schedule(pb::scheduler::ScheduleRequest {
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let uri = sched.grpc_uri;

        node::Node::new(self.node_id.clone(), self.network, tls)
            .connect(uri)
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
}
