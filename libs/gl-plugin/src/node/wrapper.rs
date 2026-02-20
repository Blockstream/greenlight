use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Error;
use cln_grpc;
use cln_grpc::pb::{self, node_server::Node};
use cln_rpc::primitives::ChannelState;
use cln_rpc::{self};
use log::debug;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use super::PluginNodeServer;

/// `WrappedNodeServer` enables us to quickly add customizations to
/// the pure passthru of the `cln_grpc::Server`. In particular it
/// implements the guarding against RPC commands that'd require a
/// signature if no HSM is attached (that'd lock up our node) and
/// providing RouteHints for disconnected and zeroconf channels too.
#[derive(Clone)]
pub struct WrappedNodeServer {
    inner: cln_grpc::Server,
    node_server: PluginNodeServer,
}

// TODO Make node into a module and add the WrappedNodeServer as a submodule.
impl WrappedNodeServer {
    pub async fn new(node_server: PluginNodeServer) -> anyhow::Result<Self> {
        let inner =
            cln_grpc::Server::new(&node_server.rpc_path, node_server.notifications.clone()).await?;
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

        let mut rpc = cln_rpc::ClnRpc::new(self.node_server.rpc_path.clone())
            .await
            .unwrap();

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

        let res: Result<crate::responses::Invoice, cln_rpc::RpcError> =
            rpc.call_raw("invoice", &pbreq).await;

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

    async fn list_offers(
        &self,
        r: Request<pb::ListoffersRequest>,
    ) -> Result<Response<pb::ListoffersResponse>, Status> {
        self.inner.list_offers(r).await
    }

    async fn offer(
        &self,
        r: Request<pb::OfferRequest>,
    ) -> Result<Response<pb::OfferResponse>, Status> {
        self.inner.offer(r).await
    }

    async fn bkpr_list_income(
        &self,
        r: Request<pb::BkprlistincomeRequest>,
    ) -> Result<Response<pb::BkprlistincomeResponse>, Status> {
        self.inner.bkpr_list_income(r).await
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

    async fn wait_block_height(
        &self,
        r: Request<pb::WaitblockheightRequest>,
    ) -> Result<Response<pb::WaitblockheightResponse>, Status> {
        self.inner.wait_block_height(r).await
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

    async fn datastore_usage(
        &self,
        r: Request<pb::DatastoreusageRequest>,
    ) -> Result<Response<pb::DatastoreusageResponse>, Status> {
        self.inner.datastore_usage(r).await
    }

    async fn fetch_invoice(
        &self,
        request: tonic::Request<pb::FetchinvoiceRequest>,
    ) -> Result<tonic::Response<pb::FetchinvoiceResponse>, tonic::Status> {
        self.inner.fetch_invoice(request).await
    }

    async fn wait(
        &self,
        request: tonic::Request<pb::WaitRequest>,
    ) -> Result<tonic::Response<pb::WaitResponse>, tonic::Status> {
        self.inner.wait(request).await
    }

    async fn add_psbt_output(
        &self,
        request: tonic::Request<pb::AddpsbtoutputRequest>,
    ) -> Result<tonic::Response<pb::AddpsbtoutputResponse>, tonic::Status> {
        self.inner.add_psbt_output(request).await
    }

    async fn auto_clean_once(
        &self,
        request: tonic::Request<pb::AutocleanonceRequest>,
    ) -> Result<tonic::Response<pb::AutocleanonceResponse>, tonic::Status> {
        self.inner.auto_clean_once(request).await
    }

    async fn auto_clean_status(
        &self,
        request: tonic::Request<pb::AutocleanstatusRequest>,
    ) -> Result<tonic::Response<pb::AutocleanstatusResponse>, tonic::Status> {
        self.inner.auto_clean_status(request).await
    }

    async fn dev_forget_channel(
        &self,
        request: tonic::Request<pb::DevforgetchannelRequest>,
    ) -> Result<tonic::Response<pb::DevforgetchannelResponse>, tonic::Status> {
        self.inner.dev_forget_channel(request).await
    }

    async fn emergency_recover(
        &self,
        request: tonic::Request<pb::EmergencyrecoverRequest>,
    ) -> Result<tonic::Response<pb::EmergencyrecoverResponse>, tonic::Status> {
        self.inner.emergency_recover(request).await
    }

    async fn get_emergency_recover_data(
        &self,
        request: tonic::Request<pb::GetemergencyrecoverdataRequest>,
    ) -> Result<tonic::Response<pb::GetemergencyrecoverdataResponse>, tonic::Status> {
        self.inner.get_emergency_recover_data(request).await
    }

    async fn expose_secret(
        &self,
        request: tonic::Request<pb::ExposesecretRequest>,
    ) -> Result<tonic::Response<pb::ExposesecretResponse>, tonic::Status> {
        self.inner.expose_secret(request).await
    }

    async fn recover(
        &self,
        request: tonic::Request<pb::RecoverRequest>,
    ) -> Result<tonic::Response<pb::RecoverResponse>, tonic::Status> {
        self.inner.recover(request).await
    }

    async fn recover_channel(
        &self,
        request: tonic::Request<pb::RecoverchannelRequest>,
    ) -> Result<tonic::Response<pb::RecoverchannelResponse>, tonic::Status> {
        self.inner.recover_channel(request).await
    }

    async fn create_invoice_request(
        &self,
        request: tonic::Request<pb::InvoicerequestRequest>,
    ) -> Result<tonic::Response<pb::InvoicerequestResponse>, tonic::Status> {
        self.inner.create_invoice_request(request).await
    }

    async fn disable_invoice_request(
        &self,
        request: tonic::Request<pb::DisableinvoicerequestRequest>,
    ) -> Result<tonic::Response<pb::DisableinvoicerequestResponse>, tonic::Status> {
        self.inner.disable_invoice_request(request).await
    }

    async fn list_invoice_requests(
        &self,
        request: tonic::Request<pb::ListinvoicerequestsRequest>,
    ) -> Result<tonic::Response<pb::ListinvoicerequestsResponse>, tonic::Status> {
        self.inner.list_invoice_requests(request).await
    }

    async fn make_secret(
        &self,
        request: tonic::Request<pb::MakesecretRequest>,
    ) -> Result<tonic::Response<pb::MakesecretResponse>, tonic::Status> {
        self.inner.make_secret(request).await
    }

    async fn del_pay(
        &self,
        request: tonic::Request<pb::DelpayRequest>,
    ) -> Result<tonic::Response<pb::DelpayResponse>, tonic::Status> {
        self.inner.del_pay(request).await
    }

    async fn del_forward(
        &self,
        request: tonic::Request<pb::DelforwardRequest>,
    ) -> Result<tonic::Response<pb::DelforwardResponse>, tonic::Status> {
        self.inner.del_forward(request).await
    }

    async fn disable_offer(
        &self,
        request: tonic::Request<pb::DisableofferRequest>,
    ) -> Result<tonic::Response<pb::DisableofferResponse>, tonic::Status> {
        self.inner.disable_offer(request).await
    }

    async fn enable_offer(
        &self,
        request: tonic::Request<pb::EnableofferRequest>,
    ) -> Result<tonic::Response<pb::EnableofferResponse>, tonic::Status> {
        self.inner.enable_offer(request).await
    }

    async fn fund_channel_cancel(
        &self,
        request: tonic::Request<pb::FundchannelCancelRequest>,
    ) -> Result<tonic::Response<pb::FundchannelCancelResponse>, tonic::Status> {
        self.inner.fund_channel_cancel(request).await
    }

    async fn fund_channel_complete(
        &self,
        request: tonic::Request<pb::FundchannelCompleteRequest>,
    ) -> Result<tonic::Response<pb::FundchannelCompleteResponse>, tonic::Status> {
        self.inner.fund_channel_complete(request).await
    }

    async fn fund_channel_start(
        &self,
        request: tonic::Request<pb::FundchannelStartRequest>,
    ) -> Result<tonic::Response<pb::FundchannelStartResponse>, tonic::Status> {
        self.inner.fund_channel_start(request).await
    }

    async fn get_log(
        &self,
        request: tonic::Request<pb::GetlogRequest>,
    ) -> Result<tonic::Response<pb::GetlogResponse>, tonic::Status> {
        self.inner.get_log(request).await
    }

    async fn funder_update(
        &self,
        request: tonic::Request<pb::FunderupdateRequest>,
    ) -> Result<tonic::Response<pb::FunderupdateResponse>, tonic::Status> {
        self.inner.funder_update(request).await
    }

    async fn list_addresses(
        &self,
        request: tonic::Request<pb::ListaddressesRequest>,
    ) -> Result<tonic::Response<pb::ListaddressesResponse>, tonic::Status> {
        self.inner.list_addresses(request).await
    }

    async fn multi_fund_channel(
        &self,
        request: tonic::Request<pb::MultifundchannelRequest>,
    ) -> Result<tonic::Response<pb::MultifundchannelResponse>, tonic::Status> {
        self.inner.multi_fund_channel(request).await
    }

    async fn multi_withdraw(
        &self,
        request: tonic::Request<pb::MultiwithdrawRequest>,
    ) -> Result<tonic::Response<pb::MultiwithdrawResponse>, tonic::Status> {
        self.inner.multi_withdraw(request).await
    }

    async fn open_channel_abort(
        &self,
        request: tonic::Request<pb::OpenchannelAbortRequest>,
    ) -> Result<tonic::Response<pb::OpenchannelAbortResponse>, tonic::Status> {
        self.inner.open_channel_abort(request).await
    }

    async fn open_channel_bump(
        &self,
        request: tonic::Request<pb::OpenchannelBumpRequest>,
    ) -> Result<tonic::Response<pb::OpenchannelBumpResponse>, tonic::Status> {
        self.inner.open_channel_bump(request).await
    }

    async fn open_channel_init(
        &self,
        request: tonic::Request<pb::OpenchannelInitRequest>,
    ) -> Result<tonic::Response<pb::OpenchannelInitResponse>, tonic::Status> {
        self.inner.open_channel_init(request).await
    }

    async fn open_channel_signed(
        &self,
        request: tonic::Request<pb::OpenchannelSignedRequest>,
    ) -> Result<tonic::Response<pb::OpenchannelSignedResponse>, tonic::Status> {
        self.inner.open_channel_signed(request).await
    }

    async fn open_channel_update(
        &self,
        request: tonic::Request<pb::OpenchannelUpdateRequest>,
    ) -> Result<tonic::Response<pb::OpenchannelUpdateResponse>, tonic::Status> {
        self.inner.open_channel_update(request).await
    }

    async fn plugin(
        &self,
        request: tonic::Request<pb::PluginRequest>,
    ) -> Result<tonic::Response<pb::PluginResponse>, tonic::Status> {
        self.inner.plugin(request).await
    }

    async fn rene_pay_status(
        &self,
        request: tonic::Request<pb::RenepaystatusRequest>,
    ) -> Result<tonic::Response<pb::RenepaystatusResponse>, tonic::Status> {
        self.inner.rene_pay_status(request).await
    }

    async fn rene_pay(
        &self,
        request: tonic::Request<pb::RenepayRequest>,
    ) -> Result<tonic::Response<pb::RenepayResponse>, tonic::Status> {
        self.inner.rene_pay(request).await
    }

    async fn reserve_inputs(
        &self,
        request: tonic::Request<pb::ReserveinputsRequest>,
    ) -> Result<tonic::Response<pb::ReserveinputsResponse>, tonic::Status> {
        self.inner.reserve_inputs(request).await
    }

    async fn send_invoice(
        &self,
        request: tonic::Request<pb::SendinvoiceRequest>,
    ) -> Result<tonic::Response<pb::SendinvoiceResponse>, tonic::Status> {
        self.inner.send_invoice(request).await
    }

    async fn set_config(
        &self,
        request: tonic::Request<pb::SetconfigRequest>,
    ) -> Result<tonic::Response<pb::SetconfigResponse>, tonic::Status> {
        self.inner.set_config(request).await
    }

    async fn set_psbt_version(
        &self,
        request: tonic::Request<pb::SetpsbtversionRequest>,
    ) -> Result<tonic::Response<pb::SetpsbtversionResponse>, tonic::Status> {
        self.inner.set_psbt_version(request).await
    }

    async fn splice_init(
        &self,
        request: tonic::Request<pb::SpliceInitRequest>,
    ) -> Result<tonic::Response<pb::SpliceInitResponse>, tonic::Status> {
        self.inner.splice_init(request).await
    }

    async fn splice_signed(
        &self,
        request: tonic::Request<pb::SpliceSignedRequest>,
    ) -> Result<tonic::Response<pb::SpliceSignedResponse>, tonic::Status> {
        self.inner.splice_signed(request).await
    }

    async fn splice_update(
        &self,
        request: tonic::Request<pb::SpliceUpdateRequest>,
    ) -> Result<tonic::Response<pb::SpliceUpdateResponse>, tonic::Status> {
        self.inner.splice_update(request).await
    }

    async fn dev_splice(
        &self,
        request: tonic::Request<pb::DevspliceRequest>,
    ) -> Result<tonic::Response<pb::DevspliceResponse>, tonic::Status> {
        self.inner.dev_splice(request).await
    }

    async fn unreserve_inputs(
        &self,
        request: tonic::Request<pb::UnreserveinputsRequest>,
    ) -> Result<tonic::Response<pb::UnreserveinputsResponse>, tonic::Status> {
        self.inner.unreserve_inputs(request).await
    }

    async fn upgrade_wallet(
        &self,
        request: tonic::Request<pb::UpgradewalletRequest>,
    ) -> Result<tonic::Response<pb::UpgradewalletResponse>, tonic::Status> {
        self.inner.upgrade_wallet(request).await
    }

    async fn list_configs(
        &self,
        request: tonic::Request<pb::ListconfigsRequest>,
    ) -> Result<tonic::Response<pb::ListconfigsResponse>, tonic::Status> {
        self.inner.list_configs(request).await
    }

    async fn help(
        &self,
        request: tonic::Request<pb::HelpRequest>,
    ) -> Result<tonic::Response<pb::HelpResponse>, tonic::Status> {
        self.inner.help(request).await
    }

    async fn bkpr_channels_apy(
        &self,
        request: tonic::Request<pb::BkprchannelsapyRequest>,
    ) -> Result<tonic::Response<pb::BkprchannelsapyResponse>, tonic::Status> {
        self.inner.bkpr_channels_apy(request).await
    }

    async fn bkpr_dump_income_csv(
        &self,
        request: tonic::Request<pb::BkprdumpincomecsvRequest>,
    ) -> Result<tonic::Response<pb::BkprdumpincomecsvResponse>, tonic::Status> {
        self.inner.bkpr_dump_income_csv(request).await
    }

    async fn bkpr_inspect(
        &self,
        request: tonic::Request<pb::BkprinspectRequest>,
    ) -> Result<tonic::Response<pb::BkprinspectResponse>, tonic::Status> {
        self.inner.bkpr_inspect(request).await
    }

    async fn bkpr_list_account_events(
        &self,
        request: tonic::Request<pb::BkprlistaccounteventsRequest>,
    ) -> Result<tonic::Response<pb::BkprlistaccounteventsResponse>, tonic::Status> {
        self.inner.bkpr_list_account_events(request).await
    }

    async fn bkpr_list_balances(
        &self,
        request: tonic::Request<pb::BkprlistbalancesRequest>,
    ) -> Result<tonic::Response<pb::BkprlistbalancesResponse>, tonic::Status> {
        self.inner.bkpr_list_balances(request).await
    }

    async fn bkpr_edit_description_by_outpoint(
        &self,
        request: tonic::Request<pb::BkpreditdescriptionbyoutpointRequest>,
    ) -> Result<tonic::Response<pb::BkpreditdescriptionbyoutpointResponse>, tonic::Status> {
        self.inner.bkpr_edit_description_by_outpoint(request).await
    }

    async fn bkpr_edit_description_by_payment_id(
        &self,
        request: tonic::Request<pb::BkpreditdescriptionbypaymentidRequest>,
    ) -> Result<tonic::Response<pb::BkpreditdescriptionbypaymentidResponse>, tonic::Status> {
        self.inner
            .bkpr_edit_description_by_payment_id(request)
            .await
    }

    async fn blacklist_rune(
        &self,
        request: tonic::Request<pb::BlacklistruneRequest>,
    ) -> Result<tonic::Response<pb::BlacklistruneResponse>, tonic::Status> {
        self.inner.blacklist_rune(request).await
    }

    async fn check_rune(
        &self,
        request: tonic::Request<pb::CheckruneRequest>,
    ) -> Result<tonic::Response<pb::CheckruneResponse>, tonic::Status> {
        self.inner.check_rune(request).await
    }

    async fn create_rune(
        &self,
        request: tonic::Request<pb::CreateruneRequest>,
    ) -> Result<tonic::Response<pb::CreateruneResponse>, tonic::Status> {
        self.inner.create_rune(request).await
    }

    async fn show_runes(
        &self,
        request: tonic::Request<pb::ShowrunesRequest>,
    ) -> Result<tonic::Response<pb::ShowrunesResponse>, tonic::Status> {
        self.inner.show_runes(request).await
    }

    async fn ask_rene_unreserve(
        &self,
        request: tonic::Request<pb::AskreneunreserveRequest>,
    ) -> Result<tonic::Response<pb::AskreneunreserveResponse>, tonic::Status> {
        self.inner.ask_rene_unreserve(request).await
    }

    async fn ask_rene_list_layers(
        &self,
        request: tonic::Request<pb::AskrenelistlayersRequest>,
    ) -> Result<tonic::Response<pb::AskrenelistlayersResponse>, tonic::Status> {
        self.inner.ask_rene_list_layers(request).await
    }

    async fn ask_rene_create_layer(
        &self,
        request: tonic::Request<pb::AskrenecreatelayerRequest>,
    ) -> Result<tonic::Response<pb::AskrenecreatelayerResponse>, tonic::Status> {
        self.inner.ask_rene_create_layer(request).await
    }

    async fn ask_rene_remove_layer(
        &self,
        request: tonic::Request<pb::AskreneremovelayerRequest>,
    ) -> Result<tonic::Response<pb::AskreneremovelayerResponse>, tonic::Status> {
        self.inner.ask_rene_remove_layer(request).await
    }

    async fn ask_rene_reserve(
        &self,
        request: tonic::Request<pb::AskrenereserveRequest>,
    ) -> Result<tonic::Response<pb::AskrenereserveResponse>, tonic::Status> {
        self.inner.ask_rene_reserve(request).await
    }

    async fn ask_rene_age(
        &self,
        request: tonic::Request<pb::AskreneageRequest>,
    ) -> Result<tonic::Response<pb::AskreneageResponse>, tonic::Status> {
        self.inner.ask_rene_age(request).await
    }

    async fn get_routes(
        &self,
        request: tonic::Request<pb::GetroutesRequest>,
    ) -> Result<tonic::Response<pb::GetroutesResponse>, tonic::Status> {
        self.inner.get_routes(request).await
    }

    async fn ask_rene_disable_node(
        &self,
        request: tonic::Request<pb::AskrenedisablenodeRequest>,
    ) -> Result<tonic::Response<pb::AskrenedisablenodeResponse>, tonic::Status> {
        self.inner.ask_rene_disable_node(request).await
    }

    async fn ask_rene_inform_channel(
        &self,
        request: tonic::Request<pb::AskreneinformchannelRequest>,
    ) -> Result<tonic::Response<pb::AskreneinformchannelResponse>, tonic::Status> {
        self.inner.ask_rene_inform_channel(request).await
    }

    async fn ask_rene_create_channel(
        &self,
        request: tonic::Request<pb::AskrenecreatechannelRequest>,
    ) -> Result<tonic::Response<pb::AskrenecreatechannelResponse>, tonic::Status> {
        self.inner.ask_rene_create_channel(request).await
    }

    async fn ask_rene_update_channel(
        &self,
        request: tonic::Request<pb::AskreneupdatechannelRequest>,
    ) -> Result<tonic::Response<pb::AskreneupdatechannelResponse>, tonic::Status> {
        self.inner.ask_rene_update_channel(request).await
    }

    async fn ask_rene_bias_channel(
        &self,
        request: tonic::Request<pb::AskrenebiaschannelRequest>,
    ) -> Result<tonic::Response<pb::AskrenebiaschannelResponse>, tonic::Status> {
        self.inner.ask_rene_bias_channel(request).await
    }

    async fn ask_rene_list_reservations(
        &self,
        request: tonic::Request<pb::AskrenelistreservationsRequest>,
    ) -> Result<tonic::Response<pb::AskrenelistreservationsResponse>, tonic::Status> {
        self.inner.ask_rene_list_reservations(request).await
    }

    async fn inject_payment_onion(
        &self,
        request: tonic::Request<pb::InjectpaymentonionRequest>,
    ) -> Result<tonic::Response<pb::InjectpaymentonionResponse>, tonic::Status> {
        self.inner.inject_payment_onion(request).await
    }

    async fn inject_onion_message(
        &self,
        request: tonic::Request<pb::InjectonionmessageRequest>,
    ) -> Result<tonic::Response<pb::InjectonionmessageResponse>, tonic::Status> {
        self.inner.inject_onion_message(request).await
    }

    async fn xpay(
        &self,
        request: tonic::Request<pb::XpayRequest>,
    ) -> Result<tonic::Response<pb::XpayResponse>, tonic::Status> {
        self.inner.xpay(request).await
    }

    type SubscribeBlockAddedStream =
        ReceiverStream<Result<pb::BlockAddedNotification, tonic::Status>>;

    async fn subscribe_block_added(
        &self,
        _request: tonic::Request<pb::StreamBlockAddedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeBlockAddedStream>, tonic::Status> {
        // TODO: Implement this once `cln_grpc::server::NotificationStream is public
        unimplemented!();
    }

    type SubscribeChannelOpenFailedStream =
        ReceiverStream<Result<pb::ChannelOpenFailedNotification, tonic::Status>>;

    async fn subscribe_channel_open_failed(
        &self,
        _request: tonic::Request<pb::StreamChannelOpenFailedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeChannelOpenFailedStream>, tonic::Status> {
        // TODO: Implement this once `cln_grpc::server::NotificationStream is public
        unimplemented!();
    }

    type SubscribeChannelOpenedStream =
        ReceiverStream<Result<pb::ChannelOpenedNotification, tonic::Status>>;
    async fn subscribe_channel_opened(
        &self,
        _request: tonic::Request<pb::StreamChannelOpenedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeChannelOpenedStream>, tonic::Status> {
        // TODO: Implement this once `cln_grpc::server::NotificationStream is public
        unimplemented!();
    }

    type SubscribeConnectStream =
        ReceiverStream<Result<pb::PeerConnectNotification, tonic::Status>>;
    async fn subscribe_connect(
        &self,
        _request: tonic::Request<pb::StreamConnectRequest>,
    ) -> Result<tonic::Response<Self::SubscribeConnectStream>, tonic::Status> {
        // TODO: Implement this once `cln_grpc::server::NotificationStream is public
        unimplemented!();
    }

    type SubscribeCustomMsgStream =
        ReceiverStream<Result<pb::CustomMsgNotification, tonic::Status>>;
    async fn subscribe_custom_msg(
        &self,
        _request: tonic::Request<pb::StreamCustomMsgRequest>,
    ) -> Result<tonic::Response<Self::SubscribeCustomMsgStream>, tonic::Status> {
        // TODO: Implement this once `cln_grpc::server::NotificationStream is public
        unimplemented!();
    }

    type SubscribeChannelStateChangedStream =
        ReceiverStream<Result<pb::ChannelStateChangedNotification, tonic::Status>>;
    async fn subscribe_channel_state_changed(
        &self,
        _request: tonic::Request<pb::StreamChannelStateChangedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeChannelStateChangedStream>, tonic::Status> {
        // TODO: Implement this once `cln_grpc::server::NotificationStream is public
        unimplemented!();
    }
}

impl WrappedNodeServer {
    async fn get_routehints(&self, rpc: &mut cln_rpc::ClnRpc) -> Result<Vec<pb::Routehint>, Error> {
        // Get a map of active channels to peers with a status of
        // "CHANNELD_NORMAL", and it aliases.
        // Due to a bug in `listincoming` in CLN we get the real
        // `short_channel_id`, whereas we're supposed to use the remote alias if
        // the channel is unannounced. This patches the issue in GL, and should
        // work transparently once we fix `listincoming`.
        let req = cln_rpc::model::requests::ListpeerchannelsRequest { id: None };
        let res = rpc.call_typed(&req).await?;
        let active_channels: HashMap<
            cln_rpc::primitives::ShortChannelId,
            cln_rpc::primitives::ShortChannelId,
        > = match res.channels.len() {
            0 => HashMap::new(),
            _ => res
                .channels
                .into_iter()
                .filter(|c| c.peer_connected && c.state == ChannelState::CHANNELD_NORMAL)
                .filter_map(|c| {
                    c.short_channel_id.map(|scid| {
                        let value = c
                            .alias
                            .as_ref()
                            .and_then(|alias| alias.remote)
                            .unwrap_or(scid);
                        (scid, value)
                    })
                })
                .collect(),
        };

        let res: crate::responses::ListIncoming = rpc
            .call_raw("listincoming", &crate::requests::ListIncoming {})
            .await?;
        let active_incoming_channels: Vec<crate::responses::IncomingChannel> = res
            .incoming
            .into_iter()
            .filter(|i| {
                if let Ok(scid) = cln_rpc::primitives::ShortChannelId::from_str(&i.short_channel_id)
                {
                    active_channels.contains_key(&scid)
                } else {
                    false
                }
            })
            .collect();

        let route_hints = active_incoming_channels
            .into_iter()
            .map(|i| {
                let base: Option<cln_rpc::primitives::Amount> =
                    i.fee_base_msat.as_str().try_into().ok();

                let scid =
                    cln_rpc::primitives::ShortChannelId::from_str(&i.short_channel_id).unwrap();
                let alias = active_channels.get(&scid).unwrap_or(&scid);
                let alias_str = format!("{}", alias);
                pb::Routehint {
                    hops: vec![pb::RouteHop {
                        id: hex::decode(i.id).expect("hex-decoding node_id"),
                        scid: alias_str,
                        feebase: base.map(|b| b.into()),
                        feeprop: i.fee_proportional_millionths,
                        expirydelta: i.cltv_expiry_delta,
                    }],
                }
            })
            .collect();

        Ok(route_hints)
    }
}

use crate::pb::{
    node_server::Node as GlNode, Custommsg, Empty, HsmRequest, HsmResponse, LogEntry,
    LspInvoiceRequest, LspInvoiceResponse, NodeEvent, NodeEventsRequest, StreamCustommsgRequest,
    StreamLogRequest,
};

#[tonic::async_trait]
impl GlNode for WrappedNodeServer {
    type StreamCustommsgStream = ReceiverStream<Result<Custommsg, Status>>;
    type StreamHsmRequestsStream = ReceiverStream<Result<HsmRequest, Status>>;
    type StreamLogStream = ReceiverStream<Result<LogEntry, Status>>;
    type StreamNodeEventsStream = ReceiverStream<Result<NodeEvent, Status>>;

    async fn lsp_invoice(
        &self,
        req: Request<LspInvoiceRequest>,
    ) -> Result<Response<LspInvoiceResponse>, Status> {
        self.node_server.lsp_invoice(req).await
    }

    async fn respond_hsm_request(
        &self,
        req: Request<HsmResponse>,
    ) -> Result<Response<Empty>, Status> {
        self.node_server.respond_hsm_request(req).await
    }

    async fn stream_hsm_requests(
        &self,
        req: Request<Empty>,
    ) -> Result<Response<Self::StreamHsmRequestsStream>, Status> {
        // Best Effort reconnection logic
        let s = self.node_server.clone();

        // First though call the `node_server` which records the
        // signer being present.
        let res = self.node_server.stream_hsm_requests(req).await;
        tokio::spawn(async move { s.reconnect_peers().await });

        res
    }

    async fn stream_log(
        &self,
        req: Request<StreamLogRequest>,
    ) -> Result<Response<Self::StreamLogStream>, Status> {
        self.node_server.stream_log(req).await
    }

    async fn stream_custommsg(
        &self,
        req: Request<StreamCustommsgRequest>,
    ) -> Result<Response<Self::StreamCustommsgStream>, Status> {
        self.node_server.stream_custommsg(req).await
    }

    async fn configure(
        &self,
        request: tonic::Request<crate::pb::GlConfig>,
    ) -> Result<tonic::Response<crate::pb::Empty>, tonic::Status> {
        self.node_server.configure(request).await
    }

    async fn trampoline_pay(
        &self,
        request: tonic::Request<crate::pb::TrampolinePayRequest>,
    ) -> Result<tonic::Response<crate::pb::TrampolinePayResponse>, Status> {
        self.node_server.trampoline_pay(request).await
    }

    async fn stream_node_events(
        &self,
        req: tonic::Request<NodeEventsRequest>,
    ) -> Result<Response<Self::StreamNodeEventsStream>, Status> {
        self.node_server.stream_node_events(req).await
    }
}
