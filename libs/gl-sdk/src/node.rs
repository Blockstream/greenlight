use crate::{credentials::Credentials, signer::Handle, util::exec, Error};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use gl_client::credentials::NodeIdProvider;
use gl_client::lnurl::models::LnUrlHttpClient as _;
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
    /// Background task that tails the gRPC event stream and dispatches
    /// events to the installed listener. A single listener per node;
    /// installing a new one aborts the previous task. Aborted on Drop.
    event_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    network: gl_client::bitcoin::Network,
}

impl Drop for Node {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.event_task.lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
        }
    }
}

impl Node {
    /// Construct a signerless Node — credentials only, no SDK-side
    /// signer running. The actual signing happens elsewhere (a paired
    /// device, a hardware signer, the CLN node's local signer).
    /// Operations that require signing fall through to the node side.
    ///
    /// **Not a UniFFI export.** UniFFI consumers reach this via
    /// `NodeBuilder::connect(credentials, None)` (mnemonic omitted).
    /// Sibling Rust crates (e.g. `gl-sdk-napi`) call this directly
    /// when wrapping signerless flows into their own bindings.
    pub fn signerless(credentials: Credentials) -> Result<Self, Error> {
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
            signer_handle: None,
            disconnected: AtomicBool::new(false),
            event_task: Mutex::new(None),
            network: gl_client::bitcoin::Network::Bitcoin,
        })
    }
}

#[uniffi::export]
impl Node {

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
    /// * `sat_per_vbyte` — Optional fee rate in sats per virtual byte.
    ///   Pass `None` to let the node pick. Pass the value from a prior
    ///   `prepare_onchain_send` to reproduce the previewed fee.
    /// * `utxos` — Optional pinned input set. Pass the `utxos` returned
    ///   by `prepare_onchain_send` (together with the same
    ///   `sat_per_vbyte`) to broadcast a transaction with the exact
    ///   inputs and fee shown in the preview. Pass `None` to let the
    ///   node coin-select.
    ///
    /// Returns the raw transaction, txid, and PSBT once broadcast.
    /// The transaction is broadcast immediately — this is not a dry run.
    pub fn onchain_send(
        &self,
        destination: String,
        amount_or_all: String,
        sat_per_vbyte: Option<u32>,
        utxos: Option<Vec<Outpoint>>,
    ) -> Result<OnchainSendResponse, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let satoshi = parse_amount_or_all(&amount_or_all)?;

        let req = clnpb::WithdrawRequest {
            destination,
            minconf: None,
            feerate: sat_per_vbyte.map(feerate_perkw_from_sat_per_vbyte),
            satoshi: Some(satoshi),
            utxos: utxos
                .unwrap_or_default()
                .into_iter()
                .map(outpoint_to_pb)
                .collect::<Result<Vec<_>, _>>()?,
        };

