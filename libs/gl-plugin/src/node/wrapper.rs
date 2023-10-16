use crate::LightningClient;
use anyhow::Error;
use cln_grpc::pb::{self, node_server::Node};
use log::debug;
use tonic::{Request, Response, Status};

use super::PluginNodeServer;

/// `WrappedNodeServer` enables us to quickly add customizations to
/// the pure passthru of the `cln_grpc::Server`. In particular it
/// implements the guarding against RPC commands that'd require a
/// signature if no HSM is attached (that'd lock up our node) and
/// providing RouteHints for disconnected and zeroconf channels too.
pub struct WrappedNodeServer {
    inner: cln_grpc::Server,
    node_server: PluginNodeServer,
}

// TODO Make node into a module and add the WrappedNodeServer as a submodule.
impl WrappedNodeServer {
    pub async fn new(node_server: PluginNodeServer) -> anyhow::Result<Self> {
        let inner = cln_grpc::Server::new(&node_server.rpc_path).await?;
        Ok(WrappedNodeServer { inner, node_server })
    }
}

// This would be so much easier if we have some form of delegation
// already...
#[tonic::async_trait]
impl Node for WrappedNodeServer {
    async fn invoice(
        &self,
        req: Request<pb::InvoiceRequest>,
    ) -> Result<Response<pb::InvoiceResponse>, Status> {
        let req = req.into_inner();

        use crate::rpc::LightningClient;
        let mut rpc = LightningClient::new(self.node_server.rpc_path.clone());

        // First we get the incoming channels so we can force them to
        // be added to the invoice. This is best effort and will be
        // left out if the call fails, reverting to the default
        // behavior.
        let hints: Option<Vec<Vec<pb::RouteHop>>> = self
            .get_routehints(&mut rpc)
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
                    tonic::Code::Internal,
                    format!(
                        "could not convert protobuf request into JSON-RPC request: {:?}",
                        e.to_string()
                    ),
                ));
            }
        };
        pbreq.dev_routes = hints.map(|v| {
            v.into_iter()
                .map(|e| e.into_iter().map(|ee| ee.into()).collect())
                .collect()
        });

        pbreq.cltv = match pbreq.cltv {
            Some(c) => Some(c), // Keep any set value
            None => Some(144),  // Use a day if not set
        };

        let res: Result<crate::responses::Invoice, crate::rpc::Error> =
            rpc.call("invoice", pbreq).await;

        let res: Result<cln_grpc::pb::InvoiceResponse, tonic::Status> = res
            .map(|r| cln_grpc::pb::InvoiceResponse::from(r))
            .map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Internal,
                    format!("converting invoice response to grpc: {}", e),
                )
            });

        res.map(|r| Response::new(r))
    }

    async fn getinfo(
        &self,
        r: Request<pb::GetinfoRequest>,
    ) -> Result<Response<pb::GetinfoResponse>, Status> {
        self.inner.getinfo(r).await
    }

    async fn list_peers(
        &self,
        r: Request<pb::ListpeersRequest>,
    ) -> Result<Response<pb::ListpeersResponse>, Status> {
        self.inner.list_peers(r).await
    }

    async fn list_peer_channels(
        &self,
        r: Request<pb::ListpeerchannelsRequest>,
    ) -> Result<Response<pb::ListpeerchannelsResponse>, Status> {
        self.inner.list_peer_channels(r).await
    }

    async fn list_closed_channels(
        &self,
        r: Request<pb::ListclosedchannelsRequest>,
    ) -> Result<Response<pb::ListclosedchannelsResponse>, Status> {
        self.inner.list_closed_channels(r).await
    }

    async fn list_funds(
        &self,
        r: Request<pb::ListfundsRequest>,
    ) -> Result<Response<pb::ListfundsResponse>, Status> {
        self.inner.list_funds(r).await
    }

    async fn decode_pay(
        &self,
        r: Request<pb::DecodepayRequest>,
    ) -> Result<Response<pb::DecodepayResponse>, Status> {
        self.inner.decode_pay(r).await
    }

    async fn decode(
        &self,
        r: Request<pb::DecodeRequest>,
    ) -> Result<Response<pb::DecodeResponse>, Status> {
        self.inner.decode(r).await
    }

    async fn sign_invoice(
        &self,
        r: Request<pb::SigninvoiceRequest>,
    ) -> Result<Response<pb::SigninvoiceResponse>, Status> {
        self.inner.sign_invoice(r).await
    }
    async fn pre_approve_keysend(
        &self,
        r: Request<pb::PreapprovekeysendRequest>,
    ) -> Result<Response<pb::PreapprovekeysendResponse>, Status> {
        self.inner.pre_approve_keysend(r).await
    }

    async fn pre_approve_invoice(
        &self,
        r: Request<pb::PreapproveinvoiceRequest>,
    ) -> Result<Response<pb::PreapproveinvoiceResponse>, Status> {
        self.inner.pre_approve_invoice(r).await
    }

    async fn send_custom_msg(
        &self,
        r: Request<pb::SendcustommsgRequest>,
    ) -> Result<Response<pb::SendcustommsgResponse>, Status> {
        self.inner.send_custom_msg(r).await
    }

    async fn send_pay(
        &self,
        r: Request<pb::SendpayRequest>,
    ) -> Result<Response<pb::SendpayResponse>, Status> {
        self.inner.send_pay(r).await
    }

    async fn list_channels(
        &self,
        r: Request<pb::ListchannelsRequest>,
    ) -> Result<Response<pb::ListchannelsResponse>, Status> {
        self.inner.list_channels(r).await
    }

    async fn add_gossip(
        &self,
        r: Request<pb::AddgossipRequest>,
    ) -> Result<Response<pb::AddgossipResponse>, Status> {
        self.inner.add_gossip(r).await
    }

    async fn auto_clean_invoice(
        &self,
        r: Request<pb::AutocleaninvoiceRequest>,
    ) -> Result<Response<pb::AutocleaninvoiceResponse>, Status> {
        self.inner.auto_clean_invoice(r).await
    }

    async fn check_message(
        &self,
        r: Request<pb::CheckmessageRequest>,
    ) -> Result<Response<pb::CheckmessageResponse>, Status> {
        self.inner.check_message(r).await
    }

    async fn close(
        &self,
        r: Request<pb::CloseRequest>,
    ) -> Result<Response<pb::CloseResponse>, Status> {
        self.inner.close(r).await
    }

    async fn connect_peer(
        &self,
        r: Request<pb::ConnectRequest>,
    ) -> Result<Response<pb::ConnectResponse>, Status> {
        self.inner.connect_peer(r).await
    }

    async fn create_invoice(
        &self,
        r: Request<pb::CreateinvoiceRequest>,
    ) -> Result<Response<pb::CreateinvoiceResponse>, Status> {
        self.inner.create_invoice(r).await
    }

    async fn datastore(
        &self,
        r: Request<pb::DatastoreRequest>,
    ) -> Result<Response<pb::DatastoreResponse>, Status> {
        self.inner.datastore(r).await
    }

    async fn create_onion(
        &self,
        r: Request<pb::CreateonionRequest>,
    ) -> Result<Response<pb::CreateonionResponse>, Status> {
        self.inner.create_onion(r).await
    }

    async fn del_datastore(
        &self,
        r: Request<pb::DeldatastoreRequest>,
    ) -> Result<Response<pb::DeldatastoreResponse>, Status> {
        self.inner.del_datastore(r).await
    }

    async fn del_expired_invoice(
        &self,
        r: Request<pb::DelexpiredinvoiceRequest>,
    ) -> Result<Response<pb::DelexpiredinvoiceResponse>, Status> {
        self.inner.del_expired_invoice(r).await
    }

    async fn del_invoice(
        &self,
        r: Request<pb::DelinvoiceRequest>,
    ) -> Result<Response<pb::DelinvoiceResponse>, Status> {
        self.inner.del_invoice(r).await
    }

    async fn list_datastore(
        &self,
        r: Request<pb::ListdatastoreRequest>,
    ) -> Result<Response<pb::ListdatastoreResponse>, Status> {
        self.inner.list_datastore(r).await
    }

    async fn list_invoices(
        &self,
        r: Request<pb::ListinvoicesRequest>,
    ) -> Result<Response<pb::ListinvoicesResponse>, Status> {
        self.inner.list_invoices(r).await
    }

    async fn send_onion(
        &self,
        r: Request<pb::SendonionRequest>,
    ) -> Result<Response<pb::SendonionResponse>, Status> {
        self.inner.send_onion(r).await
    }

    async fn list_send_pays(
        &self,
        r: Request<pb::ListsendpaysRequest>,
    ) -> Result<Response<pb::ListsendpaysResponse>, Status> {
        self.inner.list_send_pays(r).await
    }

    async fn list_transactions(
        &self,
        r: Request<pb::ListtransactionsRequest>,
    ) -> Result<Response<pb::ListtransactionsResponse>, Status> {
        self.inner.list_transactions(r).await
    }

    async fn pay(&self, r: Request<pb::PayRequest>) -> Result<Response<pb::PayResponse>, Status> {
        let _ = self.node_server.reconnect_peers().await;
        self.inner.pay(r).await
    }

    async fn list_nodes(
        &self,
        r: Request<pb::ListnodesRequest>,
    ) -> Result<Response<pb::ListnodesResponse>, Status> {
        self.inner.list_nodes(r).await
    }

    async fn wait_any_invoice(
        &self,
        r: Request<pb::WaitanyinvoiceRequest>,
    ) -> Result<Response<pb::WaitanyinvoiceResponse>, Status> {
        self.inner.wait_any_invoice(r).await
    }

    async fn wait_invoice(
        &self,
        r: Request<pb::WaitinvoiceRequest>,
    ) -> Result<Response<pb::WaitinvoiceResponse>, Status> {
        self.inner.wait_invoice(r).await
    }

    async fn wait_send_pay(
        &self,
        r: Request<pb::WaitsendpayRequest>,
    ) -> Result<Response<pb::WaitsendpayResponse>, Status> {
        self.inner.wait_send_pay(r).await
    }

    async fn new_addr(
        &self,
        r: Request<pb::NewaddrRequest>,
    ) -> Result<Response<pb::NewaddrResponse>, Status> {
        self.inner.new_addr(r).await
    }

    async fn withdraw(
        &self,
        r: Request<pb::WithdrawRequest>,
    ) -> Result<Response<pb::WithdrawResponse>, Status> {
        self.inner.withdraw(r).await
    }

    async fn key_send(
        &self,
        r: Request<pb::KeysendRequest>,
    ) -> Result<Response<pb::KeysendResponse>, Status> {
        self.inner.key_send(r).await
    }

    async fn fund_psbt(
        &self,
        r: Request<pb::FundpsbtRequest>,
    ) -> Result<Response<pb::FundpsbtResponse>, Status> {
        self.inner.fund_psbt(r).await
    }

    async fn send_psbt(
        &self,
        r: Request<pb::SendpsbtRequest>,
    ) -> Result<Response<pb::SendpsbtResponse>, Status> {
        self.inner.send_psbt(r).await
    }

    async fn sign_psbt(
        &self,
        r: Request<pb::SignpsbtRequest>,
    ) -> Result<Response<pb::SignpsbtResponse>, Status> {
        self.inner.sign_psbt(r).await
    }

    async fn utxo_psbt(
        &self,
        r: Request<pb::UtxopsbtRequest>,
    ) -> Result<Response<pb::UtxopsbtResponse>, Status> {
        self.inner.utxo_psbt(r).await
    }

    async fn tx_discard(
        &self,
        r: Request<pb::TxdiscardRequest>,
    ) -> Result<Response<pb::TxdiscardResponse>, Status> {
        self.inner.tx_discard(r).await
    }

    async fn tx_prepare(
        &self,
        r: Request<pb::TxprepareRequest>,
    ) -> Result<Response<pb::TxprepareResponse>, Status> {
        self.inner.tx_prepare(r).await
    }

    async fn tx_send(
        &self,
        r: Request<pb::TxsendRequest>,
    ) -> Result<Response<pb::TxsendResponse>, Status> {
        self.inner.tx_send(r).await
    }

    async fn disconnect(
        &self,
        r: Request<pb::DisconnectRequest>,
    ) -> Result<Response<pb::DisconnectResponse>, Status> {
        let inner = r.into_inner();
        let id = hex::encode(inner.id.clone());
        debug!(
            "Got a disconnect request for {}, try to delete it from the datastore peerlist.",
            id
        );

        // We try to delete the peer that we disconnect from from the datastore.
        // We don't want to be overly strict on this so we don't throw an error
        // if this does not work.
        let data_res = self
            .del_datastore(Request::new(pb::DeldatastoreRequest {
                key: vec!["greenlight".to_string(), "peerlist".to_string(), id.clone()],
                generation: None,
            }))
            .await;
        if let Err(e) = data_res {
            log::debug!("Could not delete peer {} from datastore: {}", id, e);
        }

        self.inner.disconnect(Request::new(inner.clone())).await
    }

    async fn feerates(
        &self,
        r: Request<pb::FeeratesRequest>,
    ) -> Result<Response<pb::FeeratesResponse>, Status> {
        self.inner.feerates(r).await
    }

    async fn fund_channel(
        &self,
        r: Request<pb::FundchannelRequest>,
    ) -> Result<Response<pb::FundchannelResponse>, Status> {
        self.inner.fund_channel(r).await
    }

    async fn get_route(
        &self,
        r: Request<pb::GetrouteRequest>,
    ) -> Result<Response<pb::GetrouteResponse>, Status> {
        self.inner.get_route(r).await
    }

    async fn list_forwards(
        &self,
        r: Request<pb::ListforwardsRequest>,
    ) -> Result<Response<pb::ListforwardsResponse>, Status> {
        self.inner.list_forwards(r).await
    }

    async fn list_pays(
        &self,
        r: Request<pb::ListpaysRequest>,
    ) -> Result<Response<pb::ListpaysResponse>, Status> {
        self.inner.list_pays(r).await
    }

    async fn ping(
        &self,
        r: Request<pb::PingRequest>,
    ) -> Result<Response<pb::PingResponse>, Status> {
        self.inner.ping(r).await
    }

    async fn set_channel(
        &self,
        r: Request<pb::SetchannelRequest>,
    ) -> Result<Response<pb::SetchannelResponse>, Status> {
        self.inner.set_channel(r).await
    }

    async fn sign_message(
        &self,
        r: Request<pb::SignmessageRequest>,
    ) -> Result<Response<pb::SignmessageResponse>, Status> {
        self.inner.sign_message(r).await
    }

    async fn stop(
        &self,
        r: Request<pb::StopRequest>,
    ) -> Result<Response<pb::StopResponse>, Status> {
        self.inner.stop(r).await
    }

    async fn static_backup(
        &self,
        r: Request<pb::StaticbackupRequest>,
    ) -> Result<Response<pb::StaticbackupResponse>, Status> {
        self.inner.static_backup(r).await
    }

    async fn list_htlcs(
        &self,
        r: Request<pb::ListhtlcsRequest>,
    ) -> Result<Response<pb::ListhtlcsResponse>, Status> {
        self.inner.list_htlcs(r).await
    }
}

