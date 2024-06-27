use crate::LightningClient;
use anyhow::Error;
use cln_grpc::pb::{self, node_server::Node};
use log::debug;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
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

    async fn list_funds(
        &self,
        r: Request<pb::ListfundsRequest>,
    ) -> Result<Response<pb::ListfundsResponse>, Status> {
        self.inner.list_funds(r).await
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

    async fn add_psbt_output(
        &self,
        r: tonic::Request<cln_grpc::pb::AddpsbtoutputRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::AddpsbtoutputResponse>, Status> {
        self.inner.add_psbt_output(r).await
    }

    async fn auto_clean_invoice(
        &self,
        r: Request<pb::AutocleaninvoiceRequest>,
    ) -> Result<Response<pb::AutocleaninvoiceResponse>, Status> {
        self.inner.auto_clean_invoice(r).await
    }

    async fn auto_clean_once(
        &self,
        r: tonic::Request<cln_grpc::pb::AutocleanonceRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::AutocleanonceResponse>, Status> {
        self.inner.auto_clean_once(r).await
    }

    async fn auto_clean_status(
        &self,
        r: tonic::Request<cln_grpc::pb::AutocleanstatusRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::AutocleanstatusResponse>, Status> {
        self.inner.auto_clean_status(r).await
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

    async fn datastore_usage(
        &self,
        r: Request<pb::DatastoreusageRequest>,
    ) -> Result<Response<pb::DatastoreusageResponse>, Status> {
        self.inner.datastore_usage(r).await
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

    /// Is unimplemented as it is a dev command that is dangerous to use.
    async fn dev_forget_channel(
        &self,
        _: tonic::Request<cln_grpc::pb::DevforgetchannelRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::DevforgetchannelResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "dev commands are not activated on greenlight",
        ))
    }

    async fn emergency_recover(
        &self,
        r: tonic::Request<cln_grpc::pb::EmergencyrecoverRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::EmergencyrecoverResponse>, Status> {
        self.inner.emergency_recover(r).await
    }

    async fn recover(
        &self,
        r: tonic::Request<cln_grpc::pb::RecoverRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::RecoverResponse>, Status> {
        self.inner.recover(r).await
    }

    async fn recover_channel(
        &self,
        r: tonic::Request<cln_grpc::pb::RecoverchannelRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::RecoverchannelResponse>, Status> {
        self.inner.recover_channel(r).await
    }

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

    async fn create_invoice_request(
        &self,
        r: tonic::Request<cln_grpc::pb::InvoicerequestRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::InvoicerequestResponse>, Status> {
        self.inner.create_invoice_request(r).await
    }

    async fn disable_invoice_request(
        &self,
        r: tonic::Request<cln_grpc::pb::DisableinvoicerequestRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::DisableinvoicerequestResponse>, Status> {
        self.inner.disable_invoice_request(r).await
    }

    async fn list_invoice_requests(
        &self,
        r: tonic::Request<cln_grpc::pb::ListinvoicerequestsRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::ListinvoicerequestsResponse>, Status> {
        self.inner.list_invoice_requests(r).await
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

    // Is unimplemented as greenlight uses a custom signer.
    async fn make_secret(
        &self,
        _: tonic::Request<cln_grpc::pb::MakesecretRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::MakesecretResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "make_secret is disabled on greenlight",
        ))
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

    async fn del_pay(
        &self,
        r: tonic::Request<cln_grpc::pb::DelpayRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::DelpayResponse>, Status> {
        self.inner.del_pay(r).await
    }

    async fn del_forward(
        &self,
        r: tonic::Request<cln_grpc::pb::DelforwardRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::DelforwardResponse>, Status> {
        self.inner.del_forward(r).await
    }

    async fn disable_offer(
        &self,
        r: tonic::Request<cln_grpc::pb::DisableofferRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::DisableofferResponse>, Status> {
        self.inner.disable_offer(r).await
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

    async fn fetch_invoice(
        &self,
        request: tonic::Request<pb::FetchinvoiceRequest>,
    ) -> Result<tonic::Response<pb::FetchinvoiceResponse>, tonic::Status> {
        self.inner.fetch_invoice(request).await
    }

    async fn fund_channel_cancel(
        &self,
        r: tonic::Request<cln_grpc::pb::FundchannelCancelRequest>,
    ) -> Result<tonic::Response<pb::FundchannelCancelResponse>, Status> {
        self.inner.fund_channel_cancel(r).await
    }

    async fn fund_channel_complete(
        &self,
        r: tonic::Request<cln_grpc::pb::FundchannelCompleteRequest>,
    ) -> Result<tonic::Response<pb::FundchannelCompleteResponse>, Status> {
        self.inner.fund_channel_complete(r).await
    }

    async fn fund_channel(
        &self,
        r: Request<pb::FundchannelRequest>,
    ) -> Result<Response<pb::FundchannelResponse>, Status> {
        self.inner.fund_channel(r).await
    }

    async fn fund_channel_start(
        &self,
        r: tonic::Request<cln_grpc::pb::FundchannelStartRequest>,
    ) -> Result<tonic::Response<pb::FundchannelStartResponse>, Status> {
        self.inner.fund_channel_start(r).await
    }

    async fn get_log(
        &self,
        r: tonic::Request<cln_grpc::pb::GetlogRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::GetlogResponse>, Status> {
        self.inner.get_log(r).await
    }

    async fn funder_update(
        &self,
        r: tonic::Request<cln_grpc::pb::FunderupdateRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::FunderupdateResponse>, Status> {
        self.inner.funder_update(r).await
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

    async fn list_offers(
        &self,
        r: Request<pb::ListoffersRequest>,
    ) -> Result<Response<pb::ListoffersResponse>, Status> {
        self.inner.list_offers(r).await
    }

    async fn list_pays(
        &self,
        r: Request<pb::ListpaysRequest>,
    ) -> Result<Response<pb::ListpaysResponse>, Status> {
        self.inner.list_pays(r).await
    }

    async fn list_htlcs(
        &self,
        r: Request<pb::ListhtlcsRequest>,
    ) -> Result<Response<pb::ListhtlcsResponse>, Status> {
        self.inner.list_htlcs(r).await
    }

    async fn multi_fund_channel(
        &self,
        r: tonic::Request<cln_grpc::pb::MultifundchannelRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::MultifundchannelResponse>, Status> {
        self.inner.multi_fund_channel(r).await
    }

    async fn multi_withdraw(
        &self,
        r: tonic::Request<cln_grpc::pb::MultiwithdrawRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::MultiwithdrawResponse>, Status> {
        self.inner.multi_withdraw(r).await
    }

    async fn offer(
        &self,
        r: Request<pb::OfferRequest>,
    ) -> Result<Response<pb::OfferResponse>, Status> {
        self.inner.offer(r).await
    }

    async fn open_channel_abort(
        &self,
        r: tonic::Request<cln_grpc::pb::OpenchannelAbortRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::OpenchannelAbortResponse>, Status> {
        self.inner.open_channel_abort(r).await
    }

    async fn open_channel_bump(
        &self,
        r: tonic::Request<cln_grpc::pb::OpenchannelBumpRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::OpenchannelBumpResponse>, Status> {
        self.inner.open_channel_bump(r).await
    }

    async fn open_channel_init(
        &self,
        r: tonic::Request<cln_grpc::pb::OpenchannelInitRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::OpenchannelInitResponse>, Status> {
        self.inner.open_channel_init(r).await
    }

    async fn open_channel_signed(
        &self,
        r: tonic::Request<cln_grpc::pb::OpenchannelSignedRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::OpenchannelSignedResponse>, Status> {
        self.inner.open_channel_signed(r).await
    }

    async fn open_channel_update(
        &self,
        r: tonic::Request<cln_grpc::pb::OpenchannelUpdateRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::OpenchannelUpdateResponse>, Status> {
        self.inner.open_channel_update(r).await
    }

    async fn ping(
        &self,
        r: Request<pb::PingRequest>,
    ) -> Result<Response<pb::PingResponse>, Status> {
        self.inner.ping(r).await
    }

    /// Is unimplemented as greenlight does not support plugins.
    async fn plugin(
        &self,
        _: tonic::Request<cln_grpc::pb::PluginRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::PluginResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "plugins are not supported on greenlight",
        ))
    }

    /// Is unimplemented as rene_pay is still experimental and under
    /// development.
    async fn rene_pay_status(
        &self,
        _: tonic::Request<cln_grpc::pb::RenepaystatusRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::RenepaystatusResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "rene_pay_status is dissabled on greenlight",
        ))
    }

    /// Is unimplemented as rene_pay is still experimental and under
    /// development.
    async fn rene_pay(
        &self,
        _: tonic::Request<cln_grpc::pb::RenepayRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::RenepayResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "rene_pay is dissabled on greenlight",
        ))
    }

    async fn reserve_inputs(
        &self,
        r: tonic::Request<cln_grpc::pb::ReserveinputsRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::ReserveinputsResponse>, Status> {
        self.inner.reserve_inputs(r).await
    }

    async fn send_custom_msg(
        &self,
        r: Request<pb::SendcustommsgRequest>,
    ) -> Result<Response<pb::SendcustommsgResponse>, Status> {
        self.inner.send_custom_msg(r).await
    }

    async fn send_invoice(
        &self,
        r: tonic::Request<cln_grpc::pb::SendinvoiceRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SendinvoiceResponse>, Status> {
        self.inner.send_invoice(r).await
    }

    async fn send_onion_message(
        &self,
        r: tonic::Request<cln_grpc::pb::SendonionmessageRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SendonionmessageResponse>, Status> {
        self.inner.send_onion_message(r).await
    }

    async fn set_channel(
        &self,
        r: Request<pb::SetchannelRequest>,
    ) -> Result<Response<pb::SetchannelResponse>, Status> {
        self.inner.set_channel(r).await
    }

    /// Is unimplemented as the config is set by greenlight.
    async fn set_config(
        &self,
        _: tonic::Request<cln_grpc::pb::SetconfigRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SetconfigResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "set_config is dissabled on greenlight",
        ))
    }

    async fn set_psbt_version(
        &self,
        r: tonic::Request<cln_grpc::pb::SetpsbtversionRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SetpsbtversionResponse>, Status> {
        self.inner.set_psbt_version(r).await
    }

    async fn sign_invoice(
        &self,
        r: Request<pb::SigninvoiceRequest>,
    ) -> Result<Response<pb::SigninvoiceResponse>, Status> {
        self.inner.sign_invoice(r).await
    }

    async fn sign_message(
        &self,
        r: Request<pb::SignmessageRequest>,
    ) -> Result<Response<pb::SignmessageResponse>, Status> {
        self.inner.sign_message(r).await
    }

    /// Is unimplemented as splicing it is still experimental.
    async fn splice_init(
        &self,
        _: tonic::Request<cln_grpc::pb::SpliceInitRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SpliceInitResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "splice_init is dissabled on greenlight",
        ))
    }

    /// Is unimplemented as splicing it is still experimental.
    async fn splice_signed(
        &self,
        _: tonic::Request<cln_grpc::pb::SpliceSignedRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SpliceSignedResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "splice_signed is dissabled on greenlight",
        ))
    }

    /// Is unimplemented as splicing it is still experimental.
    async fn splice_update(
        &self,
        _: tonic::Request<cln_grpc::pb::SpliceUpdateRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::SpliceUpdateResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "splice_update is dissabled on greenlight",
        ))
    }

    async fn unreserve_inputs(
        &self,
        r: tonic::Request<cln_grpc::pb::UnreserveinputsRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::UnreserveinputsResponse>, Status> {
        self.inner.unreserve_inputs(r).await
    }

    /// Is unimplemented as greenlight does not have ps2h outputs.
    async fn upgrade_wallet(
        &self,
        _: tonic::Request<cln_grpc::pb::UpgradewalletRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::UpgradewalletResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "upgrade_wallet is dissabled on greenlight",
        ))
    }

    async fn wait_block_height(
        &self,
        r: Request<pb::WaitblockheightRequest>,
    ) -> Result<Response<pb::WaitblockheightResponse>, Status> {
        self.inner.wait_block_height(r).await
    }

    async fn wait(
        &self,
        request: tonic::Request<pb::WaitRequest>,
    ) -> Result<tonic::Response<pb::WaitResponse>, tonic::Status> {
        self.inner.wait(request).await
    }

    /// Is unimplemented as greenlight sets the config.
    async fn list_configs(
        &self,
        _: tonic::Request<cln_grpc::pb::ListconfigsRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::ListconfigsResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "list_configs is dissabled on greenlight",
        ))
    }

    async fn stop(
        &self,
        r: Request<pb::StopRequest>,
    ) -> Result<Response<pb::StopResponse>, Status> {
        self.inner.stop(r).await
    }

    async fn help(
        &self,
        r: tonic::Request<cln_grpc::pb::HelpRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::HelpResponse>, Status> {
        self.inner.help(r).await
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

    async fn static_backup(
        &self,
        r: Request<pb::StaticbackupRequest>,
    ) -> Result<Response<pb::StaticbackupResponse>, Status> {
        self.inner.static_backup(r).await
    }

    async fn bkpr_channels_apy(
        &self,
        r: tonic::Request<cln_grpc::pb::BkprchannelsapyRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::BkprchannelsapyResponse>, Status> {
        self.inner.bkpr_channels_apy(r).await
    }

    async fn bkpr_dump_income_csv(
        &self,
        r: tonic::Request<cln_grpc::pb::BkprdumpincomecsvRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::BkprdumpincomecsvResponse>, Status> {
        self.inner.bkpr_dump_income_csv(r).await
    }

    async fn bkpr_inspect(
        &self,
        r: tonic::Request<cln_grpc::pb::BkprinspectRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::BkprinspectResponse>, Status> {
        self.inner.bkpr_inspect(r).await
    }

    async fn bkpr_list_account_events(
        &self,
        r: tonic::Request<cln_grpc::pb::BkprlistaccounteventsRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::BkprlistaccounteventsResponse>, Status> {
        self.inner.bkpr_list_account_events(r).await
    }

    async fn bkpr_list_balances(
        &self,
        r: tonic::Request<cln_grpc::pb::BkprlistbalancesRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::BkprlistbalancesResponse>, Status> {
        self.inner.bkpr_list_balances(r).await
    }

    async fn bkpr_list_income(
        &self,
        r: Request<pb::BkprlistincomeRequest>,
    ) -> Result<Response<pb::BkprlistincomeResponse>, Status> {
        self.inner.bkpr_list_income(r).await
    }

    /// Is unimplemented as runes need to be handled by gl-signer in
    /// greenlight.
    async fn blacklist_rune(
        &self,
        _: tonic::Request<cln_grpc::pb::BlacklistruneRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::BlacklistruneResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "blacklist_rune is dissabled on greenlight",
        ))
    }

    /// Is unimplemented as runes need to be handled by gl-signer in
    /// greenlight.
    async fn check_rune(
        &self,
        _: tonic::Request<cln_grpc::pb::CheckruneRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::CheckruneResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "check_rune is dissabled on greenlight",
        ))
    }

    /// Is unimplemented as runes need to be handled by gl-signer in
    /// greenlight.
    async fn create_rune(
        &self,
        _: tonic::Request<cln_grpc::pb::CreateruneRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::CreateruneResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "create_rune is dissabled on greenlight",
        ))
    }

    /// Is unimplemented as runes need to be handled by gl-signer in
    /// greenlight.
    async fn show_runes(
        &self,
        _: tonic::Request<cln_grpc::pb::ShowrunesRequest>,
    ) -> Result<tonic::Response<cln_grpc::pb::ShowrunesResponse>, Status> {
        Err(Status::new(
            tonic::Code::Unimplemented,
            "show_runes is dissabled on greenlight",
        ))
    }

    type SubscribeBlockAddedStream =
        ReceiverStream<Result<cln_grpc::pb::BlockAddedNotification, Status>>;

    async fn subscribe_block_added(
        &self,
        r: tonic::Request<cln_grpc::pb::StreamBlockAddedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeBlockAddedStream>, Status> {
        // Fixme in cln_grpc (nepet):
        // Workaround since the custom type NotificationStream is not
        // publically accessible and can not be set as
        // `type SubscribeBlockAddedStream`.
        let mut inner_stream = self.inner.subscribe_block_added(r).await?.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(result) = inner_stream.next().await {
                let msg = match result {
                    Ok(note) => Ok(note),
                    Err(e) => {
                        debug!("got an error listening to block_added notifications {e}");
                        Err(e)
                    }
                };
                if let Err(e) = tx.send(msg.clone()).await {
                    debug!("failed to send notification {:?} to client {}", msg, e);
                    return;
                };
            }
        });

        Ok(tonic::Response::new(rx.into()))
    }

    type SubscribeChannelOpenFailedStream =
        ReceiverStream<Result<cln_grpc::pb::ChannelOpenFailedNotification, Status>>;

    async fn subscribe_channel_open_failed(
        &self,
        r: tonic::Request<cln_grpc::pb::StreamChannelOpenFailedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeChannelOpenFailedStream>, Status> {
        // Fixme in cln_grpc (nepet):
        // Workaround since the custom type NotificationStream is not
        // publically accessible and can not be set as
        // `type SubscribeChannelOpenFailedStream`.
        let mut inner_stream = self
            .inner
            .subscribe_channel_open_failed(r)
            .await?
            .into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(result) = inner_stream.next().await {
                let msg = match result {
                    Ok(note) => Ok(note),
                    Err(e) => {
                        debug!("got an error listening to channel_open_failed notifications {e}");
                        Err(e)
                    }
                };
                if let Err(e) = tx.send(msg.clone()).await {
                    debug!("failed to send notification {:?} to client {}", msg, e);
                    return;
                };
            }
        });

        Ok(tonic::Response::new(rx.into()))
    }

    type SubscribeChannelOpenedStream =
        ReceiverStream<Result<cln_grpc::pb::ChannelOpenedNotification, Status>>;

    async fn subscribe_channel_opened(
        &self,
        r: tonic::Request<cln_grpc::pb::StreamChannelOpenedRequest>,
    ) -> Result<tonic::Response<Self::SubscribeChannelOpenedStream>, Status> {
        // Fixme in cln_grpc (nepet):
        // Workaround since the custom type NotificationStream is not
        // publically accessible and can not be set as
        // `type SubscribeChannelOpenedStream`.
        let mut inner_stream = self.inner.subscribe_channel_opened(r).await?.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(result) = inner_stream.next().await {
                let msg = match result {
                    Ok(note) => Ok(note),
                    Err(e) => {
                        debug!("got an error listening to channel_opened notifications {e}");
                        Err(e)
                    }
                };
                if let Err(e) = tx.send(msg.clone()).await {
                    debug!("failed to send notification {:?} to client {}", msg, e);
                    return;
                };
            }
        });

        Ok(tonic::Response::new(rx.into()))
    }

    type SubscribeConnectStream =
        ReceiverStream<Result<cln_grpc::pb::PeerConnectNotification, Status>>;

    async fn subscribe_connect(
        &self,
        r: tonic::Request<cln_grpc::pb::StreamConnectRequest>,
    ) -> Result<tonic::Response<Self::SubscribeConnectStream>, Status> {
        // Fixme in cln_grpc (nepet):
        // Workaround since the custom type NotificationStream is not
        // publically accessible and can not be set as
        // `type SubscribeConnectStream`.
        let mut inner_stream = self.inner.subscribe_connect(r).await?.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(result) = inner_stream.next().await {
                let msg = match result {
                    Ok(note) => Ok(note),
                    Err(e) => {
                        debug!("got an error listening to connect notifications {e}");
                        Err(e)
                    }
                };
                if let Err(e) = tx.send(msg.clone()).await {
                    debug!("failed to send notification {:?} to client {}", msg, e);
                    return;
                };
            }
        });

        Ok(tonic::Response::new(rx.into()))
    }

    type SubscribeCustomMsgStream =
        ReceiverStream<Result<cln_grpc::pb::CustomMsgNotification, Status>>;

    async fn subscribe_custom_msg(
        &self,
        r: tonic::Request<cln_grpc::pb::StreamCustomMsgRequest>,
    ) -> Result<tonic::Response<Self::SubscribeCustomMsgStream>, Status> {
        // Fixme in cln_grpc (nepet):
        // Workaround since the custom type NotificationStream is not
        // publically accessible and can not be set as
        // `type SubscribeCustomMsgStream`.
        let mut inner_stream = self.inner.subscribe_custom_msg(r).await?.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(result) = inner_stream.next().await {
                let msg = match result {
                    Ok(note) => Ok(note),
                    Err(e) => {
                        debug!("got an error listening to custom_msg notifications {e}");
                        Err(e)
                    }
                };
                if let Err(e) = tx.send(msg.clone()).await {
                    debug!("failed to send notification {:?} to client {}", msg, e);
                    return;
                };
            }
        });

        Ok(tonic::Response::new(rx.into()))
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
                        scid: aliases
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

use crate::pb::{
    node_server::Node as GlNode, Custommsg, Empty, HsmRequest, HsmResponse, IncomingPayment,
    LogEntry, StreamCustommsgRequest, StreamIncomingFilter, StreamLogRequest,
};

#[tonic::async_trait]
impl GlNode for WrappedNodeServer {
    type StreamCustommsgStream = ReceiverStream<Result<Custommsg, Status>>;
    type StreamHsmRequestsStream = ReceiverStream<Result<HsmRequest, Status>>;
    type StreamLogStream = ReceiverStream<Result<LogEntry, Status>>;
    type StreamIncomingStream = ReceiverStream<Result<IncomingPayment, Status>>;

    async fn stream_incoming(
        &self,
        req: tonic::Request<StreamIncomingFilter>,
    ) -> Result<Response<Self::StreamIncomingStream>, Status> {
        self.node_server.stream_incoming(req).await
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
}