        exec(cln_client.withdraw(req))
            .map_err(|e| Error::Rpc(e.to_string()))
            .map(|r| r.into_inner().into())
    }

    /// Preview an on-chain send without broadcasting or reserving UTXOs.
    ///
    /// Runs CLN's coin selection at the given fee rate and returns the
    /// inputs that would be spent, the fee, and the amount the recipient
    /// would receive. Safe to call repeatedly (e.g. while the user
    /// adjusts a fee slider) — nothing is locked.
    ///
    /// To broadcast with the previewed values, pass the returned
    /// `utxos` and `sat_per_vbyte` back to `onchain_send`. Identical
    /// inputs at the same fee rate yield the same fee.
    ///
    /// **Use this for "Send Max" UIs.** `recipient_sat` is the only
    /// authoritative post-fee amount the destination will receive
    /// for a sweep. `NodeState.onchain_balance_msat` includes the
    /// emergency reserve and the fee — neither of which leaves the
    /// wallet with the recipient. For the entry-point button label
    /// (a pre-fee approximation that updates without an RPC), use
    /// `OnchainBalanceState::Available.withdrawable_sat`.
    ///
    /// # Arguments
    /// * `destination` — A Bitcoin address (bech32, p2sh, or p2tr).
    /// * `amount_or_all` — Amount to send. Accepts:
    ///   - `"50000"` or `"50000sat"` — 50,000 satoshis
    ///   - `"50000msat"` — 50,000 millisatoshis
    ///   - `"all"` — sweep the entire on-chain balance
    /// * `sat_per_vbyte` — Fee rate in sats per virtual byte. Pass
    ///   `None` to use the node's "normal" priority feerate; the
    ///   effective rate CLN picked is reported back in the result's
    ///   `sat_per_vbyte` field, which can be passed to `onchain_send`
    ///   to reproduce it.
    pub fn prepare_onchain_send(
        &self,
        destination: String,
        amount_or_all: String,
        sat_per_vbyte: Option<u32>,
    ) -> Result<PreparedOnchainSend, Error> {
        self.check_connected()?;
        let cln_client = exec(self.get_cln_client())?.clone();

        let satoshi = parse_amount_or_all(&amount_or_all)?;
        let is_sweep = matches!(satoshi.value, Some(clnpb::amount_or_all::Value::All(true)));

        // `startweight` must cover everything CLN does NOT add itself
        // during fundpsbt: the base tx overhead and the destination
        // output. CLN accumulates per-input spend weights and the
        // change output weight on top of this. See
        // lightning/plugins/spender/multiwithdraw.c:339 for the
        // canonical formula CLN uses for its own withdraw plugin.
        let startweight = BASE_TX_CORE_WEIGHT + output_weight_for_address(&destination);

        let feerate = match sat_per_vbyte {
            Some(rate) => feerate_perkw_from_sat_per_vbyte(rate),
            None => clnpb::Feerate {
                style: Some(clnpb::feerate::Style::Normal(true)),
            },
        };

        let req = clnpb::FundpsbtRequest {
            satoshi: Some(satoshi),
            feerate: Some(feerate),
            startweight,
            // `reserve = 0` is the whole point: CLN runs coin selection
            // and returns the would-be inputs but does not lock them.
            reserve: Some(0),
            minconf: None,
            locktime: None,
            min_witness_weight: None,
            // For non-sweep sends any leftover after the requested
            // amount + fee becomes change. For sweeps there is no
            // requested amount so the leftover is the recipient amount
            // and CLN reports it via `excess_msat`.
            excess_as_change: Some(!is_sweep),
            nonwrapped: None,
            opening_anchor_channel: None,
        };

        // Run fund_psbt and feerates concurrently. The latter is used
        // only to validate the requested rate against the network's
        // relay floor — without this check, a too-low `sat_per_vbyte`
        // produces a confusing post-broadcast `min relay fee not met`
        // failure instead of a clean pre-confirmation error.
        let (fund_res, feerates_res) = exec(async {
            let mut c_fund = cln_client.clone();
            let mut c_rates = cln_client.clone();
            tokio::join!(
                c_fund.fund_psbt(req),
                c_rates.feerates(clnpb::FeeratesRequest {
                    style: clnpb::feerates_request::FeeratesStyle::Perkw as i32,
                }),
            )
        });

        // Reject below-relay rates up front when the caller specified
        // one. If `feerates` itself failed, skip the check — a stale
        // bitcoind connection shouldn't block a prepare.
        if let (Some(rate), Ok(rates)) = (sat_per_vbyte, feerates_res.as_ref())
            && let Some(perkw) = rates.get_ref().perkw.as_ref()
        {
            let min_sat_per_vbyte =
                sat_per_vbyte_from_perkw(perkw.min_acceptable).max(1);
            if (rate as u64) < min_sat_per_vbyte {
                return Err(Error::Argument(
                    "sat_per_vbyte".to_owned(),
                    format!(
                        "{} sat/vbyte is below the network minimum of {} sat/vbyte",
                        rate, min_sat_per_vbyte
                    ),
                ));
            }
        }

        let res = fund_res
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();

        // CLN only emits the `reservations` array when `reserve > 0`
        // (see lightning/wallet/reservation.c:421 — `if (reserve)`).
        // We deliberately pass `reserve=0` to avoid locking UTXOs, so
        // we extract the chosen inputs from the returned PSBT instead.
        let psbt = bitcoin::Psbt::from_str(&res.psbt)
            .map_err(|e| Error::Rpc(format!("invalid psbt from fund_psbt: {}", e)))?;
        let utxos: Vec<Outpoint> = psbt
            .unsigned_tx
            .input
            .iter()
            .map(|tx_in| Outpoint {
                txid: tx_in.previous_output.txid.to_string(),
                vout: tx_in.previous_output.vout,
            })
            .collect();

        // BIP-141: feerate_per_kw is sats per 1000 weight units, so
        // fee_sat = weight_wu × feerate_per_kw / 1000. The proto-level
        // `estimated_final_weight` already includes the destination
        // output we declared via `startweight`, plus any change output.
        let fee_sat: u64 =
            (res.estimated_final_weight as u64 * res.feerate_per_kw as u64) / 1000;

        // Sum input values directly from the PSBT. Each PSBT input
        // carries its prevout amount in `witness_utxo` (segwit) or
        // `non_witness_utxo` (legacy). This is the one source of truth
        // and works for sweeps that include an emergency-reserve
        // change output (anchor-channel wallets) — see
        // lightning/wallet/reservation.c:443 `change_for_emergency`,
        // which carves out `emergency_sat` even from `satoshi=All`.
        let mut total_input_sat: u64 = 0;
        for (i, input) in psbt.inputs.iter().enumerate() {
            let value = if let Some(ref txout) = input.witness_utxo {
                txout.value
            } else if let Some(ref tx) = input.non_witness_utxo {
                let vout = psbt.unsigned_tx.input[i].previous_output.vout as usize;
                tx.output
                    .get(vout)
                    .map(|o| o.value)
                    .ok_or_else(|| {
                        Error::Rpc("psbt non_witness_utxo missing vout".to_owned())
                    })?
            } else {
                return Err(Error::Rpc(format!(
                    "psbt input {} has no witness_utxo or non_witness_utxo",
                    i
                )));
            };
            total_input_sat = total_input_sat.saturating_add(value.to_sat());
        }

        let recipient_sat: u64 = if is_sweep {
            // For `satoshi=All` CLN reports the post-fee, post-emergency
            // leftover via `excess_msat`; that's what the recipient
            // receives. Any difference between `total_input_sat` and
            // `recipient_sat + fee_sat` is the emergency-reserve change
            // CLN keeps in the wallet for anchor channels.
            res.excess_msat.as_ref().map(|a| a.msat).unwrap_or(0) / 1000
        } else {
            match parse_amount_or_all(&amount_or_all)?.value {
                Some(clnpb::amount_or_all::Value::Amount(a)) => a.msat / 1000,
                _ => 0,
            }
        };

        // Round up so passing this back to `onchain_send` produces a
        // feerate at least as high as the previewed one; that way the
        // broadcast fee is never below what the user agreed to.
        let effective_sat_per_vbyte: u32 =
            (res.feerate_per_kw as u64).div_ceil(250) as u32;

        Ok(PreparedOnchainSend {
            utxos,
            total_input_sat,
            fee_sat,
            recipient_sat,
            sat_per_vbyte: effective_sat_per_vbyte,
        })
    }

    /// Classify the on-chain wallet for the withdraw entry-point UI.
    ///
    /// Runs three RPCs concurrently:
    /// * `list_funds` — current confirmed/unconfirmed/immature on-chain
    ///   balances.
    /// * `list_peer_channels` — pending channel-close payouts that
    ///   haven't yet hit the wallet.
    /// * `fund_psbt(satoshi=All, reserve=0, normal feerate)` — a
    ///   non-locking probe whose response tells us **exactly** how
    ///   much CLN will carve as the anchor-channel emergency reserve
    ///   for this specific node, no client-side guessing required.
    ///   The carved amount is computed from the response as
    ///   `total_inputs − excess − fee`, which is identical to what
    ///   CLN would carve on a real broadcast.
    ///
    /// Cheaper to call than `node_state()` and answers a different
    /// question. Wallets typically call it once per render of the
    /// home screen.
    ///
    /// For the *exact* post-fee recipient amount of a withdraw, use
    /// `prepare_onchain_send`; the `withdrawable_sat` returned here
    /// is a pre-fee, reserve-aware figure for the entry-point label.
    pub fn onchain_balance_state(&self) -> Result<OnchainBalanceState, Error> {
        self.check_connected()?;
        let cln_client = exec(self.get_cln_client())?.clone();

        // Run the three RPCs concurrently. The probe is allowed to
        // fail (e.g. empty wallet, insufficient funds for any spend);
        // we treat that as "no reserve applicable" and let the rest
        // of the classification proceed.
        let (funds_res, channels_res, probe_res) = exec(async {
            let mut c_funds = cln_client.clone();
            let mut c_channels = cln_client.clone();
            let mut c_probe = cln_client.clone();
            let probe_req = clnpb::FundpsbtRequest {
                satoshi: Some(clnpb::AmountOrAll {
                    value: Some(clnpb::amount_or_all::Value::All(true)),
                }),
                feerate: Some(clnpb::Feerate {
                    style: Some(clnpb::feerate::Style::Normal(true)),
                }),
                // Assume a P2WPKH destination for the probe — we don't
                // have a real address here. The output type only
                // affects fee estimation by a handful of weight units;
                // it does not affect the carved emergency reserve.
                startweight: BASE_TX_CORE_WEIGHT + 124,
                reserve: Some(0),
                minconf: None,
                locktime: None,
                min_witness_weight: None,
                excess_as_change: Some(false),
                nonwrapped: None,
                opening_anchor_channel: None,
            };
            tokio::join!(
                c_funds.list_funds(clnpb::ListfundsRequest { spent: None }),
                c_channels.list_peer_channels(clnpb::ListpeerchannelsRequest { id: None }),
                c_probe.fund_psbt(probe_req),
            )
        });

        let funds: ListFundsResponse = funds_res
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner()
            .into();
        let channels: ListPeerChannelsResponse = channels_res
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner()
            .into();

        let mut confirmed_sat: u64 = 0;
        let mut unconfirmed_sat: u64 = 0;
        let mut immature_sat: u64 = 0;
        for output in &funds.outputs {
            if output.reserved {
                continue;
            }
            let value_sat = output.amount_msat / 1000;
            match output.status {
                OutputStatus::Confirmed => confirmed_sat += value_sat,
                OutputStatus::Unconfirmed => unconfirmed_sat += value_sat,
                OutputStatus::Immature => immature_sat += value_sat,
                OutputStatus::Spent => {}
            }
        }

        let mut pending_close_sat: u64 = 0;
        for ch in &channels.channels {
            if channel_payout_still_pending(ch) {
                pending_close_sat += ch.to_us_msat.unwrap_or(0) / 1000;
            }
        }

        // Derive the actual emergency reserve from the probe. CLN's
        // `change_for_emergency` runs server-side in the same
        // `fund_psbt` call we just made; the difference between the
        // input total and (recipient + fee) is exactly what would be
        // carved as change on a real sweep.
        let reserve_sat = match probe_res {
            Ok(resp) => {
                let resp = resp.into_inner();
                // CLN-managed UTXOs are always segwit, so each PSBT
                // input carries `witness_utxo` with the prevout
                // amount. Sum to get total input value.
                let total_input_sat = bitcoin::Psbt::from_str(&resp.psbt)
                    .ok()
                    .map(|p| {
                        p.inputs
                            .iter()
                            .filter_map(|i| {
                                i.witness_utxo.as_ref().map(|t| t.value.to_sat())
                            })
                            .sum::<u64>()
                    })
                    .unwrap_or(0);
                let excess_sat = resp
                    .excess_msat
                    .as_ref()
                    .map(|a| a.msat / 1000)
                    .unwrap_or(0);
                let fee_sat = (resp.estimated_final_weight as u64
                    * resp.feerate_per_kw as u64)
                    / 1000;
                total_input_sat
                    .saturating_sub(excess_sat)
                    .saturating_sub(fee_sat)
            }
            // `fund_psbt` errors are expected on empty wallets or when
            // every UTXO is dust-uneconomic at the chosen feerate;
            // treat as "no reserve applicable."
            Err(_) => 0,
        };

        Ok(classify_onchain_balance(
            confirmed_sat,
            reserve_sat,
            unconfirmed_sat,
            immature_sat,
            pending_close_sat,
        ))
    }

    /// On-chain fee rates, in sats per virtual byte, at several
    /// confirmation targets.
    ///
    /// Sourced from the connected node's view of the network — no
    /// 3rd-party HTTP calls. Use as the basis for a fee-picker UI;
    /// `minimum_relay_sat_per_vbyte` is the relay floor enforced at
    /// broadcast time and should be the lower bound of any slider.
    pub fn onchain_fee_rates(&self) -> Result<OnchainFeeRates, Error> {
        self.check_connected()?;
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::FeeratesRequest {
            style: clnpb::feerates_request::FeeratesStyle::Perkw as i32,
        };
        let res = exec(cln_client.feerates(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(compute_fee_rates(res.perkw.as_ref()))
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

    /// Collect a diagnostic snapshot of the node and SDK state.
    ///
    /// Returns a pretty-printed JSON string with shape:
    /// `{ "timestamp": <unix-secs>, "node": { ... }, "sdk": { "version": ..., "node_state": ... } }`.
    /// The `node` object contains one entry per CLN RPC (`getinfo`,
    /// `listpeerchannels`, `listfunds`); each value is the serialized
    /// response, or `{ "error": "..." }` if that RPC failed. Payment and
    /// invoice history are deliberately excluded to avoid leaking
    /// preimages, payment hashes, bolt11 strings, and labels into support
    /// dumps. Intended for support tickets.
    pub fn generate_diagnostic_data(&self) -> Result<String, Error> {
        self.check_connected()?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let getinfo = render_section(self.get_info());
        let listpeerchannels = render_section(self.list_peer_channels());
        let listfunds = render_section(self.list_funds());
        let node_state = render_section(self.node_state());

        build_diagnostic_json(
            timestamp,
            env!("CARGO_PKG_VERSION"),
            getinfo,
            listpeerchannels,
            listfunds,
            node_state,
        )
    }

    // ── LNURL methods ───────────────────────────────────────────

    /// Execute an LNURL-pay flow (LUD-06).
    ///
    /// Sends the chosen amount (and optional comment) to the service's
    /// callback, receives and validates a BOLT11 invoice, pays it, and
    /// processes any success action (LUD-09/10).
    ///
    /// Call the top-level `parse_input` first to obtain the
    /// `LnUrlPayRequestData`, then build an `LnUrlPayRequest` with the
    /// user's chosen amount.
    pub fn lnurl_pay(
        &self,
        request: crate::lnurl::LnUrlPayRequest,
    ) -> Result<crate::lnurl::LnUrlPayResult, Error> {
        self.check_connected()?;
        validate_lnurl_pay_input(&request)?;

        let http_client = gl_client::lnurl::models::LnUrlHttpClearnetClient::new();

        // Phase 1: Get invoice from service callback
        let comment = request.comment.as_deref();
        let (invoice_str, success_action) = match exec(
            gl_client::lnurl::pay::fetch_invoice(
                &http_client,
                &request.data.callback,
                request.amount_msat,
                comment,
            ),
        ) {
            Ok(v) => v,
            Err(e) => {
                let msg = e.to_string();
                let reason = msg
                    .strip_prefix(gl_client::lnurl::pay::LNURL_SERVICE_ERROR_PREFIX)
                    .unwrap_or(&msg)
                    .to_string();
                return Ok(crate::lnurl::LnUrlPayResult::EndpointError {
                    data: crate::lnurl::LnUrlErrorData { reason },
                });
            }
        };

        if let Some(reason) = invoice_network_mismatch(&invoice_str, self.network) {
            return Ok(crate::lnurl::LnUrlPayResult::EndpointError {
                data: crate::lnurl::LnUrlErrorData { reason },
            });
        }

        // Phase 2: Pay the invoice
        let mut cln_client = exec(self.get_cln_client())?.clone();
        let pay_response = match exec(cln_client.pay(clnpb::PayRequest {
            bolt11: invoice_str.clone(),
            ..Default::default()
        })) {
            Ok(r) => r.into_inner(),
            Err(e) => {
                let payment_hash = invoice_str
                    .parse::<Bolt11Invoice>()
                    .ok()
                    .map(|inv| inv.payment_hash().to_string())
                    .unwrap_or_default();
                return Ok(crate::lnurl::LnUrlPayResult::PayError {
                    data: crate::lnurl::LnUrlPayErrorData {
                        payment_hash,
                        reason: e.to_string(),
                    },
                });
            }
        };

        // Phase 3: Process success action if present
        let validate_url = request.validate_success_action_url.unwrap_or(true);
        let processed_action = match success_action {
            Some(action) => {
                let processed = action
                    .process(&pay_response.payment_preimage)
                    .map_err(|e| Error::Other(e.to_string()))?;
                if validate_url {
                    if let gl_client::lnurl::models::ProcessedSuccessAction::Url {
                        url, ..
                    } = &processed
                    {
                        if let Some(reason) =
                            url_action_domain_mismatch(&request.data.callback, url)
                        {
                            return Err(Error::Other(reason));
                        }
                    }
                }
                Some(processed.into())
            }
            None => None,
        };

        Ok(crate::lnurl::LnUrlPayResult::EndpointSuccess {
            data: crate::lnurl::LnUrlPaySuccessData {
                payment_preimage: hex::encode(&pay_response.payment_preimage),
                success_action: processed_action,
            },
        })
    }

    /// Execute an LNURL-withdraw flow (LUD-03).
    ///
    /// Creates an invoice on this node for the requested amount, sends
    /// it to the service's callback URL, and the service pays it
    /// asynchronously.
    ///
    /// Call the top-level `parse_input` first to obtain the
    /// `LnUrlWithdrawRequestData`, then build an `LnUrlWithdrawRequest`
    /// with the user's chosen amount.
    pub fn lnurl_withdraw(
        &self,
        request: crate::lnurl::LnUrlWithdrawRequest,
    ) -> Result<crate::lnurl::LnUrlWithdrawResult, Error> {
        self.check_connected()?;

        let http_client = gl_client::lnurl::models::LnUrlHttpClearnetClient::new();

        // Step 1: Create an invoice on our node
        let description = request
            .description
            .unwrap_or(request.data.default_description.clone());

        let invoice_response = self.receive(
            format!("lnurl-withdraw-{}", request.data.k1),
            description,
            Some(request.amount_msat),
        )?;

        // Step 2: Build callback URL and submit invoice to service
        let callback_url = gl_client::lnurl::withdraw::build_withdraw_callback_url(
            &request.data.callback,
            &request.data.k1,
            &invoice_response.bolt11,
        )
        .map_err(|e| Error::Other(e.to_string()))?;

        // Step 3: Send invoice to service
        match exec(http_client.send_invoice_for_withdraw_request(&callback_url)) {
            Ok(_) => Ok(crate::lnurl::LnUrlWithdrawResult::Ok {
                data: crate::lnurl::LnUrlWithdrawSuccessData {
                    invoice: invoice_response.bolt11,
                },
            }),
            Err(e) => Ok(crate::lnurl::LnUrlWithdrawResult::ErrorStatus {
                data: crate::lnurl::LnUrlErrorData {
                    reason: e.to_string(),
                },
            }),
        }
    }
}

fn render_section<T: serde::Serialize>(result: Result<T, Error>) -> serde_json::Value {
    match result {
        Ok(v) => serde_json::to_value(&v)
            .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() })),
        Err(e) => serde_json::json!({ "error": e.to_string() }),
    }
}

