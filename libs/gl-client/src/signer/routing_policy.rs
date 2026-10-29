//! The wrapper adds amount-dependent fee limits while preserving VLS’s payment accounting and avoiding a fork.
//! The tradeoff is delegation code we must maintain when upgrading VLS.
//! We want to move this logic into VLS and remove the wrapper in the future.
//!
//! VLS upgrade checklist: audit this delegation against every Validator method,
//! including provided methods. New trait defaults need not fail compilation and
//! can differ from the stock validator's behavior.

use super::fee_budget::FeeBudgetLimits;
use lightning_signer::bitcoin::{
    bip32::DerivationPath,
    block::Header as BlockHeader,
    hash_types::FilterHeader,
    secp256k1::{PublicKey, SecretKey},
    sighash::{EcdsaSighashType, SegwitV0Sighash},
    BlockHash, Network, OutPoint, ScriptBuf, Transaction,
};
use lightning_signer::channel::{ChannelId, ChannelSetup, ChannelSlot};
use lightning_signer::invoice::Invoice;
use lightning_signer::lightning::{
    ln::chan_utils::{ClosingTransaction, HTLCOutputInCommitment, TxCreationKeys},
    offers::invoice::UnsignedBolt12Invoice,
};
use lightning_signer::policy::{
    error::ValidationError,
    simple_validator::{SimplePolicy, SimpleValidatorFactory},
    validator::{ChainState, CommitmentSignatures, EnforcementState, Validator, ValidatorFactory},
    Policy,
};
use lightning_signer::signer::vls_channel_signer::VlsChannelSigner;
use lightning_signer::tx::tx::{CommitmentInfo, CommitmentInfo2};
use lightning_signer::txoo::proof::TxoProof;
use lightning_signer::wallet::Wallet;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone)]
pub(super) struct RoutingValidatorFactory {
    policy: SimplePolicy,
    limits: FeeBudgetLimits,
}

impl RoutingValidatorFactory {
    pub(super) fn new(mut policy: SimplePolicy, limits: FeeBudgetLimits) -> Self {
        policy.max_routing_fee_msat = limits.payment_fee_exempt_msat;
        Self { policy, limits }
    }

    fn stock(
        &self,
        network: Network,
        node_id: PublicKey,
        channel_id: Option<ChannelId>,
        fee: u64,
    ) -> Arc<dyn Validator> {
        let mut policy = self.policy.clone();
        policy.max_routing_fee_msat = fee;
        SimpleValidatorFactory::new_with_policy(policy).make_validator(network, node_id, channel_id)
    }
}

impl ValidatorFactory for RoutingValidatorFactory {
    fn make_validator(
        &self,
        network: Network,
        node_id: PublicKey,
        channel_id: Option<ChannelId>,
    ) -> Arc<dyn Validator> {
        Arc::new(RoutingValidator {
            inner: self.stock(
                network,
                node_id,
                channel_id.clone(),
                self.limits.payment_fee_exempt_msat,
            ),
            factory: self.clone(),
            network,
            node_id,
            channel_id,
        })
    }

    fn policy(&self, _network: Network) -> Box<dyn Policy> {
        Box::new(self.policy.clone())
    }
}

struct RoutingValidator {
    inner: Arc<dyn Validator>,
    factory: RoutingValidatorFactory,
    network: Network,
    node_id: PublicKey,
    channel_id: Option<ChannelId>,
}

// Forward every other method, including provided methods: their trait defaults
// can differ from SimpleValidator (notably enforce_balance and CLTV checks).
macro_rules! delegate_to_vls {
    ($(fn $name:ident(&self $(, $arg:ident: $ty:ty)* $(,)?) -> $result:ty;)*) => {
        $(fn $name(&self, $($arg: $ty),*) -> $result {
            self.inner.$name($($arg),*)
        })*
    };
}

impl Validator for RoutingValidator {
    fn validate_payment_balance(
        &self,
        incoming_msat: u64,
        outgoing_msat: u64,
        invoiced_amount_msat: Option<u64>,
    ) -> Result<(), ValidationError> {
        let Some(amount) = invoiced_amount_msat else {
            return self
                .inner
                .validate_payment_balance(incoming_msat, outgoing_msat, None);
        };
        let fee = match self.factory.limits.payment_fee_msat(amount) {
            Ok(fee) => fee,
            Err(e) => {
                return self
                    .inner
                    .policy()
                    .policy_error("policy-routing-balanced".into(), e.to_string())
            }
        };
        // VLS adds these u64 values internally. Reject overflow before dispatch.
        if incoming_msat
            .checked_add(amount)
            .and_then(|a| a.checked_add(fee))
            .is_none()
        {
            return self.inner.policy().policy_error(
                "policy-routing-balanced".into(),
                "payment balance overflow".into(),
            );
        }
        // Only this call gets the amount-dependent allowance. Principal, node
        // state, filters, and other validators remain untouched.
        self.factory
            .stock(self.network, self.node_id, self.channel_id.clone(), fee)
            .validate_payment_balance(incoming_msat, outgoing_msat, Some(amount))
    }

