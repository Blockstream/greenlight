use crate::pb::HsmRequestContext;
use crate::persist::{
    psbt_shape_from_base64, psbt_shape_from_psbt, transaction_shape, SpliceOrigin, SplicePhase,
    SpliceSessionV1, State,
};
use crate::signer::model::Request;
use anyhow::{anyhow, bail};
use lightning_signer::bitcoin::secp256k1::PublicKey;
use lightning_signer::channel::ChannelId;
use vls_protocol::msgs::{
    CheckOutpoint, LockOutpoint, Message, SetupChannel, SignSpliceTx, SignWithdrawal,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpliceOperation {
    SignWithdrawal,
    SetupChannel,
    SignSpliceTx,
    CheckOutpoint,
    LockOutpoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VlsProof {
    SpliceSigning,
    CandidateFunding,
    NoLocalLoss,
    OutpointBurial,
    OutpointLock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SplicePolicyViolation {
    InvalidHsmContext,
    MissingSpliceSession,
    MissingSpliceIntent,
    InvalidPhase,
    MissingSignPsbtIntent,
    MissingSignPsbtAuth,
    PsbtShapeMismatch,
    UnknownWalletInput,
    SignOnlyViolation,
    OldFundingInputSelected,
    TxPsbtMismatch,
    OldFundingInputMismatch,
    CandidateMismatch,
    PeerLocalLoss,
    DevSpliceUnresolved,
    UnknownOutpoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SplicePolicyDecision {
    NotSpliceRelated,
    RequiresVlsProof(VlsProof),
    Rejected(SplicePolicyViolation),
}

impl SpliceOperation {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SignWithdrawal => "sign_withdrawal",
            Self::SetupChannel => "setup_channel",
            Self::SignSpliceTx => "sign_splice_tx",
            Self::CheckOutpoint => "check_outpoint",
            Self::LockOutpoint => "lock_outpoint",
        }
    }
}

impl VlsProof {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SpliceSigning => "splice_signing",
            Self::CandidateFunding => "candidate_funding",
            Self::NoLocalLoss => "no_local_loss",
            Self::OutpointBurial => "outpoint_burial",
            Self::OutpointLock => "outpoint_lock",
        }
    }
}

pub(crate) fn operation(message: &Message) -> Option<SpliceOperation> {
    match message {
        Message::SignWithdrawal(_) => Some(SpliceOperation::SignWithdrawal),
        Message::SetupChannel(_) => Some(SpliceOperation::SetupChannel),
        Message::SignSpliceTx(_) => Some(SpliceOperation::SignSpliceTx),
        Message::CheckOutpoint(_) => Some(SpliceOperation::CheckOutpoint),
        Message::LockOutpoint(_) => Some(SpliceOperation::LockOutpoint),
        _ => None,
    }
}

pub(crate) fn node_channel_id_hex(
    local_node_id: &[u8],
    context: &HsmRequestContext,
) -> anyhow::Result<String> {
    let local_node_id = PublicKey::from_slice(local_node_id)
        .map_err(|e| anyhow!("invalid local node id in signer context: {e}"))?;
    let peer_id: [u8; 33] = context.node_id.as_slice().try_into().map_err(|_| {
        anyhow!(
            "invalid peer id length in HSM context: expected 33 bytes, got {}",
            context.node_id.len()
        )
    })?;
    if context.dbid == 0 {
        bail!("channel-scoped HSM context has zero dbid");
    }
    let channel_id = ChannelId::new_from_peer_id_and_oid(&peer_id, context.dbid);
    Ok(hex::encode(
        vls_persist::model::NodeChannelId::new(&local_node_id, &channel_id).0,
    ))
}

pub(crate) fn classify(
    message: &Message,
    pending_requests: &[Request],
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
) -> anyhow::Result<SplicePolicyDecision> {
    match message {
        Message::SignWithdrawal(request) => {
            classify_sign_withdrawal(request, pending_requests, state)
        }
        Message::SetupChannel(request) => {
            classify_setup_channel(request, state, local_node_id, hsm_context)
        }
        Message::SignSpliceTx(request) => {
            classify_sign_splice_tx(request, pending_requests, state, local_node_id, hsm_context)
        }
        Message::CheckOutpoint(request) => {
            classify_check_outpoint(request, state, local_node_id, hsm_context)
        }
        Message::LockOutpoint(request) => {
            classify_lock_outpoint(request, state, local_node_id, hsm_context)
        }
        _ => Ok(SplicePolicyDecision::NotSpliceRelated),
    }
}

fn reject(violation: SplicePolicyViolation) -> anyhow::Result<SplicePolicyDecision> {
    Ok(SplicePolicyDecision::Rejected(violation))
}

fn requires_vls_proof(proof: VlsProof) -> anyhow::Result<SplicePolicyDecision> {
    Ok(SplicePolicyDecision::RequiresVlsProof(proof))
}

fn channel_session(
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
) -> anyhow::Result<Option<(String, SpliceSessionV1)>> {
    let Some(context) = hsm_context else {
        return Ok(None);
    };
    let Ok(node_channel_id_hex) = node_channel_id_hex(local_node_id, context) else {
        return Ok(None);
    };
    let session = state.get_splice_session(&node_channel_id_hex)?;
    Ok(session.map(|session| (node_channel_id_hex, session)))
}