fn build_diagnostic_json(
    timestamp: u64,
    sdk_version: &str,
    getinfo: serde_json::Value,
    listpeerchannels: serde_json::Value,
    listfunds: serde_json::Value,
    node_state: serde_json::Value,
) -> Result<String, Error> {
    let envelope = serde_json::json!({
        "timestamp": timestamp,
        "node": {
            "getinfo": getinfo,
            "listpeerchannels": listpeerchannels,
            "listfunds": listfunds,
        },
        "sdk": {
            "version": sdk_version,
            "node_state": node_state,
        }
    });
    serde_json::to_string_pretty(&envelope).map_err(|e| Error::Other(e.to_string()))
}

/// Returns a human-readable reason if the invoice's BOLT-11 currency
/// prefix does not match the node's configured network.
///
/// Not a LUD-06 requirement; this is a wallet-side safety check that
/// prevents attempting to pay e.g. a testnet invoice from a mainnet
/// wallet. The payment would fail at the node layer regardless, but
/// this surfaces a clean error earlier.
fn invoice_network_mismatch(
    invoice_str: &str,
    node_network: gl_client::bitcoin::Network,
) -> Option<String> {
    use lightning_invoice::Currency;
    let invoice = invoice_str.parse::<Bolt11Invoice>().ok()?;
    let expected = match node_network {
        gl_client::bitcoin::Network::Bitcoin => Currency::Bitcoin,
        gl_client::bitcoin::Network::Testnet => Currency::BitcoinTestnet,
        gl_client::bitcoin::Network::Signet => Currency::Signet,
        gl_client::bitcoin::Network::Regtest => Currency::Regtest,
        _ => return None,
    };
    if invoice.currency() == expected {
        None
    } else {
        Some(format!(
            "invoice is for {:?}, but this node is on {:?}",
            invoice.currency(),
            node_network
        ))
    }
}