    delegate_to_vls! {
        fn validate_setup_channel(
            &self,
            wallet: &dyn Wallet,
            setup: &ChannelSetup,
            holder_shutdown_key_path: &DerivationPath,
        ) -> Result<(), ValidationError>;
        fn validate_channel_value(&self, setup: &ChannelSetup) -> Result<(), ValidationError>;
        fn validate_onchain_tx(
            &self,
            wallet: &dyn Wallet,
            channels: Vec<Option<Arc<Mutex<ChannelSlot>>>>,
            tx: &Transaction,
            segwit_flags: &[bool],
            values_sat: &[u64],
            opaths: &[DerivationPath],
            weight_lower_bound: usize,
        ) -> Result<u64, ValidationError>;
        fn decode_commitment_tx(
            &self,
            keys: &VlsChannelSigner,
            setup: &ChannelSetup,
            is_counterparty: bool,
            tx: &Transaction,
            output_witscripts: &[Vec<u8>],
        ) -> Result<CommitmentInfo, ValidationError>;
        fn validate_counterparty_commitment_tx(
            &self,
            estate: &EnforcementState,
            commit_num: u64,
            commitment_point: &PublicKey,
            setup: &ChannelSetup,
            cstate: &ChainState,
            info2: &CommitmentInfo2,
        ) -> Result<(), ValidationError>;
        fn validate_holder_commitment_tx(
            &self,
            estate: &EnforcementState,
            commit_num: u64,
            commitment_point: &PublicKey,
            setup: &ChannelSetup,
            cstate: &ChainState,
            info2: &CommitmentInfo2,
        ) -> Result<(), ValidationError>;
        fn validate_counterparty_revocation(
            &self,
            state: &EnforcementState,
            revoke_num: u64,
            commitment_secret: &SecretKey,
        ) -> Result<(), ValidationError>;
        fn decode_and_validate_htlc_tx(
            &self,
            is_counterparty: bool,
            setup: &ChannelSetup,
            txkeys: &TxCreationKeys,
            tx: &Transaction,
            redeemscript: &ScriptBuf,
            htlc_amount_sat: u64,
            output_witscript: &ScriptBuf,
        ) -> Result<(u32, HTLCOutputInCommitment, SegwitV0Sighash, EcdsaSighashType), ValidationError>;
        fn validate_htlc_tx(
            &self,
            setup: &ChannelSetup,
            cstate: &ChainState,
            is_counterparty: bool,
            htlc: &HTLCOutputInCommitment,
            feerate_per_kw: u32,
        ) -> Result<(), ValidationError>;
        fn decode_and_validate_mutual_close_tx(
            &self,
            wallet: &dyn Wallet,
            setup: &ChannelSetup,
            state: &EnforcementState,
            tx: &Transaction,
            opaths: &[DerivationPath],
        ) -> Result<ClosingTransaction, ValidationError>;
        fn validate_mutual_close_tx(
            &self,
            wallet: &dyn Wallet,
            setup: &ChannelSetup,
            state: &EnforcementState,
            to_holder_value_sat: u64,
            to_counterparty_value_sat: u64,
            holder_shutdown_script: &Option<ScriptBuf>,
            counterparty_shutdown_script: &Option<ScriptBuf>,
            holder_wallet_path_hint: &DerivationPath,
        ) -> Result<(), ValidationError>;
        fn validate_delayed_sweep(
            &self,
            wallet: &dyn Wallet,
            setup: &ChannelSetup,
            cstate: &ChainState,
            tx: &Transaction,
            input: usize,
            amount_sat: u64,
            key_path: &DerivationPath,
        ) -> Result<(), ValidationError>;
        fn validate_counterparty_htlc_sweep(
            &self,
            wallet: &dyn Wallet,
            setup: &ChannelSetup,
            cstate: &ChainState,
            tx: &Transaction,
            redeemscript: &ScriptBuf,
            input: usize,
            amount_sat: u64,
            key_path: &DerivationPath,
        ) -> Result<(), ValidationError>;
        fn validate_justice_sweep(
            &self,
            wallet: &dyn Wallet,
            setup: &ChannelSetup,
            cstate: &ChainState,
            tx: &Transaction,
            input: usize,
            amount_sat: u64,
            key_path: &DerivationPath,
        ) -> Result<(), ValidationError>;
        fn validate_payment_cltv(
            &self,
            incoming_cltv: u32,
            outgoing_cltv: u32,
        ) -> Result<(), ValidationError>;
        fn enforce_balance(&self) -> bool;
        fn minimum_initial_balance(&self, holder_value_msat: u64) -> u64;
        fn is_ready(&self, cstate: &ChainState) -> bool;
        fn policy(&self) -> Box<&dyn Policy>;
        fn set_next_holder_commit_num(
            &self,
            estate: &mut EnforcementState,
            num: u64,
            current_commitment_info: CommitmentInfo2,
            counterparty_signatures: CommitmentSignatures,
        ) -> Result<(), ValidationError>;
        fn get_current_holder_commitment_info(
            &self,
            estate: &mut EnforcementState,
            commitment_number: u64,
        ) -> Result<CommitmentInfo2, ValidationError>;
        fn set_next_counterparty_commit_num(
            &self,
            estate: &mut EnforcementState,
            num: u64,
            current_point: PublicKey,
            current_commitment_info: CommitmentInfo2,
        ) -> Result<(), ValidationError>;
        fn set_next_counterparty_revoke_num(
            &self,
            estate: &mut EnforcementState,
            num: u64,
        ) -> Result<(), ValidationError>;
        fn validate_block(
            &self,
            proof: &TxoProof,
            height: u32,
            header: &BlockHeader,
            external_block_hash: Option<&BlockHash>,
            prev_filter_header: &FilterHeader,
            outpoint_watches: &[OutPoint],
            trusted_oracle_pubkeys: &Vec<PublicKey>,
        ) -> Result<(), ValidationError>;
        fn validate_invoice(&self, invoice: &Invoice, now: Duration) -> Result<(), ValidationError>;
        fn validate_bolt12_invoice_unsigned(
            &self,
            invoice: &UnsignedBolt12Invoice,
            now: Duration,
            bolt12_pubkey: PublicKey,
        ) -> Result<(), ValidationError>;
    }
}
