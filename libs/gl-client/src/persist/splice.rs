use super::canonical::canonical_json_bytes;
use super::{State, StateEntry, CHANNEL_PREFIX};
use crate::bitcoin::consensus::encode::deserialize;
use crate::bitcoin::psbt::Psbt;
use crate::bitcoin::Transaction;
use anyhow::{anyhow, bail};
use base64::{engine::general_purpose, Engine as _};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

const SPLICE_SESSION_PREFIX: &str = "splices";
const SPLICE_OUTPOINT_PREFIX: &str = "splice_outpoints";
const SPLICE_WALLET_PSBT_PREFIX: &str = "splice_wallet_psbts";

fn splice_session_key(node_channel_id_hex: &str) -> String {
    format!("{SPLICE_SESSION_PREFIX}/{node_channel_id_hex}")
}

fn splice_outpoint_key(txid: &str, vout: u32) -> String {
    format!("{SPLICE_OUTPOINT_PREFIX}/{txid}:{vout}")
}

fn wallet_psbt_key(psbt_shape_hash: &str) -> String {
    format!("{SPLICE_WALLET_PSBT_PREFIX}/{psbt_shape_hash}")
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
pub enum WalletInputSource {
    FundPsbt,
    SignPsbt,
    Unknown,
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
pub struct NormalizedRpcAuth {
    pub schema: String,
    pub schema_version: u16,
    pub uri: String,
    pub request_hash: String,
    pub caller_pubkey_hex: String,
    pub timestamp_ms: u64,
}

impl NormalizedRpcAuth {
    pub fn new(
        uri: String,
        request_hash: String,
        caller_pubkey_hex: String,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            schema: "NormalizedRpcAuth".to_string(),
            schema_version: 1,
            uri,
            request_hash,
            caller_pubkey_hex,
            timestamp_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsbtShapeInputV1 {
    pub prev_txid: String,
    pub prev_vout: u32,
    pub sequence: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsbtShapeOutputV1 {
    pub value_sat: u64,
    pub script_pubkey_hex: String,
    pub script_pubkey_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsbtShapeV1 {
    pub schema: String,
    pub version: i32,
    pub locktime: u32,
    pub inputs: Vec<PsbtShapeInputV1>,
    pub outputs: Vec<PsbtShapeOutputV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OldSpliceState {
    pub funding_outpoint: FundingOutpoint,
    pub channel_value_sat: u64,
    pub local_balance_sat: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpliceAuthState {
    pub splice_init_auth: Option<NormalizedRpcAuth>,
    pub latest_splice_update_auth: Option<NormalizedRpcAuth>,
    pub splice_signed_auth: Option<NormalizedRpcAuth>,
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
    pub splice_init_auth: NormalizedRpcAuth,
    pub authorized_relative_amount_sat: i64,
    pub fee_policy: FeePolicy,
    pub initial_psbt_shape_hash: Option<String>,
    pub initial_psbt_shape: Option<PsbtShapeV1>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplicePsbtState {
    pub candidate_psbt_shape_hash: Option<String>,
    pub candidate_psbt_shape: Option<PsbtShapeV1>,
    pub frozen_psbt_shape_hash: Option<String>,
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
pub struct SpliceSessionV1 {
    pub schema: String,
    pub schema_version: u16,
    pub origin: SpliceOrigin,
    pub phase: SplicePhase,
    pub node_id_hex: String,
    pub channel_id_hex: String,
    pub node_channel_id_hex: String,
    pub old: OldSpliceState,
    pub auth: SpliceAuthState,
    pub intent: SpliceIntentState,
    pub psbt: SplicePsbtState,
    pub cand: SpliceCandidateState,
    pub delta: SpliceDeltaState,
    pub linked_wallet_psbt_shape_hashes: Vec<String>,
    pub request_history: Vec<NormalizedRpcAuth>,
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
        splice_init_auth: Option<NormalizedRpcAuth>,
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
            auth: SpliceAuthState {
                splice_init_auth: splice_init_auth.clone(),
                latest_splice_update_auth: None,
                splice_signed_auth: None,
            },
            intent: SpliceIntentState {
                authorized_relative_amount_sat,
                fee_policy,
            },
            psbt: SplicePsbtState::default(),
            cand: SpliceCandidateState::default(),
            delta: SpliceDeltaState::default(),
            linked_wallet_psbt_shape_hashes: Vec::new(),
            request_history: splice_init_auth.into_iter().collect(),
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
    pub source: WalletInputSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpliceWalletPsbtContextV1 {
    pub schema: String,
    pub schema_version: u16,
    pub psbt_shape_hash: String,
    pub latest_psbt_shape: PsbtShapeV1,
    pub fundpsbt_auth: Option<NormalizedRpcAuth>,
    pub signpsbt_auth: Option<NormalizedRpcAuth>,
    pub signonly: Vec<u32>,
    pub wallet_inputs: Vec<WalletInput>,
    pub change_outnum: Option<u32>,
    pub linked_node_channel_id_hex: Option<String>,
    pub linked_splice_psbt_shape_hash: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl SpliceWalletPsbtContextV1 {
    pub fn new(
        psbt_shape_hash: String,
        latest_psbt_shape: PsbtShapeV1,
        fundpsbt_auth: Option<NormalizedRpcAuth>,
        signpsbt_auth: Option<NormalizedRpcAuth>,
        wallet_inputs: Vec<WalletInput>,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            schema: "SpliceWalletPsbtContextV1".to_string(),
            schema_version: 1,
            psbt_shape_hash,
            latest_psbt_shape,
            fundpsbt_auth,
            signpsbt_auth,
            signonly: Vec::new(),
            wallet_inputs,
            change_outnum: None,
            linked_node_channel_id_hex: None,
            linked_splice_psbt_shape_hash: None,
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
    pub psbt_shape_hash: String,
    pub psbt_shape: PsbtShapeV1,
    pub fundpsbt_auth: NormalizedRpcAuth,
    pub wallet_inputs: Vec<WalletInput>,
    pub change_outnum: Option<u32>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignPsbtResponseFacts {
    pub psbt_shape_hash: String,
    pub psbt_shape: PsbtShapeV1,
    pub signpsbt_auth: NormalizedRpcAuth,
    pub signonly: Vec<u32>,
    pub timestamp_ms: u64,
}

pub type SignPsbtIntentFacts = SignPsbtResponseFacts;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceUpdateResponseFacts {
    pub node_channel_id_hex: String,
    pub psbt_shape_hash: String,
    pub psbt_shape: PsbtShapeV1,
    pub splice_update_auth: NormalizedRpcAuth,
    pub commitments_secured: bool,
    pub signatures_secured: Option<bool>,
    pub timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceSignedResponseFacts {
    pub node_channel_id_hex: String,
    pub psbt_shape_hash: String,
    pub psbt_shape: PsbtShapeV1,
    pub splice_signed_auth: NormalizedRpcAuth,
    pub candidate: Option<CandidateFundingFacts>,
    pub timestamp_ms: u64,
}

pub(crate) fn transaction_shape(tx: &Transaction) -> PsbtShapeV1 {
    PsbtShapeV1 {
        schema: "PsbtShapeV1".to_string(),
        version: tx.version.0,
        locktime: tx.lock_time.to_consensus_u32(),
        inputs: tx
            .input
            .iter()
            .map(|input| PsbtShapeInputV1 {
                prev_txid: input.previous_output.txid.to_string(),
                prev_vout: input.previous_output.vout,
                sequence: input.sequence.to_consensus_u32(),
            })
            .collect(),
        outputs: tx
            .output
            .iter()
            .map(|output| {
                let script_bytes = output.script_pubkey.as_bytes();
                PsbtShapeOutputV1 {
                    value_sat: output.value.to_sat(),
                    script_pubkey_hex: hex::encode(script_bytes),
                    script_pubkey_hash: sha256::digest(script_bytes),
                }
            })
            .collect(),
    }
}

pub fn psbt_shape_hash(shape: &PsbtShapeV1) -> anyhow::Result<String> {
    let value = serde_json::to_value(shape)
        .map_err(|e| anyhow!("failed to encode PSBT shape for hashing: {e}"))?;
    let bytes = canonical_json_bytes(&value)?;
    Ok(sha256::digest(bytes.as_slice()))
}

pub fn psbt_shape_from_base64(psbt: &str) -> anyhow::Result<(String, PsbtShapeV1)> {
    let raw = general_purpose::STANDARD
        .decode(psbt)
        .map_err(|e| anyhow!("failed to decode PSBT base64: {e}"))?;
    let psbt = Psbt::deserialize(&raw).map_err(|e| anyhow!("failed to parse PSBT: {e}"))?;
    psbt_shape_from_psbt(&psbt)
}

pub(crate) fn psbt_shape_from_psbt(psbt: &Psbt) -> anyhow::Result<(String, PsbtShapeV1)> {
    let shape = transaction_shape(&psbt.unsigned_tx);
    let hash = psbt_shape_hash(&shape)?;
    Ok((hash, shape))
}

fn parse_psbt(psbt: &str) -> anyhow::Result<Psbt> {
    let raw = general_purpose::STANDARD
        .decode(psbt)
        .map_err(|e| anyhow!("failed to decode PSBT base64: {e}"))?;
    Psbt::deserialize(&raw).map_err(|e| anyhow!("failed to parse PSBT: {e}"))
}

fn psbt_input_value_sat(psbt: &Psbt, input_index: usize) -> anyhow::Result<u64> {
    let input = psbt
        .inputs
        .get(input_index)
        .ok_or_else(|| anyhow!("missing PSBT input {}", input_index))?;
    if let Some(txout) = &input.witness_utxo {
        return Ok(txout.value.to_sat());
    }

    let Some(prev_tx) = &input.non_witness_utxo else {
        bail!(
            "PSBT input {} has no witness_utxo or non_witness_utxo",
            input_index
        );
    };
    let vout = psbt.unsigned_tx.input[input_index].previous_output.vout as usize;
    prev_tx
        .output
        .get(vout)
        .map(|txout| txout.value.to_sat())
        .ok_or_else(|| anyhow!("PSBT input {} non_witness_utxo missing vout", input_index))
}

pub fn wallet_inputs_from_psbt(
    psbt: &str,
    reservations: &[WalletInputReservation],
    source: WalletInputSource,
) -> anyhow::Result<Vec<WalletInput>> {
    let psbt = parse_psbt(psbt)?;
    psbt.unsigned_tx
        .input
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let txid = input.previous_output.txid.to_string();
            let vout = input.previous_output.vout;
            let reservation = reservations
                .iter()
                .find(|reservation| reservation.txid == txid && reservation.vout == vout);
            Ok(WalletInput {
                txid,
                vout,
                value_sat: psbt_input_value_sat(&psbt, index)?,
                reserved_to_block: reservation
                    .and_then(|reservation| reservation.reserved_to_block),
                source: source.clone(),
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
    let psbt = parse_psbt(psbt)?;
    let old_input_index = psbt
        .unsigned_tx
        .input
        .iter()
        .position(|input| {
            input.previous_output.txid.to_string() == old_funding_outpoint.txid
                && input.previous_output.vout == old_funding_outpoint.vout
        })
        .ok_or_else(|| anyhow!("splice PSBT does not spend old funding outpoint"))?;
    let output = psbt
        .unsigned_tx
        .output
        .get(funding_vout as usize)
        .ok_or_else(|| anyhow!("splice funding output index {} is missing", funding_vout))?;

    Ok(CandidateFundingFacts {
        funding_outpoint: FundingOutpoint {
            txid: funding_txid.to_string(),
            vout: funding_vout,
        },
        value_sat: output.value.to_sat(),
        script_pubkey_hash: sha256::digest(output.script_pubkey.as_bytes()),
        sign_splice_tx_input_index: old_input_index as u32,
        remote_funding_key_hex: None,
    })
}

pub fn candidate_funding_facts_from_tx(
    tx: &[u8],
    funding_txid: &str,
    funding_vout: u32,
    sign_splice_tx_input_index: u32,
) -> anyhow::Result<CandidateFundingFacts> {
    let tx: Transaction =
        deserialize(tx).map_err(|e| anyhow!("failed to parse splice transaction: {e}"))?;
    let output = tx
        .output
        .get(funding_vout as usize)
        .ok_or_else(|| anyhow!("splice funding output index {} is missing", funding_vout))?;
    Ok(CandidateFundingFacts {
        funding_outpoint: FundingOutpoint {
            txid: funding_txid.to_string(),
            vout: funding_vout,
        },
        value_sat: output.value.to_sat(),
        script_pubkey_hash: sha256::digest(output.script_pubkey.as_bytes()),
        sign_splice_tx_input_index,
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

    fn link_splice_shape_context(
        &mut self,
        session: &mut SpliceSessionV1,
        psbt_shape_hash: &str,
        psbt_shape: &PsbtShapeV1,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let source_shape_hashes = session.linked_wallet_psbt_shape_hashes.clone();
        let mut context = self
            .get_splice_wallet_psbt_context(psbt_shape_hash)?
            .unwrap_or_else(|| {
                SpliceWalletPsbtContextV1::new(
                    psbt_shape_hash.to_string(),
                    psbt_shape.clone(),
                    None,
                    None,
                    Vec::new(),
                    updated_at_ms,
                )
            });
        if let Some(linked_channel) = context.linked_node_channel_id_hex.as_deref() {
            if linked_channel != session.node_channel_id_hex {
                bail!(
                    "PSBT shape {} is already linked to splice channel {}",
                    psbt_shape_hash,
                    linked_channel
                );
            }
        }

        context.latest_psbt_shape = psbt_shape.clone();
        context.linked_node_channel_id_hex = Some(session.node_channel_id_hex.clone());
        context.linked_splice_psbt_shape_hash = Some(psbt_shape_hash.to_string());
        context.updated_at_ms = updated_at_ms;
        if !session
            .linked_wallet_psbt_shape_hashes
            .iter()
            .any(|hash| hash == psbt_shape_hash)
        {
            session
                .linked_wallet_psbt_shape_hashes
                .push(psbt_shape_hash.to_string());
        }
        self.put_splice_wallet_psbt_context(context)?;
        for source_shape_hash in source_shape_hashes {
            self.inherit_splice_wallet_context(
                &source_shape_hash,
                psbt_shape_hash,
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
        if session.auth.splice_init_auth.is_none() {
            bail!("local splice sessions require splice_init_auth");
        }
        if session.intent.authorized_relative_amount_sat.is_none() {
            bail!("local splice sessions require authorized relative amount");
        }
        self.create_new_splice_session(session, SpliceOrigin::LocalInitiator)
    }

    pub fn record_local_splice_intent(&mut self, intent: LocalSpliceIntent) -> anyhow::Result<()> {
        match (
            intent.initial_psbt_shape_hash.as_ref(),
            intent.initial_psbt_shape.as_ref(),
        ) {
            (Some(_), Some(_)) | (None, None) => {}
            _ => bail!("initial PSBT shape hash and shape must be recorded together"),
        }

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
            session.auth.splice_init_auth = Some(intent.splice_init_auth.clone());
            session.intent.authorized_relative_amount_sat =
                Some(intent.authorized_relative_amount_sat);
            session.intent.fee_policy = intent.fee_policy;
            session.psbt.candidate_psbt_shape_hash = intent.initial_psbt_shape_hash;
            session.psbt.candidate_psbt_shape = intent.initial_psbt_shape;
            session.request_history.push(intent.splice_init_auth);
            session.updated_at_ms = intent.timestamp_ms;
            if let (Some(hash), Some(shape)) = (
                session.psbt.candidate_psbt_shape_hash.clone(),
                session.psbt.candidate_psbt_shape.clone(),
            ) {
                self.link_splice_shape_context(&mut session, &hash, &shape, intent.timestamp_ms)?;
            }
            return self.put_splice_session(&session);
        }

        let mut session = SpliceSessionV1::new(
            SpliceOrigin::LocalInitiator,
            intent.node_id_hex,
            intent.channel_id_hex,
            intent.node_channel_id_hex,
            intent.old,
            Some(intent.splice_init_auth),
            Some(intent.authorized_relative_amount_sat),
            intent.fee_policy,
            intent.timestamp_ms,
        );
        session.psbt.candidate_psbt_shape_hash = intent.initial_psbt_shape_hash;
        session.psbt.candidate_psbt_shape = intent.initial_psbt_shape;
        if let (Some(hash), Some(shape)) = (
            session.psbt.candidate_psbt_shape_hash.clone(),
            session.psbt.candidate_psbt_shape.clone(),
        ) {
            self.link_splice_shape_context(&mut session, &hash, &shape, intent.timestamp_ms)?;
        }
        self.create_local_splice_session(session)
    }

    pub fn record_fundpsbt_response(&mut self, facts: FundPsbtResponseFacts) -> anyhow::Result<()> {
        let existing = self.get_splice_wallet_psbt_context(&facts.psbt_shape_hash)?;
        let mut context = existing.unwrap_or_else(|| {
            SpliceWalletPsbtContextV1::new(
                facts.psbt_shape_hash.clone(),
                facts.psbt_shape.clone(),
                None,
                None,
                Vec::new(),
                facts.timestamp_ms,
            )
        });
        context.latest_psbt_shape = facts.psbt_shape;
        context.fundpsbt_auth = Some(facts.fundpsbt_auth);
        context.wallet_inputs = facts.wallet_inputs;
        context.change_outnum = facts.change_outnum;
        context.updated_at_ms = facts.timestamp_ms;
        self.put_splice_wallet_psbt_context(context)
    }

    pub fn record_signpsbt_response(&mut self, facts: SignPsbtResponseFacts) -> anyhow::Result<()> {
        let existing = self.get_splice_wallet_psbt_context(&facts.psbt_shape_hash)?;
        let mut context = existing.unwrap_or_else(|| {
            SpliceWalletPsbtContextV1::new(
                facts.psbt_shape_hash.clone(),
                facts.psbt_shape.clone(),
                None,
                None,
                Vec::new(),
                facts.timestamp_ms,
            )
        });
        context.latest_psbt_shape = facts.psbt_shape;
        context.signpsbt_auth = Some(facts.signpsbt_auth);
        context.signonly = facts.signonly;
        context.updated_at_ms = facts.timestamp_ms;
        self.put_splice_wallet_psbt_context(context)
    }

    pub fn record_signpsbt_intent(&mut self, facts: SignPsbtIntentFacts) -> anyhow::Result<()> {
        self.record_signpsbt_response(facts)
    }

    pub fn record_splice_update_response(
        &mut self,
        facts: SpliceUpdateResponseFacts,
    ) -> anyhow::Result<()> {
        let linked_psbt_shape_hash = facts.psbt_shape_hash.clone();
        let linked_psbt_shape = facts.psbt_shape.clone();
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

        session.auth.latest_splice_update_auth = Some(facts.splice_update_auth.clone());
        session.request_history.push(facts.splice_update_auth);

        if facts.commitments_secured {
            session.psbt.frozen_psbt_shape_hash = Some(facts.psbt_shape_hash.clone());
            session.psbt.candidate_psbt_shape_hash = Some(facts.psbt_shape_hash);
            session.psbt.candidate_psbt_shape = Some(facts.psbt_shape);
            session.phase = if facts.signatures_secured == Some(true) {
                SplicePhase::SignaturesExchanging
            } else {
                SplicePhase::CommitmentsSecured
            };
        } else {
            if session.phase != SplicePhase::Negotiating {
                bail!(
                    "candidate PSBT shape can only change while negotiating, current phase {:?}",
                    session.phase
                );
            }
            session.psbt.candidate_psbt_shape_hash = Some(facts.psbt_shape_hash);
            session.psbt.candidate_psbt_shape = Some(facts.psbt_shape);
        }

        session.updated_at_ms = facts.timestamp_ms;
        self.link_splice_shape_context(
            &mut session,
            &linked_psbt_shape_hash,
            &linked_psbt_shape,
            facts.timestamp_ms,
        )?;
        self.put_splice_session(&session)
    }

    pub fn record_splice_signed_response(
        &mut self,
        facts: SpliceSignedResponseFacts,
    ) -> anyhow::Result<()> {
        let linked_psbt_shape_hash = facts.psbt_shape_hash.clone();
        let linked_psbt_shape = facts.psbt_shape.clone();
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

        session.auth.splice_signed_auth = Some(facts.splice_signed_auth.clone());
        session.request_history.push(facts.splice_signed_auth);
        session.psbt.candidate_psbt_shape_hash = Some(facts.psbt_shape_hash.clone());
        session.psbt.candidate_psbt_shape = Some(facts.psbt_shape);
        if session.psbt.frozen_psbt_shape_hash.is_none() {
            session.psbt.frozen_psbt_shape_hash = Some(facts.psbt_shape_hash.clone());
        }
        session.phase = SplicePhase::SignaturesExchanging;
        session.updated_at_ms = facts.timestamp_ms;

        self.link_splice_shape_context(
            &mut session,
            &linked_psbt_shape_hash,
            &linked_psbt_shape,
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
        if session.auth.splice_init_auth.is_some() {
            bail!("peer-initiated splice sessions must not include splice_init_auth");
        }
        if session.intent.authorized_relative_amount_sat.is_some() {
            bail!("peer-initiated splice sessions must not include local relative amount intent");
        }
        self.create_new_splice_session(session, SpliceOrigin::PeerInitiated)
    }

    pub fn create_dev_splice_session(&mut self, session: SpliceSessionV1) -> anyhow::Result<()> {
        self.create_new_splice_session(session, SpliceOrigin::DevSpliceUnresolved)
    }

    pub fn update_splice_candidate_shape(
        &mut self,
        node_channel_id_hex: &str,
        psbt_shape_hash: String,
        psbt_shape: PsbtShapeV1,
        auth: NormalizedRpcAuth,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        if session.phase != SplicePhase::Negotiating {
            bail!(
                "candidate PSBT shape can only change while negotiating, current phase {:?}",
                session.phase
            );
        }
        session.auth.latest_splice_update_auth = Some(auth.clone());
        session.psbt.candidate_psbt_shape_hash = Some(psbt_shape_hash);
        session.psbt.candidate_psbt_shape = Some(psbt_shape);
        session.request_history.push(auth);
        session.updated_at_ms = updated_at_ms;
        self.put_splice_session(&session)
    }

    pub fn freeze_splice_candidate(
        &mut self,
        node_channel_id_hex: &str,
        frozen_psbt_shape_hash: String,
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
        session.psbt.frozen_psbt_shape_hash = Some(frozen_psbt_shape_hash);
        session.cand = candidate.into();
        session.updated_at_ms = updated_at_ms;
        self.put_splice_session(&session)?;
        self.put_splice_outpoint_index(&candidate_outpoint, &index)
    }

    pub fn mark_splice_signatures_exchanging(
        &mut self,
        node_channel_id_hex: &str,
        auth: NormalizedRpcAuth,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        if !matches!(
            session.phase,
            SplicePhase::CommitmentsSecured | SplicePhase::SignaturesExchanging
        ) {
            bail!(
                "splice_signed auth requires commitments secured, current phase {:?}",
                session.phase
            );
        }
        session.phase = SplicePhase::SignaturesExchanging;
        session.auth.splice_signed_auth = Some(auth.clone());
        session.request_history.push(auth);
        session.updated_at_ms = updated_at_ms;
        self.put_splice_session(&session)
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
        context: SpliceWalletPsbtContextV1,
    ) -> anyhow::Result<()> {
        self.put_splice(&wallet_psbt_key(&context.psbt_shape_hash), &context)
    }

    pub fn get_splice_wallet_psbt_context(
        &self,
        psbt_shape_hash: &str,
    ) -> anyhow::Result<Option<SpliceWalletPsbtContextV1>> {
        self.get_splice(&wallet_psbt_key(psbt_shape_hash))
    }

    pub fn link_splice_wallet_psbt(
        &mut self,
        psbt_shape_hash: &str,
        node_channel_id_hex: &str,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        let mut session = self
            .get_splice_session(node_channel_id_hex)?
            .ok_or_else(|| anyhow!("missing splice session for channel {}", node_channel_id_hex))?;
        let mut context = self
            .get_splice_wallet_psbt_context(psbt_shape_hash)?
            .ok_or_else(|| anyhow!("missing wallet PSBT context {}", psbt_shape_hash))?;
        let shape_matches_splice = session.psbt.candidate_psbt_shape_hash.as_deref()
            == Some(psbt_shape_hash)
            || session.psbt.frozen_psbt_shape_hash.as_deref() == Some(psbt_shape_hash);
        if !shape_matches_splice {
            bail!(
                "wallet PSBT shape {} does not match splice candidate for channel {}",
                psbt_shape_hash,
                node_channel_id_hex
            );
        }

        context.linked_node_channel_id_hex = Some(node_channel_id_hex.to_string());
        context.linked_splice_psbt_shape_hash = Some(psbt_shape_hash.to_string());
        context.updated_at_ms = updated_at_ms;
        if !session
            .linked_wallet_psbt_shape_hashes
            .iter()
            .any(|hash| hash == psbt_shape_hash)
        {
            session
                .linked_wallet_psbt_shape_hashes
                .push(psbt_shape_hash.to_string());
        }
        session.updated_at_ms = updated_at_ms;
        self.put_splice_wallet_psbt_context(context)?;
        self.put_splice_session(&session)
    }

    pub fn inherit_splice_wallet_context(
        &mut self,
        source_psbt_shape_hash: &str,
        candidate_psbt_shape_hash: &str,
        node_channel_id_hex: &str,
        updated_at_ms: u64,
    ) -> anyhow::Result<()> {
        if source_psbt_shape_hash == candidate_psbt_shape_hash {
            return Ok(());
        }
        let Some(source) = self.get_splice_wallet_psbt_context(source_psbt_shape_hash)? else {
            return Ok(());
        };
        let mut candidate = self
            .get_splice_wallet_psbt_context(candidate_psbt_shape_hash)?
            .ok_or_else(|| {
                anyhow!(
                    "missing splice candidate PSBT context {}",
                    candidate_psbt_shape_hash
                )
            })?;
        if candidate.linked_node_channel_id_hex.as_deref() != Some(node_channel_id_hex) {
            bail!(
                "splice candidate PSBT context {} is not linked to channel {}",
                candidate_psbt_shape_hash,
                node_channel_id_hex
            );
        }

        if candidate.fundpsbt_auth.is_none() {
            candidate.fundpsbt_auth = source.fundpsbt_auth;
        }
        for wallet_input in source.wallet_inputs {
            let remains_in_candidate = candidate.latest_psbt_shape.inputs.iter().any(|input| {
                input.prev_txid == wallet_input.txid && input.prev_vout == wallet_input.vout
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
        self.put_splice_wallet_psbt_context(candidate)
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
                .linked_wallet_psbt_shape_hashes
                .iter()
                .map(|hash| wallet_psbt_key(hash)),
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
    use lightning_signer::channel::{ChannelSetup, CommitmentType};
    use lightning_signer::policy::validator::EnforcementState;
    use serde_json::json;
    use std::str::FromStr;

    fn auth(uri: &str, request_hash: &str, timestamp_ms: u64) -> NormalizedRpcAuth {
        NormalizedRpcAuth::new(
            uri.to_string(),
            request_hash.to_string(),
            "02".repeat(33),
            timestamp_ms,
        )
    }

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

    fn shape(hash_suffix: &str) -> PsbtShapeV1 {
        PsbtShapeV1 {
            schema: "PsbtShapeV1".to_string(),
            version: 2,
            locktime: 0,
            inputs: vec![PsbtShapeInputV1 {
                prev_txid: format!("{}{}", "11".repeat(31), hash_suffix),
                prev_vout: 0,
                sequence: 0xffff_ffff,
            }],
            outputs: vec![PsbtShapeOutputV1 {
                value_sat: 1000,
                script_pubkey_hex: "0014".to_string(),
                script_pubkey_hash: "aa".repeat(32),
            }],
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
            Some(auth("/cln.Node/SpliceInit", &"66".repeat(32), 1)),
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
    fn record_local_splice_intent_keeps_authority_out_of_psbt_shape() {
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
                splice_init_auth: auth("/cln.Node/SpliceInit", &"66".repeat(32), 1),
                authorized_relative_amount_sat: 50_000,
                fee_policy: FeePolicy {
                    feerate_per_kw: Some(253),
                    force_feerate: Some(false),
                },
                initial_psbt_shape_hash: Some("shape-init".to_string()),
                initial_psbt_shape: Some(shape("04")),
                timestamp_ms: 1,
            })
            .unwrap();

        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.origin, SpliceOrigin::LocalInitiator);
        assert_eq!(session.phase, SplicePhase::Negotiating);
        assert_eq!(session.intent.authorized_relative_amount_sat, Some(50_000));
        assert_eq!(session.intent.fee_policy.feerate_per_kw, Some(253));
        assert_eq!(
            session.auth.splice_init_auth.as_ref().unwrap().uri,
            "/cln.Node/SpliceInit"
        );
        assert_eq!(
            session.psbt.candidate_psbt_shape_hash.as_deref(),
            Some("shape-init")
        );
        let wallet_context = state
            .get_splice_wallet_psbt_context("shape-init")
            .unwrap()
            .expect("splice candidate creates a linked PSBT context");
        assert_eq!(
            wallet_context.linked_node_channel_id_hex.as_deref(),
            Some("4444444444444444444444444444444444444444444444444444444444444444")
        );

        let psbt_shape_value =
            serde_json::to_value(session.psbt.candidate_psbt_shape.as_ref().unwrap()).unwrap();
        assert!(psbt_shape_value
            .get("authorized_relative_amount_sat")
            .is_none());
        assert!(psbt_shape_value.get("fee_policy").is_none());
        assert!(psbt_shape_value.get("splice_init_auth").is_none());
        assert!(psbt_shape_value.get("request_hash").is_none());
        assert!(psbt_shape_value.get("caller_pubkey_hex").is_none());
    }

    #[test]
    fn record_local_splice_intent_rejects_half_recorded_initial_psbt_shape() {
        let mut state = State::new();
        let mut intent = LocalSpliceIntent {
            node_id_hex: "02".repeat(33),
            channel_id_hex: "33".repeat(32),
            node_channel_id_hex: "44".repeat(32),
            old: OldSpliceState {
                funding_outpoint: outpoint(&"55".repeat(32), 0),
                channel_value_sat: 1_000_000,
                local_balance_sat: 600_000,
            },
            splice_init_auth: auth("/cln.Node/SpliceInit", &"66".repeat(32), 1),
            authorized_relative_amount_sat: 50_000,
            fee_policy: FeePolicy {
                feerate_per_kw: Some(253),
                force_feerate: Some(false),
            },
            initial_psbt_shape_hash: Some("shape-init".to_string()),
            initial_psbt_shape: None,
            timestamp_ms: 1,
        };

        let err = state
            .record_local_splice_intent(intent.clone())
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("initial PSBT shape hash and shape must be recorded together"));

        intent.initial_psbt_shape_hash = None;
        intent.initial_psbt_shape = Some(shape("05"));
        let err = state.record_local_splice_intent(intent).unwrap_err();
        assert!(err
            .to_string()
            .contains("initial PSBT shape hash and shape must be recorded together"));
    }

    #[test]
    fn psbt_shape_and_wallet_inputs_are_derived_from_psbt_without_authority() {
        let prev_txid = "11".repeat(32);
        let psbt = psbt_fixture(
            &prev_txid,
            7,
            55_000,
            vec![(50_000, "00142222222222222222222222222222222222222222")],
        );

        let (hash, parsed_shape) = psbt_shape_from_base64(&psbt).unwrap();
        assert_eq!(hash, psbt_shape_hash(&parsed_shape).unwrap());
        assert_eq!(parsed_shape.inputs[0].prev_txid, prev_txid);
        assert_eq!(parsed_shape.inputs[0].prev_vout, 7);
        assert_eq!(parsed_shape.outputs[0].value_sat, 50_000);

        let wallet_inputs = wallet_inputs_from_psbt(
            &psbt,
            &[WalletInputReservation {
                txid: "11".repeat(32),
                vout: 7,
                reserved_to_block: Some(42),
            }],
            WalletInputSource::FundPsbt,
        )
        .unwrap();
        assert_eq!(wallet_inputs.len(), 1);
        assert_eq!(wallet_inputs[0].value_sat, 55_000);
        assert_eq!(wallet_inputs[0].reserved_to_block, Some(42));

        let shape_value = serde_json::to_value(parsed_shape).unwrap();
        assert!(shape_value.get("splice_init_auth").is_none());
        assert!(shape_value.get("signpsbt_auth").is_none());
    }

    #[test]
    fn records_fundpsbt_and_signpsbt_response_facts_by_shape_hash() {
        let mut state = State::new();
        let psbt = psbt_fixture(
            &"11".repeat(32),
            0,
            25_000,
            vec![(20_000, "00143333333333333333333333333333333333333333")],
        );
        let (shape_hash, parsed_shape) = psbt_shape_from_base64(&psbt).unwrap();
        let wallet_inputs =
            wallet_inputs_from_psbt(&psbt, &[], WalletInputSource::FundPsbt).unwrap();

        state
            .record_fundpsbt_response(FundPsbtResponseFacts {
                psbt_shape_hash: shape_hash.clone(),
                psbt_shape: parsed_shape.clone(),
                fundpsbt_auth: auth("/cln.Node/FundPsbt", &"ab".repeat(32), 2),
                wallet_inputs,
                change_outnum: Some(0),
                timestamp_ms: 2,
            })
            .unwrap();
        state
            .record_signpsbt_response(SignPsbtResponseFacts {
                psbt_shape_hash: shape_hash.clone(),
                psbt_shape: parsed_shape,
                signpsbt_auth: auth("/cln.Node/SignPsbt", &"cd".repeat(32), 3),
                signonly: vec![0],
                timestamp_ms: 3,
            })
            .unwrap();

        let context = state
            .get_splice_wallet_psbt_context(&shape_hash)
            .unwrap()
            .unwrap();
        assert!(context.fundpsbt_auth.is_some());
        assert!(context.signpsbt_auth.is_some());
        assert_eq!(context.signonly, vec![0]);
        assert_eq!(context.change_outnum, Some(0));
        assert_eq!(context.wallet_inputs[0].value_sat, 25_000);
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
                splice_init_auth: auth("/cln.Node/SpliceInit", &"66".repeat(32), 1),
                authorized_relative_amount_sat: 50_000,
                fee_policy: FeePolicy::default(),
                initial_psbt_shape_hash: Some("shape-a".to_string()),
                initial_psbt_shape: Some(shape("06")),
                timestamp_ms: 1,
            })
            .unwrap();

        state
            .record_signpsbt_intent(SignPsbtIntentFacts {
                psbt_shape_hash: "shape-a".to_string(),
                psbt_shape: shape("06"),
                signpsbt_auth: auth("/cln.Node/SignPsbt", &"77".repeat(32), 2),
                signonly: vec![0],
                timestamp_ms: 2,
            })
            .unwrap();

        let context = state
            .get_splice_wallet_psbt_context("shape-a")
            .unwrap()
            .unwrap();
        assert!(context.signpsbt_auth.is_some());
        assert_eq!(context.signonly, vec![0]);
        assert_eq!(
            context.linked_node_channel_id_hex.as_deref(),
            Some("4444444444444444444444444444444444444444444444444444444444444444")
        );
    }

    #[test]
    fn splice_candidate_inherits_wallet_inputs_from_initial_psbt() {
        let mut state = State::new();
        let source_shape = shape("07");
        let mut candidate_shape = source_shape.clone();
        candidate_shape.outputs.push(PsbtShapeOutputV1 {
            value_sat: 50_000,
            script_pubkey_hex: "0014".to_string(),
            script_pubkey_hash: "bb".repeat(32),
        });
        state
            .record_fundpsbt_response(FundPsbtResponseFacts {
                psbt_shape_hash: "source-shape".to_string(),
                psbt_shape: source_shape.clone(),
                fundpsbt_auth: auth("/cln.Node/FundPsbt", &"77".repeat(32), 1),
                wallet_inputs: vec![WalletInput {
                    txid: source_shape.inputs[0].prev_txid.clone(),
                    vout: source_shape.inputs[0].prev_vout,
                    value_sat: 25_000,
                    reserved_to_block: Some(100),
                    source: WalletInputSource::FundPsbt,
                }],
                change_outnum: None,
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
                splice_init_auth: auth("/cln.Node/SpliceInit", &"66".repeat(32), 2),
                authorized_relative_amount_sat: 50_000,
                fee_policy: FeePolicy::default(),
                initial_psbt_shape_hash: Some("candidate-shape".to_string()),
                initial_psbt_shape: Some(candidate_shape),
                timestamp_ms: 2,
            })
            .unwrap();

        state
            .inherit_splice_wallet_context("source-shape", "candidate-shape", &"44".repeat(32), 2)
            .unwrap();

        let candidate = state
            .get_splice_wallet_psbt_context("candidate-shape")
            .unwrap()
            .unwrap();
        assert!(candidate.fundpsbt_auth.is_some());
        assert_eq!(candidate.wallet_inputs.len(), 1);
        assert_eq!(candidate.wallet_inputs[0].reserved_to_block, Some(100));

        let mut updated_shape = candidate.latest_psbt_shape.clone();
        updated_shape.outputs[0].value_sat += 1;
        state
            .record_splice_update_response(SpliceUpdateResponseFacts {
                node_channel_id_hex: "44".repeat(32),
                psbt_shape_hash: "updated-shape".to_string(),
                psbt_shape: updated_shape,
                splice_update_auth: auth("/cln.Node/SpliceUpdate", &"88".repeat(32), 3),
                commitments_secured: false,
                signatures_secured: None,
                timestamp_ms: 3,
            })
            .unwrap();

        let updated = state
            .get_splice_wallet_psbt_context("updated-shape")
            .unwrap()
            .unwrap();
        assert!(updated.fundpsbt_auth.is_some());
        assert_eq!(updated.wallet_inputs.len(), 1);
        assert_eq!(updated.wallet_inputs[0].reserved_to_block, Some(100));
    }

    #[test]
    fn records_splice_update_and_signed_response_phase_facts() {
        let mut state = State::new();
        state.create_local_splice_session(local_session()).unwrap();
        let old_txid = "55".repeat(32);
        let psbt = psbt_fixture(
            &old_txid,
            0,
            1_000_000,
            vec![(1_050_000, "00144444444444444444444444444444444444444444")],
        );
        let (shape_hash, parsed_shape) = psbt_shape_from_base64(&psbt).unwrap();

        state
            .record_splice_update_response(SpliceUpdateResponseFacts {
                node_channel_id_hex: "44".repeat(32),
                psbt_shape_hash: shape_hash.clone(),
                psbt_shape: parsed_shape.clone(),
                splice_update_auth: auth("/cln.Node/SpliceUpdate", &"ef".repeat(32), 2),
                commitments_secured: true,
                signatures_secured: Some(false),
                timestamp_ms: 2,
            })
            .unwrap();
        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.phase, SplicePhase::CommitmentsSecured);
        assert_eq!(
            session.psbt.frozen_psbt_shape_hash.as_deref(),
            Some(shape_hash.as_str())
        );
        assert!(session.cand.funding_outpoint.is_none());

        let candidate = candidate_funding_facts_from_psbt(
            &psbt,
            &"99".repeat(32),
            0,
            &session.old.funding_outpoint,
        )
        .unwrap();
        state
            .record_splice_signed_response(SpliceSignedResponseFacts {
                node_channel_id_hex: "44".repeat(32),
                psbt_shape_hash: shape_hash,
                psbt_shape: parsed_shape,
                splice_signed_auth: auth("/cln.Node/SpliceSigned", &"12".repeat(32), 3),
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
    fn session_serializes_to_grouped_schema() {
        let session = local_session();

        let value = serde_json::to_value(session).unwrap();

        assert_eq!(value["schema"], json!("SpliceSessionV1"));
        assert!(value.get("old").is_some());
        assert!(value.get("auth").is_some());
        assert!(value.get("intent").is_some());
        assert!(value.get("psbt").is_some());
        assert!(value.get("cand").is_some());
        assert!(value.get("delta").is_some());
        assert!(value.get("linked_wallet_psbt_shape_hashes").is_some());
        assert!(value.get("old_funding_outpoint").is_none());
        assert!(value.get("splice_init_auth").is_none());
        assert!(value.get("candidate_funding_outpoint").is_none());
        assert_eq!(
            value["auth"]["splice_init_auth"]["uri"],
            json!("/cln.Node/SpliceInit")
        );
        assert!(value["auth"]["splice_init_auth"].get("request").is_none());
        assert!(value["auth"]["splice_init_auth"].get("payload").is_none());
    }

    #[test]
    fn local_session_updates_freezes_and_tombstones_related_keys() {
        let mut state = State::new();
        state.create_local_splice_session(local_session()).unwrap();

        state
            .update_splice_candidate_shape(
                &"44".repeat(32),
                "shape-a".to_string(),
                shape("01"),
                auth("/cln.Node/SpliceUpdate", &"77".repeat(32), 2),
                2,
            )
            .unwrap();
        let candidate = CandidateFundingFacts {
            funding_outpoint: outpoint(&"88".repeat(32), 1),
            value_sat: 1_050_000,
            script_pubkey_hash: "99".repeat(32),
            sign_splice_tx_input_index: 0,
            remote_funding_key_hex: Some("03".repeat(33)),
        };
        state
            .freeze_splice_candidate(&"44".repeat(32), "shape-a".to_string(), candidate, 3)
            .unwrap();

        let index_key = splice_outpoint_key(&"88".repeat(32), 1);
        let index = state.values.get(&index_key).unwrap();
        assert_eq!(
            index.value,
            json!({
                "schema": "SpliceOutpointIndexV1",
                "schema_version": 1,
                "splice_session_key": format!("splices/{}", "44".repeat(32)),
            })
        );

        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.phase, SplicePhase::CommitmentsSecured);
        assert_eq!(
            session.psbt.frozen_psbt_shape_hash.as_deref(),
            Some("shape-a")
        );

        state
            .mark_splice_signatures_exchanging(
                &"44".repeat(32),
                auth("/cln.Node/SpliceSigned", &"aa".repeat(32), 4),
                4,
            )
            .unwrap();
        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.phase, SplicePhase::SignaturesExchanging);

        state.mark_splice_pending_lock(&"44".repeat(32), 5).unwrap();
        let session = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(session.phase, SplicePhase::PendingLock);

        let by_outpoint = state
            .get_splice_by_outpoint(&"88".repeat(32), 1)
            .unwrap()
            .unwrap();
        assert_eq!(by_outpoint.node_channel_id_hex, "44".repeat(32));

        state.tombstone_splice_session(&"44".repeat(32)).unwrap();
        assert!(state
            .get_splice_session(&"44".repeat(32))
            .unwrap()
            .is_none());
        assert!(state
            .get_splice_by_outpoint(&"88".repeat(32), 1)
            .unwrap()
            .is_none());
    }

    #[test]
    fn peer_and_dev_sessions_store_unresolved_origins_without_local_auth() {
        let mut state = State::new();
        let mut peer = local_session();
        peer.origin = SpliceOrigin::PeerInitiated;
        peer.auth.splice_init_auth = None;
        peer.intent.authorized_relative_amount_sat = None;
        peer.delta.computed = false;
        peer.delta.no_local_loss = false;
        state.create_peer_splice_session(peer.clone()).unwrap();

        let mut dev = local_session();
        dev.origin = SpliceOrigin::DevSpliceUnresolved;
        dev.node_channel_id_hex = "45".repeat(32);
        state.create_dev_splice_session(dev.clone()).unwrap();

        let stored_peer = state.get_splice_session(&"44".repeat(32)).unwrap().unwrap();
        assert_eq!(stored_peer.origin, SpliceOrigin::PeerInitiated);
        assert!(stored_peer.auth.splice_init_auth.is_none());
        assert!(stored_peer.intent.authorized_relative_amount_sat.is_none());

        let stored_dev = state.get_splice_session(&"45".repeat(32)).unwrap().unwrap();
        assert_eq!(stored_dev.origin, SpliceOrigin::DevSpliceUnresolved);
        assert_eq!(stored_dev.phase, SplicePhase::Negotiating);
    }

    #[test]
    fn wallet_context_links_by_shape_and_distinguishes_fundpsbt_from_signpsbt_auth() {
        let mut state = State::new();
        let mut session = local_session();
        session.psbt.candidate_psbt_shape_hash = Some("shape-a".to_string());
        state.create_local_splice_session(session).unwrap();

        let context = SpliceWalletPsbtContextV1::new(
            "shape-a".to_string(),
            shape("02"),
            Some(auth("/cln.Node/FundPsbt", &"aa".repeat(32), 2)),
            None,
            vec![WalletInput {
                txid: "bb".repeat(32),
                vout: 0,
                value_sat: 25_000,
                reserved_to_block: Some(100),
                source: WalletInputSource::FundPsbt,
            }],
            2,
        );
        assert!(context.signpsbt_auth.is_none());
        state.put_splice_wallet_psbt_context(context).unwrap();

        state
            .link_splice_wallet_psbt("shape-a", &"44".repeat(32), 3)
            .unwrap();

        let linked = state
            .get_splice_wallet_psbt_context("shape-a")
            .unwrap()
            .unwrap();
        let expected_channel = "44".repeat(32);
        assert_eq!(
            linked.linked_node_channel_id_hex.as_deref(),
            Some(expected_channel.as_str())
        );
        assert_eq!(
            linked.linked_splice_psbt_shape_hash.as_deref(),
            Some("shape-a")
        );

        let mut signed = linked;
        signed.signpsbt_auth = Some(auth("/cln.Node/SignPsbt", &"cc".repeat(32), 4));
        assert!(signed.signpsbt_auth.is_some());

        state.tombstone_splice_session(&"44".repeat(32)).unwrap();
        assert!(state
            .get_splice_wallet_psbt_context("shape-a")
            .unwrap()
            .is_none());
    }

    #[test]
    fn outpoint_lookup_survives_restart_and_tombstone_rejects_stale_merge() {
        let mut state = State::new();
        state.create_local_splice_session(local_session()).unwrap();
        state
            .update_splice_candidate_shape(
                &"44".repeat(32),
                "shape-a".to_string(),
                shape("03"),
                auth("/cln.Node/SpliceUpdate", &"dd".repeat(32), 2),
                2,
            )
            .unwrap();
        state
            .freeze_splice_candidate(
                &"44".repeat(32),
                "shape-a".to_string(),
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
    }
}