fn url_action_domain_mismatch(callback_url: &str, action_url: &str) -> Option<String> {
    let cb = url::Url::parse(callback_url).ok()?;
    let action = url::Url::parse(action_url).ok()?;
    let cb_domain = cb.domain()?;
    let action_domain = action.domain()?;
    if cb_domain == action_domain {
        None
    } else {
        Some(format!(
            "success action URL domain ({}) does not match the callback domain ({})",
            action_domain, cb_domain
        ))
    }
}

fn validate_lnurl_pay_input(request: &crate::lnurl::LnUrlPayRequest) -> Result<(), Error> {
    let data = &request.data;
    if request.amount_msat < data.min_sendable {
        return Err(Error::Other(format!(
            "amount_msat {} is below the service's min_sendable ({})",
            request.amount_msat, data.min_sendable
        )));
    }
    if request.amount_msat > data.max_sendable {
        return Err(Error::Other(format!(
            "amount_msat {} is above the service's max_sendable ({})",
            request.amount_msat, data.max_sendable
        )));
    }
    if let Some(comment) = request.comment.as_deref() {
        if data.comment_allowed == 0 && !comment.is_empty() {
            return Err(Error::Other(
                "this LNURL service does not accept comments".to_string(),
            ));
        }
        if (comment.len() as u64) > data.comment_allowed {
            return Err(Error::Other(format!(
                "comment length {} exceeds the service's comment_allowed ({})",
                comment.len(),
                data.comment_allowed
            )));
        }
    }
    Ok(())
}

