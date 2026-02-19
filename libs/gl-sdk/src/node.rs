use crate::{credentials::Credentials, util::exec, Error};
use gl_client::credentials::NodeIdProvider;
use gl_client::node::{Client as GlClient, ClnClient, Node as ClientNode};

use gl_client::pb::cln as clnpb;
use tokio::sync::OnceCell;

/// The `Node` is an RPC stub representing the node running in the
/// cloud. It is the main entrypoint to interact with the node.
#[derive(uniffi::Object, Clone)]
#[allow(unused)]
pub struct Node {
    inner: ClientNode,
    cln_client: OnceCell<ClnClient>,
    gl_client: OnceCell<GlClient>,
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
        })
    }

    /// Stop the node if it is currently running.
    pub fn stop(&self) -> Result<(), Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::StopRequest {};

        // It's ok, the error here is expected and should just be
        // telling us that we've lost the connection. This is to
        // be expected on shutdown, so we clamp this to success.
        let _ = exec(cln_client.stop(req));
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
        Ok(ReceiveResponse { bolt11: res.bolt11 })
    }

    pub fn send(&self, invoice: String, amount_msat: Option<u64>) -> Result<SendResponse, Error> {
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
    fn get_info(&self) -> Result<GetInfoResponse, Error> {
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
    fn list_peers(&self) -> Result<ListPeersResponse, Error> {
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
    fn list_peer_channels(&self) -> Result<ListPeerChannelsResponse, Error> {
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
    fn list_funds(&self) -> Result<ListFundsResponse, Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::ListfundsRequest { spent: None };

        let res = exec(cln_client.list_funds(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
    }
}

// Not exported through uniffi
impl Node {
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
    pub amount_msat: u64,
    pub amount_sent_msat: u64,
    pub parts: u32,
}

impl From<clnpb::PayResponse> for SendResponse {
    fn from(other: clnpb::PayResponse) -> Self {
        Self {
            status: other.status.into(),
            preimage: other.payment_preimage,
            amount_msat: other.amount_msat.unwrap().msat,
            amount_sent_msat: other.amount_sent_msat.unwrap().msat,
            parts: other.parts,
        }
    }
}

#[derive(uniffi::Record)]
pub struct ReceiveResponse {
    pub bolt11: String,
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
    id: Vec<u8>,
    connected: bool,
    num_channels: Option<u32>,
    netaddr: Vec<String>,
    remote_addr: Option<String>,
    features: Option<Vec<u8>>,
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
    peer_id: Vec<u8>,
    peer_connected: bool,
    state: ChannelState,
    short_channel_id: Option<String>,
    channel_id: Option<Vec<u8>>,
    funding_txid: Option<Vec<u8>>,
    funding_outnum: Option<u32>,
    to_us_msat: Option<u64>,
    total_msat: Option<u64>,
    spendable_msat: Option<u64>,
    receivable_msat: Option<u64>,
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
    txid: Vec<u8>,
    output: u32,
    amount_msat: u64,
    status: OutputStatus,
    address: Option<String>,
    blockheight: Option<u32>,
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
    peer_id: Vec<u8>,
    our_amount_msat: u64,
    amount_msat: u64,
    funding_txid: Vec<u8>,
    funding_output: u32,
    connected: bool,
    state: ChannelState,
    short_channel_id: Option<String>,
    channel_id: Option<Vec<u8>>,
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