fn session_violation(
    session: &SpliceSessionV1,
    allowed_phases: &[SplicePhase],
) -> Option<SplicePolicyViolation> {
    if session.origin == SpliceOrigin::DevSpliceUnresolved {
        return Some(SplicePolicyViolation::DevSpliceUnresolved);
    }
    if !allowed_phases.contains(&session.phase) {
        return Some(SplicePolicyViolation::InvalidPhase);
    }
    if session.origin == SpliceOrigin::LocalInitiator
        && (session.auth.splice_init_auth.is_none()
            || session.intent.authorized_relative_amount_sat.is_none())
    {
        return Some(SplicePolicyViolation::MissingSpliceIntent);
    }
    None
}

fn peer_policy_decision(session: &SpliceSessionV1) -> Option<SplicePolicyDecision> {
    if session.origin != SpliceOrigin::PeerInitiated {
        return None;
    }
    if !session.delta.computed {
        return Some(SplicePolicyDecision::RequiresVlsProof(
            VlsProof::NoLocalLoss,
        ));
    }
    if !session.delta.no_local_loss {
        return Some(SplicePolicyDecision::Rejected(
            SplicePolicyViolation::PeerLocalLoss,
        ));
    }
    None
}

fn classify_setup_channel(
    request: &SetupChannel,
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
) -> anyhow::Result<SplicePolicyDecision> {
    let txid = request.funding_txid.to_string();
    let by_outpoint = state.get_splice_by_outpoint(&txid, request.funding_txout as u32)?;
    let by_context = channel_session(state, local_node_id, hsm_context)?;
    let session = match (by_outpoint, by_context) {
        (None, None) => return Ok(SplicePolicyDecision::NotSpliceRelated),
        (Some(session), Some((context_id, _))) if context_id == session.node_channel_id_hex => {
            session
        }
        (Some(_), _) => return reject(SplicePolicyViolation::InvalidHsmContext),
        (None, Some((_, session))) => session,
    };

    if let Some(violation) = session_violation(
        &session,
        &[
            SplicePhase::Negotiating,
            SplicePhase::CommitmentsSecured,
            SplicePhase::SignaturesExchanging,
        ],
    ) {
        return reject(violation);
    }
    let Some(candidate_outpoint) = session.cand.funding_outpoint.as_ref() else {
        return requires_vls_proof(VlsProof::CandidateFunding);
    };
    let Some(candidate_value_sat) = session.cand.value_sat else {
        return requires_vls_proof(VlsProof::CandidateFunding);
    };
    if candidate_outpoint.txid != txid
        || candidate_outpoint.vout != request.funding_txout as u32
        || candidate_value_sat != request.channel_value
    {
        return reject(SplicePolicyViolation::CandidateMismatch);
    }
    if let Some(remote_funding_key_hex) = session.cand.remote_funding_key_hex.as_deref() {
        if remote_funding_key_hex != hex::encode(request.remote_funding_pubkey.0) {
            return reject(SplicePolicyViolation::CandidateMismatch);
        }
    }
    if let Some(decision) = peer_policy_decision(&session) {
        return Ok(decision);
    }

    requires_vls_proof(VlsProof::CandidateFunding)
}

fn pending_splice_psbt_matches(
    pending_requests: &[Request],
    session: &SpliceSessionV1,
    expected_shape_hash: &str,
) -> bool {
    pending_requests.iter().any(|pending| {
        let (channel_id, psbt) = match pending {
            Request::SpliceInit(request) => {
                let Some(psbt) = request.initialpsbt.as_deref() else {
                    return false;
                };
                (request.channel_id.as_slice(), psbt)
            }
            Request::SpliceUpdate(request) => {
                (request.channel_id.as_slice(), request.psbt.as_str())
            }
            Request::SpliceSigned(request) => {
                (request.channel_id.as_slice(), request.psbt.as_str())
            }
            _ => return false,
        };
        hex::encode(channel_id) == session.channel_id_hex
            && psbt_shape_from_base64(psbt)
                .map(|(shape_hash, _)| shape_hash == expected_shape_hash)
                .unwrap_or(false)
    })
}