// Not exported through uniffi
impl Node {
    /// Install a listener that receives real-time node events.
    ///
    /// Spawns a background task that tails the gRPC event stream and
    /// invokes `listener.on_event(...)` for every event. Each `Node`
    /// holds at most one listener — calling again replaces it. The task
    /// stops when the stream ends, errors, or the `Node` is dropped.
    ///
    /// Crate-private — installed via `NodeBuilder::with_event_listener`
    /// at construction time so events emitted during node bring-up
    /// aren't missed.
    pub(crate) fn set_event_listener(
        &self,
        listener: std::sync::Arc<dyn NodeEventListener>,
    ) -> Result<(), Error> {
        self.check_connected()?;
        let mut gl_client = exec(self.get_gl_client())?.clone();
        let req = glpb::NodeEventsRequest {};
        let stream = exec(gl_client.stream_node_events(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();

        let mut guard = self
            .event_task
            .lock()
            .map_err(|e| Error::Other(e.to_string()))?;
        if let Some(prev) = guard.take() {
            prev.abort();
        }

        let task = crate::util::get_runtime().spawn(async move {
            let mut stream = stream;
            loop {
                match stream.message().await {
                    Ok(Some(raw)) => {
                        if let Some(event) = node_event_from_pb(raw) {
                            listener.on_event(event);
                        }
                    }
                    Ok(None) => break,
                    Err(e) if e.code() == tonic::Code::Unknown => break,
                    Err(_) => break,
                }
            }
        });
        *guard = Some(task);
        Ok(())
    }

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
        network: gl_client::bitcoin::Network,
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
            event_task: Mutex::new(None),
            network,
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

/// A specific on-chain output, identified by its outpoint.
#[derive(Clone, uniffi::Record)]
pub struct Outpoint {
    /// Transaction id as lowercase hex (64 chars).
    pub txid: String,
    /// Output index within that transaction.
    pub vout: u32,
}

/// On-chain fee rates in sats per virtual byte at various
/// confirmation targets, derived from the connected node's view of
/// network mempool conditions. Use as the basis for a fee-picker UI.
#[derive(Clone, uniffi::Record)]
pub struct OnchainFeeRates {
    /// Target the next block (~10 min).
    pub next_block_sat_per_vbyte: u64,
    /// ~30 minute confirmation target (3 blocks).
    pub half_hour_sat_per_vbyte: u64,
    /// ~1 hour confirmation target (6 blocks).
    pub hour_sat_per_vbyte: u64,
    /// ~1 day confirmation target (144 blocks). Suitable for
    /// non-urgent sweeps.
    pub day_sat_per_vbyte: u64,
    /// Network minimum relay fee. Anything below this will be
    /// rejected by mempool policy at broadcast time. Use as the
    /// lower bound of any user-facing fee slider.
    pub minimum_relay_sat_per_vbyte: u64,
}

/// Convert sat/kw to sat/vbyte, rounding up so we never undershoot
/// the relay floor when the caller submits the value back.
fn sat_per_vbyte_from_perkw(perkw: u32) -> u64 {
    (perkw as u64).div_ceil(250)
}

/// Pick the smallest-blockcount estimate that is `≥ target_blocks`.
/// If none exists (the longest estimate is shorter than `target_blocks`),
/// fall back to the longest estimate available. Returns the rate in
/// sat per kilo-weight (perkw).
fn pick_perkw_for_target(
    estimates: &[clnpb::FeeratesPerkwEstimates],
    target_blocks: u32,
) -> Option<u32> {
    let above = estimates
        .iter()
        .filter(|e| e.blockcount >= target_blocks)
        .min_by_key(|e| e.blockcount)
        .map(|e| e.feerate);
    above.or_else(|| {
        estimates
            .iter()
            .max_by_key(|e| e.blockcount)
            .map(|e| e.feerate)
    })
}

/// Map CLN's perkw feerates into an `OnchainFeeRates` (sat/vbyte)
/// for the standard 5-bucket fee-picker UI. Rounds up at the
/// boundary so values produced here are never below the network's
/// relay floor when the caller submits them.
fn compute_fee_rates(perkw: Option<&clnpb::FeeratesPerkw>) -> OnchainFeeRates {
    // Universal fallback when the node hasn't reported feerates yet
    // (e.g. just after startup, no connection to bitcoind).
    const FALLBACK_SAT_PER_VBYTE: u64 = 1;

    let Some(p) = perkw else {
        return OnchainFeeRates {
            next_block_sat_per_vbyte: FALLBACK_SAT_PER_VBYTE,
            half_hour_sat_per_vbyte: FALLBACK_SAT_PER_VBYTE,
            hour_sat_per_vbyte: FALLBACK_SAT_PER_VBYTE,
            day_sat_per_vbyte: FALLBACK_SAT_PER_VBYTE,
            minimum_relay_sat_per_vbyte: FALLBACK_SAT_PER_VBYTE,
        };
    };

    let minimum_relay_sat_per_vbyte =
        sat_per_vbyte_from_perkw(p.min_acceptable).max(FALLBACK_SAT_PER_VBYTE);

    let bucket = |target_blocks: u32| -> u64 {
        pick_perkw_for_target(&p.estimates, target_blocks)
            .map(sat_per_vbyte_from_perkw)
            .unwrap_or(minimum_relay_sat_per_vbyte)
            .max(minimum_relay_sat_per_vbyte)
    };

    OnchainFeeRates {
        next_block_sat_per_vbyte: bucket(1),
        half_hour_sat_per_vbyte: bucket(3),
        hour_sat_per_vbyte: bucket(6),
        day_sat_per_vbyte: bucket(144),
        minimum_relay_sat_per_vbyte,
    }
}

/// Classifies the on-chain wallet into discrete cases that a wallet
/// UI can switch on to render the correct entry-point for the
/// withdraw flow. Derived purely from `NodeState` — no RPC.
#[derive(Clone, uniffi::Enum)]
pub enum OnchainBalanceState {
    /// No funds on-chain in any form (confirmed, unconfirmed,
    /// immature, or pending channel-close payouts are all zero).
    /// Don't render a withdraw entry point.
    Unavailable,

    /// Funds are spendable now. Render the withdraw entry point
    /// enabled with `withdrawable_sat` as the headline.
    Available {
        /// `onchain_balance_sat - emergency_reserve_sat`. Use as
        /// the displayed amount on the entry point.
        withdrawable_sat: u64,
        /// Held back by CLN for anchor-channel safety; cannot be
        /// withdrawn without closing channels first.
        emergency_reserve_sat: u64,
        /// Inbound on-chain funds not yet confirmed. Informational
        /// only — not part of `withdrawable_sat`.
        unconfirmed_sat: u64,
    },

    /// On-chain funds exist but are entirely locked as the
    /// anchor-channel emergency reserve. Render the entry point
    /// disabled with an explainer (e.g. "close channels to free
    /// these funds").
    ReserveOnly { reserve_sat: u64 },

    /// Inbound on-chain funds are awaiting confirmation. Render a
    /// "pending" indicator instead of an enabled withdraw button.
    PendingConfirmation { unconfirmed_sat: u64 },

    /// Funds exist as CSV-timelocked outputs from a recent channel
    /// close and can't be spent until the relative locktime
    /// expires. Render the entry point disabled with a
    /// "channel closing" explainer.
    Immature { immature_sat: u64 },
}

/// Pure variant classifier. Given the five sat-denominated balance
/// figures, decide which `OnchainBalanceState` variant applies. The
/// public method `Node::onchain_balance_state` gathers the figures
/// from CLN and calls this.
fn classify_onchain_balance(
    confirmed_sat: u64,
    reserve_sat: u64,
    unconfirmed_sat: u64,
    immature_sat: u64,
    pending_close_sat: u64,
) -> OnchainBalanceState {
    let withdrawable_sat = confirmed_sat.saturating_sub(reserve_sat);

    if confirmed_sat == 0
        && unconfirmed_sat == 0
        && immature_sat == 0
        && pending_close_sat == 0
    {
        return OnchainBalanceState::Unavailable;
    }
    if withdrawable_sat > ONCHAIN_DUST_THRESHOLD_SAT {
        return OnchainBalanceState::Available {
            withdrawable_sat,
            emergency_reserve_sat: reserve_sat,
            unconfirmed_sat,
        };
    }
    if confirmed_sat > 0 && reserve_sat > 0 {
        return OnchainBalanceState::ReserveOnly { reserve_sat };
    }
    if unconfirmed_sat > 0 {
        return OnchainBalanceState::PendingConfirmation { unconfirmed_sat };
    }
    OnchainBalanceState::Immature { immature_sat }
}

/// Preview of an on-chain send: the inputs CLN would select at the
/// given fee rate, the resulting fee, and the amount the recipient
/// would receive. Inputs are NOT reserved — the wallet is free to
/// spend them via other paths until `onchain_send` actually broadcasts.
///
/// Pass `utxos` and `sat_per_vbyte` back to `onchain_send` to broadcast
/// with identical inputs and fee.
///
/// Amounts are in satoshis: on-chain transactions cannot carry sub-sat
/// precision, so msat denomination would be misleading here.
#[derive(uniffi::Record)]
pub struct PreparedOnchainSend {
    /// UTXOs that would be spent, in selection order.
    pub utxos: Vec<Outpoint>,
    /// Sum of all input UTXO values, in satoshis.
    pub total_input_sat: u64,
    /// Fee that would be paid, in satoshis.
    pub fee_sat: u64,
    /// Amount the recipient would receive, in satoshis.
    /// For a sweep ("all") this equals `total_input_sat - fee_sat`.
    /// For a fixed amount this equals the requested amount.
    pub recipient_sat: u64,
    /// Effective fee rate (sat per virtual byte) the node used to
    /// compute this preview. Equal to the caller's `sat_per_vbyte` if
    /// one was supplied; otherwise the rate the node picked at
    /// "normal" priority. Pass this back to `onchain_send` to
    /// reproduce the previewed fee.
    pub sat_per_vbyte: u32,
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

/// Parse an `amount_or_all` argument into the protobuf `AmountOrAll`.
/// Accepts `"all"`, `"<n>"`, `"<n>sat"`, or `"<n>msat"`.
fn parse_amount_or_all(amount_or_all: &str) -> Result<clnpb::AmountOrAll, Error> {
    let (num, suffix): (String, String) =
        amount_or_all.chars().partition(|c| c.is_ascii_digit());

    let num = if num.is_empty() {
        0
    } else {
        num.parse::<u64>()
            .map_err(|_| Error::Argument("amount_or_all".to_owned(), amount_or_all.to_owned()))?
    };

    match (num, suffix.as_str()) {
        (n, "") | (n, "sat") => Ok(clnpb::AmountOrAll {
            value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount {
                msat: n * 1000,
            })),
        }),
        (n, "msat") => Ok(clnpb::AmountOrAll {
            value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount { msat: n })),
        }),
        (0, "all") => Ok(clnpb::AmountOrAll {
            value: Some(clnpb::amount_or_all::Value::All(true)),
        }),
        _ => Err(Error::Argument(
            "amount_or_all".to_owned(),
            amount_or_all.to_owned(),
        )),
    }
}

