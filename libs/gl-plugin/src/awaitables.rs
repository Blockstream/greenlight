use cln_rpc::{
    model::{
        requests::{ConnectRequest, GetinfoRequest, GetrouteRequest, ListpeerchannelsRequest},
        responses::GetrouteResponse,
    },
    primitives::{Amount, PublicKey, ShortChannelId},
    ClnRpc,
};
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    time::Duration,
};
use thiserror;
use tokio::time::Instant;

// The delay between consecutive rpc calls of the same type.
const RPC_CALL_DELAY_MSEC: u64 = 250;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown peer {0}")]
    PeerUnknown(String),
    #[error("Can't connect to peer {0}")]
    PeerConnectionFailure(String),
    #[error("Channel error: {0}")]
    Channel(&'static str),
    #[error("RPC error: {0}")]
    Rpc(#[from] cln_rpc::RpcError),
    #[error("Error talking to a GL service: {0}")]
    Service(String),
}

/// A struct to track the status of a peer connection.
pub struct AwaitablePeer {
    peer_id: PublicKey,
    rpc_path: PathBuf,

    ensure_peer_connection: Option<Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>>,
}

impl AwaitablePeer {
    pub fn new(peer_id: PublicKey, rpc_path: PathBuf) -> Self {
        AwaitablePeer {
            peer_id,
            rpc_path,
            ensure_peer_connection: None,
        }
    }

    pub async fn wait(&mut self) -> Result<(), Error> {
        ensure_peer_connection(&self.rpc_path, self.peer_id).await
    }
}

impl Future for AwaitablePeer {
    type Output = Result<(), Error>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // Ensure that the peer is connected.

        if self.ensure_peer_connection.is_none() {
            let fut = Box::pin(ensure_peer_connection(
                self.rpc_path.clone(),
                self.peer_id.clone(),
            ));
            self.ensure_peer_connection = Some(fut);
        }

        let ensure_peer_connection = self.ensure_peer_connection.as_mut().unwrap();
        match ensure_peer_connection.as_mut().poll(cx) {
            std::task::Poll::Ready(result) => std::task::Poll::Ready(result),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A struct to track the status of a channel. It implements `Future` to
/// await an operable channel state before returning the spendable amount
/// on this channel.
pub struct AwaitableChannel {
    scid: ShortChannelId,
    peer_id: PublicKey,
    rpc_path: PathBuf,

    version: Option<String>,
    peer_connected: bool,
    channel_ready: bool,
    route_found: bool,

    last_check: Option<Instant>,
    rpc_call_delay: Duration,

    get_version: Option<Pin<Box<dyn Future<Output = Result<String, Error>> + Send>>>,
    ensure_peer_connection: Option<Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>>,
    billboard: Option<Pin<Box<dyn Future<Output = Result<Vec<String>, Error>> + Send>>>,
    get_route: Option<Pin<Box<dyn Future<Output = Result<GetrouteResponse, Error>> + Send>>>,
    spendable_msat: Option<Pin<Box<dyn Future<Output = Result<Amount, Error>> + Send>>>,
}

impl Future for AwaitableChannel {
    type Output = Result<Amount, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let now = Instant::now();

        if let Some(last_check) = self.last_check {
            // We already checked and still need to wait before we retry
            let elapsed = now.duration_since(last_check);
            if elapsed < self.rpc_call_delay {
                return std::task::Poll::Pending;
            }
        }

        // Get version if not set already.
        if self.version.is_none() {
            if self.get_version.is_none() {
                let fut = Box::pin(get_version(self.rpc_path.clone()));
                self.get_version = Some(fut);
            }

            let get_version = self.get_version.as_mut().unwrap();
            match get_version.as_mut().poll(cx) {
                std::task::Poll::Ready(v) => {
                    self.version = Some(v?);
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }

        // Ensure that the peer is connected.
        if !self.peer_connected {
            if self.ensure_peer_connection.is_none() {
                let fut = Box::pin(ensure_peer_connection(
                    self.rpc_path.clone(),
                    self.peer_id.clone(),
                ));
                self.ensure_peer_connection = Some(fut);
            }

            let ensure_peer_connection = self.ensure_peer_connection.as_mut().unwrap();
            match ensure_peer_connection.as_mut().poll(cx) {
                std::task::Poll::Ready(result) => {
                    result?;
                    log::debug!("Peer {} is connected", self.peer_id.to_string());
                    self.peer_connected = true;
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }

        // Ensure that the channel is reestablished.
        if !self.channel_ready {
            if self.billboard.is_none() {
                let fut = Box::pin(billboard(
                    self.rpc_path.clone(),
                    self.version.as_ref().unwrap().clone(),
                    self.peer_id.clone(),
                    self.scid,
                ));
                self.billboard = Some(fut);
            }

            let billboard = self.billboard.as_mut().unwrap();
            match billboard.as_mut().poll(cx) {
                std::task::Poll::Ready(result) => {
                    let result = result?;
                    if !result.into_iter().any(|s| {
                        s.find("Channel ready").is_some()
                            || s.find("Reconnected, and reestablished").is_some()
                    }) {
                        // Reset billboard and last_check to back-off for a bit.
                        self.last_check = Some(Instant::now());
                        self.billboard = None;
                        return std::task::Poll::Pending;
                    }
                    log::debug!("Channel {} is established", self.scid);
                    self.channel_ready = true;
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }

        // Ensure that the channel can be used to route an htlc to the peer.
        if !self.route_found {
            if self.get_route.is_none() {
                let fut = Box::pin(get_route(self.rpc_path.clone(), self.peer_id.clone()));
                self.get_route = Some(fut);
            }

            let get_route = self.get_route.as_mut().unwrap();
            match get_route.as_mut().poll(cx) {
                std::task::Poll::Ready(route) => {
                    if route.is_ok() {
                        log::debug!("Peer {:?} is routable", self.peer_id.to_string());
                        self.route_found = true;
                    } else {
                        // Reset get_route and last_check to back-off for a bit.
                        self.last_check = Some(Instant::now());
                        self.get_route = None;
                        return std::task::Poll::Pending;
                    };
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }

        // Return the amount that can be send via this channel.
        if self.spendable_msat.is_none() {
            let fut = Box::pin(spendable_msat(
                self.rpc_path.clone(),
                self.version.as_ref().unwrap().clone(),
                self.peer_id.clone(),
                self.scid,
            ));
            self.spendable_msat = Some(fut);
        }

        let spendable_msat = self.spendable_msat.as_mut().unwrap();
        match spendable_msat.as_mut().poll(cx) {
            std::task::Poll::Ready(amount) => std::task::Poll::Ready(amount),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl AwaitableChannel {
    pub async fn new(peer_id: PublicKey, scid: ShortChannelId, rpc_path: PathBuf) -> Self {
        AwaitableChannel {
            peer_id,
            scid,
            rpc_path,
            version: None,
            peer_connected: false,
            channel_ready: false,
            route_found: false,
            last_check: None,
            rpc_call_delay: Duration::from_millis(RPC_CALL_DELAY_MSEC),
            get_version: None,
            ensure_peer_connection: None,
            billboard: None,
            get_route: None,
            spendable_msat: None,
        }
    }
}

async fn connect(rpc_path: impl AsRef<Path>) -> Result<ClnRpc, Error> {
    ClnRpc::new(rpc_path)
        .await
        .map_err(|e| Error::Service(format!("cant connect to rpc {}", e.to_string())))
}

/// Try to connect to the peer if we are not already connected.
async fn ensure_peer_connection(
    rpc_path: impl AsRef<Path>,
    peer_id: PublicKey,
) -> Result<(), Error> {
    log::debug!("Checking if peer {} is connected", peer_id);

    let mut rpc = connect(rpc_path).await?;
    let res = rpc
        .call_typed(&cln_rpc::model::requests::ListpeersRequest {
            id: Some(peer_id),
            level: None,
        })
        .await?;
    let peer = res
        .peers
        .first()
        .ok_or(Error::PeerUnknown(peer_id.to_string()))?;

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
            .map_err(|_| Error::PeerConnectionFailure(peer_id.to_string()))?;

        log::debug!("Connect call to {} resulted in {:?}", peer_id, res);
    }
    Ok(())
}

async fn get_version(rpc_path: impl AsRef<Path>) -> Result<String, Error> {
    let mut rpc = connect(rpc_path).await?;
    let info = rpc.call_typed(&GetinfoRequest {}).await?;
    Ok(info.version)
}

async fn billboard(
    rpc_path: impl AsRef<Path>,
    version: String,
    peer_id: PublicKey,
    scid: ShortChannelId,
) -> Result<Vec<String>, Error> {
    let mut rpc = connect(rpc_path).await?;
    if *version >= *"v23.05gl1" {
        Ok(rpc
            .call_typed(&ListpeerchannelsRequest { id: Some(peer_id) })
            .await
            .map_err(|e| Error::Rpc(e))?
            .channels
            .into_iter()
            .filter(|c| {
                c.short_channel_id == Some(scid)
                    || c.alias.clone().and_then(|a| a.local) == Some(scid)
            })
            .nth(0)
            .ok_or(Error::Channel(
                "Could not find the channel in listpeerchannels",
            ))?
            .status
            .ok_or(Error::Channel("Status not found"))?)
    } else {
        return Err(Error::Service(format!(
            "Not supported in this version of core-lightning: {}, need at least v23.05gl1",
            version,
        )));
    }
}

async fn get_route(
    rpc_path: impl AsRef<Path>,
    peer_id: PublicKey,
) -> Result<GetrouteResponse, Error> {
    let mut rpc = connect(rpc_path).await?;
    Ok(rpc
        .call_typed(&GetrouteRequest {
            id: peer_id,
            amount_msat: cln_rpc::primitives::Amount::from_msat(1),
            riskfactor: 1,
            cltv: None,
            fromid: None,
            fuzzpercent: Some(0),
            exclude: None,
            maxhops: Some(1),
        })
        .await?)
}

async fn spendable_msat(
    rpc_path: impl AsRef<Path>,
    version: String,
    peer_id: PublicKey,
    scid: ShortChannelId,
) -> Result<Amount, Error> {
    let mut rpc = connect(rpc_path).await?;
    if *version >= *"v23.05gl1" {
        Ok(rpc
            .call_typed(&ListpeerchannelsRequest { id: Some(peer_id) })
            .await
            .map_err(|e| Error::Rpc(e))?
            .channels
            .into_iter()
            .filter(|c| {
                c.short_channel_id == Some(scid)
                    || c.alias.clone().and_then(|a| a.local) == Some(scid)
            })
            .nth(0)
            .ok_or(Error::Channel(
                "Could not find the channel in listpeerchannels",
            ))?
            .spendable_msat
            .ok_or(Error::Channel("No amount found"))?)
    } else {
        return Err(Error::Service(format!(
            "Not supported in this version of core-lightning: {}, need at least v23.05gl1",
            version,
        )));
    }
}

pub fn assert_send<T: Send>(_: T) {}
