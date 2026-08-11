use super::{State, StateEntry, CHANNEL_PREFIX};
use crate::bitcoin::{OutPoint, Txid};
use crate::psbt::ParsedPsbt;
use anyhow::{anyhow, bail};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const SPLICE_SESSION_PREFIX: &str = "splices";
const SPLICE_OUTPOINT_PREFIX: &str = "splice_outpoints";
const SPLICE_WALLET_PSBT_PREFIX: &str = "splice_wallet_psbts";

fn splice_session_key(node_channel_id_hex: &str) -> String {
    format!("{SPLICE_SESSION_PREFIX}/{node_channel_id_hex}")
}

fn splice_outpoint_key(txid: &str, vout: u32) -> String {
    format!("{SPLICE_OUTPOINT_PREFIX}/{txid}:{vout}")
}

fn wallet_psbt_key(psbt_fingerprint: &str) -> String {
    format!("{SPLICE_WALLET_PSBT_PREFIX}/{psbt_fingerprint}")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpliceOrigin {
    LocalInitiator,
    PeerInitiated,
    DevSpliceUnresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplicePhase {
    Negotiating,
    CommitmentsSecured,
    SignaturesExchanging,
    PendingLock,
    Locked,
    Aborted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaSource {
    Vls,
    Cln,
    Unresolved,
}

impl Default for DeltaSource {
    fn default() -> Self {
        Self::Unresolved
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpliceTerminalReason {
    Locked,
    Aborted,
    ChannelDeleted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FundingOutpoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OldSpliceState {
    pub funding_outpoint: FundingOutpoint,
    pub channel_value_sat: u64,
    pub local_balance_sat: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeePolicy {
    pub feerate_per_kw: Option<u32>,
    pub force_feerate: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpliceIntentState {
    pub authorized_relative_amount_sat: Option<i64>,
    pub fee_policy: FeePolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalSpliceIntent {
    pub node_id_hex: String,
    pub channel_id_hex: String,
    pub node_channel_id_hex: String,
    pub old: OldSpliceState,
    pub authorized_relative_amount_sat: i64,
    pub fee_policy: FeePolicy,
    pub initial_psbt_fingerprint: String,
    pub initial_psbt_input_outpoints: Vec<FundingOutpoint>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplicePsbtState {
    pub candidate_fingerprint: Option<String>,
    pub frozen_fingerprint: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpliceCandidateState {
    pub funding_outpoint: Option<FundingOutpoint>,
    pub value_sat: Option<u64>,
    pub script_pubkey_hash: Option<String>,
    pub sign_splice_tx_input_index: Option<u32>,
    pub remote_funding_key_hex: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateFundingFacts {
    pub funding_outpoint: FundingOutpoint,
    pub value_sat: u64,
    pub script_pubkey_hash: String,
    pub sign_splice_tx_input_index: u32,
    pub remote_funding_key_hex: Option<String>,
}

impl From<CandidateFundingFacts> for SpliceCandidateState {
    fn from(value: CandidateFundingFacts) -> Self {
        Self {
            funding_outpoint: Some(value.funding_outpoint),
            value_sat: Some(value.value_sat),
            script_pubkey_hash: Some(value.script_pubkey_hash),
            sign_splice_tx_input_index: Some(value.sign_splice_tx_input_index),
            remote_funding_key_hex: value.remote_funding_key_hex,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpliceDeltaState {
    pub computed: bool,
    pub channel_delta_sat: i64,
    pub wallet_input_delta_sat: i64,
    pub wallet_output_delta_sat: i64,
    pub fee_burden_sat: i64,
    pub no_local_loss: bool,
    pub source: DeltaSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerRequestRecord {
    pub request_type: String,
    pub request_hash: String,
    pub phase: SplicePhase,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// TODO: Reconsider these fields once the VLS splice integration shape is settled.
// Keep observed Greenlight request facts here, and move authenticated intent and protocol
// state to the signer/VLS integration boundary.
pub struct SpliceSessionV1 {
    pub schema: String,
    pub schema_version: u16,
    pub origin: SpliceOrigin,
    pub phase: SplicePhase,
    pub node_id_hex: String,
    pub channel_id_hex: String,
    pub node_channel_id_hex: String,
    pub old: OldSpliceState,
    pub intent: SpliceIntentState,
    pub psbt: SplicePsbtState,
    pub cand: SpliceCandidateState,
    pub delta: SpliceDeltaState,
    pub linked_wallet_psbt_fingerprints: Vec<String>,
    pub signer_request_history: Vec<SignerRequestRecord>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub terminal_reason: Option<SpliceTerminalReason>,
}

impl SpliceSessionV1 {
    pub fn new(
        origin: SpliceOrigin,
        node_id_hex: String,
        channel_id_hex: String,
        node_channel_id_hex: String,
        old: OldSpliceState,
        authorized_relative_amount_sat: Option<i64>,
        fee_policy: FeePolicy,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            schema: "SpliceSessionV1".to_string(),
            schema_version: 1,
            origin,
            phase: SplicePhase::Negotiating,
            node_id_hex,
            channel_id_hex,
            node_channel_id_hex,
            old,
            intent: SpliceIntentState {
                authorized_relative_amount_sat,
                fee_policy,
            },
            psbt: SplicePsbtState::default(),
            cand: SpliceCandidateState::default(),
            delta: SpliceDeltaState::default(),
            linked_wallet_psbt_fingerprints: Vec::new(),
            signer_request_history: Vec::new(),
            created_at_ms: timestamp_ms,
            updated_at_ms: timestamp_ms,
            terminal_reason: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpliceOutpointIndexV1 {
    pub schema: String,
    pub schema_version: u16,
    pub splice_session_key: String,
}

impl SpliceOutpointIndexV1 {
    fn for_session(node_channel_id_hex: &str) -> Self {
        Self {
            schema: "SpliceOutpointIndexV1".to_string(),
            schema_version: 1,
            splice_session_key: splice_session_key(node_channel_id_hex),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletInput {
    pub txid: String,
    pub vout: u32,
    pub value_sat: u64,
    pub reserved_to_block: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// TODO: Reconsider which wallet-PSBT fields must remain durable once splice intent is
// supplied through the VLS approver interface.
pub struct SpliceWalletPsbtContextV1 {
    pub schema: String,
    pub schema_version: u16,
    pub signonly: Vec<u32>,
    pub wallet_inputs: Vec<WalletInput>,
    pub linked_node_channel_id_hex: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl SpliceWalletPsbtContextV1 {
    pub fn new(wallet_inputs: Vec<WalletInput>, timestamp_ms: u64) -> Self {
        Self {
            schema: "SpliceWalletPsbtContextV1".to_string(),
            schema_version: 1,
            signonly: Vec::new(),
            wallet_inputs,
            linked_node_channel_id_hex: None,
            created_at_ms: timestamp_ms,
            updated_at_ms: timestamp_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletInputReservation {
    pub txid: String,
    pub vout: u32,
    pub reserved_to_block: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundPsbtResponseFacts {
    pub psbt_fingerprint: String,
    pub wallet_inputs: Vec<WalletInput>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignPsbtIntentFacts {
    pub psbt_fingerprint: String,
    pub signonly: Vec<u32>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceUpdateResponseFacts {
    pub node_channel_id_hex: String,
    pub psbt_fingerprint: String,
    pub psbt_input_outpoints: Vec<FundingOutpoint>,
    pub commitments_secured: bool,
    pub signatures_secured: Option<bool>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceSignedResponseFacts {
    pub node_channel_id_hex: String,
    pub psbt_fingerprint: String,
    pub psbt_input_outpoints: Vec<FundingOutpoint>,
    pub candidate: Option<CandidateFundingFacts>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PsbtCaptureFacts {
    pub fingerprint: String,
    pub input_outpoints: Vec<FundingOutpoint>,
}

pub fn parse_base64_psbt(psbt: &str) -> anyhow::Result<PsbtCaptureFacts> {
    let psbt = ParsedPsbt::from_base64(psbt)?;
    Ok(PsbtCaptureFacts {
        fingerprint: psbt.fingerprint().to_string(),
        input_outpoints: psbt
            .input_outpoints()
            .into_iter()
            .map(|outpoint| FundingOutpoint {
                txid: outpoint.txid.to_string(),
                vout: outpoint.vout,
        })
            .collect(),
    })
}

pub fn wallet_inputs_from_psbt(
    psbt: &str,
    reservations: &[WalletInputReservation],
) -> anyhow::Result<Vec<WalletInput>> {
    let psbt = ParsedPsbt::from_base64(psbt)?;
    let funding_utxos = psbt.funding_utxos()?;
    psbt.input_outpoints()
        .into_iter()
        .zip(funding_utxos)
        .map(|(outpoint, funding_utxo)| {
            let txid = outpoint.txid.to_string();
            let vout = outpoint.vout;
            let reservation = reservations
                .iter()
                .find(|reservation| reservation.txid == txid && reservation.vout == vout);
            Ok(WalletInput {
                txid,
                vout,
                value_sat: funding_utxo.value.to_sat(),
                reserved_to_block: reservation
                    .and_then(|reservation| reservation.reserved_to_block),
            })
        })
        .collect()
}

pub fn candidate_funding_facts_from_psbt(
    psbt: &str,
    funding_txid: &str,
    funding_vout: u32,
    old_funding_outpoint: &FundingOutpoint,
) -> anyhow::Result<CandidateFundingFacts> {
    let psbt = ParsedPsbt::from_base64(psbt)?;
    let funding_txid =
        Txid::from_str(funding_txid).map_err(|e| anyhow!("invalid splice funding txid: {e}"))?;
    let old_funding_outpoint = OutPoint::new(
        Txid::from_str(&old_funding_outpoint.txid)
            .map_err(|e| anyhow!("invalid old funding txid: {e}"))?,
        old_funding_outpoint.vout,
    );
    let candidate = psbt.candidate_funding(funding_txid, funding_vout, old_funding_outpoint)?;

    Ok(CandidateFundingFacts {
        funding_outpoint: FundingOutpoint {
            txid: candidate.outpoint.txid.to_string(),
            vout: candidate.outpoint.vout,
        },
        value_sat: candidate.txout.value.to_sat(),
        script_pubkey_hash: sha256::digest(candidate.txout.script_pubkey.as_bytes()),
        sign_splice_tx_input_index: candidate.sign_splice_tx_input_index,
        remote_funding_key_hex: None,
    })
}

impl State {
    pub fn node_channel_id_for_funding_outpoint(
        &self,
        node_id_hex: &str,
        funding_outpoint: &FundingOutpoint,
    ) -> anyhow::Result<Option<String>> {
        let node_id = hex::decode(node_id_hex)
            .map_err(|e| anyhow!("invalid node id hex for channel lookup: {e}"))?;
        if node_id.len() != 33 {
            bail!(
                "invalid node id length for channel lookup: expected 33 bytes, got {}",
                node_id.len()
            );
        }

        let key_prefix = format!("{CHANNEL_PREFIX}/{node_id_hex}");
        let mut matches = Vec::new();
        for (key, entry) in self.values.iter() {
            if !key.starts_with(&key_prefix) || self.is_tombstone(key) {
                continue;
            }
            let channel: vls_persist::model::ChannelEntry =
                serde_json::from_value(entry.value.clone()).map_err(|e| {
                    anyhow!("failed to decode channel state value for key {key}: {e}")
                })?;
            let Some(setup) = channel.channel_setup else {
                continue;
            };
            if setup.funding_outpoint.txid.to_string() == funding_outpoint.txid
                && setup.funding_outpoint.vout == funding_outpoint.vout
            {
                matches.push(
                    key.strip_prefix(&format!("{CHANNEL_PREFIX}/"))
                        .expect("channel key prefix checked")
                        .to_string(),
                );
            }
        }

        match matches.as_slice() {
            [] => Ok(None),
            [node_channel_id_hex] => Ok(Some(node_channel_id_hex.clone())),
            _ => bail!(
                "multiple channels match funding outpoint {}:{}",
                funding_outpoint.txid,
                funding_outpoint.vout
            ),
        }
    }

    fn get_splice<T>(&self, key: &str) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        if self.is_tombstone(key) {
            return Ok(None);
        }
        let Some(entry) = self.values.get(key) else {
            return Ok(None);
        };
        serde_json::from_value(entry.value.clone())
            .map(Some)
            .map_err(|e| anyhow!("failed to decode splice state value for key {}: {}", key, e))
    }

    fn put_splice<T>(&mut self, key: &str, value: &T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        if self.is_tombstone(key) {
            anyhow::bail!("key {} has been deleted", key);
        }
        let value = serde_json::to_value(value)
            .map_err(|e| anyhow!("failed to encode splice state value for key {}: {}", key, e))?;
        let version = self.next_version(key);
        self.values
            .insert(key.to_owned(), StateEntry::new(version, value));
        Ok(())
    }

    pub fn get_splice_session(
        &self,
        node_channel_id_hex: &str,
    ) -> anyhow::Result<Option<SpliceSessionV1>> {
        self.get_splice(&splice_session_key(node_channel_id_hex))
    }

    fn put_splice_session(&mut self, session: &SpliceSessionV1) -> anyhow::Result<()> {
        self.put_splice(&splice_session_key(&session.node_channel_id_hex), session)
    }

    fn link_splice_psbt_context(
        &mut self,
        session: &mut SpliceSessionV1,
        psbt_fingerprint: &str,
        psbt_input_outpoints: &[FundingOutpoint],
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let source_fingerprints = session.linked_wallet_psbt_fingerprints.clone();
        let mut context = self
            .get_psbt_context(psbt_fingerprint)?
            .unwrap_or_else(|| SpliceWalletPsbtContextV1::new(Vec::new(), updated_at_ms));
        if let Some(linked_channel) = context.linked_node_channel_id_hex.as_deref() {
            if linked_channel != session.node_channel_id_hex {
                bail!(
                    "PSBT {} is already linked to splice channel {}",
                    psbt_fingerprint,
                    linked_channel
                );
            }
        }

        context.linked_node_channel_id_hex = Some(session.node_channel_id_hex.clone());
        context.updated_at_ms = updated_at_ms;
        if !session
            .linked_wallet_psbt_fingerprints
            .iter()
            .any(|fingerprint| fingerprint == psbt_fingerprint)
        {
            session
                .linked_wallet_psbt_fingerprints
                .push(psbt_fingerprint.to_string());
        }
        self.put_splice_wallet_psbt_context(psbt_fingerprint, context)?;
        for source_fingerprint in source_fingerprints {
            self.inherit_splice_wallet_context(
                &source_fingerprint,
                psbt_fingerprint,
                psbt_input_outpoints,
                &session.node_channel_id_hex,
                updated_at_ms,
            )?;
        }
        Ok(())
    }

    fn create_new_splice_session(
        &mut self,
        session: SpliceSessionV1,
        expected_origin: SpliceOrigin,
    ) -> anyhow::Result<()> {
        if session.origin != expected_origin {
            bail!(
                "splice session origin {:?} does not match expected {:?}",
                session.origin,
                expected_origin
            );
        }
        if session.phase != SplicePhase::Negotiating {
            bail!("new splice sessions must start in negotiating phase");
        }
        if self
            .get_splice_session(&session.node_channel_id_hex)?
            .is_some()
        {
            bail!(
                "live splice session already exists for channel {}",
                session.node_channel_id_hex
            );
        }
        self.put_splice_session(&session)
    }

    pub fn create_local_splice_session(&mut self, session: SpliceSessionV1) -> anyhow::Result<()> {
        if session.intent.authorized_relative_amount_sat.is_none() {
            bail!("local splice sessions require authorized relative amount");
        }
        self.create_new_splice_session(session, SpliceOrigin::LocalInitiator)
    }

    pub fn record_local_splice_intent(&mut self, intent: LocalSpliceIntent) -> anyhow::Result<()> {
        let psbt_fingerprint = intent.initial_psbt_fingerprint.clone();
        let psbt_input_outpoints = intent.initial_psbt_input_outpoints.clone();

        if let Some(mut session) = self.get_splice_session(&intent.node_channel_id_hex)? {
            if session.origin != SpliceOrigin::LocalInitiator {
                bail!(
                    "cannot replace {:?} splice session with local intent",
                    session.origin
                );
            }
            if session.phase != SplicePhase::Negotiating {
                bail!(
                    "splice_init intent can only update negotiating session, current phase {:?}",
                    session.phase
                );
            }
            session.node_id_hex = intent.node_id_hex;
            session.channel_id_hex = intent.channel_id_hex;
            session.old = intent.old;
            session.intent.authorized_relative_amount_sat =
                Some(intent.authorized_relative_amount_sat);
            session.intent.fee_policy = intent.fee_policy;
            session.psbt.candidate_fingerprint = Some(psbt_fingerprint.clone());
            session.updated_at_ms = intent.timestamp_ms;
            self.link_splice_psbt_context(
                &mut session,
                &psbt_fingerprint,
                &psbt_input_outpoints,
                intent.timestamp_ms,
            )?;
            return self.put_splice_session(&session);
        }

        let mut session = SpliceSessionV1::new(
            SpliceOrigin::LocalInitiator,
            intent.node_id_hex,
            intent.channel_id_hex,
            intent.node_channel_id_hex,
            intent.old,
            Some(intent.authorized_relative_amount_sat),
            intent.fee_policy,
            intent.timestamp_ms,
        );
        session.psbt.candidate_fingerprint = Some(psbt_fingerprint.clone());
        self.link_splice_psbt_context(
            &mut session,
            &psbt_fingerprint,
            &psbt_input_outpoints,
            intent.timestamp_ms,
        )?;
        self.create_local_splice_session(session)
    }

    pub fn record_fundpsbt_response(&mut self, facts: FundPsbtResponseFacts) -> anyhow::Result<()> {
        let existing = self.get_psbt_context(&facts.psbt_fingerprint)?;
        let mut context = existing
            .unwrap_or_else(|| SpliceWalletPsbtContextV1::new(Vec::new(), facts.timestamp_ms));
        context.wallet_inputs = facts.wallet_inputs;
        context.updated_at_ms = facts.timestamp_ms;
        self.put_splice_wallet_psbt_context(&facts.psbt_fingerprint, context)
    }

    pub fn record_signpsbt_intent(&mut self, facts: SignPsbtIntentFacts) -> anyhow::Result<()> {
        let existing = self.get_psbt_context(&facts.psbt_fingerprint)?;
        let mut context = existing
            .unwrap_or_else(|| SpliceWalletPsbtContextV1::new(Vec::new(), facts.timestamp_ms));
        context.signonly = facts.signonly;
        context.updated_at_ms = facts.timestamp_ms;
        self.put_splice_wallet_psbt_context(&facts.psbt_fingerprint, context)
    }

    pub fn record_splice_update_response(
        &mut self,
        facts: SpliceUpdateResponseFacts,
    ) -> anyhow::Result<()> {
        let psbt_fingerprint = facts.psbt_fingerprint.clone();
        let psbt_input_outpoints = facts.psbt_input_outpoints.clone();
        let mut session = self
            .get_splice_session(&facts.node_channel_id_hex)?
            .ok_or_else(|| {
                anyhow!(
                    "missing splice session for channel {}",
                    facts.node_channel_id_hex
                )
            })?;

        if matches!(session.phase, SplicePhase::Locked | SplicePhase::Aborted) {
            bail!(
                "splice_update cannot update terminal splice phase {:?}",
                session.phase
            );
        }

        if facts.commitments_secured {
            session.psbt.frozen_fingerprint = Some(facts.psbt_fingerprint.clone());
            session.psbt.candidate_fingerprint = Some(facts.psbt_fingerprint);
            session.phase = if facts.signatures_secured == Some(true) {
                SplicePhase::SignaturesExchanging
            } else {
                SplicePhase::CommitmentsSecured
            };
        } else {
            if session.phase != SplicePhase::Negotiating {
                bail!(
                    "candidate PSBT can only change while negotiating, current phase {:?}",
                    session.phase
                );
            }
            session.psbt.candidate_fingerprint = Some(facts.psbt_fingerprint);
        }

        session.updated_at_ms = facts.timestamp_ms;
        self.link_splice_psbt_context(
            &mut session,
            &psbt_fingerprint,
            &psbt_input_outpoints,
            facts.timestamp_ms,
        )?;
        self.put_splice_session(&session)
    }

    pub fn record_splice_signed_response(
        &mut self,
        facts: SpliceSignedResponseFacts,
    ) -> anyhow::Result<()> {
        let psbt_fingerprint = facts.psbt_fingerprint.clone();
        let psbt_input_outpoints = facts.psbt_input_outpoints.clone();
        let mut session = self
            .get_splice_session(&facts.node_channel_id_hex)?
            .ok_or_else(|| {
                anyhow!(
                    "missing splice session for channel {}",
                    facts.node_channel_id_hex
                )
            })?;
        if !matches!(
            session.phase,
            SplicePhase::CommitmentsSecured
                | SplicePhase::SignaturesExchanging
                | SplicePhase::PendingLock
        ) {
            bail!(
                "splice_signed response requires commitments secured, current phase {:?}",
                session.phase
            );
        }

        session.psbt.candidate_fingerprint = Some(facts.psbt_fingerprint.clone());
        if session.psbt.frozen_fingerprint.is_none() {
            session.psbt.frozen_fingerprint = Some(facts.psbt_fingerprint);
        }
        session.phase = SplicePhase::SignaturesExchanging;
        session.updated_at_ms = facts.timestamp_ms;

        self.link_splice_psbt_context(
            &mut session,
            &psbt_fingerprint,
            &psbt_input_outpoints,
            facts.timestamp_ms,
        )?;

        if let Some(candidate) = facts.candidate {
            let index = SpliceOutpointIndexV1::for_session(&facts.node_channel_id_hex);
            let candidate_outpoint = candidate.funding_outpoint.clone();
            session.phase = SplicePhase::PendingLock;
            session.cand = candidate.into();
            self.put_splice_session(&session)?;
            return self.put_splice_outpoint_index(&candidate_outpoint, &index);
        }

        self.put_splice_session(&session)
    }

    pub fn create_peer_splice_session(&mut self, session: SpliceSessionV1) -> anyhow::Result<()> {
        if session.intent.authorized_relative_amount_sat.is_some() {
            bail!("peer-initiated splice sessions must not include local relative amount intent");
        }
        self.create_new_splice_session(session, SpliceOrigin::PeerInitiated)
    }

    pub fn create_dev_splice_session(&mut self, session: SpliceSessionV1) -> anyhow::Result<()> {
        self.create_new_splice_session(session, SpliceOrigin::DevSpliceUnresolved)
    }

    pub fn update_splice_candidate(
        &mut self,
        node_channel_id_hex: &str,
        psbt_fingerprint: String,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        if session.phase != SplicePhase::Negotiating {
            bail!(
                "candidate PSBT can only change while negotiating, current phase {:?}",
                session.phase
            );
        }
        session.psbt.candidate_fingerprint = Some(psbt_fingerprint);
        session.updated_at_ms = updated_at_ms;
        self.put_splice_session(&session)
    }

    pub fn freeze_splice_candidate(
        &mut self,
        node_channel_id_hex: &str,
        frozen_psbt_fingerprint: String,
        candidate: CandidateFundingFacts,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        if session.phase != SplicePhase::Negotiating {
            bail!(
                "candidate can only be frozen from negotiating phase, current phase {:?}",
                session.phase
            );
        }
        let index = SpliceOutpointIndexV1::for_session(node_channel_id_hex);
        let candidate_outpoint = candidate.funding_outpoint.clone();
        session.phase = SplicePhase::CommitmentsSecured;
        session.psbt.frozen_fingerprint = Some(frozen_psbt_fingerprint);
        session.cand = candidate.into();
        session.updated_at_ms = updated_at_ms;
        self.put_splice_session(&session)?;
        self.put_splice_outpoint_index(&candidate_outpoint, &index)
    }

    pub fn mark_splice_pending_lock(
        &mut self,
        node_channel_id_hex: &str,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        if !matches!(
            session.phase,
            SplicePhase::CommitmentsSecured
                | SplicePhase::SignaturesExchanging
                | SplicePhase::PendingLock
        ) {
            bail!(
                "pending lock requires secured commitments or signatures, current phase {:?}",
                session.phase
            );
        }
        session.phase = SplicePhase::PendingLock;
        session.updated_at_ms = updated_at_ms;
        self.put_splice_session(&session)
    }

    pub fn put_splice_outpoint_index(
        &mut self,
        candidate_outpoint: &FundingOutpoint,
        index: &SpliceOutpointIndexV1,
    ) -> anyhow::Result<()> {
        self.put_splice(
            &splice_outpoint_key(&candidate_outpoint.txid, candidate_outpoint.vout),
            index,
        )
    }

    pub fn get_splice_by_outpoint(
        &self,
        txid: &str,
        vout: u32,
    ) -> anyhow::Result<Option<SpliceSessionV1>> {
        let Some(index): Option<SpliceOutpointIndexV1> =
            self.get_splice(&splice_outpoint_key(txid, vout))?
        else {
            return Ok(None);
        };
        self.get_splice(&index.splice_session_key)
    }

    pub fn put_splice_wallet_psbt_context(
        &mut self,
        psbt_fingerprint: &str,
        context: SpliceWalletPsbtContextV1,
    ) -> anyhow::Result<()> {
        self.put_splice(&wallet_psbt_key(psbt_fingerprint), &context)
    }

    pub fn get_psbt_context(
        &self,
        psbt_fingerprint: &str,
    ) -> anyhow::Result<Option<SpliceWalletPsbtContextV1>> {
        self.get_splice(&wallet_psbt_key(psbt_fingerprint))
    }

    pub fn link_splice_wallet_psbt(
        &mut self,
        psbt_fingerprint: &str,
        node_channel_id_hex: &str,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        let mut context = self
            .get_psbt_context(psbt_fingerprint)?
            .ok_or_else(|| anyhow!("missing wallet PSBT context {}", psbt_fingerprint))?;
        let fingerprint_matches_splice = session.psbt.candidate_fingerprint.as_deref()
            == Some(psbt_fingerprint)
            || session.psbt.frozen_fingerprint.as_deref() == Some(psbt_fingerprint);
        if !fingerprint_matches_splice {
            bail!(
                "wallet PSBT {} does not match splice candidate for channel {}",
                psbt_fingerprint,
                node_channel_id_hex
            );
        }

        context.linked_node_channel_id_hex = Some(node_channel_id_hex.to_string());
        context.updated_at_ms = updated_at_ms;
        if !session
            .linked_wallet_psbt_fingerprints
            .iter()
            .any(|fingerprint| fingerprint == psbt_fingerprint)
        {
            session
                .linked_wallet_psbt_fingerprints
                .push(psbt_fingerprint.to_string());
        }
        session.updated_at_ms = updated_at_ms;
        self.put_splice_wallet_psbt_context(psbt_fingerprint, context)?;
        self.put_splice_session(&session)
    }

    pub fn inherit_splice_wallet_context(
        &mut self,
        source_psbt_fingerprint: &str,
        candidate_psbt_fingerprint: &str,
        candidate_input_outpoints: &[FundingOutpoint],
        node_channel_id_hex: &str,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        if source_psbt_fingerprint == candidate_psbt_fingerprint {
            return Ok(());
        }
        let Some(source) = self.get_psbt_context(source_psbt_fingerprint)? else {
            return Ok(());
        };
        let mut candidate = self
            .get_psbt_context(candidate_psbt_fingerprint)?
            .ok_or_else(|| {
                anyhow!(
                    "missing splice candidate PSBT context {}",
                    candidate_psbt_fingerprint
                )
            })?;
        if candidate.linked_node_channel_id_hex.as_deref() != Some(node_channel_id_hex) {
            bail!(
                "splice candidate PSBT context {} is not linked to channel {}",
                candidate_psbt_fingerprint,
                node_channel_id_hex
            );
        }

        for wallet_input in source.wallet_inputs {
            let remains_in_candidate = candidate_input_outpoints.iter().any(|outpoint| {
                outpoint.txid == wallet_input.txid && outpoint.vout == wallet_input.vout
            });
            let already_known = candidate
                .wallet_inputs
                .iter()
                .any(|input| input.txid == wallet_input.txid && input.vout == wallet_input.vout);
            if remains_in_candidate && !already_known {
                candidate.wallet_inputs.push(wallet_input);
            }
        }
        candidate.updated_at_ms = updated_at_ms;
        self.put_splice_wallet_psbt_context(candidate_psbt_fingerprint, candidate)
    }

    pub fn tombstone_splice_session(&mut self, node_channel_id_hex: &str) -> anyhow::Result<()> {
        let Some(session) = self.get_splice_session(node_channel_id_hex)? else {
            return Ok(());
        };
        let mut keys = vec![splice_session_key(node_channel_id_hex)];
        if let Some(outpoint) = &session.cand.funding_outpoint {
            keys.push(splice_outpoint_key(&outpoint.txid, outpoint.vout));
        }
        keys.extend(
            session
                .linked_wallet_psbt_fingerprints
                .iter()
                .map(|fingerprint| wallet_psbt_key(fingerprint)),
        );
        keys.sort();
        keys.dedup();

        for key in keys {
            self.put_tombstone(&key);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::absolute::LockTime;
    use crate::bitcoin::psbt::Psbt;
    use crate::bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
    use crate::bitcoin::transaction::Version;
    use crate::bitcoin::{
        Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
    };
    use crate::lightning::ln::chan_utils::ChannelPublicKeys;
    use crate::lightning::ln::channel_keys::{
        DelayedPaymentBasepoint, HtlcBasepoint, RevocationBasepoint,
    };
    use crate::pb::SignerStateEntry;
    use base64::{engine::general_purpose, Engine as _};
    use lightning_signer::channel::{ChannelSetup, CommitmentType};
    use lightning_signer::policy::validator::EnforcementState;
    use serde_json::json;
    use std::str::FromStr;

    use crate::psbt::CLN_PSBT_V2;

    fn outpoint(txid: &str, vout: u32) -> FundingOutpoint {
        FundingOutpoint {
            txid: txid.to_string(),
            vout,
        }
    }

    fn channel_entry(txid: &str, vout: u32) -> vls_persist::model::ChannelEntry {
        let secret = SecretKey::from_slice(&[1; 32]).unwrap();
        let pubkey = PublicKey::from_secret_key(&Secp256k1::signing_only(), &secret);
        vls_persist::model::ChannelEntry {
            channel_value_satoshis: 1_000_000,
            channel_setup: Some(ChannelSetup {
                is_outbound: true,
                channel_value_sat: 1_000_000,
                push_value_msat: 0,
                funding_outpoint: OutPoint {
                    txid: Txid::from_str(txid).unwrap(),
                    vout,
                },
                holder_selected_contest_delay: 6,
                holder_shutdown_script: None,
                counterparty_points: ChannelPublicKeys {
                    funding_pubkey: pubkey,
                    revocation_basepoint: RevocationBasepoint(pubkey),
                    payment_point: pubkey,
                    delayed_payment_basepoint: DelayedPaymentBasepoint(pubkey),
                    htlc_basepoint: HtlcBasepoint(pubkey),
                },
                counterparty_selected_contest_delay: 6,
                counterparty_shutdown_script: None,
                commitment_type: CommitmentType::StaticRemoteKey,
            }),
            id: None,
            enforcement_state: EnforcementState::new(600_000),
            blockheight: None,
        }
    }

    fn psbt_fixture(
        prev_txid: &str,
        prev_vout: u32,
        input_value_sat: u64,
        outputs: Vec<(u64, &str)>,
    ) -> String {
        let input_script =
            ScriptBuf::from_hex("00140000000000000000000000000000000000000000").unwrap();
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_str(prev_txid).unwrap(),
                    vout: prev_vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: outputs
                .into_iter()
                .map(|(value_sat, script_hex)| TxOut {
                    value: Amount::from_sat(value_sat),
                    script_pubkey: ScriptBuf::from_hex(script_hex).unwrap(),
                })
                .collect(),
        };
        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(input_value_sat),
            script_pubkey: input_script,
        });
        general_purpose::STANDARD.encode(psbt.serialize())
    }

    fn local_session() -> SpliceSessionV1 {
        SpliceSessionV1::new(
            SpliceOrigin::LocalInitiator,
            "02".repeat(33),
            "33".repeat(32),
            "44".repeat(32),
            OldSpliceState {
                funding_outpoint: outpoint(&"55".repeat(32), 0),
                channel_value_sat: 1_000_000,
                local_balance_sat: 600_000,
            },
            Some(50_000),
            FeePolicy {
                feerate_per_kw: Some(253),
                force_feerate: Some(false),
            },
            1,
        )
    }

    #[test]
    fn resolves_canonical_node_channel_id_from_old_funding_outpoint() {
        let mut state = State::new();
        let node_id_hex = "02".repeat(33);
        let channel_id = format!("{}{}", node_id_hex, "03".repeat(41));
        let funding_txid = "11".repeat(32);
        state
            .insert_channel(&channel_id, channel_entry(&funding_txid, 7))
            .unwrap();

        let resolved = state
            .node_channel_id_for_funding_outpoint(
                &node_id_hex,
                &FundingOutpoint {
                    txid: funding_txid,
                    vout: 7,
                },
            )
            .unwrap();

        assert_eq!(resolved.as_deref(), Some(channel_id.as_str()));
    }

    #[test]
    fn rejects_ambiguous_node_channel_id_for_funding_outpoint() {
        let mut state = State::new();
        let node_id_hex = "02".repeat(33);
        let funding_txid = "11".repeat(32);
        for suffix in ["03", "04"] {
            let channel_id = format!("{}{}", node_id_hex, suffix.repeat(41));
            state
                .insert_channel(&channel_id, channel_entry(&funding_txid, 7))
                .unwrap();
        }

        let error = state
            .node_channel_id_for_funding_outpoint(
                &node_id_hex,
                &FundingOutpoint {
                    txid: funding_txid,
                    vout: 7,
                },
            )
            .unwrap_err();

        assert!(error.to_string().contains("multiple channels"));
    }

    #[test]
    fn record_local_splice_intent_persists_facts_and_fingerprint() {
        let mut state = State::new();
        let fingerprint = "77".repeat(32);

        state
            .record_local_splice_intent(LocalSpliceIntent {
                node_id_hex: "02".repeat(33),
                channel_id_hex: "33".repeat(32),
                node_channel_id_hex: "44".repeat(32),
                old: OldSpliceState {
                    funding_outpoint: outpoint(&"55".repeat(32), 0),
                    channel_value_sat: 1_000_000,
                    local_balance_sat: 600_000,
                },
                authorized_relative_amount_sat: 50_000,
                fee_policy: FeePolicy {
                    feerate_per_kw: Some(253),
                    force_feerate: Some(false),
                },
                initial_psbt_fingerprint: fingerprint.clone(),
                initial_psbt_input_outpoints: vec![outpoint(&"11".repeat(32), 7)],
                timestamp_ms: 1,
            })
            .unwrap();

        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.origin, SpliceOrigin::LocalInitiator);
        assert_eq!(session.phase, SplicePhase::Negotiating);
        assert_eq!(session.intent.authorized_relative_amount_sat, Some(50_000));
        assert_eq!(session.intent.fee_policy.feerate_per_kw, Some(253));
        assert_eq!(
            session.psbt.candidate_fingerprint.as_deref(),
            Some(fingerprint.as_str())
        );
        let wallet_context = state
            .get_psbt_context(&fingerprint)
            .unwrap()
            .expect("splice candidate creates a linked PSBT context");
        assert_eq!(
            wallet_context.linked_node_channel_id_hex.as_deref(),
            Some("4444444444444444444444444444444444444444444444444444444444444444")
        );

        let session_value = serde_json::to_value(session).unwrap();
        assert!(session_value.get("auth").is_none());
        assert!(session_value.get("request_history").is_none());
    }

    #[test]
    fn wallet_inputs_psbt_values_and_reservations() {
        let prev_txid = "11".repeat(32);
        let psbt = psbt_fixture(
            &prev_txid,
            7,
            55_000,
            vec![(50_000, "00142222222222222222222222222222222222222222")],
        );

        let wallet_inputs = wallet_inputs_from_psbt(
            &psbt,
            &[WalletInputReservation {
                txid: "11".repeat(32),
                vout: 7,
                reserved_to_block: Some(42),
            }],
        )
        .unwrap();
        assert_eq!(wallet_inputs.len(), 1);
        assert_eq!(wallet_inputs[0].txid, prev_txid);
        assert_eq!(wallet_inputs[0].vout, 7);
        assert_eq!(wallet_inputs[0].value_sat, 55_000);
        assert_eq!(wallet_inputs[0].reserved_to_block, Some(42));
    }

    #[test]
    fn psbt_v2_and_candidate_facts_use_the_adapter() {
        let psbt = parse_base64_psbt(CLN_PSBT_V2).unwrap();
        assert_eq!(
            psbt.input_outpoints,
            vec![outpoint(
                "c85f81844094f9f0eec1e41f8d63e0a99e9f73dc725d7319871c9c4121d90a0b",
                0,
            )]
        );

        let candidate = candidate_funding_facts_from_psbt(
            CLN_PSBT_V2,
            &"99".repeat(32),
            0,
            &psbt.input_outpoints[0],
        )
        .unwrap();
        assert_eq!(candidate.value_sat, 800_000_000);
        assert_eq!(candidate.sign_splice_tx_input_index, 0);
        assert_eq!(
            candidate.script_pubkey_hash,
            sha256::digest(hex::decode("0014c430f64c4756da310dbd1a085572ef299926272c").unwrap())
        );
    }

    #[test]
    fn records_fundpsbt_response_and_signpsbt_intent_by_fingerprint() {
        let mut state = State::new();
        let serialized_psbt = psbt_fixture(
            &"11".repeat(32),
            0,
            25_000,
            vec![(20_000, "00143333333333333333333333333333333333333333")],
        );
        let psbt = parse_base64_psbt(&serialized_psbt).unwrap();
        let wallet_inputs = wallet_inputs_from_psbt(&serialized_psbt, &[]).unwrap();

        state
            .record_fundpsbt_response(FundPsbtResponseFacts {
                psbt_fingerprint: psbt.fingerprint.clone(),
                wallet_inputs,
                timestamp_ms: 2,
            })
            .unwrap();
        state
            .record_signpsbt_intent(SignPsbtIntentFacts {
                psbt_fingerprint: psbt.fingerprint.clone(),
                signonly: vec![0],
                timestamp_ms: 3,
            })
            .unwrap();

        let context = state.get_psbt_context(&psbt.fingerprint).unwrap().unwrap();
        assert_eq!(context.signonly, vec![0]);
        assert_eq!(context.wallet_inputs[0].value_sat, 25_000);
        let value = serde_json::to_value(context).unwrap();
        assert!(value.get("fundpsbt_auth").is_none());
        assert!(value.get("signpsbt_auth").is_none());
    }

    #[test]
    fn signpsbt_intent_preserves_existing_splice_link() {
        let mut state = State::new();
        state
            .record_local_splice_intent(LocalSpliceIntent {
                node_id_hex: "02".repeat(33),
                channel_id_hex: "33".repeat(32),
                node_channel_id_hex: "44".repeat(32),
                old: OldSpliceState {
                    funding_outpoint: outpoint(&"55".repeat(32), 0),
                    channel_value_sat: 1_000_000,
                    local_balance_sat: 600_000,
                },
                authorized_relative_amount_sat: 50_000,
                fee_policy: FeePolicy::default(),
                initial_psbt_fingerprint: "aa".repeat(32),
                initial_psbt_input_outpoints: vec![],
                timestamp_ms: 1,
            })
            .unwrap();

        state
            .record_signpsbt_intent(SignPsbtIntentFacts {
                psbt_fingerprint: "aa".repeat(32),
                signonly: vec![0],
                timestamp_ms: 2,
            })
            .unwrap();

        let context = state.get_psbt_context(&"aa".repeat(32)).unwrap().unwrap();
        assert_eq!(context.signonly, vec![0]);
        assert_eq!(
            context.linked_node_channel_id_hex.as_deref(),
            Some("4444444444444444444444444444444444444444444444444444444444444444")
        );
    }

    #[test]
    fn splice_candidate_inherits_wallet_inputs_from_initial_psbt() {
        let mut state = State::new();
        let source_fingerprint = "aa".repeat(32);
        let candidate_fingerprint = "bb".repeat(32);
        let updated_fingerprint = "cc".repeat(32);
        let wallet_outpoint = outpoint(&"11".repeat(32), 7);
        state
            .record_fundpsbt_response(FundPsbtResponseFacts {
                psbt_fingerprint: source_fingerprint.clone(),
                wallet_inputs: vec![WalletInput {
                    txid: wallet_outpoint.txid.clone(),
                    vout: wallet_outpoint.vout,
                    value_sat: 25_000,
                    reserved_to_block: Some(100),
                }],
                timestamp_ms: 1,
            })
            .unwrap();
        state
            .record_local_splice_intent(LocalSpliceIntent {
                node_id_hex: "02".repeat(33),
                channel_id_hex: "33".repeat(32),
                node_channel_id_hex: "44".repeat(32),
                old: OldSpliceState {
                    funding_outpoint: outpoint(&"55".repeat(32), 0),
                    channel_value_sat: 1_000_000,
                    local_balance_sat: 600_000,
                },
                authorized_relative_amount_sat: 50_000,
                fee_policy: FeePolicy::default(),
                initial_psbt_fingerprint: candidate_fingerprint.clone(),
                initial_psbt_input_outpoints: vec![wallet_outpoint.clone()],
                timestamp_ms: 2,
            })
            .unwrap();

        state
            .inherit_splice_wallet_context(
                &source_fingerprint,
                &candidate_fingerprint,
                std::slice::from_ref(&wallet_outpoint),
                &"44".repeat(32),
                2,
            )
            .unwrap();

        let candidate = state
            .get_psbt_context(&candidate_fingerprint)
            .unwrap()
            .unwrap();
        assert_eq!(candidate.wallet_inputs.len(), 1);
        assert_eq!(candidate.wallet_inputs[0].reserved_to_block, Some(100));

        state
            .record_splice_update_response(SpliceUpdateResponseFacts {
                node_channel_id_hex: "44".repeat(32),
                psbt_fingerprint: updated_fingerprint.clone(),
                psbt_input_outpoints: vec![wallet_outpoint],
                commitments_secured: false,
                signatures_secured: None,
                timestamp_ms: 3,
            })
            .unwrap();

        let updated = state
            .get_psbt_context(&updated_fingerprint)
            .unwrap()
            .unwrap();
        assert_eq!(updated.wallet_inputs.len(), 1);
        assert_eq!(updated.wallet_inputs[0].reserved_to_block, Some(100));
    }

    #[test]
    fn records_splice_update_and_signed_response_phase_facts() {
        let mut state = State::new();
        state.create_local_splice_session(local_session()).unwrap();
        let old_txid = "55".repeat(32);
        let serialized_psbt = psbt_fixture(
            &old_txid,
            0,
            1_000_000,
            vec![(1_050_000, "00144444444444444444444444444444444444444444")],
        );
        let psbt = parse_base64_psbt(&serialized_psbt).unwrap();

        state
            .record_splice_update_response(SpliceUpdateResponseFacts {
                node_channel_id_hex: "44".repeat(32),
                psbt_fingerprint: psbt.fingerprint.clone(),
                psbt_input_outpoints: psbt.input_outpoints.clone(),
                commitments_secured: true,
                signatures_secured: Some(false),
                timestamp_ms: 2,
            })
            .unwrap();
        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.phase, SplicePhase::CommitmentsSecured);
        assert_eq!(
            session.psbt.frozen_fingerprint.as_deref(),
            Some(psbt.fingerprint.as_str())
        );
        assert!(session.cand.funding_outpoint.is_none());

        let candidate = candidate_funding_facts_from_psbt(
            &serialized_psbt,
            &"99".repeat(32),
            0,
            &session.old.funding_outpoint,
        )
        .unwrap();
        state
            .record_splice_signed_response(SpliceSignedResponseFacts {
                node_channel_id_hex: "44".repeat(32),
                psbt_fingerprint: psbt.fingerprint,
                psbt_input_outpoints: psbt.input_outpoints,
                candidate: Some(candidate),
                timestamp_ms: 3,
            })
            .unwrap();

        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.phase, SplicePhase::PendingLock);
        assert_eq!(
            session.cand.funding_outpoint.as_ref().unwrap().txid,
            "99".repeat(32)
        );
        assert!(state
            .get_splice_by_outpoint(&"99".repeat(32), 0)
            .unwrap()
            .is_some());
    }

    #[test]
    fn session_schema_omits_unverifiable_rpc_auth() {
        let mut session = local_session();
        session.psbt.candidate_fingerprint = Some("candidate".to_string());
        session.psbt.frozen_fingerprint = Some("frozen".to_string());

        let value = serde_json::to_value(&session).unwrap();

        assert_eq!(value["schema"], json!("SpliceSessionV1"));
        assert!(value.get("old").is_some());
        assert!(value.get("auth").is_none());
        assert!(value.get("intent").is_some());
        assert!(value.get("psbt").is_some());
        assert!(value.get("cand").is_some());
        assert!(value.get("delta").is_some());
        assert!(value.get("linked_wallet_psbt_fingerprints").is_some());
        assert!(value.get("request_history").is_none());
        assert!(value.get("old_funding_outpoint").is_none());
        assert!(value.get("splice_init_auth").is_none());
        assert!(value.get("candidate_funding_outpoint").is_none());
        assert_eq!(
            value["psbt"],
            json!({
                "candidate_fingerprint": "candidate",
                "frozen_fingerprint": "frozen",
            })
        );
        assert_eq!(
            serde_json::from_value::<SpliceSessionV1>(value).unwrap(),
            session
        );
    }

    #[test]
    fn outpoint_lookup_survives_restart_and_tombstone_rejects_stale_merge() {
        let mut state = State::new();
        let fingerprint = "aa".repeat(32);
        state.create_local_splice_session(local_session()).unwrap();
        state
            .update_splice_candidate(&"44".repeat(32), fingerprint.clone(), 2)
            .unwrap();
        state
            .put_splice_wallet_psbt_context(&fingerprint, SpliceWalletPsbtContextV1::new(vec![], 2))
            .unwrap();
        state
            .link_splice_wallet_psbt(&fingerprint, &"44".repeat(32), 2)
            .unwrap();
        state
            .freeze_splice_candidate(
                &"44".repeat(32),
                fingerprint.clone(),
                CandidateFundingFacts {
                    funding_outpoint: outpoint(&"ee".repeat(32), 2),
                    value_sat: 1_050_000,
                    script_pubkey_hash: "ff".repeat(32),
                    sign_splice_tx_input_index: 0,
                    remote_funding_key_hex: None,
                },
                3,
            )
            .unwrap();

        let stale_state = state.clone();
        let entries: Vec<SignerStateEntry> = state.clone().into();
        let restored = State::try_from(entries.as_slice()).unwrap();
        assert!(restored
            .get_splice_by_outpoint(&"ee".repeat(32), 2)
            .unwrap()
            .is_some());
        assert!(restored.get_psbt_context(&fingerprint).unwrap().is_some());

        state.tombstone_splice_session(&"44".repeat(32)).unwrap();
        state.merge(&stale_state).unwrap();

        assert!(state
            .get_splice_session(&"44".repeat(32))
            .unwrap()
            .is_none());
        assert!(state
            .get_splice_by_outpoint(&"ee".repeat(32), 2)
            .unwrap()
            .is_none());
        assert!(state.get_psbt_context(&fingerprint).unwrap().is_none());
    }
}