/// Build a CLN `Feerate` from a sat/vbyte value. CLN measures rates
/// in sat per 1000 weight units, and 1 vbyte = 4 weight units, so
/// `sat/kw = sat/vbyte × 250`.
fn feerate_perkw_from_sat_per_vbyte(sat_per_vbyte: u32) -> clnpb::Feerate {
    clnpb::Feerate {
        style: Some(clnpb::feerate::Style::Perkw(sat_per_vbyte * 250)),
    }
}

/// Convert a public `Outpoint` (hex txid) into the protobuf form
/// (raw txid bytes) used by CLN's `WithdrawRequest`.
fn outpoint_to_pb(o: Outpoint) -> Result<clnpb::Outpoint, Error> {
    let txid = hex::decode(&o.txid)
        .map_err(|_| Error::Argument("utxos.txid".to_owned(), o.txid.clone()))?;
    Ok(clnpb::Outpoint {
        txid,
        outnum: o.vout,
    })
}

/// Base transaction overhead in BIP-141 weight units, for a typical
/// segwit transaction with 1–252 inputs and 1–252 outputs:
/// `(version=4 + input_count_varint=1 + output_count_varint=1 +
/// locktime=4) × 4 + segwit_marker_flag=2 = 42 wu`. This is the
/// `bitcoin_tx_core_weight(1, 1)` value from CLN
/// (`lightning/bitcoin/tx.c:849`). At 253+ inputs/outputs the varints
/// grow to 3 bytes (8 wu more), which is rare enough to ignore.
const BASE_TX_CORE_WEIGHT: u32 = 42;

/// Conservative dust gate for the on-chain entry-point: highest dust
/// threshold across common output types at Bitcoin Core's default
/// `DUST_RELAY_TX_FEE = 3000` (sat/kvB). P2PKH is 546 sat, P2WPKH
/// 294 sat, P2TR/P2WSH 330 sat. Using 546 means `Available` implies
/// the user can plausibly send to any common address type.
/// Source: Bitcoin Core `policy/policy.cpp::GetDustThreshold` and
/// `bitcoin::Script::minimal_non_dust` at the default relay fee.
const ONCHAIN_DUST_THRESHOLD_SAT: u64 = 546;

