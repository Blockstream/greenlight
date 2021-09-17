use crate::pb::scheduler_client::SchedulerClient;
use crate::{node, pb, signer::Signer, utils, Network};
use anyhow::{anyhow, Result};
use tonic::transport::{Channel, ClientTlsConfig};

type Client = SchedulerClient<Channel>;

pub struct Scheduler {
    node_id: Vec<u8>,
    client: Client,
    tls: Option<ClientTlsConfig>,
    network: Network,
}

impl Scheduler {
    pub async fn new(node_id: Vec<u8>, network: Network) -> Result<Scheduler> {
        let tls = crate::tls::NOBODY_CONFIG.clone();
        let scheduler_uri = utils::scheduler_uri();

	debug!("Connecting to scheduler at {}", scheduler_uri);
        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(tls.clone())?
            .connect()
            .await?;

        let client = SchedulerClient::new(channel);

        Ok(Scheduler {
            tls: None,
            client,
            node_id,
            network,
        })
    }

    pub async fn register(&self, signer: Signer) -> Result<pb::RegistrationResponse> {
        let challenge = self
            .client
            .clone()
            .get_challenge(pb::ChallengeRequest {
                scope: pb::ChallengeScope::Register as i32,
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let signature = signer.sign_challenge(challenge.challenge.clone())?;
        let network: &str = self.network.into();

        let res = self
            .client
            .clone()
            .register(pb::RegistrationRequest {
                node_id: self.node_id.clone(),
                bip32_key: signer.bip32_ext_key(),
                network: network.to_string(),
                challenge: challenge.challenge,
                email: "".to_string(),
                signature: signature,
            })
            .await?;

        Ok(res.into_inner())
    }

    pub async fn recover(&self, signer: &Signer) -> Result<pb::RecoveryResponse> {
        let challenge = self
            .client
            .clone()
            .get_challenge(pb::ChallengeRequest {
                scope: pb::ChallengeScope::Recover as i32,
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let signature = signer.sign_challenge(challenge.challenge.clone())?;

        let res = self
            .client
            .clone()
            .recover(pb::RecoveryRequest {
                node_id: self.node_id.clone(),
                challenge: challenge.challenge,
                signature: signature,
            })
            .await?;

        Ok(res.into_inner())
    }

    pub async fn schedule(&self) -> Result<node::Client> {
        let tls = match &self.tls {
            Some(tls) => tls.clone(),
            None => {
                return Err(anyhow!(
                    "Cannot connect to the node without first setting the device mTLS certificate"
                ))
            }
        };

        let sched = self
            .client
            .clone()
            .schedule(pb::ScheduleRequest {
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let uri = sched.grpc_uri;

        node::Builder::new(self.node_id.clone(), self.network, tls)
            .connect(uri)
            .await
    }
}