impl WrappedNodeServer {
    async fn get_routehints(&self, rpc: &mut LightningClient) -> Result<Vec<pb::Routehint>, Error> {
        use crate::responses::Peer;

        // Get a list of active channels to peers so we can filter out
        // offline peers or peers with unconfirmed or closing
        // channels.
        let res = rpc
            .listpeers(None)
            .await?
            .peers
            .into_iter()
            .filter(|p| p.channels.len() > 0)
            .collect::<Vec<Peer>>();

        // Now project channels to their state and flatten into a vec
        // of short_channel_ids
        let active: Vec<String> = res
            .iter()
            .map(|p| {
                p.channels
                    .iter()
                    .filter(|c| c.state == "CHANNELD_NORMAL")
                    .filter_map(|c| c.clone().short_channel_id)
            })
            .flatten()
            .collect();

        /* Due to a bug in `listincoming` in CLN we get the real
         * `short_channel_id`, whereas we're supposed to use the
         * remote alias if the channel is unannounced. This patches
         * the issue in GL, and should work transparently once we fix
         * `listincoming`. */
        use std::collections::HashMap;
        let aliases: HashMap<String, String> = HashMap::from_iter(
            res.iter()
                .map(|p| {
                    p.channels
                        .iter()
                        .filter(|c| {
                            c.short_channel_id.is_some()
                                && c.alias.is_some()
                                && c.alias.as_ref().unwrap().remote.is_some()
                        })
                        .map(|c| {
                            (
                                c.short_channel_id.clone().unwrap(),
                                c.alias.clone().unwrap().remote.unwrap(),
                            )
                        })
                })
                .flatten(),
        );

        // Now we can listincoming, filter with the above active list,
        // and then map to become `pb::Routehint` instances
        Ok(rpc
            .listincoming()
            .await?
            .incoming
            .into_iter()
            .filter(|i| active.contains(&i.short_channel_id))
            .map(|i| {
                let base: Option<cln_rpc::primitives::Amount> =
                    i.fee_base_msat.as_str().try_into().ok();

                pb::Routehint {
                    hops: vec![pb::RouteHop {
                        id: hex::decode(i.id).expect("hex-decoding node_id"),
                        short_channel_id: aliases
                            .get(&i.short_channel_id)
                            .or(Some(&i.short_channel_id))
                            .unwrap()
                            .to_owned(),
                        feebase: base.map(|b| b.into()),
                        feeprop: i.fee_proportional_millionths,
                        expirydelta: i.cltv_expiry_delta,
                    }],
                }
            })
            .collect())
    }
}
