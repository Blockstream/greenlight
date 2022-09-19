tonic::include_proto!("greenlight");
use crate::{messages, requests, responses};
use anyhow::{anyhow, Context, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use cln_rpc::primitives::ShortChannelId;
use log::warn;
use std::str::FromStr;

impl TryFrom<responses::GetInfo> for GetInfoResponse {
    type Error = anyhow::Error;
    fn try_from(i: responses::GetInfo) -> Result<GetInfoResponse> {
        Ok(GetInfoResponse {
            alias: i.alias,
            blockheight: i.blockheight as u32,
            color: hex::decode(i.color)?,
            addresses: vec![],
            node_id: hex::decode(i.id)?,
            version: i.version,
            num_peers: i.num_peers as u32,
            network: i.network,
        })
    }
}
impl From<responses::Connect> for ConnectResponse {
    fn from(c: responses::Connect) -> ConnectResponse {
        ConnectResponse {
            node_id: c.id,
            features: c.features,
        }
    }
}

impl From<&responses::Channel> for Channel {
    fn from(c: &responses::Channel) -> Self {
        Channel {
            state: c.state.to_string(),
            owner: c.owner.clone().unwrap_or_default(),
            short_channel_id: c.short_channel_id.clone().unwrap_or_default(),
            direction: c.direction.unwrap_or_default() as u32,
            channel_id: c.channel_id.clone(),
            funding_txid: c.funding_txid.clone(),
            close_to_addr: c.close_to_addr.clone().unwrap_or_default(),
            close_to: c.close_to.clone().unwrap_or_default(),
            private: c.private,
            total: c.total_msat.to_string(),
            dust_limit: c.dust_limit_msat.to_string(),
            spendable: c.spendable_msat.to_string(),
            receivable: c.receivable_msat.to_string(),
            their_to_self_delay: c.their_to_self_delay as u32,
            our_to_self_delay: c.our_to_self_delay as u32,
            status: vec![],
            htlcs: vec![], // TODO implement
        }
    }
}

impl TryFrom<&responses::Peer> for Peer {
    type Error = anyhow::Error;

    fn try_from(p: &responses::Peer) -> Result<Peer> {
        let features = match &p.features {
            Some(f) => f.to_string(),
            None => "".to_string(),
        };

        Ok(Peer {
            id: hex::decode(&p.id)?,
            connected: p.connected,
            features: features,
            addresses: vec![], // TODO Add field
            channels: p.channels.iter().map(|c| c.into()).collect(),
        })
    }
}

impl TryFrom<responses::ListPeers> for ListPeersResponse {
    type Error = anyhow::Error;
    fn try_from(lp: responses::ListPeers) -> Result<ListPeersResponse> {
        let peers: Result<Vec<Peer>> = lp.peers.iter().map(|p| p.try_into()).collect();

        Ok(ListPeersResponse { peers: peers? })
    }
}

impl From<&str> for OutputStatus {
    fn from(s: &str) -> Self {
        match s {
            "confirmed" => OutputStatus::Confirmed,
            "unconfirmed" => OutputStatus::Unconfirmed,
            _ => panic!("Unknown output status {}", s),
        }
    }
}

impl From<&responses::ListFundsOutput> for ListFundsOutput {
    fn from(o: &responses::ListFundsOutput) -> Self {
        let status: OutputStatus = o.status[..].into();
        ListFundsOutput {
            address: o.address.clone(),
            amount: Some(Amount {
                unit: Some(amount::Unit::Millisatoshi(o.amount_msat.0)),
            }),
            output: Some(Outpoint {
                outnum: o.output as u32,
                txid: hex::decode(&o.txid).unwrap(),
            }),
            status: status as i32,
        }
    }
}

/// Small helper to encode short_channel_ids as protobuf ints
fn parse_scid(scid: &Option<String>) -> u64 {
    match &scid {
        Some(i) => match ShortChannelId::from_str(&i) {
            Ok(i) => (i.block() as u64) << 40 | (i.txindex() as u64) << 16 | (i.outnum() as u64),
            Err(e) => {
                warn!(
                    "JSON-RPC returned an unparseable short_channel_id {}: {}",
                    i, e
                );
                0
            }
        },
        None => 0,
    }
}

impl From<&responses::ListFundsChannel> for ListFundsChannel {
    fn from(c: &responses::ListFundsChannel) -> Self {
        ListFundsChannel {
            peer_id: hex::decode(&c.peer_id).unwrap(),
            connected: c.connected,
            short_channel_id: parse_scid(&c.short_channel_id),
            our_amount_msat: c.our_amount_msat.0,
            amount_msat: c.amount_msat.0,
            funding_txid: hex::decode(&c.funding_txid).unwrap(),
            funding_output: c.funding_output as u32,
        }
    }
}

impl From<responses::ListFunds> for ListFundsResponse {
    fn from(lf: responses::ListFunds) -> Self {
        ListFundsResponse {
            outputs: lf.outputs.iter().map(|o| o.into()).collect(),
            channels: lf.channels.iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<responses::Withdraw> for WithdrawResponse {
    fn from(r: responses::Withdraw) -> Self {
        WithdrawResponse {
            tx: hex::decode(r.tx).unwrap(),
            txid: hex::decode(r.txid).unwrap(),
        }
    }
}

impl TryFrom<FundChannelRequest> for requests::FundChannel {
    type Error = anyhow::Error;
    fn try_from(f: FundChannelRequest) -> Result<Self> {
        let amount: requests::Amount = match f.amount {
            Some(v) => v.try_into()?,
            None => return Err(anyhow!("Funding amount cannot be omitted.")),
        };

        if let requests::Amount::Any = amount {
            return Err(anyhow!(
                "Funding amount cannot be 'any'. Did you mean to use 'all'?"
            ));
        }

        if let requests::Amount::Millisatoshi(a) = amount {
            if a % 1000 != 0 {
                return Err(anyhow!("Funding amount must be expressed integer satoshis. Millisatoshi amount {} is not.", a));
            }
        }

        Ok(requests::FundChannel {
            id: hex::encode(f.node_id),
            amount: amount,
            feerate: None,
            announce: Some(f.announce),
            minconf: f.minconf.map(|c| c.blocks),
            close_to: match f.close_to.as_ref() {
                "" => None,
                v => Some(v.to_string()),
            },
        })
    }
}

impl From<responses::FundChannel> for FundChannelResponse {
    fn from(r: responses::FundChannel) -> Self {
        FundChannelResponse {
            tx: hex::decode(r.tx).unwrap(),
            outpoint: Some(Outpoint {
                txid: hex::decode(r.txid).unwrap(),
                outnum: r.outpoint,
            }),
            channel_id: hex::decode(r.channel_id).unwrap(),
            close_to: r.close_to.unwrap_or("".to_string()),
        }
    }
}

impl HsmRequestContext {
    pub fn to_client_hsmfd_msg(&self) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(2 + 33 + 8 + 8);

        buf.put_u16(9); // client_hsmfd type
        buf.put_slice(&self.node_id[..]);
        buf.put_u64(self.dbid);
        buf.put_u64(self.capabilities);

        Ok(buf.freeze())
    }

    pub fn from_client_hsmfd_msg(msg: &mut Bytes) -> Result<HsmRequestContext> {
        let typ = msg.get_u16();

        if typ != 9 {
            return Err(anyhow!("message is not an init"));
        }

        let mut node_id = [0u8; 33];
        msg.copy_to_slice(&mut node_id);
        let dbid = msg.get_u64();
        let caps = msg.get_u64();

        Ok(HsmRequestContext {
            node_id: node_id.to_vec(),
            dbid: dbid,
            capabilities: caps,
        })
    }
}

impl HsmRequest {
    pub fn get_type(&self) -> u16 {
        (self.raw[0] as u16) << 8 | (self.raw[1] as u16)
    }
}

impl TryFrom<responses::CloseChannel> for CloseChannelResponse {
    type Error = anyhow::Error;
    fn try_from(c: responses::CloseChannel) -> Result<Self, Self::Error> {
        Ok(CloseChannelResponse {
            close_type: match c.close_type.as_ref() {
                "mutual" => CloseChannelType::Mutual as i32,
                "unilateral" => CloseChannelType::Unilateral as i32,
                _ => return Err(anyhow!("Unexpected close type: {}", c.close_type)),
            },
            tx: hex::decode(c.tx).unwrap(),
            txid: hex::decode(c.txid).unwrap(),
        })
    }
}
impl From<CloseChannelRequest> for requests::CloseChannel {
    fn from(r: CloseChannelRequest) -> Self {
        requests::CloseChannel {
            node_id: hex::encode(r.node_id),
            timeout: match r.unilateraltimeout {
                Some(v) => Some(v.seconds),
                None => None,
            },
            destination: match r.destination {
                None => None,
                Some(v) => Some(v.address),
            },
        }
    }
}

impl TryFrom<InvoiceRequest> for requests::Invoice {
    type Error = anyhow::Error;

    fn try_from(i: InvoiceRequest) -> Result<Self, Self::Error> {
        if i.label == "" {
            return Err(anyhow!(
                "Label must be set, not empty and unique from all other invoice labels."
            ));
        }

        if let None = i.amount {
            return Err(anyhow!(
                "No amount specified. Use Amount(any=true) if you want an amountless invoice"
            ));
        };

        // We can unwrap since we checked the None case above.
        let amt = i.amount.unwrap();

        let preimage = if i.preimage.len() == 0 {
            None
        } else {
            Some(hex::encode(i.preimage))
        };

        if let Some(amount::Unit::All(_)) = amt.unit {
            return Err(anyhow!(
		"Amount cannot be set to `all`. Use Amount(any=true) if you want an amountless invoice"
	    ));
        }

        Ok(requests::Invoice {
            amount: amt.try_into()?,
            label: i.label,
            description: i.description,
            exposeprivatechannels: None,
            dev_routes: None,
            preimage,
        })
    }
}

impl From<u64> for Amount {
    fn from(i: u64) -> Self {
        Amount {
            unit: Some(amount::Unit::Millisatoshi(i)),
        }
    }
}

/// Converts the result of the `invoice` call into the shared format
/// for invoices in protobuf, hence the subset of fields that are
/// initialized to default values.
impl From<responses::Invoice> for Invoice {
    fn from(i: responses::Invoice) -> Invoice {
        Invoice {
            label: "".to_string(),
            description: "".to_string(),
            payment_preimage: vec![],
            amount: None,
            received: None,
            payment_time: 0,
            status: InvoiceStatus::Unpaid as i32,
            bolt11: i.bolt11,
            payment_hash: hex::decode(i.payment_hash).unwrap(),
            expiry_time: i.expiry_time,
        }
    }
}

impl TryFrom<Amount> for requests::Amount {
    type Error = anyhow::Error;
    fn try_from(a: Amount) -> Result<Self, Self::Error> {
        match a.unit {
            Some(amount::Unit::Millisatoshi(v)) => Ok(requests::Amount::Millisatoshi(v)),
            Some(amount::Unit::Satoshi(v)) => Ok(requests::Amount::Satoshi(v)),
            Some(amount::Unit::Bitcoin(v)) => Ok(requests::Amount::Bitcoin(v)),
            Some(amount::Unit::All(_)) => Ok(requests::Amount::All),
            Some(amount::Unit::Any(_)) => Ok(requests::Amount::Any),
            None => Err(anyhow!("cannot convert a unit-less amount")),
        }
    }
}

impl From<responses::Pay> for Payment {
    fn from(p: responses::Pay) -> Self {
        Payment {
            destination: hex::decode(p.destination).unwrap(),
            payment_hash: hex::decode(p.payment_hash).unwrap(),
            payment_preimage: p
                .payment_preimage
                .map_or(vec![], |p| hex::decode(p).unwrap()),
            amount: Some(Amount {
                unit: Some(amount::Unit::Millisatoshi(p.msatoshi)),
            }),
            amount_sent: Some(Amount {
                unit: Some(amount::Unit::Millisatoshi(p.msatoshi_sent)),
            }),
            status: match p.status.as_ref() {
                "pending" => PayStatus::Pending as i32,
                "complete" => PayStatus::Complete as i32,
                "failed" => PayStatus::Failed as i32,
                o => panic!("Unmapped pay status: {}", o),
            },
            bolt11: p.bolt11.unwrap_or("".to_string()),
            created_at: p.created_at,
        }
    }
}

impl From<PayRequest> for requests::Pay {
    fn from(p: PayRequest) -> Self {
        requests::Pay {
            bolt11: p.bolt11,
            amount: None,
            retry_for: match p.timeout {
                0 => None,
                v => Some(v),
            },
        }
    }
}

impl TryFrom<Feerate> for requests::Feerate {
    type Error = anyhow::Error;

    fn try_from(value: Feerate) -> Result<requests::Feerate, anyhow::Error> {
        use requests as r;
        let res = match value.value {
            Some(v) => match v {
                feerate::Value::Preset(p) => match p {
                    0 => r::Feerate::Normal,
                    1 => r::Feerate::Slow,
                    2 => r::Feerate::Urgent,
                    n => return Err(anyhow!("no such feerate preset {}", n)),
                },
                feerate::Value::Perkw(v) => r::Feerate::PerKw(v),
                feerate::Value::Perkb(v) => r::Feerate::PerKb(v),
            },
            None => return Err(anyhow!("Feerate must have a value set")),
        };
        Ok(res)
    }
}

impl TryFrom<Outpoint> for requests::Outpoint {
    type Error = anyhow::Error;

    fn try_from(value: Outpoint) -> Result<Self, Self::Error> {
        if value.txid.len() != 32 {
            return Err(anyhow!(
                "{} is not a valid transaction ID",
                hex::encode(&value.txid)
            ));
        }

        Ok(requests::Outpoint {
            txid: value.txid,
            outnum: value.outnum.try_into().context("outnum out of range")?,
        })
    }
}

impl TryFrom<ListPaymentsRequest> for requests::ListPays {
    type Error = anyhow::Error;

    fn try_from(v: ListPaymentsRequest) -> Result<Self, Self::Error> {
        match v.identifier {
            Some(identifier) => match identifier.id {
                Some(payment_identifier::Id::Bolt11(b)) => Ok(requests::ListPays {
                    payment_hash: None,
                    bolt11: Some(b),
                }),
                Some(payment_identifier::Id::PaymentHash(h)) => Ok(requests::ListPays {
                    payment_hash: Some(hex::encode(h)),
                    bolt11: None,
                }),
                None => Ok(requests::ListPays {
                    bolt11: None,
                    payment_hash: None,
                }),
            },
            None => Ok(requests::ListPays {
                bolt11: None,
                payment_hash: None,
            }),
        }
    }
}

impl TryFrom<responses::ListPays> for ListPaymentsResponse {
    type Error = anyhow::Error;

    fn try_from(v: responses::ListPays) -> Result<Self, Self::Error> {
        let payments: Result<Vec<Payment>> = v.pays.iter().map(|p| p.clone().try_into()).collect();
        Ok(ListPaymentsResponse {
            payments: payments?,
        })
    }
}

impl TryFrom<responses::ListPaysPay> for Payment {
    type Error = anyhow::Error;
    fn try_from(p: responses::ListPaysPay) -> Result<Self, Self::Error> {
        Ok(Payment {
            destination: hex::decode(p.destination).unwrap(),
            payment_hash: hex::decode(p.payment_hash).unwrap(),
            payment_preimage: p
                .payment_preimage
                .map_or(vec![], |p| hex::decode(p).unwrap()),
            status: match p.status.as_ref() {
                "pending" => PayStatus::Pending as i32,
                "complete" => PayStatus::Complete as i32,
                "failed" => PayStatus::Failed as i32,
                o => panic!("Unmapped pay status: {}", o),
            },
            amount: p.amount_msat.map(|a| a.try_into().unwrap()),
            amount_sent: Some(p.amount_sent_msat.try_into()?),
            bolt11: p.bolt11.unwrap_or("".to_string()),
            created_at: p.created_at,
        })
    }
}

impl TryFrom<String> for Amount {
    type Error = anyhow::Error;
    fn try_from(s: String) -> Result<Amount, Self::Error> {
        match s.strip_suffix("msat") {
            Some(v) => Ok(Amount {
                unit: Some(amount::Unit::Millisatoshi(v.parse()?)),
            }),
            None => Err(anyhow!("Amount {} does not have 'msat' suffix", s)),
        }
    }
}

impl TryFrom<String> for InvoiceStatus {
    type Error = anyhow::Error;

    fn try_from(v: String) -> Result<Self, anyhow::Error> {
        match v.as_ref() {
            "unpaid" => Ok(InvoiceStatus::Unpaid),
            "paid" => Ok(InvoiceStatus::Paid),
            "expired" => Ok(InvoiceStatus::Expired),
            s => Err(anyhow!("{} is not a valid InvoiceStatus", s)),
        }
    }
}

impl TryFrom<ListInvoicesRequest> for requests::ListInvoices {
    type Error = anyhow::Error;

    fn try_from(v: ListInvoicesRequest) -> Result<Self, Self::Error> {
        match v.identifier {
            None => Ok(requests::ListInvoices {
                label: None,
                invstring: None,
                payment_hash: None,
            }),
            Some(i) => match i.id {
                None => Ok(requests::ListInvoices {
                    label: None,
                    invstring: None,
                    payment_hash: None,
                }),
                Some(invoice_identifier::Id::Label(s)) => Ok(requests::ListInvoices {
                    label: Some(s),
                    invstring: None,
                    payment_hash: None,
                }),
                Some(invoice_identifier::Id::PaymentHash(s)) => Ok(requests::ListInvoices {
                    label: None,
                    invstring: None,
                    payment_hash: Some(hex::encode(s)),
                }),
                Some(invoice_identifier::Id::Invstring(s)) => Ok(requests::ListInvoices {
                    label: None,
                    invstring: Some(s),
                    payment_hash: None,
                }),
            },
        }
    }
}

impl From<&responses::ListInvoiceInvoice> for Invoice {
    fn from(i: &responses::ListInvoiceInvoice) -> Invoice {
        let status: InvoiceStatus = i.status.clone().try_into().unwrap();
        let amount: Amount = if i.amount == None {
            Amount {
                unit: Some(crate::pb::amount::Unit::Any(true)),
            }
        } else {
            i.amount.clone().unwrap().try_into().unwrap()
        };

        Invoice {
            amount: Some(amount),
            bolt11: i.bolt11.clone(),
            description: i.description.clone(),
            expiry_time: i.expiry_time,
            label: i.label.clone(),
            payment_hash: hex::decode(&i.payment_hash).unwrap(),
            payment_preimage: i
                .payment_preimage
                .clone()
                .map(|p| hex::decode(p).unwrap())
                .unwrap_or_default(),
            status: status as i32,
            received: i.received.clone().map(|i| i.try_into().unwrap()),
            payment_time: i.payment_time.clone().unwrap_or_default(),
        }
    }
}

impl TryFrom<responses::ListInvoices> for ListInvoicesResponse {
    type Error = anyhow::Error;
    fn try_from(l: responses::ListInvoices) -> Result<Self, Self::Error> {
        let invoices: Vec<Invoice> = l.invoices.iter().map(|i| i.into()).collect();
        Ok(ListInvoicesResponse { invoices })
    }
}

impl From<messages::TlvField> for TlvField {
    fn from(f: messages::TlvField) -> TlvField {
        TlvField {
            r#type: f.typ,
            value: hex::decode(f.value).unwrap(),
        }
    }
}

impl TryFrom<KeysendRequest> for requests::Keysend {
    type Error = anyhow::Error;
    fn try_from(r: KeysendRequest) -> Result<requests::Keysend> {
        use std::collections::HashMap;
        // Transform the extratlvs into aa key-value dict:
        let mut tlvs: HashMap<u64, String> = HashMap::new();

        for e in r.extratlvs {
            tlvs.insert(e.r#type, hex::encode(e.value));
        }

        let mut routehints = vec![];
        for rh in r.routehints {
            routehints.push(rh.into())
        }

        Ok(requests::Keysend {
            destination: hex::encode(r.node_id),
            msatoshi: r.amount.unwrap().try_into()?,
            label: r.label,
            exemptfee: None,
            maxfeepercent: None,
            maxdelay: None,
            extratlvs: Some(tlvs),
            routehints: Some(routehints),
            retry_for: None,
        })
    }
}

impl From<responses::Keysend> for Payment {
    fn from(r: responses::Keysend) -> Payment {
        use std::time::SystemTime;
        Payment {
            destination: hex::decode(r.destination).unwrap(),
            payment_hash: hex::decode(r.payment_hash).unwrap(),
            payment_preimage: hex::decode(r.payment_preimage.unwrap_or("".to_string())).unwrap(),
            status: match r.status.as_ref() {
                "pending" => PayStatus::Pending as i32,
                "complete" => PayStatus::Complete as i32,
                "failed" => PayStatus::Failed as i32,
                o => panic!("Unmapped pay status: {}", o),
            },
            amount: Some(Amount {
                unit: Some(amount::Unit::Millisatoshi(r.msatoshi)),
            }),
            amount_sent: Some(Amount {
                unit: Some(amount::Unit::Millisatoshi(r.msatoshi_sent)),
            }),
            bolt11: "".to_string(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    }
}

impl From<RoutehintHop> for requests::RoutehintHop {
    fn from(r: RoutehintHop) -> requests::RoutehintHop {
        requests::RoutehintHop {
            id: hex::encode(r.node_id),
            scid: r.short_channel_id,
            feebase: r.fee_base,
            feeprop: r.fee_prop,
            expirydelta: r.cltv_expiry_delta as u16,
        }
    }
}

impl From<Routehint> for Vec<requests::RoutehintHop> {
    fn from(r: Routehint) -> Vec<requests::RoutehintHop> {
        let mut hops = vec![];
        for h in r.hops {
            hops.push(h.into())
        }
        hops
    }
}

impl From<RoutehintHop> for requests::RoutehintHopDev {
    fn from(r: RoutehintHop) -> requests::RoutehintHopDev {
        requests::RoutehintHopDev {
            id: hex::encode(r.node_id),
            short_channel_id: r.short_channel_id,
            fee_base_msat: r.fee_base,
            fee_proportional_millionths: r.fee_prop,
            cltv_expiry_delta: r.cltv_expiry_delta as u16,
        }
    }
}
