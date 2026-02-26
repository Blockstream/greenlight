use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

// ============================================================
// GetInfo
// ============================================================

#[derive(Serialize)]
pub struct GetInfoOutput {
    pub id: String,
    pub alias: Option<String>,
    pub color: String,
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

impl From<glsdk::GetInfoResponse> for GetInfoOutput {
    fn from(r: glsdk::GetInfoResponse) -> Self {
        Self {
            id: hex::encode(&r.id),
            alias: r.alias,
            color: hex::encode(&r.color),
            num_peers: r.num_peers,
            num_pending_channels: r.num_pending_channels,
            num_active_channels: r.num_active_channels,
            num_inactive_channels: r.num_inactive_channels,
            version: r.version,
            lightning_dir: r.lightning_dir,
            blockheight: r.blockheight,
            network: r.network,
            fees_collected_msat: r.fees_collected_msat,
        }
    }
}

// ============================================================
// ListPeers
// ============================================================

#[derive(Serialize)]
pub struct ListPeersOutput {
    pub peers: Vec<PeerOutput>,
}

#[derive(Serialize)]
pub struct PeerOutput {
    pub id: String,
    pub connected: bool,
    pub num_channels: Option<u32>,
    pub netaddr: Vec<String>,
    pub remote_addr: Option<String>,
    pub features: Option<String>,
}

impl From<glsdk::ListPeersResponse> for ListPeersOutput {
    fn from(r: glsdk::ListPeersResponse) -> Self {
        Self {
            peers: r.peers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<glsdk::Peer> for PeerOutput {
    fn from(p: glsdk::Peer) -> Self {
        Self {
            id: hex::encode(&p.id),
            connected: p.connected,
            num_channels: p.num_channels,
            netaddr: p.netaddr,
            remote_addr: p.remote_addr,
            features: p.features.map(|f| hex::encode(&f)),
        }
    }
}

// ============================================================
// ListPeerChannels
// ============================================================

#[derive(Serialize)]
pub struct ListPeerChannelsOutput {
    pub channels: Vec<PeerChannelOutput>,
}

#[derive(Serialize)]
pub struct PeerChannelOutput {
    pub peer_id: String,
    pub peer_connected: bool,
    pub state: String,
    pub short_channel_id: Option<String>,
    pub channel_id: Option<String>,
    pub funding_txid: Option<String>,
    pub funding_outnum: Option<u32>,
    pub to_us_msat: Option<u64>,
    pub total_msat: Option<u64>,
    pub spendable_msat: Option<u64>,
    pub receivable_msat: Option<u64>,
}

impl From<glsdk::ListPeerChannelsResponse> for ListPeerChannelsOutput {
    fn from(r: glsdk::ListPeerChannelsResponse) -> Self {
        Self {
            channels: r.channels.into_iter().map(Into::into).collect(),
        }
    }
}

fn channel_state_str(s: &glsdk::ChannelState) -> &'static str {
    match s {
        glsdk::ChannelState::Openingd => "OPENINGD",
        glsdk::ChannelState::ChanneldAwaitingLockin => "CHANNELD_AWAITING_LOCKIN",
        glsdk::ChannelState::ChanneldNormal => "CHANNELD_NORMAL",
        glsdk::ChannelState::ChanneldShuttingDown => "CHANNELD_SHUTTING_DOWN",
        glsdk::ChannelState::ClosingdSigexchange => "CLOSINGD_SIGEXCHANGE",
        glsdk::ChannelState::ClosingdComplete => "CLOSINGD_COMPLETE",
        glsdk::ChannelState::AwaitingUnilateral => "AWAITING_UNILATERAL",
        glsdk::ChannelState::FundingSpendSeen => "FUNDING_SPEND_SEEN",
        glsdk::ChannelState::Onchain => "ONCHAIN",
        glsdk::ChannelState::DualopendOpenInit => "DUALOPEND_OPEN_INIT",
        glsdk::ChannelState::DualopendAwaitingLockin => "DUALOPEND_AWAITING_LOCKIN",
        glsdk::ChannelState::DualopendOpenCommitted => "DUALOPEND_OPEN_COMMITTED",
        glsdk::ChannelState::DualopendOpenCommitReady => "DUALOPEND_OPEN_COMMIT_READY",
    }
}

impl From<glsdk::PeerChannel> for PeerChannelOutput {
    fn from(c: glsdk::PeerChannel) -> Self {
        Self {
            peer_id: hex::encode(&c.peer_id),
            peer_connected: c.peer_connected,
            state: channel_state_str(&c.state).to_string(),
            short_channel_id: c.short_channel_id,
            channel_id: c.channel_id.map(|v| hex::encode(&v)),
            funding_txid: c.funding_txid.map(|v| hex::encode(&v)),
            funding_outnum: c.funding_outnum,
            to_us_msat: c.to_us_msat,
            total_msat: c.total_msat,
            spendable_msat: c.spendable_msat,
            receivable_msat: c.receivable_msat,
        }
    }
}

// ============================================================
// ListFunds
// ============================================================

#[derive(Serialize)]
pub struct ListFundsOutput {
    pub outputs: Vec<FundOutputOutput>,
    pub channels: Vec<FundChannelOutput>,
}

#[derive(Serialize)]
pub struct FundOutputOutput {
    pub txid: String,
    pub output: u32,
    pub amount_msat: u64,
    pub status: String,
    pub address: Option<String>,
    pub blockheight: Option<u32>,
}

#[derive(Serialize)]
pub struct FundChannelOutput {
    pub peer_id: String,
    pub our_amount_msat: u64,
    pub amount_msat: u64,
    pub funding_txid: String,
    pub funding_output: u32,
    pub connected: bool,
    pub state: String,
    pub short_channel_id: Option<String>,
    pub channel_id: Option<String>,
}

fn output_status_str(s: &glsdk::OutputStatus) -> &'static str {
    match s {
        glsdk::OutputStatus::Unconfirmed => "unconfirmed",
        glsdk::OutputStatus::Confirmed => "confirmed",
        glsdk::OutputStatus::Spent => "spent",
        glsdk::OutputStatus::Immature => "immature",
    }
}

impl From<glsdk::ListFundsResponse> for ListFundsOutput {
    fn from(r: glsdk::ListFundsResponse) -> Self {
        Self {
            outputs: r.outputs.into_iter().map(Into::into).collect(),
            channels: r.channels.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<glsdk::FundOutput> for FundOutputOutput {
    fn from(o: glsdk::FundOutput) -> Self {
        Self {
            txid: hex::encode(&o.txid),
            output: o.output,
            amount_msat: o.amount_msat,
            status: output_status_str(&o.status).to_string(),
            address: o.address,
            blockheight: o.blockheight,
        }
    }
}

impl From<glsdk::FundChannel> for FundChannelOutput {
    fn from(c: glsdk::FundChannel) -> Self {
        Self {
            peer_id: hex::encode(&c.peer_id),
            our_amount_msat: c.our_amount_msat,
            amount_msat: c.amount_msat,
            funding_txid: hex::encode(&c.funding_txid),
            funding_output: c.funding_output,
            connected: c.connected,
            state: channel_state_str(&c.state).to_string(),
            short_channel_id: c.short_channel_id,
            channel_id: c.channel_id.map(|v| hex::encode(&v)),
        }
    }
}

// ============================================================
// Receive / Send / Onchain
// ============================================================

#[derive(Serialize)]
pub struct ReceiveOutput {
    pub bolt11: String,
}

impl From<glsdk::ReceiveResponse> for ReceiveOutput {
    fn from(r: glsdk::ReceiveResponse) -> Self {
        Self { bolt11: r.bolt11 }
    }
}

#[derive(Serialize)]
pub struct SendOutput {
    pub status: String,
    pub preimage: String,
    pub amount_msat: u64,
    pub amount_sent_msat: u64,
    pub parts: u32,
}

fn pay_status_str(s: &glsdk::PayStatus) -> &'static str {
    match s {
        glsdk::PayStatus::COMPLETE => "complete",
        glsdk::PayStatus::PENDING => "pending",
        glsdk::PayStatus::FAILED => "failed",
    }
}

impl From<glsdk::SendResponse> for SendOutput {
    fn from(r: glsdk::SendResponse) -> Self {
        Self {
            status: pay_status_str(&r.status).to_string(),
            preimage: hex::encode(&r.preimage),
            amount_msat: r.amount_msat,
            amount_sent_msat: r.amount_sent_msat,
            parts: r.parts,
        }
    }
}

#[derive(Serialize)]
pub struct OnchainReceiveOutput {
    pub bech32: String,
    pub p2tr: String,
}

impl From<glsdk::OnchainReceiveResponse> for OnchainReceiveOutput {
    fn from(r: glsdk::OnchainReceiveResponse) -> Self {
        Self {
            bech32: r.bech32,
            p2tr: r.p2tr,
        }
    }
}

#[derive(Serialize)]
pub struct OnchainSendOutput {
    pub tx: String,
    pub txid: String,
    pub psbt: String,
}

impl From<glsdk::OnchainSendResponse> for OnchainSendOutput {
    fn from(r: glsdk::OnchainSendResponse) -> Self {
        Self {
            tx: hex::encode(&r.tx),
            txid: hex::encode(&r.txid),
            psbt: r.psbt,
        }
    }
}

// ============================================================
// NodeEvent
// ============================================================

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum NodeEventOutput {
    #[serde(rename = "invoice_paid")]
    InvoicePaid {
        payment_hash: String,
        bolt11: String,
        preimage: String,
        label: String,
        amount_msat: u64,
    },
    #[serde(rename = "unknown")]
    Unknown,
}

impl From<glsdk::NodeEvent> for NodeEventOutput {
    fn from(e: glsdk::NodeEvent) -> Self {
        match e {
            glsdk::NodeEvent::InvoicePaid { details } => NodeEventOutput::InvoicePaid {
                payment_hash: hex::encode(&details.payment_hash),
                bolt11: details.bolt11,
                preimage: hex::encode(&details.preimage),
                label: details.label,
                amount_msat: details.amount_msat,
            },
            glsdk::NodeEvent::Unknown => NodeEventOutput::Unknown,
        }
    }
}

// ============================================================
// Signer node-id
// ============================================================

#[derive(Serialize)]
pub struct NodeIdOutput {
    pub node_id: String,
}
