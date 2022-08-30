use crate::config::Config;
use crate::pb::{self, node_server::Node};
use crate::rpc::LightningClient;
use crate::stager;
use anyhow::{Context, Error, Result};
use governor::{
    clock::DefaultClock, state::direct::NotKeyed, state::InMemoryState, Quota, RateLimiter,
};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    time,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::ServerTlsConfig, Code, Request, Response, Status};

lazy_static! {
    static ref LIMITER: RateLimiter<NotKeyed, InMemoryState, DefaultClock> =
        RateLimiter::direct(Quota::per_minute(core::num::NonZeroU32::new(300).unwrap()));
    static ref HSM_ID_COUNT: AtomicUsize = AtomicUsize::new(0);
    static ref RPC_BCAST: broadcast::Sender<super::Event> = broadcast::channel(4).0;
}

/// The PluginNodeServer is the interface that is exposed to client devices
/// and is in charge of coordinating the various user-controlled
/// entities. This includes dispatching incoming RPC calls to the JSON-RPC
/// interface, as well as staging requests from the HSM so that they can be
/// streamed and replied to by devices that have access to the signing keys.
#[derive(Clone)]
pub struct PluginNodeServer {
    pub tls: ServerTlsConfig,
    pub stage: Arc<stager::Stage>,
    pub rpc: Arc<Mutex<LightningClient>>,
    events: tokio::sync::broadcast::Sender<super::Event>,
    grpc_binding: String,
}

impl PluginNodeServer {
    pub fn new(
        stage: Arc<stager::Stage>,
        config: Config,
        events: tokio::sync::broadcast::Sender<super::Event>,
    ) -> Result<Self, Error> {
        let tls = ServerTlsConfig::new()
            .identity(config.identity.id)
            .client_ca_root(config.identity.ca);

        let mut sock_path = std::env::current_dir().unwrap();
        sock_path.push("lightning-rpc");
        info!("Connecting to lightning-rpc at {:?}", sock_path);

        let rpc = Arc::new(Mutex::new(LightningClient::new(sock_path)));

        // Bridge the RPC_BCAST into the events queue
        let tx = events.clone();
        tokio::spawn(async move {
            let mut rx = RPC_BCAST.subscribe();
            loop {
                if let Ok(e) = rx.recv().await {
                    let _ = tx.send(e);
                }
            }
        });

        Ok(PluginNodeServer {
            tls,
            rpc,
            stage,
            events,
            grpc_binding: config.node_grpc_binding,
        })
    }

    pub async fn get_rpc(&self) -> LightningClient {
        let rpc = self.rpc.lock().await;
        let r = rpc.clone();
        drop(rpc);
        r
    }

    /// Fetch a list of usable single-hop routehints corresponding to
    /// our active incoming channels. The tricky part here is that we
    /// listen to `listpeers` to enumerate the channels we might want
    /// to return, and then ask `listchannels` for the routing
    /// details, which comes from a different subsystem. So we need to
    /// match the two up, using `listpeers` as the minimal viable
    /// result, and then loop until that minimal result is satisfied.
    async fn get_routehints(&self) -> Result<Vec<pb::Routehint>, Error> {
        use tokio::time::{sleep, Duration};
        info!("Retrieving incoming channels for routehints");
        let rpc = self.get_rpc().await;
        let myid = rpc.getinfo().await?.id;

        // Retrieve peers so we know what to expect at a minimum.
        let peers = rpc.listpeers(None).await?.peers;
        let peer_ids: Vec<String> = peers
            .iter()
            .filter(|p| p.channels.iter().any(|c| c.state == "CHANNELD_NORMAL"))
            .map(|p| p.id.clone())
            .collect();

        info!(
            "We have {} channels recorded for peers: {:?}",
            peer_ids.len(),
            peer_ids
        );

        // Now try for up to 5 seconds to see if we get the channel's
        // we expect in the `listchannels` output.
        for _ in 1..=10 {
            let found: Vec<String> = rpc
                .listchannels(None, None, Some(myid.clone()))
                .await?
                .channels
                .into_iter()
                .map(|c| c.source)
                .collect();

            // Check that all peer_ids are also in the channels we
            // found. May mismatch the actual channel IDs, due to
            // aliasing.
            if peer_ids.iter().all(|item| found.contains(&item)) {
                break;
            } else {
                warn!(
                    "Missing channels from `listchannels`: {:?} > {:?}",
                    peer_ids, found
                );
                sleep(Duration::from_millis(500)).await;
            }
        }

        let chans = dbg!(rpc.listchannels(None, None, Some(myid.clone())).await?).channels;
        info!(
            "Currently have {} channels from listchannels: {:?}",
            chans.len(),
            chans
        );
        let hops: Vec<pb::Routehint> = chans
            .into_iter()
            .map(|i| pb::Routehint {
                hops: vec![pb::RoutehintHop {
                    node_id: hex::decode(i.source).expect("hex-decoding source node_id"),
                    short_channel_id: i.short_channel_id,
                    fee_base: i.base_fee_millisatoshi,
                    fee_prop: i.fee_per_millionth,
                    cltv_expiry_delta: i.delay,
                }],
            })
            .collect();

        info!("Returning {} hops: {:?}", hops.len(), hops);
        return Ok(hops);
    }
}

