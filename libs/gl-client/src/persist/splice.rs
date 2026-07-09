use super::{State, StateEntry};
use anyhow::{anyhow, bail};
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
pub(crate) enum SpliceOrigin {
    LocalInitiator,
    PeerInitiated,
    DevSpliceUnresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SplicePhase {
    Negotiating,
    CommitmentsSecured,
    SignaturesExchanging,
    PendingLock,
    Locked,
    Aborted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DeltaSource {
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
pub(crate) enum WalletInputSource {
    FundPsbt,
    SignPsbt,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SpliceTerminalReason {
    Locked,
    Aborted,
    ChannelDeleted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FundingOutpoint {
    pub(crate) txid: String,
    pub(crate) vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NormalizedRpcAuth {
    pub(crate) schema: String,
    pub(crate) schema_version: u16,
    pub(crate) uri: String,
    pub(crate) request_hash: String,
    pub(crate) caller_pubkey_hex: String,
    pub(crate) timestamp_ms: u64,
}

impl NormalizedRpcAuth {
    pub(crate) fn new(
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
pub(crate) struct PsbtShapeInputV1 {
    pub(crate) prev_txid: String,
    pub(crate) prev_vout: u32,
    pub(crate) sequence: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PsbtShapeOutputV1 {
    pub(crate) value_sat: u64,
    pub(crate) script_pubkey_hex: String,
    pub(crate) script_pubkey_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PsbtShapeV1 {
    pub(crate) schema: String,
    pub(crate) version: i32,
    pub(crate) locktime: u32,
    pub(crate) inputs: Vec<PsbtShapeInputV1>,
    pub(crate) outputs: Vec<PsbtShapeOutputV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OldSpliceState {
    pub(crate) funding_outpoint: FundingOutpoint,
    pub(crate) channel_value_sat: u64,
    pub(crate) local_balance_sat: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpliceAuthState {
    pub(crate) splice_init_auth: Option<NormalizedRpcAuth>,
    pub(crate) latest_splice_update_auth: Option<NormalizedRpcAuth>,
    pub(crate) splice_signed_auth: Option<NormalizedRpcAuth>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FeePolicy {
    pub(crate) feerate_per_kw: Option<u32>,
    pub(crate) force_feerate: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpliceIntentState {
    pub(crate) authorized_relative_amount_sat: Option<i64>,
    pub(crate) fee_policy: FeePolicy,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SplicePsbtState {
    pub(crate) candidate_psbt_shape_hash: Option<String>,
    pub(crate) candidate_psbt_shape: Option<PsbtShapeV1>,
    pub(crate) frozen_psbt_shape_hash: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpliceCandidateState {
    pub(crate) funding_outpoint: Option<FundingOutpoint>,
    pub(crate) value_sat: Option<u64>,
    pub(crate) script_pubkey_hash: Option<String>,
    pub(crate) sign_splice_tx_input_index: Option<u32>,
    pub(crate) remote_funding_key_hex: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CandidateFundingFacts {
    pub(crate) funding_outpoint: FundingOutpoint,
    pub(crate) value_sat: u64,
    pub(crate) script_pubkey_hash: String,
    pub(crate) sign_splice_tx_input_index: u32,
    pub(crate) remote_funding_key_hex: Option<String>,
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
pub(crate) struct SpliceDeltaState {
    pub(crate) computed: bool,
    pub(crate) channel_delta_sat: i64,
    pub(crate) wallet_input_delta_sat: i64,
    pub(crate) wallet_output_delta_sat: i64,
    pub(crate) fee_burden_sat: i64,
    pub(crate) no_local_loss: bool,
    pub(crate) source: DeltaSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SignerRequestRecord {
    pub(crate) request_type: String,
    pub(crate) request_hash: String,
    pub(crate) phase: SplicePhase,
    pub(crate) timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpliceSessionV1 {
    pub(crate) schema: String,
    pub(crate) schema_version: u16,
    pub(crate) origin: SpliceOrigin,
    pub(crate) phase: SplicePhase,
    pub(crate) node_id_hex: String,
    pub(crate) channel_id_hex: String,
    pub(crate) node_channel_id_hex: String,
    pub(crate) old: OldSpliceState,
    pub(crate) auth: SpliceAuthState,
    pub(crate) intent: SpliceIntentState,
    pub(crate) psbt: SplicePsbtState,
    pub(crate) cand: SpliceCandidateState,
    pub(crate) delta: SpliceDeltaState,
    pub(crate) linked_wallet_psbt_shape_hashes: Vec<String>,
    pub(crate) request_history: Vec<NormalizedRpcAuth>,
    pub(crate) signer_request_history: Vec<SignerRequestRecord>,
    pub(crate) created_at_ms: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) terminal_reason: Option<SpliceTerminalReason>,
}

impl SpliceSessionV1 {
    pub(crate) fn new(
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
pub(crate) struct SpliceOutpointIndexV1 {
    pub(crate) schema: String,
    pub(crate) schema_version: u16,
    pub(crate) splice_session_key: String,
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
pub(crate) struct WalletInput {
    pub(crate) txid: String,
    pub(crate) vout: u32,
    pub(crate) value_sat: u64,
    pub(crate) reserved_to_block: Option<u32>,
    pub(crate) source: WalletInputSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpliceWalletPsbtContextV1 {
    pub(crate) schema: String,
    pub(crate) schema_version: u16,
    pub(crate) psbt_shape_hash: String,
    pub(crate) latest_psbt_shape: PsbtShapeV1,
    pub(crate) fundpsbt_auth: Option<NormalizedRpcAuth>,
    pub(crate) signpsbt_auth: Option<NormalizedRpcAuth>,
    pub(crate) signonly: Vec<u32>,
    pub(crate) wallet_inputs: Vec<WalletInput>,
    pub(crate) change_outnum: Option<u32>,
    pub(crate) linked_node_channel_id_hex: Option<String>,
    pub(crate) linked_splice_psbt_shape_hash: Option<String>,
    pub(crate) created_at_ms: u64,
    pub(crate) updated_at_ms: u64,
}

impl SpliceWalletPsbtContextV1 {
    pub(crate) fn new(
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

impl State {
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

    pub(crate) fn get_splice_session(
        &self,
        node_channel_id_hex: &str,
    ) -> anyhow::Result<Option<SpliceSessionV1>> {
        self.get_splice(&splice_session_key(node_channel_id_hex))
    }

    fn put_splice_session(&mut self, session: &SpliceSessionV1) -> anyhow::Result<()> {
        self.put_splice(&splice_session_key(&session.node_channel_id_hex), session)
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

    pub(crate) fn create_local_splice_session(
        &mut self,
        session: SpliceSessionV1,
    ) -> anyhow::Result<()> {
        if session.auth.splice_init_auth.is_none() {
            bail!("local splice sessions require splice_init_auth");
        }
        if session.intent.authorized_relative_amount_sat.is_none() {
            bail!("local splice sessions require authorized relative amount");
        }
        self.create_new_splice_session(session, SpliceOrigin::LocalInitiator)
    }

    pub(crate) fn create_peer_splice_session(
        &mut self,
        session: SpliceSessionV1,
    ) -> anyhow::Result<()> {
        if session.auth.splice_init_auth.is_some() {
            bail!("peer-initiated splice sessions must not include splice_init_auth");
        }
        if session.intent.authorized_relative_amount_sat.is_some() {
            bail!("peer-initiated splice sessions must not include local relative amount intent");
        }
        self.create_new_splice_session(session, SpliceOrigin::PeerInitiated)
    }

    pub(crate) fn create_dev_splice_session(
        &mut self,
        session: SpliceSessionV1,
    ) -> anyhow::Result<()> {
        self.create_new_splice_session(session, SpliceOrigin::DevSpliceUnresolved)
    }

    pub(crate) fn update_splice_candidate_shape(
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

    pub(crate) fn freeze_splice_candidate(
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

    pub(crate) fn mark_splice_signatures_exchanging(
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

    pub(crate) fn mark_splice_pending_lock(
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

    pub(crate) fn put_splice_outpoint_index(
        &mut self,
        candidate_outpoint: &FundingOutpoint,
        index: &SpliceOutpointIndexV1,
    ) -> anyhow::Result<()> {
        self.put_splice(
            &splice_outpoint_key(&candidate_outpoint.txid, candidate_outpoint.vout),
            index,
        )
    }

    pub(crate) fn get_splice_by_outpoint(
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

    pub(crate) fn put_splice_wallet_psbt_context(
        &mut self,
        context: SpliceWalletPsbtContextV1,
    ) -> anyhow::Result<()> {
        self.put_splice(&wallet_psbt_key(&context.psbt_shape_hash), &context)
    }

    pub(crate) fn get_splice_wallet_psbt_context(
        &self,
        psbt_shape_hash: &str,
    ) -> anyhow::Result<Option<SpliceWalletPsbtContextV1>> {
        self.get_splice(&wallet_psbt_key(psbt_shape_hash))
    }

    pub(crate) fn link_splice_wallet_psbt(
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

    pub(crate) fn tombstone_splice_session(
        &mut self,
        node_channel_id_hex: &str,
    ) -> anyhow::Result<()> {
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
    use crate::pb::SignerStateEntry;
    use serde_json::json;

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