fn classify_sign_splice_tx(
    request: &SignSpliceTx,
    pending_requests: &[Request],
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
) -> anyhow::Result<SplicePolicyDecision> {
    let Some(context) = hsm_context else {
        return reject(SplicePolicyViolation::InvalidHsmContext);
    };
    let node_channel_id_hex = match node_channel_id_hex(local_node_id, context) {
        Ok(node_channel_id_hex) => node_channel_id_hex,
        Err(_) => return reject(SplicePolicyViolation::InvalidHsmContext),
    };
    let Some(session) = state.get_splice_session(&node_channel_id_hex)? else {
        return reject(SplicePolicyViolation::MissingSpliceSession);
    };
    if let Some(violation) = session_violation(
        &session,
        &[
            SplicePhase::CommitmentsSecured,
            SplicePhase::SignaturesExchanging,
        ],
    ) {
        return reject(violation);
    }

    let psbt = &request.psbt.0.inner;
    let (shape_hash, psbt_shape) = psbt_shape_from_psbt(psbt)?;
    if transaction_shape(&request.tx.0) != psbt_shape {
        return reject(SplicePolicyViolation::TxPsbtMismatch);
    }
    let expected_shape_hash = session
        .psbt
        .frozen_psbt_shape_hash
        .as_deref()
        .or(session.psbt.candidate_psbt_shape_hash.as_deref());
    if expected_shape_hash != Some(shape_hash.as_str())
        || session.psbt.candidate_psbt_shape.as_ref() != Some(&psbt_shape)
    {
        return reject(SplicePolicyViolation::PsbtShapeMismatch);
    }
    if session.origin == SpliceOrigin::LocalInitiator
        && !pending_splice_psbt_matches(pending_requests, &session, &shape_hash)
    {
        return reject(SplicePolicyViolation::MissingSpliceIntent);
    }

    let old_input_indices: Vec<usize> = request
        .tx
        .0
        .input
        .iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let old = &session.old.funding_outpoint;
            (input.previous_output.txid.to_string() == old.txid
                && input.previous_output.vout == old.vout)
                .then_some(index)
        })
        .collect();
    if old_input_indices.as_slice() != [request.input_index as usize] {
        return reject(SplicePolicyViolation::OldFundingInputMismatch);
    }
    match session.cand.sign_splice_tx_input_index {
        Some(input_index) if input_index != request.input_index => {
            return reject(SplicePolicyViolation::OldFundingInputMismatch)
        }
        None => return requires_vls_proof(VlsProof::CandidateFunding),
        Some(_) => {}
    }

    let Some(candidate_outpoint) = session.cand.funding_outpoint.as_ref() else {
        return requires_vls_proof(VlsProof::CandidateFunding);
    };
    let Some(candidate_value_sat) = session.cand.value_sat else {
        return requires_vls_proof(VlsProof::CandidateFunding);
    };
    let Some(candidate_script_hash) = session.cand.script_pubkey_hash.as_deref() else {
        return requires_vls_proof(VlsProof::CandidateFunding);
    };
    let Some(candidate_output) = request.tx.0.output.get(candidate_outpoint.vout as usize) else {
        return reject(SplicePolicyViolation::CandidateMismatch);
    };
    if candidate_outpoint.txid != request.tx.0.compute_txid().to_string()
        || candidate_output.value.to_sat() != candidate_value_sat
        || sha256::digest(candidate_output.script_pubkey.as_bytes()) != candidate_script_hash
    {
        return reject(SplicePolicyViolation::CandidateMismatch);
    }
    if let Some(remote_funding_key_hex) = session.cand.remote_funding_key_hex.as_deref() {
        if remote_funding_key_hex != hex::encode(request.remote_funding_key.0) {
            return reject(SplicePolicyViolation::CandidateMismatch);
        }
    }
    if let Some(decision) = peer_policy_decision(&session) {
        return Ok(decision);
    }

    requires_vls_proof(VlsProof::SpliceSigning)
}

fn classify_check_outpoint(
    request: &CheckOutpoint,
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
) -> anyhow::Result<SplicePolicyDecision> {
    classify_outpoint(
        request.funding_txid.to_string(),
        request.funding_txout as u32,
        state,
        local_node_id,
        hsm_context,
        VlsProof::OutpointBurial,
    )
}

fn classify_lock_outpoint(
    request: &LockOutpoint,
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
) -> anyhow::Result<SplicePolicyDecision> {
    classify_outpoint(
        request.funding_txid.to_string(),
        request.funding_txout as u32,
        state,
        local_node_id,
        hsm_context,
        VlsProof::OutpointLock,
    )
}

fn classify_outpoint(
    txid: String,
    vout: u32,
    state: &State,
    local_node_id: &[u8],
    hsm_context: Option<&HsmRequestContext>,
    proof: VlsProof,
) -> anyhow::Result<SplicePolicyDecision> {
    let Some(session) = state.get_splice_by_outpoint(&txid, vout)? else {
        if channel_session(state, local_node_id, hsm_context)?.is_some() {
            return reject(SplicePolicyViolation::UnknownOutpoint);
        }
        return Ok(SplicePolicyDecision::NotSpliceRelated);
    };
    if let Some(context) = hsm_context {
        if context.dbid != 0 {
            let context_id = match node_channel_id_hex(local_node_id, context) {
                Ok(context_id) => context_id,
                Err(_) => return reject(SplicePolicyViolation::InvalidHsmContext),
            };
            if context_id != session.node_channel_id_hex {
                return reject(SplicePolicyViolation::InvalidHsmContext);
            }
        }
    }
    if let Some(violation) = session_violation(&session, &[SplicePhase::PendingLock]) {
        return reject(violation);
    }
    if session
        .cand
        .funding_outpoint
        .as_ref()
        .map(|outpoint| (&outpoint.txid, outpoint.vout))
        != Some((&txid, vout))
    {
        return reject(SplicePolicyViolation::UnknownOutpoint);
    }
    if let Some(decision) = peer_policy_decision(&session) {
        return Ok(decision);
    }

    requires_vls_proof(proof)
}