#[tonic::async_trait]
impl Node for PluginNodeServer {
    type StreamHsmRequestsStream = ReceiverStream<Result<pb::HsmRequest, Status>>;
    type StreamLogStream = ReceiverStream<Result<pb::LogEntry, Status>>;

    async fn get_info(
        &self,
        _: Request<pb::GetInfoRequest>,
    ) -> Result<Response<pb::GetInfoResponse>, Status> {
        LIMITER.until_ready().await;
        let rpc = self.get_rpc().await;

        let res: Result<crate::responses::GetInfo, clightningrpc::Error> =
            rpc.call("getinfo", json!({})).await;

        match res {
            Ok(r) => Ok(Response::new(
                r.try_into()
                    .expect("conversion to pb::GetInfoResponse failed"),
            )),
            Err(e) => Err(Status::new(Code::Unknown, e.to_string())),
        }
    }

    async fn stop(
        &self,
        _: Request<pb::StopRequest>,
    ) -> Result<Response<pb::StopResponse>, Status> {
        self.events
            .send(super::Event::Stop(self.stage.clone()))
            .unwrap();
        self.terminate().await
    }

    async fn connect_peer(
        &self,
        r: Request<pb::ConnectRequest>,
    ) -> Result<Response<pb::ConnectResponse>, Status> {
        let r = r.into_inner();
        let req = clightningrpc::requests::Connect {
            id: &r.node_id,
            host: match r.addr.as_ref() {
                "" => None,
                v => Some(v),
            },
        };

        let rpc = self.get_rpc().await;

        match rpc.connect(&req).await {
            Ok(s) => Ok(Response::new(s.into())),
            Err(e) => Err(Status::new(Code::Unknown, e.to_string())),
        }
    }

    async fn list_peers(
        &self,
        r: Request<pb::ListPeersRequest>,
    ) -> Result<Response<pb::ListPeersResponse>, Status> {
        LIMITER.until_ready().await;
        let req = r.into_inner();
        let rpc = self.rpc.lock().await;

        let node_id = match req.node_id.as_ref() {
            "" => None,
            _ => Some(req.node_id.as_str()),
        };

        match rpc.listpeers(node_id).await {
            Ok(s) => match s.try_into() {
                Ok(s) => Ok(Response::new(s)),
                Err(e) => Err(Status::new(Code::Unknown, e.to_string())),
            },
            Err(e) => Err(Status::new(Code::Unknown, e.to_string())),
        }
    }