/// Serialized weight (BIP-141 weight units) of a single output paying
/// to the given address. Used (with `BASE_TX_CORE_WEIGHT`) as
/// `startweight` for `FundPsbt`, which only accounts for inputs and
/// change on top of what the caller declares.
///
/// Output bytes = 8 (value) + varint(script_len) + script_pubkey.
/// All standard scripts are < 253 bytes so the varint is 1 byte. The
/// total is then × 4 since outputs are non-witness data.
///
/// Falls back to 172 wu for unparseable inputs so the fee is over-
/// rather than under-estimated.
fn output_weight_for_address(addr: &str) -> u32 {
    match bitcoin::Address::from_str(addr) {
        Ok(a) => {
            let spk_len = a.assume_checked().script_pubkey().len();
            let varint_len = if spk_len < 0xfd { 1 } else { 3 };
            ((8 + varint_len + spk_len) * 4) as u32
        }
        Err(_) => 172,
    }
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

#[derive(uniffi::Enum, Clone, serde::Serialize)]
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
#[derive(Clone, serde::Serialize, uniffi::Record)]
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
#[derive(Clone, serde::Serialize, uniffi::Record)]
pub struct ListPeerChannelsResponse {
    pub channels: Vec<PeerChannel>,
}

#[allow(unused)]
#[derive(Clone, serde::Serialize, uniffi::Record)]
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
#[derive(Clone, serde::Serialize, uniffi::Enum)]
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

#[derive(Clone, serde::Serialize, uniffi::Enum)]
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
#[derive(Clone, serde::Serialize, uniffi::Record)]
pub struct ListFundsResponse {
    pub outputs: Vec<FundOutput>,
    pub channels: Vec<FundChannel>,
}

#[allow(unused)]
#[derive(Clone, serde::Serialize, uniffi::Record)]
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

#[derive(Clone, serde::Serialize, uniffi::Enum)]
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
#[derive(Clone, serde::Serialize, uniffi::Record)]
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

#[derive(Clone, serde::Serialize, uniffi::Enum)]
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

#[derive(Clone, serde::Serialize, uniffi::Record)]
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

#[derive(Clone, serde::Serialize, uniffi::Record)]
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

#[derive(Clone, serde::Serialize, uniffi::Record)]
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

#[derive(Clone, serde::Serialize, uniffi::Record)]
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
#[derive(Clone, serde::Serialize, uniffi::Record)]
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

/// Callback interface for receiving node events.
///
/// `on_event` is invoked from the SDK's internal event-dispatch task.
/// Implementations should be cheap and non-blocking; to update UI,
/// dispatch to the main thread from inside the handler.
///
/// Installed via `NodeBuilder::with_event_listener(...)` so events
/// emitted during node bring-up are captured. The polling-style
/// `Node::stream_node_events()` API is still available for callers
/// that prefer to drive events themselves.
#[uniffi::export(callback_interface)]
pub trait NodeEventListener: Send + Sync {
    fn on_event(&self, event: NodeEvent);
}

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
        // Loop over wire events, skipping any the SDK doesn't recognise,
        // until we either decode a known event, the stream ends, or it
        // errors. The public `NodeEvent` enum is a closed set —
        // unknown server-side events are silently dropped here.
        loop {
            match exec(stream.message()) {
                Ok(Some(raw)) => {
                    if let Some(event) = node_event_from_pb(raw) {
                        return Ok(Some(event));
                    }
                    // Unknown event — fall through to next iteration.
                }
                Ok(None) => return Ok(None),
                Err(e) if e.code() == tonic::Code::Unknown => return Ok(None),
                Err(e) => return Err(Error::Rpc(e.to_string())),
            }
        }
    }
}

