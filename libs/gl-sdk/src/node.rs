use crate::{credentials::Credentials, signer::Handle, util::exec, Error};
use std::sync::atomic::{AtomicBool, Ordering};
use gl_client::credentials::NodeIdProvider;
use gl_client::node::{Client as GlClient, ClnClient, Node as ClientNode};
use gl_client::pb::{self as glpb, cln as clnpb};
use lightning_invoice::Bolt11Invoice;
use std::sync::{Arc, Mutex};
use tokio::sync::OnceCell;

/// The `Node` is an RPC stub representing the node running in the
/// cloud. It is the main entrypoint to interact with the node.
#[derive(uniffi::Object)]
#[allow(unused)]
pub struct Node {
    inner: ClientNode,
    cln_client: OnceCell<ClnClient>,
    gl_client: OnceCell<GlClient>,
    stored_credentials: Option<Credentials>,
    signer_handle: Option<Handle>,
    disconnected: AtomicBool,
}

#[uniffi::export]
impl Node {
    #[uniffi::constructor()]
    pub fn new(credentials: &Credentials) -> Result<Self, Error> {
        let node_id = credentials
            .inner
            .node_id()
            .map_err(|_e| Error::UnparseableCreds())?;
        let inner = ClientNode::new(node_id, credentials.inner.clone())
            .expect("infallible client instantiation");

        let cln_client = OnceCell::const_new();
        let gl_client = OnceCell::const_new();
        Ok(Node {
            inner,
            cln_client,
            gl_client,
            stored_credentials: Some(credentials.clone()),
            signer_handle: None,
            disconnected: AtomicBool::new(false),
        })
    }

    /// Stop the node if it is currently running.
    pub fn stop(&self) -> Result<(), Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::StopRequest {};