    async fn disconnect(
        &self,
        r: Request<pb::DisconnectRequest>,
    ) -> Result<Response<pb::DisconnectResponse>, Status> {
        let req = r.into_inner();
        let rpc = self.get_rpc().await;

        let node_id = match req.node_id.as_ref() {
            "" => {
                return Err(Status::new(
                    Code::InvalidArgument,
                    "Must specify a node ID to disconnect from",
                ))
            }
            _ => req.node_id.as_str(),
        };

        match rpc.disconnect(node_id, req.force).await {
            Ok(()) => Ok(Response::new(pb::DisconnectResponse {})),
            Err(e) => Err(Status::new(Code::Unknown, e.to_string())),
        }
    }

    async fn stream_log(
        &self,
        _: Request<pb::StreamLogRequest>,
    ) -> Result<Response<Self::StreamLogStream>, Status> {
        match async {
            let (tx, rx) = mpsc::channel(1);
            let mut lines = linemux::MuxedLines::new()?;
            lines.add_file("/tmp/log").await?;

            // TODO: Yes, this may produce duplicate lines, when new
            // log entries are produced while we're streaming the
            // backlog out, but do we care?
            use tokio::io::{AsyncBufReadExt, BufReader};
            let file = tokio::fs::File::open("/tmp/log").await?;
            let mut file = BufReader::new(file).lines();

            tokio::spawn(async move {
                match async {
                    while let Some(line) = file.next_line().await? {
                        tx.send(Ok(pb::LogEntry { line: line })).await?
                    }

                    while let Ok(Some(line)) = lines.next_line().await {
                        tx.send(Ok(pb::LogEntry {
                            line: line.line().to_string(),
                        }))
                        .await?;
                    }
                    Ok(())
                }
                .await as Result<(), anyhow::Error>
                {
                    Ok(()) => {}
                    Err(e) => {
                        warn!("error streaming logs to client: {}", e);
                    }
                }
            });
            Ok(ReceiverStream::new(rx))
        }
        .await as Result<Self::StreamLogStream, anyhow::Error>
        {
            Ok(v) => Ok(Response::new(v)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn stream_hsm_requests(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<Self::StreamHsmRequestsStream>, Status> {
        let hsm_id = HSM_ID_COUNT.fetch_add(1, Ordering::SeqCst);

        info!(
            "New signer with hsm_id={} attached, streaming requests",
            hsm_id
        );

        let (tx, rx) = mpsc::channel(10);
        let mut stream = self.stage.mystream().await;
        tokio::spawn(async move {
            trace!("hsmd hsm_id={} request processor started", hsm_id);
            loop {
                let req = match stream.next().await {
                    Err(e) => {
                        error!(
                            "Could not get next request from stage: {:?} for hsm_id={}",
                            e, hsm_id
                        );
                        break;
                    }
                    Ok(r) => r,
                };
                trace!(
                    "Sending request={} to hsm_id={}",
                    req.request.request_id,
                    hsm_id
                );

                if let Err(e) = tx.send(Ok(req.request)).await {
                    warn!("Error streaming request {:?} to hsm_id={}", e, hsm_id);
                    break;
                }
            }
            info!("Signer hsm_id={} exited", hsm_id);
        });

        // Now that we have an hsmd we can actually reconnect to our peers
        let c2 = self.clone();
        tokio::spawn(async move {
            let rpc = c2.get_rpc().await;
            let res = rpc.listpeers(None).await;
            if res.is_err() {
                warn!("Could not list peers to reconnect: {:?}", res);
            }

            for p in res.unwrap().peers.iter() {
                if p.connected {
                    debug!("Already connected to {}, not reconnecting", p.id);
                    continue;
                }

                trace!("Calling connect: {:?}", &p.id);
                let res = rpc
                    .connect(&clightningrpc::requests::Connect {
                        id: &p.id,
                        host: None, // TODO Maybe we can have an extra lookup service?
                    })
                    .await;
                trace!("Connect returned: {:?} -> {:?}", &p.id, res);

                match res {
                    Ok(r) => info!("Connection to {} established: {:?}", p.id, r),
                    Err(e) => warn!("Could not connect to {}: {:?}", p.id, e),
                }
            }
        });
        trace!("Returning stream_hsm_request channel");
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn new_addr(
        &self,
        r: Request<pb::NewAddrRequest>,
    ) -> Result<Response<pb::NewAddrResponse>, Status> {
        LIMITER.until_ready().await;
        let req = r.into_inner();
        let rpc = self.rpc.lock().await;
        match rpc.newaddr(req.address_type()).await {
            Ok(a) => Ok(Response::new(pb::NewAddrResponse {
                address_type: req.address_type,
                address: a,
            })),
            Err(e) => Err(Status::new(
                Code::Internal,
                format!("could not generate a new address: {}", e.to_string()),
            )),
        }
    }

    async fn respond_hsm_request(
        &self,
        request: Request<pb::HsmResponse>,
    ) -> Result<Response<pb::Empty>, Status> {
        if let Err(e) = self.stage.respond(request.into_inner()).await {
            warn!("Suppressing error: {:?}", e);
        }
        Ok(Response::new(pb::Empty::default()))
    }

    async fn list_funds(
        &self,
        _: Request<pb::ListFundsRequest>,
    ) -> Result<Response<pb::ListFundsResponse>, Status> {
        LIMITER.until_ready().await;
        // TODO Add the spent parameter to the call
        let rpc = self.rpc.lock().await;

        let res: Result<clightningrpc::responses::ListFunds, clightningrpc::Error> =
            rpc.call("listfunds", crate::requests::ListFunds {}).await;

        match res {
            Ok(f) => Ok(Response::new(f.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn withdraw(
        &self,
        r: Request<pb::WithdrawRequest>,
    ) -> Result<Response<pb::WithdrawResponse>, Status> {
        let req = r.into_inner();
        let rpc = self.get_rpc().await;

        // protobufs create really dumb nested things, so unwrap them here.
        let amt: crate::requests::Amount = match req.amount {
            None => return Err(Status::new(Code::InvalidArgument, "amount must be set")),
            Some(a) => match a.try_into() {
                Ok(crate::requests::Amount::Any) => {
                    return Err(Status::new(
                        Code::InvalidArgument,
                        "withdraw requires a valid amount, not 'any'",
                    ))
                }
                Ok(a) => a,
                Err(e) => return Err(Status::new(Code::InvalidArgument, e.to_string())),
            },
        };

        let feerate: Option<crate::requests::Feerate> = match req.feerate {
            None => None,
            Some(v) => match v.try_into() {
                Ok(v) => Some(v),
                Err(e) => {
                    return Err(Status::new(
                        Code::InvalidArgument,
                        format!("Error parsing request: {}", e),
                    ))
                }
            },
        };

        let utxos: Result<Vec<crate::requests::Outpoint>, Error> =
            req.utxos.into_iter().map(|o| o.try_into()).collect();

        let utxos = match utxos {
            Err(e) => return Err(Status::new(Code::Internal, e.to_string())),
            Ok(u) => u,
        };

        let req = crate::requests::Withdraw {
            destination: req.destination,
            amount: amt,
            minconf: match req.minconf {
                None => None,
                Some(a) => Some(a.blocks),
            },
            utxos: match utxos.len() {
                0 => None,
                _ => Some(utxos),
            },
            feerate: feerate,
        };

        let res: Result<crate::responses::Withdraw, clightningrpc::Error> =
            rpc.call("withdraw", req).await;

        match res {
            Ok(w) => Ok(Response::new(w.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn fund_channel(
        &self,
        req: Request<pb::FundChannelRequest>,
    ) -> Result<Response<pb::FundChannelResponse>, Status> {
        let rpc = self.get_rpc().await;

        let r: crate::requests::FundChannel = match req.into_inner().try_into() {
            Ok(v) => v,
            Err(e) => return Err(Status::new(Code::InvalidArgument, e.to_string())),
        };

        let response: Result<crate::responses::FundChannel, clightningrpc::Error> =
            rpc.call("fundchannel", r).await;

        match response {
            Ok(v) => Ok(Response::new(v.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn close_channel(
        &self,
        req: Request<pb::CloseChannelRequest>,
    ) -> Result<Response<pb::CloseChannelResponse>, Status> {
        use crate::{requests, responses};
        let req = req.into_inner();
        let r = self.get_rpc().await;

        let req: requests::CloseChannel = req.into();
        let res: Result<responses::CloseChannel, clightningrpc::Error> = r.call("close", req).await;

        match res {
            // Conversion may fail, so handle that case here.
            Ok(v) => Ok(Response::new(match v.try_into() {
                Ok(v) => v,
                Err(e) => {
                    return Err(Status::new(
                        Code::Internal,
                        format!("error converting response: {}", e.to_string()),
                    ))
                }
            })),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn create_invoice(
        &self,
        req: Request<pb::InvoiceRequest>,
    ) -> Result<Response<pb::Invoice>, Status> {
        LIMITER.until_ready().await;
        let req = req.into_inner();
        let rpc = self.get_rpc().await;

        // First we get the incoming channels so we can force them to
        // be added to the invoice. This is best effort and will be
        // left out if the call fails, reverting to the default
        // behavior.
        let hints: Option<Vec<Vec<pb::RoutehintHop>>> = self
            .get_routehints()
            .await
            .map(
                // Map Result to Result
                |v| {
                    v.into_iter()
                        .map(
                            // map each vector element
                            |rh| rh.hops,
                        )
                        .collect()
                },
            )
            .ok();

        let mut pbreq: crate::requests::Invoice = match req.clone().try_into() {
            Ok(v) => v,
            Err(e) => {
                return Err(Status::new(
                    Code::Internal,
                    format!(
                        "could not convert protobuf request into JSON-RPC request: {:?}",
                        e.to_string()
                    ),
                ))
            }
        };
        pbreq.dev_routes = hints.map(|v| {
            v.into_iter()
                .map(|e| e.into_iter().map(|ee| ee.into()).collect())
                .collect()
        });

        let res: Result<crate::responses::Invoice, clightningrpc::Error> =
            rpc.call("invoice", pbreq).await;

        match res {
            Ok(v) => {
                // Ok, we got the invoice created, now backfill some
                // of the information that is not returned by the
                // c-lightning RPC
                let mut res: pb::Invoice = v.into();
                res.label = req.label;
                res.description = req.description;
                res.amount = req.amount;

                Ok(Response::new(res))
            }
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn pay(&self, req: Request<pb::PayRequest>) -> Result<Response<pb::Payment>, Status> {
        let rpc = self.get_rpc().await;
        let req = req.into_inner();
        let req: crate::requests::Pay = req.into();

        let res: Result<crate::responses::Pay, clightningrpc::Error> = rpc.call("pay", req).await;

        match res {
            // Conversion may fail, so handle that case here.
            Ok(v) => Ok(Response::new(v.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn list_payments(
        &self,
        req: tonic::Request<pb::ListPaymentsRequest>,
    ) -> Result<tonic::Response<pb::ListPaymentsResponse>, tonic::Status> {
        LIMITER.until_ready().await;
        let rpc = self.rpc.lock().await;
        let req: crate::requests::ListPays = match req.into_inner().try_into() {
            Ok(v) => v,
            Err(e) => {
                return Err(Status::new(
                    Code::InvalidArgument,
                    format!(
                        "Could not convert argument to valid JSON-RPC request: {}",
                        e
                    ),
                ))
            }
        };

        let res: Result<crate::responses::ListPays, clightningrpc::Error> =
            rpc.call("listpays", req).await;

        match res {
            Ok(v) => Ok(Response::new(v.try_into().unwrap())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    async fn list_invoices(
        &self,
        req: tonic::Request<pb::ListInvoicesRequest>,
    ) -> Result<tonic::Response<pb::ListInvoicesResponse>, tonic::Status> {
        LIMITER.until_ready().await;
        let req = req.into_inner();
        let req: crate::requests::ListInvoices = match req.try_into() {
            Ok(v) => v,
            Err(e) => return Err(Status::new(Code::InvalidArgument, e.to_string())),
        };
        let rpc = self.rpc.lock().await;
        let res: Result<crate::responses::ListInvoices, clightningrpc::Error> =
            rpc.call("listinvoices", req).await;

        match res {
            Ok(v) => Ok(Response::new(v.try_into().unwrap())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }

    type StreamIncomingStream = ReceiverStream<Result<pb::IncomingPayment, Status>>;

    async fn stream_incoming(
        &self,
        _req: tonic::Request<pb::StreamIncomingFilter>,
    ) -> Result<Response<Self::StreamIncomingStream>, Status> {
        // TODO See if we can just return the broadcast::Receiver
        // instead of pulling off broadcast and into an mpsc.
        let (tx, rx) = mpsc::channel(1);
        let mut bcast = self.events.subscribe();
        tokio::spawn(async move {
            while let Ok(p) = bcast.recv().await {
                match p {
                    super::Event::IncomingPayment(p) => {
                        let _ = tx.send(Ok(p)).await;
                    }
                    _ => {}
                }
            }
        });

        return Ok(Response::new(ReceiverStream::new(rx)));
    }

    async fn keysend(
        &self,
        request: tonic::Request<pb::KeysendRequest>,
    ) -> Result<tonic::Response<pb::Payment>, tonic::Status> {
        match async {
            let rpcreq: crate::requests::Keysend = request.into_inner().try_into().unwrap();
            let rpc = self.get_rpc().await;
            let res: Result<crate::responses::Keysend, clightningrpc::Error> =
                rpc.call("keysend", rpcreq).await;

            res
        }
        .await
        {
            Ok(v) => Ok(Response::new(v.into())),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        }
    }
}

use nix::sys::signal;
use nix::unistd::getppid;
impl PluginNodeServer {
    pub async fn run(self) -> Result<()> {
        let addr = self.grpc_binding.parse().unwrap();
        let router = tonic::transport::Server::builder()
            .tcp_keepalive(Some(tokio::time::Duration::from_secs(1)))
            .tls_config(self.tls.clone())?
            .add_service(crate::pb::node_server::NodeServer::with_interceptor(
                self.clone(),
                intercept,
            ));
        info!("Starting grpc server on {}", addr);

        let rpc = self.rpc.clone();
        tokio::spawn(async move {
            debug!("Locking grpc interface until the JSON-RPC interface becomes available.");
            use tokio::time::{sleep, Duration};
            let rpc = rpc.lock().await;
            loop {
                let res: Result<crate::responses::GetInfo, clightningrpc::Error> =
                    rpc.call("getinfo", json!({})).await;
                match res {
                    Ok(_) => break,
                    Err(e) => {
                        trace!(
                            "JSON-RPC interface not yet available. Delaying 50ms. {:?}",
                            e
                        );
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        });

        router
            .serve(addr)
            .await
            .context("grpc interface exited with error")
    }

    /// Do your best to kill `lightningd`, by sending a TERM signal, give
    /// it a couple of seconds and then sending a KILL.
    async fn kill(&self) -> ! {
        let ppid = getppid();
        signal::kill(ppid, signal::Signal::SIGTERM).expect("sending SIGTERM");
        time::sleep(time::Duration::from_secs(5)).await;
        signal::kill(ppid, signal::Signal::SIGKILL).expect("sending SIGKILL");
        std::process::exit(0);
    }

    async fn terminate(&self) -> ! {
        // Give the node some time to stop gracefully, otherwise kill
        // it. We need to call `stop` in a task because we might not
        // have completed startup. This can happen if we're stuck
        // waiting on the signer.
        let rpc = self.get_rpc().await;
        tokio::spawn(async move {
            rpc.stop().await.expect("calling `stop`");
        });
        time::sleep(time::Duration::from_secs(2)).await;
        self.kill().await
    }
}

fn intercept(req: Request<()>) -> Result<Request<()>, Status> {
    // Spawn a tokio task so we can make use to the state guarded by
    // an async lock.
    let _ = RPC_BCAST.clone().send(super::Event::RpcCall);
    Ok(req)
}
