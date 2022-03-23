use crate::pb::scheduler_client::SchedulerClient;
use crate::tls::TlsConfig;
use crate::{node, pb, signer::Signer, utils};
use anyhow::Result;
use bitcoin::Network;
use tonic::transport::Channel;

type Client = SchedulerClient<Channel>;

#[derive(Clone)]
pub struct Scheduler {
    node_id: Vec<u8>,
    client: Client,
    network: Network,
}

impl Scheduler {
    pub async fn new(node_id: Vec<u8>, network: Network) -> Result<Scheduler> {
        let tls = crate::tls::TlsConfig::new()?;
        let scheduler_uri = utils::scheduler_uri();

        debug!("Connecting to scheduler at {}", scheduler_uri);
        let channel = Channel::from_shared(scheduler_uri)?
            .tls_config(tls.inner.clone())?
            .connect()
            .await?;

        let client = SchedulerClient::new(channel);

        Ok(Scheduler {
            client,
            node_id,
            network,
        })
    }

    pub async fn register(&self, signer: &Signer) -> Result<pb::RegistrationResponse> {
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

        let res = self
            .client
            .clone()
            .register(pb::RegistrationRequest {
                node_id: self.node_id.clone(),
                bip32_key: signer.bip32_ext_key(),
                network: self.network.to_string(),
                challenge: challenge.challenge,
                email: "".to_string(),
                signature,
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
                signature,
            })
            .await?;

        Ok(res.into_inner())
    }

    pub async fn schedule(&self, tls: TlsConfig) -> Result<node::Client> {
        let sched = self
            .client
            .clone()
            .schedule(pb::ScheduleRequest {
                node_id: self.node_id.clone(),
            })
            .await?
            .into_inner();

        let uri = sched.grpc_uri;

        node::Node::new(self.node_id.clone(), self.network, tls)
            .connect(uri)
            .await
    }
}