/// A real-time event from the node.
#[derive(Clone, uniffi::Enum)]
pub enum NodeEvent {
    /// An invoice was paid.
    InvoicePaid { details: InvoicePaidEvent },
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

/// Convert a wire-level `glpb::NodeEvent` into the typed SDK enum.
///
/// Returns `None` for events the SDK doesn't recognise (e.g. a future
/// server-side event type added after the client was built). Callers
/// silently skip `None` so unknown events never reach the foreign
/// bindings — the public `NodeEvent` is a closed set.
fn node_event_from_pb(other: glpb::NodeEvent) -> Option<NodeEvent> {
    match other.event {
        Some(glpb::node_event::Event::InvoicePaid(paid)) => Some(NodeEvent::InvoicePaid {
            details: InvoicePaidEvent {
                payment_hash: hex::encode(&paid.payment_hash),
                bolt11: paid.bolt11,
                preimage: hex::encode(&paid.preimage),
                label: paid.label,
                amount_msat: paid.amount_msat,
            },
        }),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_amount_or_all_handles_all_variants() {
        let all = parse_amount_or_all("all").unwrap();
        assert!(matches!(all.value, Some(clnpb::amount_or_all::Value::All(true))));

        let plain = parse_amount_or_all("50000").unwrap();
        assert!(matches!(
            plain.value,
            Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount { msat: 50_000_000 }))
        ));

        let sat = parse_amount_or_all("50000sat").unwrap();
        assert!(matches!(
            sat.value,
            Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount { msat: 50_000_000 }))
        ));

        let msat = parse_amount_or_all("50000msat").unwrap();
        assert!(matches!(
            msat.value,
            Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount { msat: 50_000 }))
        ));

        assert!(parse_amount_or_all("notanumber").is_err());
        assert!(parse_amount_or_all("50000btc").is_err());
    }

    #[test]
    fn classify_onchain_balance_unavailable_when_empty() {
        assert!(matches!(
            classify_onchain_balance(0, 0, 0, 0, 0),
            OnchainBalanceState::Unavailable
        ));
    }

    #[test]
    fn classify_onchain_balance_available_with_room_above_dust() {
        // confirmed=100k, reserve=25k, unconfirmed=5k → Available
        match classify_onchain_balance(100_000, 25_000, 5_000, 0, 0) {
            OnchainBalanceState::Available {
                withdrawable_sat,
                emergency_reserve_sat,
                unconfirmed_sat,
            } => {
                assert_eq!(withdrawable_sat, 75_000);
                assert_eq!(emergency_reserve_sat, 25_000);
                assert_eq!(unconfirmed_sat, 5_000);
            }
            other => panic!("expected Available, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn classify_onchain_balance_reserve_only_when_balance_equals_reserve() {
        // 25k confirmed, 25k reserve → withdrawable = 0
        match classify_onchain_balance(25_000, 25_000, 0, 0, 0) {
            OnchainBalanceState::ReserveOnly { reserve_sat } => {
                assert_eq!(reserve_sat, 25_000);
            }
            other => panic!(
                "expected ReserveOnly, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn classify_onchain_balance_pending_when_only_unconfirmed() {
        match classify_onchain_balance(0, 0, 50_000, 0, 0) {
            OnchainBalanceState::PendingConfirmation { unconfirmed_sat } => {
                assert_eq!(unconfirmed_sat, 50_000);
            }
            other => panic!(
                "expected PendingConfirmation, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn classify_onchain_balance_immature_when_only_immature() {
        match classify_onchain_balance(0, 0, 0, 100_000, 0) {
            OnchainBalanceState::Immature { immature_sat } => {
                assert_eq!(immature_sat, 100_000);
            }
            other => panic!("expected Immature, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn classify_onchain_balance_real_wallet_small_onchain_with_active_channels() {
        // Captured from a live mainnet wallet: 2 active channels,
        // ~1,228 sat confirmed on-chain, 25,000 sat reserve carved
        // (anchor-channel default). Withdrawable = 0 → ReserveOnly.
        match classify_onchain_balance(1_228, 25_000, 0, 0, 0) {
            OnchainBalanceState::ReserveOnly { reserve_sat } => {
                assert_eq!(reserve_sat, 25_000);
            }
            other => panic!(
                "expected ReserveOnly, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn classify_onchain_balance_real_wallet_onchain_just_above_reserve() {
        // Same wallet after a top-up: 28,228 sat confirmed, 25k
        // reserve → withdrawable = 3,228, well above the dust gate
        // → Available.
        match classify_onchain_balance(28_228, 25_000, 0, 0, 0) {
            OnchainBalanceState::Available {
                withdrawable_sat,
                emergency_reserve_sat,
                unconfirmed_sat,
            } => {
                assert_eq!(withdrawable_sat, 3_228);
                assert_eq!(emergency_reserve_sat, 25_000);
                assert_eq!(unconfirmed_sat, 0);
            }
            other => panic!(
                "expected Available, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn classify_onchain_balance_dust_only_above_reserve_is_not_available() {
        // Withdrawable would be 100 sat — below the 546 dust gate.
        // Falls through Available; lands on ReserveOnly.
        match classify_onchain_balance(25_100, 25_000, 0, 0, 0) {
            OnchainBalanceState::ReserveOnly { reserve_sat } => {
                assert_eq!(reserve_sat, 25_000);
            }
            other => panic!(
                "expected ReserveOnly, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn classify_onchain_balance_real_user_no_anchor_no_reserve() {
        // The user-reported case: 28,228 sat on-chain, channels open
        // but NOT anchor type, so probe returns 0 reserve. The
        // entry-point should show Available with the full balance.
        match classify_onchain_balance(28_228, 0, 0, 0, 0) {
            OnchainBalanceState::Available {
                withdrawable_sat,
                emergency_reserve_sat,
                unconfirmed_sat,
            } => {
                assert_eq!(withdrawable_sat, 28_228);
                assert_eq!(emergency_reserve_sat, 0);
                assert_eq!(unconfirmed_sat, 0);
            }
            other => panic!(
                "expected Available, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    fn perkw_with(estimates: Vec<(u32, u32)>, min_acceptable: u32) -> clnpb::FeeratesPerkw {
        clnpb::FeeratesPerkw {
            min_acceptable,
            max_acceptable: 0,
            opening: None,
            mutual_close: None,
            unilateral_close: None,
            unilateral_anchor_close: None,
            delayed_to_us: None,
            htlc_resolution: None,
            penalty: None,
            estimates: estimates
                .into_iter()
                .map(|(blockcount, feerate)| clnpb::FeeratesPerkwEstimates {
                    blockcount,
                    feerate,
                    smoothed_feerate: feerate,
                })
                .collect(),
            floor: None,
        }
    }

    #[test]
    fn fee_rates_maps_perkw_to_buckets() {
        // Typical CLN response with estimates at 2/6/12/144 blocks.
        // perkw values: 1 sat/vbyte = 250, 5 sat/vbyte = 1250.
        let perkw = perkw_with(
            vec![(2, 5000), (6, 2000), (12, 1500), (144, 500)],
            253, // min_acceptable just over 1 sat/vbyte
        );
        let r = compute_fee_rates(Some(&perkw));
        // 5000 perkw = 20 sat/vbyte (next-block target picks blockcount=2)
        assert_eq!(r.next_block_sat_per_vbyte, 20);
        // half_hour target is 3 blocks; smallest estimate ≥3 is 6 → 2000 perkw = 8 sat/vbyte
        assert_eq!(r.half_hour_sat_per_vbyte, 8);
        // hour target is 6 blocks → 2000 perkw = 8 sat/vbyte
        assert_eq!(r.hour_sat_per_vbyte, 8);
        // day target is 144 blocks → 500 perkw = 2 sat/vbyte
        assert_eq!(r.day_sat_per_vbyte, 2);
        // min_acceptable 253 perkw rounds up to 2 sat/vbyte
        assert_eq!(r.minimum_relay_sat_per_vbyte, 2);
    }

    #[test]
    fn fee_rates_fall_back_to_minimum_when_no_estimates() {
        let perkw = perkw_with(vec![], 750); // min ~3 sat/vbyte
        let r = compute_fee_rates(Some(&perkw));
        assert_eq!(r.minimum_relay_sat_per_vbyte, 3);
        // Empty estimates → all buckets fall back to minimum
        assert_eq!(r.next_block_sat_per_vbyte, 3);
        assert_eq!(r.half_hour_sat_per_vbyte, 3);
        assert_eq!(r.hour_sat_per_vbyte, 3);
        assert_eq!(r.day_sat_per_vbyte, 3);
    }

    #[test]
    fn fee_rates_no_perkw_at_all_returns_safe_floor() {
        let r = compute_fee_rates(None);
        assert_eq!(r.minimum_relay_sat_per_vbyte, 1);
        assert_eq!(r.next_block_sat_per_vbyte, 1);
        assert_eq!(r.day_sat_per_vbyte, 1);
    }

    #[test]
    fn fee_rates_buckets_never_below_minimum() {
        // 144-block estimate below min_acceptable — bucket must be
        // clamped to minimum so we never recommend below network relay.
        let perkw = perkw_with(
            vec![(2, 1500), (144, 250)], // 144→1 sat/vbyte, but min=1000 perkw = 4 sat/vbyte
            1000,
        );
        let r = compute_fee_rates(Some(&perkw));
        assert_eq!(r.minimum_relay_sat_per_vbyte, 4);
        // Even though the 144-block estimate is 1 sat/vbyte, we clamp up.
        assert_eq!(r.day_sat_per_vbyte, 4);
    }

    #[test]
    fn fee_rates_target_above_all_estimates_uses_largest() {
        // Only short-target estimates; day(144) should fall back to
        // the longest estimate available.
        let perkw = perkw_with(vec![(2, 5000), (6, 2500)], 250);
        let r = compute_fee_rates(Some(&perkw));
        // Longest available estimate is 6 blocks → 2500 perkw = 10 sat/vbyte
        assert_eq!(r.day_sat_per_vbyte, 10);
    }

    #[test]
    fn output_weight_for_address_per_script_type() {
        // P2WPKH — script_pubkey is 22 bytes, output = (8+1+22)*4 = 124
        // BIP-173 test vector.
        assert_eq!(
            output_weight_for_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"),
            124
        );

        // P2TR — script_pubkey is 34 bytes, output = (8+1+34)*4 = 172
        // BIP-341 test vector.
        assert_eq!(
            output_weight_for_address(
                "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0"
            ),
            172
        );

        // P2SH — script_pubkey is 23 bytes, output = (8+1+23)*4 = 128
        assert_eq!(output_weight_for_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"), 128);

        // P2PKH — script_pubkey is 25 bytes, output = (8+1+25)*4 = 136
        assert_eq!(output_weight_for_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"), 136);

        // Garbage falls back to the conservative 172 wu.
        assert_eq!(output_weight_for_address("not-an-address"), 172);
    }
}
