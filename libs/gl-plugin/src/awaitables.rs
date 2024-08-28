use std::time::Duration;

use cln_rpc::{
    model::requests::{
        ConnectRequest, GetinfoRequest, GetrouteRequest, ListpeerchannelsRequest, ListpeersRequest,
    },
    primitives::{Amount, PublicKey, ShortChannelId},
    ClnRpc,
};
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Peer error: {0}")]
    Peer(&'static str),
    #[error("Channel error: {0}")]
    Channel(&'static str),
    #[error("RPC error: {0}")]
    Rpc(#[from] cln_rpc::RpcError),
}

/// A struct to track the status of a peer connection.
pub struct AwaitablePeer {
    peer_id: PublicKey,
    rpc: ClnRpc,
}

impl AwaitablePeer {
    pub fn new(peer_id: PublicKey, rpc: ClnRpc) -> Self {
        AwaitablePeer { peer_id, rpc }
    }

    pub async fn wait(&mut self) -> Result<(), Error> {
        ensure_peer_connection(&mut self.rpc, self.peer_id).await
    }
}

/// A struct to track the status of a channel.
pub struct AwaitableChannel {
    scid: ShortChannelId,
    peer_id: PublicKey,
    rpc: ClnRpc,
    version: String,
}

impl AwaitableChannel {
    pub async fn new(
        peer_id: PublicKey,
        scid: ShortChannelId,
        mut rpc: ClnRpc,
    ) -> Result<Self, Error> {
        let info = rpc
            .call_typed(&GetinfoRequest {})
            .await
            .map_err(|_| Error::Peer("unable to connect"))?;
        let version = info.version;
        Ok(AwaitableChannel {
            peer_id,
            scid,
            rpc,
            version,
        })
    }

    pub async fn wait(&mut self) -> Result<Amount, Error> {
        use tokio::time::sleep;
        // Ensure that we are connected to the peer, return an Error if
        // we can not connect to the peer.
        ensure_peer_connection(&mut self.rpc, self.peer_id).await?;

        // Next step is to wait for the channel to be
        // re-established. For this we look into the billboard and
        // wait for some magic strings to appear (yeah, I know...)
        log::debug!("Checking if channel {} is ready", self.scid);
        while !self.billboard().await?.into_iter().any(|s| {
            s.find("Channel ready").is_some() || s.find("Reconnected, and reestablished").is_some()
        }) {
            sleep(Duration::from_millis(250)).await;
        }
        log::debug!("Channel {} is established", self.scid);

        // Finally, we need to check that we have the gossip required
        // to route through the channel. We could check for channels
        // individually, but we can check them all at once by using
        // `getroute` to the peer.
        loop {
            let route = self
                .rpc
                .call_typed(&GetrouteRequest {
                    id: self.peer_id,
                    amount_msat: cln_rpc::primitives::Amount::from_msat(1),
                    riskfactor: 1,
                    cltv: None,
                    fromid: None,
                    fuzzpercent: Some(0),
                    exclude: None,
                    maxhops: Some(1),
                })
                .await;

            if route.is_ok() {
                log::debug!("Peer {:?} is routable", self.peer_id.to_string());
                break;
            } else {
                sleep(Duration::from_millis(500)).await;
            }
        }

        self.spendable_msat().await
    }

    /// Retrieve the spendable amount for the channel.
    async fn spendable_msat(&mut self) -> Result<Amount, Error> {
        if *self.version >= *"v23.05gl1" {
            Ok(self
                .rpc
                .call_typed(&ListpeerchannelsRequest {
                    id: Some(self.peer_id),
                })
                .await
                .map_err(|e| Error::Rpc(e))?
                .channels
                .ok_or(Error::Channel("No channels found"))?
                .into_iter()
                .filter(|c| {
                    c.short_channel_id == Some(self.scid)
                        || c.alias.clone().and_then(|a| a.local) == Some(self.scid)
                })
                .nth(0)
                .ok_or(Error::Channel(
                    "Could not find the channel in listpeerchannels",
                ))?
                .spendable_msat
                .ok_or(Error::Channel("No amount found"))?)
        } else {
            #[allow(deprecated)]
            Ok(self
                .rpc
                .call_typed(&ListpeersRequest {
                    id: Some(self.peer_id),
                    level: None,
                })
                .await
                .map_err(|e| Error::Rpc(e))?
                .peers
                .into_iter()
                .nth(0)
                .ok_or(Error::Peer("Has no peerlist"))?
                .channels
                .into_iter()
                .nth(0)
                .ok_or(Error::Channel("Empty channel list"))?
                .into_iter()
                .filter(|c| c.short_channel_id == Some(self.scid))
                .nth(0)
                .ok_or(Error::Channel("No channel with scid"))?
                .spendable_msat
                .ok_or(Error::Channel("No amount found"))?)
        }
    }

    async fn billboard(&mut self) -> Result<Vec<String>, Error> {
        if *self.version >= *"v23.05gl1" {
            Ok(self
                .rpc
                .call_typed(&ListpeerchannelsRequest {
                    id: Some(self.peer_id),
                })
                .await
                .map_err(|e| Error::Rpc(e))?
                .channels
                .unwrap()
                .into_iter()
                .filter(|c| {
                    c.short_channel_id == Some(self.scid)
                        || c.alias.clone().and_then(|a| a.local) == Some(self.scid)
                })
                .nth(0)
                .ok_or(Error::Channel(
                    "Could not find the channel in listpeerchannels",
                ))?
                .status
                .unwrap())
        } else {
            #[allow(deprecated)]
            Ok(self
                .rpc
                .call_typed(&ListpeersRequest {
                    id: Some(self.peer_id),
                    level: None,
                })
                .await
                .map_err(|e| Error::Rpc(e))?
                .peers
                .into_iter()
                .nth(0)
                .unwrap()
                .channels
                .into_iter()
                .nth(0)
                .unwrap()
                .into_iter()
                .filter(|c| c.short_channel_id == Some(self.scid))
                .nth(0)
                .unwrap()
                .status
                .unwrap())
        }
    }
}

/// Try to connect to the peer if we are not already connected.
async fn ensure_peer_connection(rpc: &mut ClnRpc, peer_id: PublicKey) -> Result<(), Error> {
    log::debug!("Checking if peer {} is connected", peer_id);
    let res = rpc
        .call_typed(&cln_rpc::model::requests::ListpeersRequest {
            id: Some(peer_id),
            level: None,
        })
        .await?;
    let peer = res.peers.first().ok_or(Error::Peer("No such peer"))?;

    if !peer.connected {
        log::debug!("Peer {} is not connected, connecting", peer_id);
        let req = ConnectRequest {
            id: peer_id.to_string(),
            host: None,
            port: None,
        };
        let res = rpc
            .call_typed(&req)
            .await
            .map_err(|_| Error::Peer("unable to connect"))?;

        log::debug!("Connect call to {} resulted in {:?}", peer_id, res);
    }
    Ok(())
}
