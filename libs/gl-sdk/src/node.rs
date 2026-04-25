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

    /// Send bitcoin on-chain to a destination address.
    ///
    /// # Arguments
    /// * `destination` — A Bitcoin address (bech32, p2sh, or p2tr).
    /// * `amount_or_all` — Amount to send. Accepts:
    ///   - `"50000"` or `"50000sat"` — 50,000 satoshis
    ///   - `"50000msat"` — 50,000 millisatoshis
    ///   - `"all"` — sweep the entire on-chain balance
    ///
    /// Returns the raw transaction, txid, and PSBT once broadcast.
    /// The transaction is broadcast immediately — this is not a dry run.
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
                value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount {
                    msat: n,
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

    /// Generate a fresh on-chain Bitcoin address for receiving funds.
    ///
    /// Returns both a bech32 (SegWit v0) and a p2tr (Taproot) address.
    /// Either can be shared with a sender. Deposited funds will appear
    /// in `node_state().onchain_balance_msat` once confirmed.
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

    /// Get a snapshot of the node's balances, capacity, and connectivity.
    ///
    /// Aggregates data from multiple RPCs into a single `NodeState`.
    /// Queries the node live on each call — not cached.
    pub fn node_state(&self) -> Result<NodeState, Error> {
        self.check_connected()?;
        let cln_client = exec(self.get_cln_client())?.clone();

        let (info_res, channels_res, funds_res) = exec(async {
            let mut c_info = cln_client.clone();
            let mut c_channels = cln_client.clone();
            let mut c_funds = cln_client.clone();
            tokio::join!(
                c_info.getinfo(clnpb::GetinfoRequest {}),
                c_channels.list_peer_channels(clnpb::ListpeerchannelsRequest { id: None }),
                c_funds.list_funds(clnpb::ListfundsRequest { spent: None }),
            )
        });

        let info: GetInfoResponse = info_res
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner()
            .into();
        let channels: ListPeerChannelsResponse = channels_res
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner()
            .into();
        let funds: ListFundsResponse = funds_res
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner()
            .into();

        let mut channels_balance_msat: u64 = 0;
        let mut max_payable_msat: u64 = 0;
        let mut total_channel_capacity_msat: u64 = 0;
        let mut max_receivable_single_payment_msat: u64 = 0;
        let mut total_inbound_liquidity_msat: u64 = 0;
        let mut pending_onchain_balance_msat: u64 = 0;
        let mut connected_channel_peer_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for ch in &channels.channels {
            if ch.state.is_open() {
                channels_balance_msat += ch.to_us_msat.unwrap_or(0);
                max_payable_msat += ch.spendable_msat.unwrap_or(0);
                total_channel_capacity_msat += ch.total_msat.unwrap_or(0);
                let receivable = ch.receivable_msat.unwrap_or(0);
                if receivable > max_receivable_single_payment_msat {
                    max_receivable_single_payment_msat = receivable;
                }
                total_inbound_liquidity_msat += receivable;
            }
            if channel_payout_still_pending(ch) {
                pending_onchain_balance_msat += ch.to_us_msat.unwrap_or(0);
            }
            if ch.peer_connected {
                connected_channel_peer_set.insert(ch.peer_id.clone());
            }
        }
        let connected_channel_peers: Vec<String> =
            connected_channel_peer_set.into_iter().collect();

        let max_chan_reserve_msat =
            channels_balance_msat.saturating_sub(max_payable_msat);

        let mut onchain_balance_msat: u64 = 0;
        let mut unconfirmed_onchain_balance_msat: u64 = 0;
        let mut immature_onchain_balance_msat: u64 = 0;
        let mut utxos: Vec<FundOutput> = Vec::with_capacity(funds.outputs.len());
        for output in &funds.outputs {
            if !matches!(output.status, OutputStatus::Spent) {
                utxos.push(output.clone());
            }
            if output.reserved {
                continue;
            }
            match output.status {
                OutputStatus::Confirmed => onchain_balance_msat += output.amount_msat,
                OutputStatus::Unconfirmed => {
                    unconfirmed_onchain_balance_msat += output.amount_msat
                }
                OutputStatus::Immature => {
                    immature_onchain_balance_msat += output.amount_msat
                }
                OutputStatus::Spent => {}
            }
        }

        let total_onchain_msat = onchain_balance_msat
            .saturating_add(unconfirmed_onchain_balance_msat)
            .saturating_add(immature_onchain_balance_msat);
        let total_balance_msat = channels_balance_msat
            .saturating_add(total_onchain_msat)
            .saturating_add(pending_onchain_balance_msat);
        let spendable_balance_msat = max_payable_msat.saturating_add(onchain_balance_msat);


        Ok(NodeState {
            id: info.id,
            block_height: info.blockheight,
            network: info.network,
            version: info.version,
            alias: info.alias,
            color: info.color,
            num_active_channels: info.num_active_channels,
            num_pending_channels: info.num_pending_channels,
            num_inactive_channels: info.num_inactive_channels,
            channels_balance_msat,
            max_payable_msat,
            total_channel_capacity_msat,
            max_chan_reserve_msat,
            onchain_balance_msat,
            unconfirmed_onchain_balance_msat,
            immature_onchain_balance_msat,
            pending_onchain_balance_msat,
            max_receivable_single_payment_msat,
            total_inbound_liquidity_msat,
            connected_channel_peers,
            utxos,
            total_onchain_msat,
            total_balance_msat,
            spendable_balance_msat,
        })
    }

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
    ///
    /// Fetches invoices and outgoing payments from the node, merges
    /// them into a unified list, and applies optional filters.
    /// Use `list_invoices`/`list_pays` for direct CLN access.
    /// Results are sorted newest-first.
    pub fn list_payments(&self, req: ListPaymentsRequest) -> Result<Vec<Payment>, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let invoices = exec(cln_client.list_invoices(clnpb::ListinvoicesRequest::default()))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();

        let mut cln_client = exec(self.get_cln_client())?.clone();
        let pays = exec(cln_client.list_pays(clnpb::ListpaysRequest::default()))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();

        let mut payments: Vec<Payment> = Vec::new();

        // Should we include received payments?
        let include_received = req
            .filters
            .as_ref()
            .map(|f| f.is_empty() || f.iter().any(|t| matches!(t, PaymentTypeFilter::Received)))
            .unwrap_or(true);

        // Should we include sent payments?
        let include_sent = req
            .filters
            .as_ref()
            .map(|f| f.is_empty() || f.iter().any(|t| matches!(t, PaymentTypeFilter::Sent)))
            .unwrap_or(true);

        if include_received {
            // Only paid invoices belong in payment history. Open
            // (unpaid) and expired invoices live behind list_invoices()
            // for callers that want to inspect them directly.
            payments.extend(
                invoices
                    .invoices
                    .into_iter()
                    .filter(|i| {
                        i.status()
                            == clnpb::listinvoices_invoices::ListinvoicesInvoicesStatus::Paid
                    })
                    .map(|i| -> Payment { i.into() }),
            );
        }
        if include_sent {
            payments.extend(pays.pays.into_iter().map(|p| -> Payment { p.into() }));
        }

        let include_failures = req.include_failures.unwrap_or(false);

        payments.retain(|p| {
            if !include_failures && matches!(p.status, PaymentStatus::Failed) {
                return false;
            }
            if let Some(from) = req.from_timestamp {
                if p.payment_time < from {
                    return false;
                }
            }
            if let Some(to) = req.to_timestamp {
                if p.payment_time > to {
                    return false;
                }
            }
            true
        });

        // Sort newest first
        payments.sort_by(|a, b| b.payment_time.cmp(&a.payment_time));

        // Apply pagination
        let offset = req.offset.unwrap_or(0) as usize;
        let limit = req.limit.unwrap_or(u32::MAX) as usize;
        let payments = payments.into_iter().skip(offset).take(limit).collect();

        Ok(payments)
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

/// Result of an on-chain send. The transaction has already been broadcast.
#[derive(uniffi::Record)]
pub struct OnchainSendResponse {
    /// The raw signed transaction bytes.
    pub tx: Vec<u8>,
    /// The transaction id as lowercase hex (64 chars).
    pub txid: String,
    /// The transaction as a Partially Signed Bitcoin Transaction string.
    pub psbt: String,
}

impl From<clnpb::WithdrawResponse> for OnchainSendResponse {
    fn from(other: clnpb::WithdrawResponse) -> Self {
        Self {
            tx: other.tx,
            txid: hex::encode(&other.txid),
            psbt: other.psbt,
        }
    }
}

/// A pair of on-chain addresses for receiving funds.
#[derive(uniffi::Record)]
pub struct OnchainReceiveResponse {
    /// SegWit v0 (bech32) address — starts with `bc1q` on mainnet.
    pub bech32: String,
    /// Taproot (bech32m) address — starts with `bc1p` on mainnet.
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
    /// Payment preimage (proof of payment) as lowercase hex (64 chars).
    pub preimage: String,
    /// Payment hash as lowercase hex (64 chars).
    pub payment_hash: String,
    /// Recipient node pubkey as lowercase hex (66 chars), if known.
    pub destination_pubkey: Option<String>,
    pub amount_msat: u64,
    pub amount_sent_msat: u64,
    pub parts: u32,
}

impl From<clnpb::PayResponse> for SendResponse {
    fn from(other: clnpb::PayResponse) -> Self {
        Self {
            status: other.status.into(),
            preimage: hex::encode(&other.payment_preimage),
            payment_hash: hex::encode(&other.payment_hash),
            destination_pubkey: other.destination.as_deref().map(hex::encode),
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
    /// Node public key as lowercase hex (66 chars).
    pub id: String,
    pub alias: Option<String>,
    /// 3-byte RGB color as lowercase hex (6 chars).
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

impl From<clnpb::GetinfoResponse> for GetInfoResponse {
    fn from(other: clnpb::GetinfoResponse) -> Self {
        Self {
            id: hex::encode(&other.id),
            alias: other.alias,
            color: hex::encode(&other.color),
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
    /// Peer node public key as lowercase hex (66 chars).
    pub id: String,
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
            id: hex::encode(&other.id),
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
    /// Peer node public key as lowercase hex (66 chars).
    pub peer_id: String,
    pub peer_connected: bool,
    pub state: ChannelState,
    pub short_channel_id: Option<String>,
    /// Channel id as lowercase hex (64 chars).
    pub channel_id: Option<String>,
    /// Funding transaction id as lowercase hex (64 chars).
    pub funding_txid: Option<String>,
    pub funding_outnum: Option<u32>,
    pub to_us_msat: Option<u64>,
    pub total_msat: Option<u64>,
    pub spendable_msat: Option<u64>,
    pub receivable_msat: Option<u64>,
    /// Which side initiated the close, if the channel is closing or closed.
    pub closer: Option<ChannelSide>,
    /// Human-readable status strings from CLN, ordered oldest to newest.
    /// For a channel in `Onchain` state, the last entry indicates whether
    /// our payout is still timelocked (`DELAYED_OUTPUT_TO_US`) or already
    /// available in the on-chain balance.
    pub status: Vec<String>,
}

/// Which side of a channel performed a given action (e.g. initiated close).
#[derive(Clone, uniffi::Enum)]
pub enum ChannelSide {
    Local,
    Remote,
}

impl ChannelSide {
    fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ChannelSide::Local),
            1 => Some(ChannelSide::Remote),
            _ => None,
        }
    }
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
    /// A state reported by the node that this SDK doesn't recognize.
    /// Returned when CLN introduces a new channel state after this SDK
    /// was built. Treated as neither open nor closing by balance math.
    Unknown,
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
            _ => ChannelState::Unknown,
        }
    }

    fn is_open(&self) -> bool {
        matches!(self, ChannelState::ChanneldNormal)
    }

}

/// Returns true when the channel still holds on-chain funds that have
/// not yet been credited to the wallet's on-chain balance.
///
/// In closing states up to `FundingSpendSeen`, the payout has not yet
/// appeared as a wallet UTXO and `to_us_msat` represents funds still
/// locked in the channel.
///
/// In `Onchain` state CLN keeps the channel around for the duration of
/// the close timelock. Once the close tx is mined the payout is visible
/// in `listfunds.outputs`, so counting `to_us_msat` again would double
/// it. The exception is when we initiated the close and our payout is
/// still timelocked (the last status entry contains `DELAYED_OUTPUT_TO_US`):
/// in that window the funds exist on-chain but are not yet spendable.
fn channel_payout_still_pending(ch: &PeerChannel) -> bool {
    match ch.state {
        ChannelState::ChanneldShuttingDown
        | ChannelState::ClosingdSigexchange
        | ChannelState::ClosingdComplete
        | ChannelState::AwaitingUnilateral
        | ChannelState::FundingSpendSeen => true,
        ChannelState::Onchain => {
            matches!(ch.closer, Some(ChannelSide::Local))
                && ch
                    .status
                    .last()
                    .is_some_and(|s| s.contains("DELAYED_OUTPUT_TO_US"))
        }
        _ => false,
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
        let closer = other.closer.and_then(ChannelSide::from_i32);
        Self {
            peer_id: hex::encode(&other.peer_id),
            peer_connected: other.peer_connected,
            state,
            short_channel_id: other.short_channel_id,
            channel_id: other.channel_id.as_deref().map(hex::encode),
            funding_txid: other.funding_txid.as_deref().map(hex::encode),
            funding_outnum: other.funding_outnum,
            to_us_msat: other.to_us_msat.map(|a| a.msat),
            total_msat: other.total_msat.map(|a| a.msat),
            spendable_msat: other.spendable_msat.map(|a| a.msat),
            receivable_msat: other.receivable_msat.map(|a| a.msat),
            closer,
            status: other.status,
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
    /// Transaction id as lowercase hex (64 chars).
    pub txid: String,
    pub output: u32,
    pub amount_msat: u64,
    pub status: OutputStatus,
    pub address: Option<String>,
    pub blockheight: Option<u32>,
    /// True when this UTXO is currently reserved by an in-flight PSBT
    /// (e.g. a channel-open or fund-send that has not been broadcast or
    /// abandoned). Reserved UTXOs are not spendable and must be excluded
    /// from the wallet's spendable balance.
    pub reserved: bool,
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
    /// Peer node public key as lowercase hex (66 chars).
    pub peer_id: String,
    pub our_amount_msat: u64,
    pub amount_msat: u64,
    /// Funding transaction id as lowercase hex (64 chars).
    pub funding_txid: String,
    pub funding_output: u32,
    pub connected: bool,
    pub state: ChannelState,
    pub short_channel_id: Option<String>,
    /// Channel id as lowercase hex (64 chars).
    pub channel_id: Option<String>,
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
            txid: hex::encode(&other.txid),
            output: other.output,
            amount_msat: other.amount_msat.map(|a| a.msat).unwrap_or(0),
            status,
            address: other.address,
            blockheight: other.blockheight,
            reserved: other.reserved,
        }
    }
}

impl From<clnpb::ListfundsChannels> for FundChannel {
    fn from(other: clnpb::ListfundsChannels) -> Self {
        let state = ChannelState::from_i32(other.state);
        Self {
            peer_id: hex::encode(&other.peer_id),
            our_amount_msat: other.our_amount_msat.map(|a| a.msat).unwrap_or(0),
            amount_msat: other.amount_msat.map(|a| a.msat).unwrap_or(0),
            funding_txid: hex::encode(&other.funding_txid),
            funding_output: other.funding_output,
            connected: other.connected,
            state,
            short_channel_id: other.short_channel_id,
            channel_id: other.channel_id.as_deref().map(hex::encode),
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
    /// Payment hash as lowercase hex (64 chars).
    pub payment_hash: String,
    pub status: InvoiceStatus,
    pub amount_msat: Option<u64>,
    pub amount_received_msat: Option<u64>,
    pub bolt11: Option<String>,
    pub bolt12: Option<String>,
    pub paid_at: Option<u64>,
    pub expires_at: u64,
    /// Payment preimage as lowercase hex (64 chars), if the invoice has been paid.
    pub payment_preimage: Option<String>,
    /// Recipient node pubkey as lowercase hex (66 chars), recovered from the bolt11.
    pub destination_pubkey: Option<String>,
}

/// Extract the payee public key from a BOLT11 invoice string as hex.
fn pubkey_from_bolt11(bolt11: &str) -> Option<String> {
    let invoice: Bolt11Invoice = bolt11.parse().ok()?;
    Some(hex::encode(invoice.recover_payee_pub_key().serialize()))
}

impl From<clnpb::ListinvoicesInvoices> for Invoice {
    fn from(other: clnpb::ListinvoicesInvoices) -> Self {
        let destination_pubkey = other.bolt11.as_deref().and_then(pubkey_from_bolt11);
        Self {
            label: other.label,
            description: other.description.unwrap_or_default(),
            payment_hash: hex::encode(&other.payment_hash),
            status: other.status.into(),
            amount_msat: other.amount_msat.map(|a| a.msat),
            amount_received_msat: other.amount_received_msat.map(|a| a.msat),
            bolt11: other.bolt11,
            bolt12: other.bolt12,
            paid_at: other.paid_at,
            expires_at: other.expires_at,
            payment_preimage: other.payment_preimage.as_deref().map(hex::encode),
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
    /// Payment hash as lowercase hex (64 chars).
    pub payment_hash: String,
    pub status: PayStatus,
    /// Recipient node pubkey as lowercase hex (66 chars), if known.
    pub destination_pubkey: Option<String>,
    pub amount_msat: Option<u64>,
    pub amount_sent_msat: Option<u64>,
    pub label: Option<String>,
    pub bolt11: Option<String>,
    pub description: Option<String>,
    pub bolt12: Option<String>,
    /// Payment preimage as lowercase hex (64 chars), if the payment completed.
    pub preimage: Option<String>,
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
            payment_hash: hex::encode(&other.payment_hash),
            status,
            destination_pubkey: other.destination.as_deref().map(hex::encode),
            amount_msat: other.amount_msat.map(|a| a.msat),
            amount_sent_msat: other.amount_sent_msat.map(|a| a.msat),
            label: other.label,
            bolt11: other.bolt11,
            description: other.description,
            bolt12: other.bolt12,
            preimage: other.preimage.as_deref().map(hex::encode),
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
// Unified list_payments request/response types
// ============================================================

#[derive(Clone, Default, uniffi::Record)]
pub struct ListPaymentsRequest {
    /// Filter by payment type (Sent, Received). None or empty = all.
    pub filters: Option<Vec<PaymentTypeFilter>>,
    /// Include only payments after this epoch timestamp (seconds).
    pub from_timestamp: Option<u64>,
    /// Include only payments before this epoch timestamp (seconds).
    pub to_timestamp: Option<u64>,
    /// Include failed payments. Default: false.
    pub include_failures: Option<bool>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Pagination limit.
    pub limit: Option<u32>,
}

#[derive(Clone, uniffi::Enum)]
pub enum PaymentTypeFilter {
    Sent,
    Received,
}

#[derive(Clone, uniffi::Record)]
pub struct Payment {
    pub id: String,
    pub payment_type: PaymentType,
    pub payment_time: u64,
    pub amount_msat: u64,
    pub fee_msat: u64,
    pub status: PaymentStatus,
    pub description: Option<String>,
    pub bolt11: Option<String>,
    /// Payment preimage as lowercase hex (64 chars), when known.
    pub preimage: Option<String>,
    /// Pubkey of the counterparty in the payment, as lowercase hex
    /// (66 chars).
    ///
    /// For `PaymentType::Sent`: the recipient node we paid (when CLN
    /// reports it).
    ///
    /// For `PaymentType::Received`: always `None`. Lightning's privacy
    /// model does not reveal the sender's pubkey to the recipient — the
    /// HTLC arrives via one of our channel peers, but that peer is
    /// usually just a router, not the original payer. The only pubkey
    /// derivable from a paid invoice is the *payee* (i.e. our own
    /// node), which is uninteresting to display per-row.
    pub destination: Option<String>,
}

#[derive(Clone, uniffi::Enum)]
pub enum PaymentType {
    Sent,
    Received,
}

#[derive(Clone, uniffi::Enum)]
pub enum PaymentStatus {
    Pending,
    Complete,
    Failed,
}

impl From<clnpb::ListinvoicesInvoices> for Payment {
    fn from(inv: clnpb::ListinvoicesInvoices) -> Self {
        let status = match inv.status() {
            clnpb::listinvoices_invoices::ListinvoicesInvoicesStatus::Paid => {
                PaymentStatus::Complete
            }
            clnpb::listinvoices_invoices::ListinvoicesInvoicesStatus::Expired => {
                PaymentStatus::Failed
            }
            clnpb::listinvoices_invoices::ListinvoicesInvoicesStatus::Unpaid => {
                PaymentStatus::Pending
            }
        };

        let payment_time = inv.paid_at.unwrap_or(inv.expires_at);
        let amount_msat = inv
            .amount_received_msat
            .or(inv.amount_msat)
            .map(|a| a.msat)
            .unwrap_or(0);

        Payment {
            id: hex::encode(&inv.payment_hash),
            payment_type: PaymentType::Received,
            payment_time,
            amount_msat,
            fee_msat: 0,
            status,
            description: inv.description,
            bolt11: inv.bolt11,
            preimage: inv.payment_preimage.as_deref().map(hex::encode),
            destination: None,
        }
    }
}

impl From<clnpb::ListpaysPays> for Payment {
    fn from(pay: clnpb::ListpaysPays) -> Self {
        let status = match pay.status() {
            clnpb::listpays_pays::ListpaysPaysStatus::Complete => PaymentStatus::Complete,
            clnpb::listpays_pays::ListpaysPaysStatus::Failed => PaymentStatus::Failed,
            clnpb::listpays_pays::ListpaysPaysStatus::Pending => PaymentStatus::Pending,
        };

        let payment_time = pay.completed_at.unwrap_or(pay.created_at);
        let amount_msat = pay.amount_msat.as_ref().map(|a| a.msat).unwrap_or(0);
        let amount_sent_msat = pay.amount_sent_msat.as_ref().map(|a| a.msat).unwrap_or(0);
        let fee_msat = amount_sent_msat.saturating_sub(amount_msat);

        Payment {
            id: hex::encode(&pay.payment_hash),
            payment_type: PaymentType::Sent,
            payment_time,
            amount_msat,
            fee_msat,
            status,
            description: pay.description,
            bolt11: pay.bolt11,
            preimage: pay.preimage.as_deref().map(hex::encode),
            destination: pay.destination.as_deref().map(hex::encode),
        }
    }
}

// ============================================================
// NodeState — unified node snapshot
// ============================================================

/// A point-in-time snapshot of the node's balances, capacity, and
/// connectivity. Returned by `node_state()`.
///
/// All amounts are in millisatoshis (1 sat = 1000 msat).
#[derive(Clone, uniffi::Record)]
pub struct NodeState {
    /// The node's public key as a lowercase hex string (66 chars).
    pub id: String,
    /// Latest block height the node has synced to.
    pub block_height: u32,
    /// The Bitcoin network this node is running on (e.g. "bitcoin", "regtest").
    pub network: String,
    /// CLN version string (e.g. "v24.11").
    pub version: String,
    /// Human-readable node alias, if set.
    pub alias: Option<String>,
    /// 3-byte RGB color of the node, as a lowercase hex string (6 chars).
    pub color: String,
    /// Number of channels that are open and operational. These are the
    /// channels that contribute to `channels_balance_msat`,
    /// `max_payable_msat`, `total_channel_capacity_msat`, and
    /// `total_inbound_liquidity_msat`.
    pub num_active_channels: u32,
    /// Number of channels that are being opened but not yet confirmed.
    /// Pending channels do not contribute to any balance or capacity
    /// field on this snapshot; their funds show up only after they
    /// transition to active.
    pub num_pending_channels: u32,
    /// Number of channels that are open but the peer is offline.
    /// Inactive channels hold balance but cannot be used for payments
    /// until the peer reconnects; they do not contribute to
    /// `max_payable_msat` or `total_inbound_liquidity_msat` (those are
    /// computed from the live `spendable_msat` / `receivable_msat`
    /// reported by CLN, which goes to zero when the peer is offline).
    pub num_inactive_channels: u32,
    /// Total our-side balance across all open channels, including amounts
    /// that protocol reserves make unspendable.
    ///
    /// This is the field a wallet's home screen should show as the
    /// user's "Lightning balance" — it reflects what they own off-chain,
    /// matching what they'd expect to see at a glance.
    ///
    /// Do **not** use this to gate a send button: some of it is locked
    /// in channel reserves. Use `max_payable_msat` for that.
    pub channels_balance_msat: u64,
    /// Aggregate spendable amount across all open channels. Equal to
    /// `channels_balance_msat - max_chan_reserve_msat`.
    ///
    /// This is the field a send screen should gate against — it is what
    /// the user can actually move right now over Lightning in total.
    ///
    /// Caveat: a single Lightning payment is additionally bounded by
    /// the largest channel's own `spendable_msat`. Reaching this full
    /// aggregate amount in one payment requires multi-path-payment
    /// support from the recipient and a working route.
    pub max_payable_msat: u64,
    /// Sum of all open channel capacities (your side + remote side).
    pub total_channel_capacity_msat: u64,
    /// Amount locked in protocol channel reserves, computed as
    /// `channels_balance_msat - max_payable_msat`. These sats are yours
    /// on paper but cannot be spent until the channel closes.
    pub max_chan_reserve_msat: u64,
    /// Confirmed on-chain balance available for spending or opening channels.
    pub onchain_balance_msat: u64,
    /// On-chain balance from transactions that have not yet been confirmed.
    pub unconfirmed_onchain_balance_msat: u64,
    /// On-chain balance confirmed but not yet spendable (e.g. coinbase
    /// outputs inside the 100-block maturation window).
    pub immature_onchain_balance_msat: u64,
    /// On-chain balance locked in channels that are being closed.
    /// These funds will become available once the close is confirmed.
    pub pending_onchain_balance_msat: u64,
    /// Largest single Lightning payment the node can receive without
    /// splitting across channels. Bounded by the inbound capacity of
    /// the largest open channel.
    pub max_receivable_single_payment_msat: u64,
    /// Total amount you can receive across all open channels combined.
    pub total_inbound_liquidity_msat: u64,
    /// Lowercase hex public keys of peers we have at least one channel
    /// with and are currently connected to. Peers we're connected to but
    /// have no channel with are not represented here; for routing-node
    /// use cases, query `list_peers()` directly.
    pub connected_channel_peers: Vec<String>,
    /// Unspent on-chain outputs owned by the node's wallet. Excludes
    /// spent outputs; includes confirmed, unconfirmed, immature, and
    /// reserved UTXOs (callers can filter by `status` and `reserved`).
    pub utxos: Vec<FundOutput>,

    // ------------------------------------------------------------------
    // Aggregate balance views. All amounts in millisatoshis, matching
    // the rest of this struct. Callers displaying sats should divide by
    // 1000 on the UI side.
    // ------------------------------------------------------------------
    /// All non-pending on-chain balance buckets summed:
    /// `onchain_balance_msat + unconfirmed_onchain_balance_msat + immature_onchain_balance_msat`.
    /// Excludes funds locked in closing channels (`pending_onchain_balance_msat`)
    /// since those are not yet on-chain UTXOs.
    pub total_onchain_msat: u64,
    /// Everything the user owns, summed: channel balance (including
    /// protocol reserves) + all on-chain buckets + funds locked in
    /// closing channels. The "total holdings" number a wallet home
    /// screen typically shows.
    pub total_balance_msat: u64,
    /// What the user can spend *right now*:
    /// `max_payable_msat + onchain_balance_msat`. Excludes reserves,
    /// unconfirmed, immature, and pending amounts. The number a
    /// send-money screen should gate against.
    pub spendable_balance_msat: u64,
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
    /// Payment hash of the paid invoice as lowercase hex (64 chars).
    pub payment_hash: String,
    /// The bolt11 invoice string.
    pub bolt11: String,
    /// Preimage that proves payment as lowercase hex (64 chars).
    pub preimage: String,
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
                    payment_hash: hex::encode(&paid.payment_hash),
                    bolt11: paid.bolt11,
                    preimage: hex::encode(&paid.preimage),
                    label: paid.label,
                    amount_msat: paid.amount_msat,
                },
            },
            None => NodeEvent::Unknown,
        }
    }
}