        // It's ok, the error here is expected and should just be
        // telling us that we've lost the connection. This is to
        // be expected on shutdown, so we clamp this to success.
        let _ = exec(cln_client.stop(req));
        Ok(())
    }

    /// Returns the serialized credentials for this node.
    /// The app should persist these bytes and pass them to connect() on next launch.
    pub fn credentials(&self) -> Result<Vec<u8>, Error> {
        match &self.stored_credentials {
            Some(creds) => creds.save(),
            None => Err(Error::Other(
                "No credentials stored. Use register/recover/connect to create a Node with credentials.".to_string(),
            )),
        }
    }

    /// Disconnects from the node and stops the signer if running.
    /// After disconnect, all RPC methods will return an error.
    /// Safe to call multiple times.
    pub fn disconnect(&self) -> Result<(), Error> {
        self.disconnected.store(true, Ordering::Relaxed);
        if let Some(ref handle) = self.signer_handle {
            handle.try_stop();
        }
        Ok(())
    }

    /// Receive an off-chain payment.
    ///
    /// This method generates a request for a payment, also called an
    /// invoice, that encodes all the information, including amount
    /// and destination, for a prospective sender to send a lightning
    /// payment. The invoice includes negotiation of an LSPS2 / JIT
    /// channel, meaning that if there is no channel sufficient to
    /// receive the requested funds, the node will negotiate an
    /// opening, and when/if executed the payment will cause a channel
    /// to be created, and the incoming payment to be forwarded.
    pub fn receive(
        &self,
        label: String,
        description: String,
        amount_msat: Option<u64>,
    ) -> Result<ReceiveResponse, Error> {
        self.check_connected()?;
        let mut gl_client = exec(self.get_gl_client())?.clone();

        let req = gl_client::pb::LspInvoiceRequest {
            amount_msat: amount_msat.unwrap_or_default(),
            description: description,
            label: label,
            lsp_id: "".to_owned(),
            token: "".to_owned(),
        };
        let res = exec(gl_client.lsp_invoice(req))
            .map_err(|s| Error::Rpc(s.to_string()))?
            .into_inner();
        Ok(ReceiveResponse {
            bolt11: res.bolt11,
            opening_fee_msat: res.opening_fee_msat,
        })
    }

    pub fn send(&self, invoice: String, amount_msat: Option<u64>) -> Result<SendResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();
        let req = clnpb::PayRequest {
            amount_msat: match amount_msat {
                Some(a) => Some(clnpb::Amount { msat: a }),
                None => None,
            },

            bolt11: invoice,
            description: None,
            exclude: vec![],
            exemptfee: None,
            label: None,
            localinvreqid: None,
            maxdelay: None,
            maxfee: None,
            maxfeepercent: None,
            partial_msat: None,
            retry_for: None,
            riskfactor: None,
        };
        exec(cln_client.pay(req))
            .map_err(|e| Error::Rpc(e.to_string()))
            .map(|r| r.into_inner().into())
    }

    pub fn onchain_send(
        &self,
        destination: String,
        amount_or_all: String,
    ) -> Result<OnchainSendResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        // Decode what the user intends to do. Either we have `all`,
        // or we have an amount that we can parse.
        let (num, suffix): (String, String) = amount_or_all.chars().partition(|c| c.is_digit(10));

        let num = if num.len() > 0 {
            num.parse::<u64>().unwrap()
        } else {
            0
        };
        let satoshi = match (num, suffix.as_ref()) {
            (n, "") | (n, "sat") => clnpb::AmountOrAll {
                // No value suffix, interpret as satoshis. This is an
                // onchain RPC method, hence the sat denomination by
                // default.
                value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount {
                    msat: n * 1000,
                })),
            },
            (n, "msat") => clnpb::AmountOrAll {
                // No value suffix, interpret as satoshis. This is an
                // onchain RPC method, hence the sat denomination by
                // default.
                value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount {
                    msat: n * 1000,
                })),
            },
            (0, "all") => clnpb::AmountOrAll {
                value: Some(clnpb::amount_or_all::Value::All(true)),
            },
            (_, _) => return Err(Error::Argument("amount_or_all".to_owned(), amount_or_all)),
        };

        let req = clnpb::WithdrawRequest {
            destination: destination,
            minconf: None,
            feerate: None,
            satoshi: Some(satoshi),
            utxos: vec![],
        };

        exec(cln_client.withdraw(req))
            .map_err(|e| Error::Rpc(e.to_string()))
            .map(|r| r.into_inner().into())
    }

    pub fn onchain_receive(&self) -> Result<OnchainReceiveResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::NewaddrRequest {
            addresstype: Some(clnpb::newaddr_request::NewaddrAddresstype::All.into()),
        };

        let res = exec(cln_client.new_addr(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// Get information about the node.
    ///
    /// Returns basic information about the node including its ID,
    /// alias, network, and channel counts.
    pub fn get_info(&self) -> Result<GetInfoResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::GetinfoRequest {};

        let res = exec(cln_client.getinfo(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// List all peers connected to this node.
    ///
    /// Returns information about all peers including their connection
    /// status.
    pub fn list_peers(&self) -> Result<ListPeersResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::ListpeersRequest {
            id: None,
            level: None,
        };

        let res = exec(cln_client.list_peers(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// List all channels with peers.
    ///
    /// Returns detailed information about all channels including their
    /// state, capacity, and balances.
    pub fn list_peer_channels(&self) -> Result<ListPeerChannelsResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::ListpeerchannelsRequest { id: None };

        let res = exec(cln_client.list_peer_channels(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// List all funds available to the node.
    ///
    /// Returns information about on-chain outputs and channel funds
    /// that are available or pending.
    pub fn list_funds(&self) -> Result<ListFundsResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::ListfundsRequest { spent: None };

        let res = exec(cln_client.list_funds(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// List all invoices (received payment requests).
    /// List invoices (received payment requests).
    /// All parameters are optional filters; pass None to fetch all.
    pub fn list_invoices(
        &self,
        label: Option<String>,
        invstring: Option<String>,
        payment_hash: Option<Vec<u8>>,
        offer_id: Option<String>,
        index: Option<ListIndex>,
        start: Option<u64>,
        limit: Option<u32>,
    ) -> Result<ListInvoicesResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::ListinvoicesRequest {
            label,
            invstring,
            payment_hash,
            offer_id,
            index: index.map(|i| i.to_i32()),
            start,
            limit,
        };

        let res = exec(cln_client.list_invoices(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// List outgoing payments.
    /// All parameters are optional filters; pass None to fetch all.
    pub fn list_pays(
        &self,
        bolt11: Option<String>,
        payment_hash: Option<Vec<u8>>,
        status: Option<PayStatus>,
        index: Option<ListIndex>,
        start: Option<u64>,
        limit: Option<u32>,
    ) -> Result<ListPaysResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        // ListpaysRequest.ListpaysStatus: PENDING=0, COMPLETE=1, FAILED=2
        let cln_status = status.map(|s| match s {
            PayStatus::PENDING => 0,
            PayStatus::COMPLETE => 1,
            PayStatus::FAILED => 2,
        });

        let req = clnpb::ListpaysRequest {
            bolt11,
            payment_hash,
            status: cln_status,
            index: index.map(|i| i.to_i32()),
            start,
            limit,
        };

        let res = exec(cln_client.list_pays(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }

    /// List payments (sent and received), merged into a single timeline.
    /// Defaults to COMPLETE only. Pass a status to override the filter,
    /// or use list_invoices/list_pays for unfiltered access.
    /// Results are sorted newest-first.
    pub fn list_payments(
        &self,
        status: Option<PaymentStatus>,
    ) -> Result<ListPaymentsResponse, Error> {
        let filter = status.unwrap_or(PaymentStatus::COMPLETE);

        let invoices = self.list_invoices(None, None, None, None, None, None, None)?;
        let pays = self.list_pays(None, None, None, None, None, None)?;

        let mut payments: Vec<Payment> = Vec::new();

        for inv in &invoices.invoices {
            payments.push(Payment::from_invoice(inv));
        }
        for pay in &pays.pays {
            payments.push(Payment::from_pay(pay));
        }

        payments.retain(|p| p.status == filter);

        // Sort newest first by created_at (None sorts last)
        payments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(ListPaymentsResponse { payments })
    }

    /// Stream real-time events from the node.
    ///
    /// Returns a `NodeEventStream` iterator. Call `next()` repeatedly
    /// to receive events as they occur (e.g., invoice payments).
    ///
    /// The `next()` method blocks the calling thread until an event
    /// is available, but does not block the underlying async runtime,
    /// so other node methods can be called concurrently from other
    /// threads.
    pub fn stream_node_events(&self) -> Result<Arc<NodeEventStream>, Error> {
        self.check_connected()?;
        let mut gl_client = exec(self.get_gl_client())?.clone();
        let req = glpb::NodeEventsRequest {};
        let stream = exec(gl_client.stream_node_events(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(Arc::new(NodeEventStream {
            inner: Mutex::new(stream),
        }))
    }
}

// Not exported through uniffi
impl Node {
    fn check_connected(&self) -> Result<(), Error> {
        if self.disconnected.load(Ordering::Relaxed) {
            return Err(Error::Other("Node is disconnected".to_string()));
        }
        Ok(())
    }

    /// Internal constructor used by the high-level register/recover/connect functions.
    /// Creates a Node with credentials and signer handle attached.
    pub(crate) fn with_signer(
        credentials: Credentials,
        handle: Handle,
    ) -> Result<Self, Error> {
        let node_id = credentials
            .inner
            .node_id()
            .map_err(|_e| Error::UnparseableCreds())?;
        let inner = ClientNode::new(node_id, credentials.inner.clone())
            .expect("infallible client instantiation");

        let cln_client = OnceCell::const_new();
        let gl_client = OnceCell::const_new();
        Ok(Node {
            inner,
            cln_client,
            gl_client,
            stored_credentials: Some(credentials),
            signer_handle: Some(handle),
            disconnected: AtomicBool::new(false),
        })
    }

    async fn get_gl_client<'a>(&'a self) -> Result<&'a GlClient, Error> {
        let inner = self.inner.clone();
        self.gl_client
            .get_or_try_init(|| async { inner.schedule::<GlClient>().await })
            .await
            .map_err(|e| Error::Rpc(e.to_string()))
    }

    async fn get_cln_client<'a>(&'a self) -> Result<&'a ClnClient, Error> {
        let inner = self.inner.clone();

        self.cln_client
            .get_or_try_init(|| async { inner.schedule::<ClnClient>().await })
            .await
            .map_err(|e| Error::Rpc(e.to_string()))
    }
}

#[derive(uniffi::Record)]
pub struct OnchainSendResponse {
    pub tx: Vec<u8>,
    pub txid: Vec<u8>,
    pub psbt: String,
}

impl From<clnpb::WithdrawResponse> for OnchainSendResponse {
    fn from(other: clnpb::WithdrawResponse) -> Self {
        Self {
            tx: other.tx,
            txid: other.txid,
            psbt: other.psbt,
        }
    }
}

#[derive(uniffi::Record)]
pub struct OnchainReceiveResponse {
    pub bech32: String,
    pub p2tr: String,
}

impl From<clnpb::NewaddrResponse> for OnchainReceiveResponse {
    fn from(other: clnpb::NewaddrResponse) -> Self {
        OnchainReceiveResponse {
            bech32: other.bech32.unwrap_or_default(),
            p2tr: other.p2tr.unwrap_or_default(),
        }
    }
}

#[derive(uniffi::Record)]
pub struct SendResponse {
    pub status: PayStatus,
    pub preimage: Vec<u8>,
    pub payment_hash: Vec<u8>,
    pub destination_pubkey: Option<Vec<u8>>,
    pub amount_msat: u64,
    pub amount_sent_msat: u64,
    pub parts: u32,
}

impl From<clnpb::PayResponse> for SendResponse {
    fn from(other: clnpb::PayResponse) -> Self {
        Self {
            status: other.status.into(),
            preimage: other.payment_preimage,
            payment_hash: other.payment_hash,
            destination_pubkey: other.destination,
            amount_msat: other.amount_msat.unwrap().msat,
            amount_sent_msat: other.amount_sent_msat.unwrap().msat,
            parts: other.parts,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ReceiveResponse {
    pub bolt11: String,
    /// The fee charged by the LSP for opening a JIT channel, in
    /// millisatoshi. This is 0 if no JIT channel was needed.
    pub opening_fee_msat: u64,
}

#[derive(uniffi::Enum, Clone)]
pub enum PayStatus {
    COMPLETE = 0,
    PENDING = 1,
    FAILED = 2,
}

impl From<clnpb::pay_response::PayStatus> for PayStatus {
    fn from(other: clnpb::pay_response::PayStatus) -> Self {
        match other {
            clnpb::pay_response::PayStatus::Complete => PayStatus::COMPLETE,
            clnpb::pay_response::PayStatus::Failed => PayStatus::FAILED,
            clnpb::pay_response::PayStatus::Pending => PayStatus::PENDING,
        }
    }
}

impl From<i32> for PayStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => PayStatus::COMPLETE,
            1 => PayStatus::PENDING,
            2 => PayStatus::FAILED,
            o => panic!("Unknown pay_status {}", o),
        }
    }
}

// ============================================================
// GetInfo response types
// ============================================================

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct GetInfoResponse {
    pub id: Vec<u8>,
    pub alias: Option<String>,
    pub color: Vec<u8>,
    pub num_peers: u32,
    pub num_pending_channels: u32,
    pub num_active_channels: u32,
    pub num_inactive_channels: u32,
    pub version: String,
    pub lightning_dir: String,
    pub blockheight: u32,
    pub network: String,
    pub fees_collected_msat: u64,
}

impl From<clnpb::GetinfoResponse> for GetInfoResponse {
    fn from(other: clnpb::GetinfoResponse) -> Self {
        Self {
            id: other.id,
            alias: other.alias,
            color: other.color,
            num_peers: other.num_peers,
            num_pending_channels: other.num_pending_channels,
            num_active_channels: other.num_active_channels,
            num_inactive_channels: other.num_inactive_channels,
            version: other.version,
            lightning_dir: other.lightning_dir,
            blockheight: other.blockheight,
            network: other.network,
            fees_collected_msat: other.fees_collected_msat.map(|a| a.msat).unwrap_or(0),
        }
    }
}

// ============================================================
// ListPeers response types
// ============================================================

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct ListPeersResponse {
    pub peers: Vec<Peer>,
}

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct Peer {
    pub id: Vec<u8>,
    pub connected: bool,
    pub num_channels: Option<u32>,
    pub netaddr: Vec<String>,
    pub remote_addr: Option<String>,
    pub features: Option<Vec<u8>>,
}

impl From<clnpb::ListpeersResponse> for ListPeersResponse {
    fn from(other: clnpb::ListpeersResponse) -> Self {
        Self {
            peers: other.peers.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<clnpb::ListpeersPeers> for Peer {
    fn from(other: clnpb::ListpeersPeers) -> Self {
        Self {
            id: other.id,
            connected: other.connected,
            num_channels: other.num_channels,
            netaddr: other.netaddr,
            remote_addr: other.remote_addr,
            features: other.features,
        }
    }
}

// ============================================================
// ListPeerChannels response types
// ============================================================

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct ListPeerChannelsResponse {
    pub channels: Vec<PeerChannel>,
}

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct PeerChannel {
    pub peer_id: Vec<u8>,
    pub peer_connected: bool,
    pub state: ChannelState,
    pub short_channel_id: Option<String>,
    pub channel_id: Option<Vec<u8>>,
    pub funding_txid: Option<Vec<u8>>,
    pub funding_outnum: Option<u32>,
    pub to_us_msat: Option<u64>,
    pub total_msat: Option<u64>,
    pub spendable_msat: Option<u64>,
    pub receivable_msat: Option<u64>,
}

#[derive(Clone, uniffi::Enum)]
pub enum ChannelState {
    Openingd,
    ChanneldAwaitingLockin,
    ChanneldNormal,
    ChanneldShuttingDown,
    ClosingdSigexchange,
    ClosingdComplete,
    AwaitingUnilateral,
    FundingSpendSeen,
    Onchain,
    DualopendOpenInit,
    DualopendAwaitingLockin,
    DualopendOpenCommitted,
    DualopendOpenCommitReady,
}

impl ChannelState {
    fn from_i32(value: i32) -> Self {
        match value {
            0 => ChannelState::Openingd,
            1 => ChannelState::ChanneldAwaitingLockin,
            2 => ChannelState::ChanneldNormal,
            3 => ChannelState::ChanneldShuttingDown,
            4 => ChannelState::ClosingdSigexchange,
            5 => ChannelState::ClosingdComplete,
            6 => ChannelState::AwaitingUnilateral,
            7 => ChannelState::FundingSpendSeen,
            8 => ChannelState::Onchain,
            9 => ChannelState::DualopendOpenInit,
            10 => ChannelState::DualopendAwaitingLockin,
            11 => ChannelState::DualopendOpenCommitted,
            12 => ChannelState::DualopendOpenCommitReady,
            _ => ChannelState::Onchain, // Default fallback
        }
    }
}

impl From<clnpb::ListpeerchannelsResponse> for ListPeerChannelsResponse {
    fn from(other: clnpb::ListpeerchannelsResponse) -> Self {
        Self {
            channels: other.channels.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<clnpb::ListpeerchannelsChannels> for PeerChannel {
    fn from(other: clnpb::ListpeerchannelsChannels) -> Self {
        let state = ChannelState::from_i32(other.state);
        Self {
            peer_id: other.peer_id,
            peer_connected: other.peer_connected,
            state,
            short_channel_id: other.short_channel_id,
            channel_id: other.channel_id,
            funding_txid: other.funding_txid,
            funding_outnum: other.funding_outnum,
            to_us_msat: other.to_us_msat.map(|a| a.msat),
            total_msat: other.total_msat.map(|a| a.msat),
            spendable_msat: other.spendable_msat.map(|a| a.msat),
            receivable_msat: other.receivable_msat.map(|a| a.msat),
        }
    }
}

// ============================================================
// ListFunds response types
// ============================================================

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct ListFundsResponse {
    pub outputs: Vec<FundOutput>,
    pub channels: Vec<FundChannel>,
}

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct FundOutput {
    pub txid: Vec<u8>,
    pub output: u32,
    pub amount_msat: u64,
    pub status: OutputStatus,
    pub address: Option<String>,
    pub blockheight: Option<u32>,
}

#[derive(Clone, uniffi::Enum)]
pub enum OutputStatus {
    Unconfirmed,
    Confirmed,
    Spent,
    Immature,
}

impl OutputStatus {
    fn from_i32(value: i32) -> Self {
        match value {
            0 => OutputStatus::Unconfirmed,
            1 => OutputStatus::Confirmed,
            2 => OutputStatus::Spent,
            3 => OutputStatus::Immature,
            _ => OutputStatus::Unconfirmed, // Default fallback
        }
    }
}

#[allow(unused)]
#[derive(Clone, uniffi::Record)]
pub struct FundChannel {
    pub peer_id: Vec<u8>,
    pub our_amount_msat: u64,
    pub amount_msat: u64,
    pub funding_txid: Vec<u8>,
    pub funding_output: u32,
    pub connected: bool,
    pub state: ChannelState,
    pub short_channel_id: Option<String>,
    pub channel_id: Option<Vec<u8>>,
}

impl From<clnpb::ListfundsResponse> for ListFundsResponse {
    fn from(other: clnpb::ListfundsResponse) -> Self {
        Self {
            outputs: other.outputs.into_iter().map(|o| o.into()).collect(),
            channels: other.channels.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<clnpb::ListfundsOutputs> for FundOutput {
    fn from(other: clnpb::ListfundsOutputs) -> Self {
        let status = OutputStatus::from_i32(other.status);
        Self {
            txid: other.txid,
            output: other.output,
            amount_msat: other.amount_msat.map(|a| a.msat).unwrap_or(0),
            status,
            address: other.address,
            blockheight: other.blockheight,
        }
    }
}

impl From<clnpb::ListfundsChannels> for FundChannel {
    fn from(other: clnpb::ListfundsChannels) -> Self {
        let state = ChannelState::from_i32(other.state);
        Self {
            peer_id: other.peer_id,
            our_amount_msat: other.our_amount_msat.map(|a| a.msat).unwrap_or(0),
            amount_msat: other.amount_msat.map(|a| a.msat).unwrap_or(0),
            funding_txid: other.funding_txid,
            funding_output: other.funding_output,
            connected: other.connected,
            state,
            short_channel_id: other.short_channel_id,
            channel_id: other.channel_id,
        }
    }
}

// ============================================================
// Shared pagination types
// ============================================================

/// Index field used by CLN's paginated list RPCs.
#[derive(Clone, uniffi::Enum)]
pub enum ListIndex {
    CREATED,
    UPDATED,
}

impl ListIndex {
    fn to_i32(&self) -> i32 {
        match self {
            ListIndex::CREATED => 0,
            ListIndex::UPDATED => 1,
        }
    }
}

// ============================================================
// ListInvoices response types
// ============================================================

#[derive(Clone, uniffi::Enum)]
pub enum InvoiceStatus {
    UNPAID,
    PAID,
    EXPIRED,
}

impl From<i32> for InvoiceStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => InvoiceStatus::UNPAID,
            1 => InvoiceStatus::PAID,
            2 => InvoiceStatus::EXPIRED,
            o => panic!("Unknown invoice status {}", o),
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct Invoice {
    pub label: String,
    pub description: String,
    pub payment_hash: Vec<u8>,
    pub status: InvoiceStatus,
    pub amount_msat: Option<u64>,
    pub amount_received_msat: Option<u64>,
    pub bolt11: Option<String>,
    pub bolt12: Option<String>,
    pub paid_at: Option<u64>,
    pub expires_at: u64,
    pub payment_preimage: Option<Vec<u8>>,
    pub destination_pubkey: Option<Vec<u8>>,
}

/// Extract the payee public key from a BOLT11 invoice string.
fn pubkey_from_bolt11(bolt11: &str) -> Option<Vec<u8>> {
    let invoice: Bolt11Invoice = bolt11.parse().ok()?;
    Some(invoice.recover_payee_pub_key().serialize().to_vec())
}

impl From<clnpb::ListinvoicesInvoices> for Invoice {
    fn from(other: clnpb::ListinvoicesInvoices) -> Self {
        let destination_pubkey = other.bolt11.as_deref().and_then(pubkey_from_bolt11);
        Self {
            label: other.label,
            description: other.description.unwrap_or_default(),
            payment_hash: other.payment_hash,
            status: other.status.into(),
            amount_msat: other.amount_msat.map(|a| a.msat),
            amount_received_msat: other.amount_received_msat.map(|a| a.msat),
            bolt11: other.bolt11,
            bolt12: other.bolt12,
            paid_at: other.paid_at,
            expires_at: other.expires_at,
            payment_preimage: other.payment_preimage,
            destination_pubkey,
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct ListInvoicesResponse {
    pub invoices: Vec<Invoice>,
}

impl From<clnpb::ListinvoicesResponse> for ListInvoicesResponse {
    fn from(other: clnpb::ListinvoicesResponse) -> Self {
        Self {
            invoices: other.invoices.into_iter().map(|i| i.into()).collect(),
        }
    }
}

// ============================================================
// ListPays response types
// ============================================================

#[derive(Clone, uniffi::Record)]
pub struct Pay {
    pub payment_hash: Vec<u8>,
    pub status: PayStatus,
    pub destination_pubkey: Option<Vec<u8>>,
    pub amount_msat: Option<u64>,
    pub amount_sent_msat: Option<u64>,
    pub label: Option<String>,
    pub bolt11: Option<String>,
    pub description: Option<String>,
    pub bolt12: Option<String>,
    pub preimage: Option<Vec<u8>>,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub number_of_parts: Option<u64>,
}

impl From<clnpb::ListpaysPays> for Pay {
    fn from(other: clnpb::ListpaysPays) -> Self {
        let status = match other.status {
            0 => PayStatus::PENDING,  // ListpaysPaysStatus::PENDING = 0
            1 => PayStatus::FAILED,   // ListpaysPaysStatus::FAILED = 1
            2 => PayStatus::COMPLETE, // ListpaysPaysStatus::COMPLETE = 2
            o => panic!("Unknown listpays status {}", o),
        };
        Self {
            payment_hash: other.payment_hash,
            status,
            destination_pubkey: other.destination,
            amount_msat: other.amount_msat.map(|a| a.msat),
            amount_sent_msat: other.amount_sent_msat.map(|a| a.msat),
            label: other.label,
            bolt11: other.bolt11,
            description: other.description,
            bolt12: other.bolt12,
            preimage: other.preimage,
            created_at: other.created_at,
            completed_at: other.completed_at,
            number_of_parts: other.number_of_parts,
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct ListPaysResponse {
    pub pays: Vec<Pay>,
}

impl From<clnpb::ListpaysResponse> for ListPaysResponse {
    fn from(other: clnpb::ListpaysResponse) -> Self {
        Self {
            pays: other.pays.into_iter().map(|p| p.into()).collect(),
        }
    }
}

// ============================================================
// Unified list_payments response types
// ============================================================

#[derive(Clone, uniffi::Enum)]
pub enum PaymentDirection {
    SENT,
    RECEIVED,
}

#[derive(Clone, PartialEq, uniffi::Enum)]
pub enum PaymentStatus {
    PENDING,
    COMPLETE,
    FAILED,
    EXPIRED,
}

#[derive(Clone, uniffi::Record)]
pub struct Payment {
    pub payment_hash: Vec<u8>,
    pub direction: PaymentDirection,
    pub status: PaymentStatus,
    pub invoice_status: Option<InvoiceStatus>,
    pub pay_status: Option<PayStatus>,
    pub amount_msat: Option<u64>,
    pub fee_msat: Option<u64>,
    pub amount_total_msat: Option<u64>,
    pub preimage: Option<Vec<u8>>,
    pub destination_pubkey: Option<Vec<u8>>,
    pub description: Option<String>,
    pub bolt11: Option<String>,
    pub label: Option<String>,
    pub created_at: Option<u64>,
    pub invoice: Option<Invoice>,
    pub pay: Option<Pay>,
}

impl Payment {
    fn from_invoice(inv: &Invoice) -> Self {
        let status = match inv.status {
            InvoiceStatus::UNPAID => PaymentStatus::PENDING,
            InvoiceStatus::PAID => PaymentStatus::COMPLETE,
            InvoiceStatus::EXPIRED => PaymentStatus::EXPIRED,
        };
        // Invoices have no created_at in CLN. Use paid_at if available,
        // fall back to expires_at for unpaid/expired invoices.
        let created_at = inv.paid_at.or(Some(inv.expires_at));
        let amount_msat = inv.amount_received_msat;
        Self {
            payment_hash: inv.payment_hash.clone(),
            direction: PaymentDirection::RECEIVED,
            status,
            invoice_status: Some(inv.status.clone()),
            pay_status: None,
            amount_msat,
            fee_msat: None,
            amount_total_msat: amount_msat,
            preimage: inv.payment_preimage.clone(),
            destination_pubkey: inv.destination_pubkey.clone(),
            description: if inv.description.is_empty() {
                None
            } else {
                Some(inv.description.clone())
            },
            bolt11: inv.bolt11.clone(),
            label: Some(inv.label.clone()),
            created_at,
            invoice: Some(inv.clone()),
            pay: None,
        }
    }

    fn from_pay(pay: &Pay) -> Self {
        let status = match pay.status {
            PayStatus::PENDING => PaymentStatus::PENDING,
            PayStatus::COMPLETE => PaymentStatus::COMPLETE,
            PayStatus::FAILED => PaymentStatus::FAILED,
        };
        let fee_msat = match (pay.amount_sent_msat, pay.amount_msat) {
            (Some(sent), Some(amt)) if sent >= amt => Some(sent - amt),
            _ => None,
        };
        Self {
            payment_hash: pay.payment_hash.clone(),
            direction: PaymentDirection::SENT,
            status,
            invoice_status: None,
            pay_status: Some(pay.status.clone()),
            amount_msat: pay.amount_msat,
            fee_msat,
            amount_total_msat: pay.amount_sent_msat,
            preimage: pay.preimage.clone(),
            destination_pubkey: pay.destination_pubkey.clone(),
            description: pay.description.clone(),
            bolt11: pay.bolt11.clone(),
            label: pay.label.clone(),
            created_at: Some(pay.created_at),
            invoice: None,
            pay: Some(pay.clone()),
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct ListPaymentsResponse {
    pub payments: Vec<Payment>,
}

// ============================================================
// NodeEvent streaming types
// ============================================================

/// A stream of node events. Call `next()` to receive the next event.
///
/// The stream is backed by a gRPC streaming connection to the node.
/// Each call to `next()` blocks the calling thread until an event is
/// available, but does not block the tokio runtime - other node
/// operations can proceed concurrently from other threads.
#[derive(uniffi::Object)]
pub struct NodeEventStream {
    inner: Mutex<tonic::codec::Streaming<glpb::NodeEvent>>,
}

#[uniffi::export]
impl NodeEventStream {
    /// Get the next event from the stream.
    ///
    /// Blocks the calling thread until an event is available or the
    /// stream ends. Returns `None` when the stream is exhausted or
    /// the connection is lost.
    pub fn next(&self) -> Result<Option<NodeEvent>, Error> {
        let mut stream = self.inner.lock().map_err(|e| Error::Other(e.to_string()))?;
        match exec(stream.message()) {
            Ok(Some(event)) => Ok(Some(event.into())),
            Ok(None) => Ok(None),
            Err(e) if e.code() == tonic::Code::Unknown => Ok(None),
            Err(e) => Err(Error::Rpc(e.to_string())),
        }
    }
}

/// A real-time event from the node.
#[derive(Clone, uniffi::Enum)]
pub enum NodeEvent {
    /// An invoice was paid.
    InvoicePaid { details: InvoicePaidEvent },
    /// An unknown event type was received. This can happen if the
    /// server sends a new event type that this client doesn't know about.
    Unknown,
}

/// Details of a paid invoice.
#[derive(Clone, uniffi::Record)]
pub struct InvoicePaidEvent {
    /// The payment hash of the paid invoice.
    pub payment_hash: Vec<u8>,
    /// The bolt11 invoice string.
    pub bolt11: String,
    /// The preimage that proves payment.
    pub preimage: Vec<u8>,
    /// The label assigned to the invoice.
    pub label: String,
    /// Amount received in millisatoshis.
    pub amount_msat: u64,
}

impl From<glpb::NodeEvent> for NodeEvent {
    fn from(other: glpb::NodeEvent) -> Self {
        match other.event {
            Some(glpb::node_event::Event::InvoicePaid(paid)) => NodeEvent::InvoicePaid {
                details: InvoicePaidEvent {
                    payment_hash: paid.payment_hash,
                    bolt11: paid.bolt11,
                    preimage: paid.preimage,
                    label: paid.label,
                    amount_msat: paid.amount_msat,
                },
            },
            None => NodeEvent::Unknown,
        }
    }
}