fn classify_sign_withdrawal(
    request: &SignWithdrawal,
    pending_requests: &[Request],
    state: &State,
) -> anyhow::Result<SplicePolicyDecision> {
    let psbt = &request.psbt.0.psbt.inner;
    let (shape_hash, shape) = psbt_shape_from_psbt(psbt)?;
    let Some(context) = state.get_splice_wallet_psbt_context(&shape_hash)? else {
        return Ok(SplicePolicyDecision::NotSpliceRelated);
    };
    let Some(node_channel_id_hex) = context.linked_node_channel_id_hex.as_deref() else {
        return Ok(SplicePolicyDecision::NotSpliceRelated);
    };
    let Some(session) = state.get_splice_session(node_channel_id_hex)? else {
        return reject(SplicePolicyViolation::MissingSpliceSession);
    };
    if let Some(violation) = session_violation(
        &session,
        &[
            SplicePhase::CommitmentsSecured,
            SplicePhase::SignaturesExchanging,
        ],
    ) {
        return reject(violation);
    }
    if context.signpsbt_auth.is_none() {
        return reject(SplicePolicyViolation::MissingSignPsbtAuth);
    }
    if context.psbt_shape_hash != shape_hash
        || context.latest_psbt_shape != shape
        || context.linked_splice_psbt_shape_hash.as_deref() != Some(shape_hash.as_str())
        || !session
            .linked_wallet_psbt_shape_hashes
            .iter()
            .any(|hash| hash == &shape_hash)
    {
        return reject(SplicePolicyViolation::PsbtShapeMismatch);
    }
    let expected_splice_shape = session
        .psbt
        .frozen_psbt_shape_hash
        .as_deref()
        .or(session.psbt.candidate_psbt_shape_hash.as_deref());
    if expected_splice_shape != Some(shape_hash.as_str()) {
        return reject(SplicePolicyViolation::PsbtShapeMismatch);
    }

    let matching_signpsbt = pending_requests.iter().any(|pending| {
        let Request::SignPsbt(pending) = pending else {
            return false;
        };
        psbt_shape_from_base64(&pending.psbt)
            .map(|(pending_hash, _)| {
                pending_hash == shape_hash && pending.signonly == context.signonly
            })
            .unwrap_or(false)
    });
    if !matching_signpsbt {
        return reject(SplicePolicyViolation::MissingSignPsbtIntent);
    }

    for utxo in request.utxos.iter() {
        let txid = utxo.txid.to_string();
        if session.old.funding_outpoint.txid == txid
            && session.old.funding_outpoint.vout == utxo.outnum
        {
            return reject(SplicePolicyViolation::OldFundingInputSelected);
        }
        let Some(input_index) = psbt.unsigned_tx.input.iter().position(|input| {
            input.previous_output.txid == utxo.txid && input.previous_output.vout == utxo.outnum
        }) else {
            return reject(SplicePolicyViolation::UnknownWalletInput);
        };
        let known_wallet_input = context.wallet_inputs.iter().any(|input| {
            input.txid == txid && input.vout == utxo.outnum && input.value_sat == utxo.amount
        });
        if !known_wallet_input {
            return reject(SplicePolicyViolation::UnknownWalletInput);
        }
        if !context.signonly.is_empty() && !context.signonly.contains(&(input_index as u32)) {
            return reject(SplicePolicyViolation::SignOnlyViolation);
        }
    }

    if let Some(decision) = peer_policy_decision(&session) {
        return Ok(decision);
    }

    requires_vls_proof(VlsProof::SpliceSigning)
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
    use crate::persist::{
        psbt_shape_from_psbt, CandidateFundingFacts, FeePolicy, FundPsbtResponseFacts,
        FundingOutpoint, LocalSpliceIntent, NormalizedRpcAuth, OldSpliceState, SignPsbtIntentFacts,
        SpliceOrigin, SpliceSessionV1, SpliceUpdateResponseFacts, State, WalletInput,
        WalletInputSource,
    };
    use crate::signer::model::{
        cln::{SignpsbtRequest, SpliceSignedRequest},
        Request,
    };
    use base64::{engine::general_purpose, Engine as _};
    use lightning_signer::channel::ChannelId;
    use std::str::FromStr;
    use vls_protocol::model::{Basepoints, PubKey, Utxo};
    use vls_protocol::msgs::{
        CheckOutpoint, LockOutpoint, Message, SetupChannel, SignSpliceTx, SignWithdrawal,
    };
    use vls_protocol::psbt::{PsbtWrapper, StreamedPSBT};
    use vls_protocol::serde_bolt::{Array, Octets, WithSize};

    fn auth(uri: &str, byte: &str, timestamp_ms: u64) -> NormalizedRpcAuth {
        NormalizedRpcAuth::new(
            uri.to_string(),
            byte.repeat(32),
            "02".repeat(33),
            timestamp_ms,
        )
    }

    struct SpliceSigningFixture {
        message: Message,
        pending: Vec<Request>,
        state: State,
        local_node_id: Vec<u8>,
        context: HsmRequestContext,
        node_channel_id_hex: String,
    }

    fn test_public_key(byte: u8) -> PublicKey {
        PublicKey::from_secret_key(
            &Secp256k1::signing_only(),
            &SecretKey::from_slice(&[byte; 32]).unwrap(),
        )
    }

    fn splice_signing_fixture(
        origin: SpliceOrigin,
        delta_computed: bool,
        no_local_loss: bool,
    ) -> SpliceSigningFixture {
        let local_node_id = test_public_key(1).serialize().to_vec();
        let peer_id = test_public_key(2);
        let remote_funding_key = test_public_key(3);
        let context = HsmRequestContext {
            node_id: peer_id.serialize().to_vec(),
            dbid: 42,
            capabilities: 0,
        };
        let node_channel_id_hex = node_channel_id_hex(&local_node_id, &context).unwrap();
        let old_txid = Txid::from_str(&"11".repeat(32)).unwrap();
        let channel_id = vec![4; 32];
        let funding_script = ScriptBuf::from_hex(
            "00203333333333333333333333333333333333333333333333333333333333333333",
        )
        .unwrap();
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: old_txid,
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(1_020_000),
                script_pubkey: funding_script,
            }],
        };
        let mut psbt = Psbt::from_unsigned_tx(tx.clone()).unwrap();
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(1_000_000),
            script_pubkey: ScriptBuf::new(),
        });
        let encoded_psbt = general_purpose::STANDARD.encode(psbt.serialize());
        let (shape_hash, shape) = psbt_shape_from_psbt(&psbt).unwrap();
        let old = OldSpliceState {
            funding_outpoint: FundingOutpoint {
                txid: old_txid.to_string(),
                vout: 0,
            },
            channel_value_sat: 1_000_000,
            local_balance_sat: 600_000,
        };
        let mut state = State::new();
        match origin {
            SpliceOrigin::LocalInitiator => state
                .record_local_splice_intent(LocalSpliceIntent {
                    node_id_hex: hex::encode(&local_node_id),
                    channel_id_hex: hex::encode(&channel_id),
                    node_channel_id_hex: node_channel_id_hex.clone(),
                    old: old.clone(),
                    splice_init_auth: auth("/cln.Node/SpliceInit", "55", 1),
                    authorized_relative_amount_sat: 20_000,
                    fee_policy: FeePolicy::default(),
                    initial_psbt_shape_hash: Some(shape_hash.clone()),
                    initial_psbt_shape: Some(shape.clone()),
                    timestamp_ms: 1,
                })
                .unwrap(),
            SpliceOrigin::PeerInitiated | SpliceOrigin::DevSpliceUnresolved => {
                let mut session = SpliceSessionV1::new(
                    origin.clone(),
                    hex::encode(&local_node_id),
                    hex::encode(&channel_id),
                    node_channel_id_hex.clone(),
                    old.clone(),
                    None,
                    None,
                    FeePolicy::default(),
                    1,
                );
                session.psbt.candidate_psbt_shape_hash = Some(shape_hash.clone());
                session.psbt.candidate_psbt_shape = Some(shape.clone());
                session.delta.computed = delta_computed;
                session.delta.no_local_loss = no_local_loss;
                if origin == SpliceOrigin::PeerInitiated {
                    state.create_peer_splice_session(session).unwrap();
                } else {
                    state.create_dev_splice_session(session).unwrap();
                }
            }
        }
        state
            .freeze_splice_candidate(
                &node_channel_id_hex,
                shape_hash,
                CandidateFundingFacts {
                    funding_outpoint: FundingOutpoint {
                        txid: tx.compute_txid().to_string(),
                        vout: 0,
                    },
                    value_sat: tx.output[0].value.to_sat(),
                    script_pubkey_hash: sha256::digest(tx.output[0].script_pubkey.as_bytes()),
                    sign_splice_tx_input_index: 0,
                    remote_funding_key_hex: Some(hex::encode(remote_funding_key.serialize())),
                },
                2,
            )
            .unwrap();

        let pending = if origin == SpliceOrigin::LocalInitiator {
            vec![Request::SpliceSigned(SpliceSignedRequest {
                channel_id,
                psbt: encoded_psbt,
                sign_first: Some(true),
            })]
        } else {
            Vec::new()
        };
        let message = Message::SignSpliceTx(SignSpliceTx {
            tx: WithSize(tx),
            psbt: WithSize(PsbtWrapper::from(psbt)),
            remote_funding_key: PubKey(remote_funding_key.serialize()),
            input_index: 0,
        });
        SpliceSigningFixture {
            message,
            pending,
            state,
            local_node_id,
            context,
            node_channel_id_hex,
        }
    }

    fn setup_channel_message(fixture: &SpliceSigningFixture) -> Message {
        let session = fixture
            .state
            .get_splice_session(&fixture.node_channel_id_hex)
            .unwrap()
            .unwrap();
        let outpoint = session.cand.funding_outpoint.unwrap();
        let remote_key = PubKey(test_public_key(3).serialize());
        Message::SetupChannel(SetupChannel {
            is_outbound: true,
            channel_value: session.cand.value_sat.unwrap(),
            push_value: 0,
            funding_txid: Txid::from_str(&outpoint.txid).unwrap(),
            funding_txout: outpoint.vout as u16,
            to_self_delay: 6,
            local_shutdown_script: Octets(Vec::new()),
            local_shutdown_wallet_index: None,
            remote_basepoints: Basepoints {
                revocation: PubKey(test_public_key(5).serialize()),
                payment: PubKey(test_public_key(6).serialize()),
                htlc: PubKey(test_public_key(7).serialize()),
                delayed_payment: PubKey(test_public_key(8).serialize()),
            },
            remote_funding_pubkey: remote_key,
            remote_to_self_delay: 6,
            remote_shutdown_script: Octets(Vec::new()),
            channel_type: Octets(Vec::new()),
        })
    }

    fn sign_withdrawal_fixture_for_origin(
        origin: SpliceOrigin,
        delta_computed: bool,
        no_local_loss: bool,
    ) -> (Message, Vec<Request>, State) {
        let old_txid = Txid::from_str(&"11".repeat(32)).unwrap();
        let wallet_txid = Txid::from_str(&"22".repeat(32)).unwrap();
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![
                TxIn {
                    previous_output: OutPoint {
                        txid: old_txid,
                        vout: 0,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::MAX,
                    witness: Witness::new(),
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: wallet_txid,
                        vout: 1,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::MAX,
                    witness: Witness::new(),
                },
            ],
            output: vec![TxOut {
                value: Amount::from_sat(1_020_000),
                script_pubkey: ScriptBuf::from_hex("00143333333333333333333333333333333333333333")
                    .unwrap(),
            }],
        };
        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(1_000_000),
            script_pubkey: ScriptBuf::new(),
        });
        psbt.inputs[1].witness_utxo = Some(TxOut {
            value: Amount::from_sat(25_000),
            script_pubkey: ScriptBuf::new(),
        });
        let encoded_psbt = general_purpose::STANDARD.encode(psbt.serialize());
        let (shape_hash, shape) = psbt_shape_from_psbt(&psbt).unwrap();
        let node_channel_id_hex = "44".repeat(74);
        let old = OldSpliceState {
            funding_outpoint: FundingOutpoint {
                txid: old_txid.to_string(),
                vout: 0,
            },
            channel_value_sat: 1_000_000,
            local_balance_sat: 600_000,
        };
        let mut state = State::new();
        state
            .record_fundpsbt_response(FundPsbtResponseFacts {
                psbt_shape_hash: shape_hash.clone(),
                psbt_shape: shape.clone(),
                fundpsbt_auth: auth("/cln.Node/FundPsbt", "55", 1),
                wallet_inputs: vec![WalletInput {
                    txid: wallet_txid.to_string(),
                    vout: 1,
                    value_sat: 25_000,
                    reserved_to_block: Some(100),
                    source: WalletInputSource::FundPsbt,
                }],
                change_outnum: None,
                timestamp_ms: 1,
            })
            .unwrap();
        match origin {
            SpliceOrigin::LocalInitiator => state
                .record_local_splice_intent(LocalSpliceIntent {
                    node_id_hex: "02".repeat(33),
                    channel_id_hex: "33".repeat(32),
                    node_channel_id_hex: node_channel_id_hex.clone(),
                    old: old.clone(),
                    splice_init_auth: auth("/cln.Node/SpliceInit", "66", 2),
                    authorized_relative_amount_sat: 20_000,
                    fee_policy: FeePolicy::default(),
                    initial_psbt_shape_hash: Some(shape_hash.clone()),
                    initial_psbt_shape: Some(shape.clone()),
                    timestamp_ms: 2,
                })
                .unwrap(),
            SpliceOrigin::PeerInitiated | SpliceOrigin::DevSpliceUnresolved => {
                let mut session = SpliceSessionV1::new(
                    origin.clone(),
                    "02".repeat(33),
                    "33".repeat(32),
                    node_channel_id_hex.clone(),
                    old,
                    None,
                    None,
                    FeePolicy::default(),
                    2,
                );
                session.delta.computed = delta_computed;
                session.delta.no_local_loss = no_local_loss;
                if origin == SpliceOrigin::PeerInitiated {
                    state.create_peer_splice_session(session).unwrap();
                } else {
                    state.create_dev_splice_session(session).unwrap();
                }
            }
        }
        state
            .record_splice_update_response(SpliceUpdateResponseFacts {
                node_channel_id_hex,
                psbt_shape_hash: shape_hash.clone(),
                psbt_shape: shape.clone(),
                splice_update_auth: auth("/cln.Node/SpliceUpdate", "77", 3),
                commitments_secured: true,
                signatures_secured: Some(false),
                timestamp_ms: 3,
            })
            .unwrap();
        state
            .record_signpsbt_intent(SignPsbtIntentFacts {
                psbt_shape_hash: shape_hash,
                psbt_shape: shape,
                signpsbt_auth: auth("/cln.Node/SignPsbt", "88", 4),
                signonly: vec![1],
                timestamp_ms: 4,
            })
            .unwrap();

        let message = Message::SignWithdrawal(SignWithdrawal {
            utxos: Array(vec![Utxo {
                txid: wallet_txid,
                outnum: 1,
                amount: 25_000,
                keyindex: 0,
                is_p2sh: false,
                script: Octets(vec![]),
                close_info: None,
                is_in_coinbase: false,
            }]),
            psbt: WithSize(StreamedPSBT::new(psbt)),
        });
        let pending = vec![Request::SignPsbt(SignpsbtRequest {
            psbt: encoded_psbt,
            signonly: vec![1],
        })];
        (message, pending, state)
    }

    fn sign_withdrawal_fixture() -> (Message, Vec<Request>, State) {
        sign_withdrawal_fixture_for_origin(SpliceOrigin::LocalInitiator, false, false)
    }

    #[test]
    fn hsm_context_resolves_canonical_node_channel_id() {
        let secp = Secp256k1::signing_only();
        let local = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[1; 32]).unwrap());
        let peer = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[2; 32]).unwrap());
        let context = HsmRequestContext {
            node_id: peer.serialize().to_vec(),
            dbid: 42,
            capabilities: 0,
        };
        let channel_id = ChannelId::new_from_peer_id_and_oid(&peer.serialize(), 42);
        let expected = hex::encode(vls_persist::model::NodeChannelId::new(&local, &channel_id).0);

        assert_eq!(
            node_channel_id_hex(&local.serialize(), &context).unwrap(),
            expected
        );
    }

    #[test]
    fn matching_splice_signwithdrawal_reaches_vls_boundary() {
        let (message, pending, state) = sign_withdrawal_fixture();

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::SpliceSigning)
        );
    }

    #[test]
    fn splice_signwithdrawal_without_current_signpsbt_rejects() {
        let (message, _, state) = sign_withdrawal_fixture();

        assert_eq!(
            classify(&message, &[], &state, &[], None).unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::MissingSignPsbtIntent)
        );
    }

    #[test]
    fn splice_signwithdrawal_with_fundpsbt_only_rejects() {
        let (message, pending, mut state) = sign_withdrawal_fixture();
        let Message::SignWithdrawal(request) = &message else {
            unreachable!();
        };
        let (shape_hash, _) = psbt_shape_from_psbt(&request.psbt.0.psbt.inner).unwrap();
        let mut context = state
            .get_splice_wallet_psbt_context(&shape_hash)
            .unwrap()
            .unwrap();
        context.signpsbt_auth = None;
        state.put_splice_wallet_psbt_context(context).unwrap();

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::MissingSignPsbtAuth)
        );
    }

    #[test]
    fn splice_signwithdrawal_outside_signonly_rejects() {
        let (message, mut pending, mut state) = sign_withdrawal_fixture();
        let Message::SignWithdrawal(request) = &message else {
            unreachable!();
        };
        let (shape_hash, _) = psbt_shape_from_psbt(&request.psbt.0.psbt.inner).unwrap();
        let mut context = state
            .get_splice_wallet_psbt_context(&shape_hash)
            .unwrap()
            .unwrap();
        context.signonly = vec![0];
        state.put_splice_wallet_psbt_context(context).unwrap();
        let Request::SignPsbt(request) = &mut pending[0] else {
            unreachable!();
        };
        request.signonly = vec![0];

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::SignOnlyViolation)
        );
    }

    #[test]
    fn splice_signwithdrawal_selecting_old_funding_input_rejects() {
        let (mut message, pending, state) = sign_withdrawal_fixture();
        let Message::SignWithdrawal(request) = &mut message else {
            unreachable!();
        };
        request.utxos.0[0].txid = Txid::from_str(&"11".repeat(32)).unwrap();
        request.utxos.0[0].outnum = 0;
        request.utxos.0[0].amount = 1_000_000;

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::OldFundingInputSelected)
        );
    }

    #[test]
    fn unresolved_peer_signwithdrawal_requires_no_local_loss_proof() {
        let (message, pending, state) =
            sign_withdrawal_fixture_for_origin(SpliceOrigin::PeerInitiated, false, false);

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::NoLocalLoss)
        );
    }

    #[test]
    fn peer_signwithdrawal_with_local_loss_rejects() {
        let (message, pending, state) =
            sign_withdrawal_fixture_for_origin(SpliceOrigin::PeerInitiated, true, false);

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::PeerLocalLoss)
        );
    }

    #[test]
    fn unresolved_dev_splice_signwithdrawal_rejects() {
        let (message, pending, state) =
            sign_withdrawal_fixture_for_origin(SpliceOrigin::DevSpliceUnresolved, false, false);

        assert_eq!(
            classify(&message, &pending, &state, &[], None).unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::DevSpliceUnresolved)
        );
    }

    #[test]
    fn matching_local_sign_splice_tx_reaches_vls_boundary() {
        let fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::SpliceSigning)
        );
    }

    #[test]
    fn local_sign_splice_tx_without_current_splice_rpc_rejects() {
        let mut fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        fixture.pending.clear();

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::MissingSpliceIntent)
        );
    }

    #[test]
    fn unresolved_dev_splice_transaction_signing_rejects() {
        let fixture = splice_signing_fixture(SpliceOrigin::DevSpliceUnresolved, false, false);

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::DevSpliceUnresolved)
        );
    }

    #[test]
    fn sign_splice_tx_with_wrong_old_funding_input_index_rejects() {
        let mut fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        let Message::SignSpliceTx(request) = &mut fixture.message else {
            unreachable!();
        };
        request.input_index = 1;

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::OldFundingInputMismatch)
        );
    }

    #[test]
    fn sign_splice_tx_with_different_transaction_and_psbt_rejects() {
        let mut fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        let Message::SignSpliceTx(request) = &mut fixture.message else {
            unreachable!();
        };
        request.tx.0.output[0].value = Amount::from_sat(1_019_999);

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::TxPsbtMismatch)
        );
    }

    #[test]
    fn unresolved_peer_sign_splice_tx_requires_no_local_loss_proof() {
        let fixture = splice_signing_fixture(SpliceOrigin::PeerInitiated, false, false);

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::NoLocalLoss)
        );
    }

    #[test]
    fn peer_sign_splice_tx_with_local_loss_rejects() {
        let fixture = splice_signing_fixture(SpliceOrigin::PeerInitiated, true, false);

        assert_eq!(
            classify(
                &fixture.message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::PeerLocalLoss)
        );
    }

    #[test]
    fn matching_splice_setup_channel_reaches_vls_boundary() {
        let fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        let message = setup_channel_message(&fixture);

        assert_eq!(
            classify(
                &message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::CandidateFunding)
        );
    }

    #[test]
    fn splice_setup_channel_with_candidate_value_mismatch_rejects() {
        let fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        let mut message = setup_channel_message(&fixture);
        let Message::SetupChannel(request) = &mut message else {
            unreachable!();
        };
        request.channel_value += 1;

        assert_eq!(
            classify(
                &message,
                &fixture.pending,
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::CandidateMismatch)
        );
    }

    #[test]
    fn unrelated_setup_channel_remains_non_splice() {
        let fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        let message = setup_channel_message(&fixture);

        assert_eq!(
            classify(
                &message,
                &[],
                &State::new(),
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::NotSpliceRelated
        );
    }

    #[test]
    fn known_splice_check_outpoint_reaches_vls_boundary() {
        let mut fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        fixture
            .state
            .mark_splice_pending_lock(&fixture.node_channel_id_hex, 3)
            .unwrap();
        let session = fixture
            .state
            .get_splice_session(&fixture.node_channel_id_hex)
            .unwrap()
            .unwrap();
        let outpoint = session.cand.funding_outpoint.unwrap();
        let message = Message::CheckOutpoint(CheckOutpoint {
            funding_txid: Txid::from_str(&outpoint.txid).unwrap(),
            funding_txout: outpoint.vout as u16,
        });

        assert_eq!(
            classify(
                &message,
                &[],
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::OutpointBurial)
        );
    }

    #[test]
    fn known_splice_lock_outpoint_reaches_vls_boundary() {
        let mut fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        fixture
            .state
            .mark_splice_pending_lock(&fixture.node_channel_id_hex, 3)
            .unwrap();
        let session = fixture
            .state
            .get_splice_session(&fixture.node_channel_id_hex)
            .unwrap()
            .unwrap();
        let outpoint = session.cand.funding_outpoint.unwrap();
        let message = Message::LockOutpoint(LockOutpoint {
            funding_txid: Txid::from_str(&outpoint.txid).unwrap(),
            funding_txout: outpoint.vout as u16,
        });

        assert_eq!(
            classify(
                &message,
                &[],
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::RequiresVlsProof(VlsProof::OutpointLock)
        );
    }

    #[test]
    fn active_splice_with_unknown_outpoint_rejects() {
        let mut fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        fixture
            .state
            .mark_splice_pending_lock(&fixture.node_channel_id_hex, 3)
            .unwrap();
        let message = Message::CheckOutpoint(CheckOutpoint {
            funding_txid: Txid::from_str(&"99".repeat(32)).unwrap(),
            funding_txout: 0,
        });

        assert_eq!(
            classify(
                &message,
                &[],
                &fixture.state,
                &fixture.local_node_id,
                Some(&fixture.context),
            )
            .unwrap(),
            SplicePolicyDecision::Rejected(SplicePolicyViolation::UnknownOutpoint)
        );
    }

    #[test]
    fn unrelated_check_outpoint_remains_non_splice() {
        let fixture = splice_signing_fixture(SpliceOrigin::LocalInitiator, false, false);
        let message = Message::CheckOutpoint(CheckOutpoint {
            funding_txid: Txid::from_str(&"99".repeat(32)).unwrap(),
            funding_txout: 0,
        });

        assert_eq!(
            classify(&message, &[], &State::new(), &fixture.local_node_id, None,).unwrap(),
            SplicePolicyDecision::NotSpliceRelated
        );
    }
}
