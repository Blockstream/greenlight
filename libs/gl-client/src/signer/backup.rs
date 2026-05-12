pub use crate::persist::PeerEntry;
use crate::persist::State;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::fs;
use std::io::Write;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

const BACKUP_VERSION: u32 = 1;
const NODE_ID_LEN: usize = 33;
const PEER_ID_LEN: usize = 33;
const CLN_DBID_LEN: usize = 8;
const CLN_CHANNEL_KEY_LEN: usize = PEER_ID_LEN + CLN_DBID_LEN;
const VLS_CHANNEL_KEY_LEN: usize = NODE_ID_LEN + CLN_CHANNEL_KEY_LEN;
const TXID_LEN: usize = 32;
const PUBKEY_LEN: usize = 33;
const SHACHAIN_SECRET_LEN: usize = 32;
const SHACHAIN_EMPTY_INDEX: u64 = 1 << 48;
const SHACHAIN_MAX_ENTRIES: usize = 49;
const SHACHAIN_MISSING_WARNING: &str = "shachain_tlv_missing";
const CHANNEL_PREFIX: &str = "channels/";
const PEER_PREFIX: &str = "peers/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignerBackupConfig {
    pub path: PathBuf,
    pub strategy: SignerBackupStrategy,
}

impl SignerBackupConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            strategy: SignerBackupStrategy::NewChannelsOnly,
        }
    }

    pub fn periodic(path: impl Into<PathBuf>, updates: u32) -> Result<Self> {
        let config = Self {
            path: path.into(),
            strategy: SignerBackupStrategy::Periodic { updates },
        };
        config.validate()?;
        Ok(config)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.strategy.validate()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignerBackupStrategy {
    NewChannelsOnly,
    Periodic { updates: u32 },
}

impl SignerBackupStrategy {
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            SignerBackupStrategy::NewChannelsOnly => Ok(()),
            SignerBackupStrategy::Periodic { updates: 0 } => {
                Err(anyhow!("periodic signer backup updates must be greater than zero"))
            }
            SignerBackupStrategy::Periodic { .. } => Ok(()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignerBackupSnapshot {
    pub version: u32,
    pub created_at: String,
    pub node_id: String,
    pub strategy: SignerBackupStrategy,
    pub state: State,
}

impl SignerBackupSnapshot {
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path)
            .with_context(|| format!("reading signer backup {}", path.display()))?;
        let snapshot: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing signer backup {}", path.display()))?;
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn recovery_data(&self) -> Result<Vec<RecoverableChannel>> {
        self.validate()?;

        let peers = peers_from_state(&self.state)?
            .into_iter()
            .map(|peer| (peer.peer_id.clone(), peer))
            .collect::<BTreeMap<_, _>>();

        recoverable_channel_values(&self.state)
            .into_iter()
            .map(|(channel_key, value)| {
                let peer_id = peer_id_from_channel_key(&channel_key)?;
                let entry: ChannelEntry = serde_json::from_value(value)
                    .with_context(|| format!("parsing recoverable channel {}", channel_key))?;
                let setup = entry.channel_setup.ok_or_else(|| {
                    anyhow!("recoverable channel {} is missing channel_setup", channel_key)
                })?;
                let peer_addr = peers
                    .get(&peer_id)
                    .and_then(|peer| (!peer.addr.is_empty()).then(|| peer.addr.clone()));
                let mut warnings = Vec::new();
                if peer_addr.is_none() {
                    warnings.push("missing_peer_addr".to_string());
                }

                Ok(RecoverableChannel {
                    channel_key,
                    peer_id,
                    peer_addr,
                    complete: warnings.is_empty(),
                    warnings,
                    funding_outpoint: setup.funding_outpoint,
                    funding_sats: setup.channel_value_sat,
                    remote_basepoints: setup.counterparty_points,
                    opener: if setup.is_outbound {
                        RecoverableChannelOpener::Local
                    } else {
                        RecoverableChannelOpener::Remote
                    },
                    remote_to_self_delay: setup.counterparty_selected_contest_delay,
                    commitment_type: setup.commitment_type,
                })
            })
            .collect()
    }

    /// Converts the snapshot's recoverable channels into a CLN-compatible backup format suitable for `recoverchannel` import.
    /// If `options.skip_incomplete` is true, channels with missing peer addresses will be skipped and
    /// included in the `skipped` list with warnings instead of causing an error.
    pub fn to_cln_backup(
        &self,
        options: CLNBackupOptions,
    ) -> Result<CLNBackup> {
        let recovery_data = self.recovery_data()?;
        let shachains = self.recoverable_channel_shachains()?;
        let total_channels = recovery_data.len();
        let mut request = RecoverchannelRequest { scb: vec![] };
        let mut channels = Vec::new();
        let mut skipped = Vec::new();

        for channel in recovery_data {
            if !channel.complete {
                if options.skip_incomplete {
                    skipped.push(RecoverchannelSkippedChannel {
                        channel_key: channel.channel_key,
                        peer_id: channel.peer_id,
                        warnings: channel.warnings,
                    });
                    continue;
                }

                return Err(anyhow!(
                    "channel {} is incomplete: {}",
                    channel.channel_key,
                    channel.warnings.join(",")
                ));
            }

            let old_secrets = shachains
                .get(&channel.channel_key)
                .ok_or_else(|| {
                    anyhow!("missing shachain state for channel {}", channel.channel_key)
                })?
                .as_deref();
            let encoded = encode_recoverchannel_scb(&channel, old_secrets).map_err(|e| {
                anyhow!(
                    "encoding recoverchannel SCB for {}: {}",
                    channel.channel_key,
                    e
                )
            })?;
            request.scb.push(encoded.scb.clone());
            channels.push(CLNBackupChannel {
                channel_key: channel.channel_key,
                peer_id: channel.peer_id,
                peer_addr: channel.peer_addr.expect("complete channel has peer_addr"),
                funding_outpoint: channel.funding_outpoint,
                cln_dbid: encoded.cln_dbid,
                channel_id: encoded.channel_id,
                scb: encoded.scb,
                warnings: encoded.warnings,
            });
        }

        if request.scb.is_empty() {
            return Err(anyhow!(
                "no complete recoverable channels available for recoverchannel export"
            ));
        }

        Ok(CLNBackup {
            request,
            total_channels,
            exported_channels: channels.len(),
            skipped_channels: skipped.len(),
            channels,
            skipped,
        })
    }

    fn validate(&self) -> Result<()> {
        if self.version != BACKUP_VERSION {
            return Err(anyhow!(
                "unsupported signer backup version {}; expected {}",
                self.version,
                BACKUP_VERSION
            ));
        }

        self.strategy.validate()
    }

    fn recoverable_channel_shachains(
        &self,
    ) -> Result<BTreeMap<String, Option<Vec<CounterpartySecret>>>> {
        recoverable_channel_values(&self.state)
            .into_iter()
            .map(|(channel_key, value)| {
                let entry: ClnChannelEntry = serde_json::from_value(value).with_context(|| {
                    format!(
                        "parsing CLN shachain state for recoverable channel {}",
                        channel_key
                    )
                })?;
                Ok((
                    channel_key,
                    entry
                        .enforcement_state
                        .and_then(|state| state.counterparty_secrets)
                        .map(|secrets| secrets.old_secrets),
                ))
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverableChannel {
    pub channel_key: String,
    pub peer_id: String,
    pub peer_addr: Option<String>,
    pub complete: bool,
    pub warnings: Vec<String>,
    pub funding_outpoint: RecoverableFundingOutpoint,
    pub funding_sats: u64,
    pub remote_basepoints: RecoverableBasepoints,
    pub opener: RecoverableChannelOpener,
    pub remote_to_self_delay: u64,
    pub commitment_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverableFundingOutpoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverableBasepoints {
    pub delayed_payment_basepoint: String,
    pub funding_pubkey: String,
    pub htlc_basepoint: String,
    pub payment_point: String,
    pub revocation_basepoint: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoverableChannelOpener {
    Local,
    Remote,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CLNBackupOptions {
    pub skip_incomplete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverchannelRequest {
    pub scb: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CLNBackup {
    pub request: RecoverchannelRequest,
    pub total_channels: usize,
    pub exported_channels: usize,
    pub skipped_channels: usize,
    pub channels: Vec<CLNBackupChannel>,
    pub skipped: Vec<RecoverchannelSkippedChannel>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CLNBackupChannel {
    pub channel_key: String,
    pub peer_id: String,
    pub peer_addr: String,
    pub funding_outpoint: RecoverableFundingOutpoint,
    pub cln_dbid: u64,
    pub channel_id: String,
    pub scb: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverchannelSkippedChannel {
    pub channel_key: String,
    pub peer_id: String,
    pub warnings: Vec<String>,
}

#[derive(Deserialize)]
struct ChannelEntry {
    channel_setup: Option<ChannelSetup>,
}

#[derive(Deserialize)]
struct ClnChannelEntry {
    enforcement_state: Option<ChannelEnforcementState>,
}

#[derive(Deserialize)]
struct ChannelEnforcementState {
    counterparty_secrets: Option<CounterpartySecrets>,
}

#[derive(Deserialize)]
struct CounterpartySecrets {
    old_secrets: Vec<CounterpartySecret>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct CounterpartySecret(Vec<u8>, u64);

#[derive(Deserialize)]
struct ChannelSetup {
    channel_value_sat: u64,
    commitment_type: String,
    counterparty_points: RecoverableBasepoints,
    counterparty_selected_contest_delay: u64,
    funding_outpoint: RecoverableFundingOutpoint,
    is_outbound: bool,
}

#[derive(Default)]
pub(crate) struct BackupRuntime {
    updates_since_backup: u32,
    snapshot_pending: bool,
    last_backed_state: Option<State>,
}

impl BackupRuntime {
    pub(crate) fn observe(
        &mut self,
        strategy: SignerBackupStrategy,
        before: &State,
        after: &State,
    ) -> bool {
        if should_snapshot_new_channels(before, after) {
            self.snapshot_pending = true;
        }

        if let SignerBackupStrategy::Periodic { updates } = strategy {
            if has_recoverable_state_update(before, after) {
                self.updates_since_backup = self.updates_since_backup.saturating_add(1);
                if updates > 0 && self.updates_since_backup >= updates {
                    self.snapshot_pending = true;
                }
            }
        }

        self.snapshot_pending && self.has_unbacked_recoverable_state(after)
    }

    pub(crate) fn snapshot_succeeded(&mut self, state: &State) {
        self.updates_since_backup = 0;
        self.snapshot_pending = false;
        self.last_backed_state = Some(state.clone());
    }

    fn has_unbacked_recoverable_state(&self, state: &State) -> bool {
        self.last_backed_state
            .as_ref()
            .map(|last| has_recoverable_state_update(last, state))
            .unwrap_or(true)
    }
}

pub(crate) fn should_snapshot_new_channels(before: &State, after: &State) -> bool {
    let before_channels = recoverable_channel_keys(before);
    recoverable_channel_keys(after)
        .iter()
        .any(|key| !before_channels.contains(key))
}

fn has_recoverable_state_update(before: &State, after: &State) -> bool {
    !recoverable_channel_keys(&before.diff_state(after)).is_empty()
}

pub(crate) fn write_snapshot(
    config: &SignerBackupConfig,
    node_id: &[u8],
    state: State,
) -> Result<()> {
    let snapshot = SignerBackupSnapshot {
        version: BACKUP_VERSION,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        node_id: hex::encode(node_id),
        strategy: config.strategy,
        state,
    };

    let dir = backup_dir(&config.path);
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .with_context(|| format!("creating temporary backup file in {}", dir.display()))?;

    serde_json::to_writer_pretty(&mut tmp, &snapshot)
        .with_context(|| format!("serializing backup snapshot for {}", config.path.display()))?;
    tmp.write_all(b"\n")
        .with_context(|| format!("finalizing backup snapshot for {}", config.path.display()))?;
    tmp.as_file_mut()
        .sync_all()
        .with_context(|| format!("syncing backup snapshot for {}", config.path.display()))?;
    tmp.persist(&config.path).map_err(|e| {
        anyhow!(
            "persisting backup snapshot to {}: {}",
            config.path.display(),
            e
        )
    })?;

    Ok(())
}

fn backup_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn recoverable_channel_entries(
    state: &State,
) -> impl Iterator<Item = (&str, &serde_json::Value)> + '_ {
    state.live_values().filter(|(key, value)| {
        key.starts_with(CHANNEL_PREFIX)
            && value
                .get("channel_setup")
                .map(|setup| !setup.is_null())
                .unwrap_or(false)
    })
}

fn recoverable_channel_keys(state: &State) -> BTreeSet<String> {
    recoverable_channel_entries(state)
        .map(|(key, _)| key.to_string())
        .collect()
}

fn recoverable_channel_values(state: &State) -> Vec<(String, serde_json::Value)> {
    recoverable_channel_entries(state)
        .map(|(key, value)| (key.to_string(), value.clone()))
        .collect()
}

fn peer_values(state: &State) -> Vec<(String, serde_json::Value)> {
    state
        .live_values()
        .filter(|(key, _)| key.starts_with(PEER_PREFIX))
        .map(|(key, value)| (key.to_string(), value.clone()))
        .collect()
}

fn peers_from_state(state: &State) -> Result<Vec<PeerEntry>> {
    let mut peers = peer_values(state)
        .into_iter()
        .map(|(key, value)| peer_from_state_value(&key, value))
        .collect::<Result<Vec<_>>>()?;
    peers.sort_by(|a, b| a.peer_id.cmp(&b.peer_id));
    Ok(peers)
}

fn peer_from_state_value(key: &str, value: serde_json::Value) -> Result<PeerEntry> {
    let key_peer_id = key
        .strip_prefix("peers/")
        .ok_or_else(|| anyhow!("invalid peer state key: {}", key))?;
    let peer: PeerEntry =
        serde_json::from_value(value).with_context(|| format!("parsing peer state {}", key))?;
    if peer.peer_id != key_peer_id {
        return Err(anyhow!(
            "peer state key {} does not match payload peer_id {}",
            key_peer_id,
            peer.peer_id
        ));
    }
    Ok(peer)
}

fn peer_id_from_channel_key(channel_key: &str) -> Result<String> {
    let encoded = channel_key
        .strip_prefix("channels/")
        .ok_or_else(|| anyhow!("invalid channel key prefix: {}", channel_key))?;
    let raw = hex::decode(encoded)
        .with_context(|| format!("decoding channel key {}", channel_key))?;
    let channel_id = raw
        .get(33..)
        .ok_or_else(|| anyhow!("channel key {} is missing node id prefix", channel_key))?;

    if channel_id.len() < 41 {
        return Err(anyhow!(
            "channel key {} does not contain a CLN-style peer id",
            channel_key
        ));
    }

    Ok(hex::encode(&channel_id[..33]))
}

struct EncodedRecoverchannelScb {
    cln_dbid: u64,
    channel_id: String,
    scb: String,
    warnings: Vec<String>,
}

struct ClnChannelKey {
    peer_id: [u8; PEER_ID_LEN],
    dbid: u64,
}

fn encode_recoverchannel_scb(
    channel: &RecoverableChannel,
    old_secrets: Option<&[CounterpartySecret]>,
) -> Result<EncodedRecoverchannelScb> {
    let channel_key = decode_cln_channel_key(&channel.channel_key)?;
    let expected_peer_id = decode_hex_array::<PEER_ID_LEN>("peer_id", &channel.peer_id)?;
    if channel_key.peer_id != expected_peer_id {
        return Err(anyhow!(
            "channel key peer id {} does not match channel peer id {}",
            hex::encode(channel_key.peer_id),
            channel.peer_id
        ));
    }

    let txid = decode_txid_for_cln_wire(&channel.funding_outpoint.txid)?;
    let channel_id = derive_v1_channel_id(txid, channel.funding_outpoint.vout);
    let wireaddr = encode_wireaddr(
        channel
            .peer_addr
            .as_deref()
            .ok_or_else(|| anyhow!("missing peer address"))?,
    )?;
    let channel_type = encode_channel_type(&channel.commitment_type)?;
    let (tlvs, warnings) = encode_scb_tlvs(channel, old_secrets)?;

    let mut scb = Vec::new();
    put_u64(&mut scb, channel_key.dbid);
    scb.extend(channel_id);
    scb.extend(channel_key.peer_id);
    scb.extend(wireaddr);
    scb.extend(txid);
    put_u32(&mut scb, channel.funding_outpoint.vout);
    put_u64(&mut scb, channel.funding_sats);
    put_u16(&mut scb, channel_type.len().try_into()?);
    scb.extend(channel_type);
    put_u32(&mut scb, tlvs.len().try_into()?);
    scb.extend(tlvs);

    Ok(EncodedRecoverchannelScb {
        cln_dbid: channel_key.dbid,
        channel_id: hex::encode(channel_id),
        scb: hex::encode(scb),
        warnings,
    })
}

fn decode_cln_channel_key(channel_key: &str) -> Result<ClnChannelKey> {
    let encoded = channel_key
        .strip_prefix("channels/")
        .ok_or_else(|| anyhow!("invalid channel key prefix: {}", channel_key))?;
    let raw = hex::decode(encoded)
        .with_context(|| format!("decoding channel key {}", channel_key))?;

    if raw.len() != VLS_CHANNEL_KEY_LEN {
        return Err(anyhow!(
            "channel key {} is not a CLN-style VLS channel key: expected {} bytes, got {}",
            channel_key,
            VLS_CHANNEL_KEY_LEN,
            raw.len()
        ));
    }

    let peer_id = raw[NODE_ID_LEN..NODE_ID_LEN + PEER_ID_LEN]
        .try_into()
        .expect("slice length checked");
    let dbid = u64::from_le_bytes(
        raw[NODE_ID_LEN + PEER_ID_LEN..]
            .try_into()
            .expect("slice length checked"),
    );

    Ok(ClnChannelKey { peer_id, dbid })
}

fn decode_txid_for_cln_wire(txid: &str) -> Result<[u8; TXID_LEN]> {
    decode_hex_array::<TXID_LEN>("funding txid", txid)
}

fn derive_v1_channel_id(txid: [u8; TXID_LEN], vout: u32) -> [u8; TXID_LEN] {
    let mut channel_id = txid;
    channel_id[TXID_LEN - 2] ^= (vout >> 8) as u8;
    channel_id[TXID_LEN - 1] ^= vout as u8;
    channel_id
}

fn encode_channel_type(commitment_type: &str) -> Result<Vec<u8>> {
    let commitment_type = match commitment_type {
        "Legacy" => lightning_signer::channel::CommitmentType::Legacy,
        "StaticRemoteKey" => lightning_signer::channel::CommitmentType::StaticRemoteKey,
        "Anchors" => lightning_signer::channel::CommitmentType::Anchors,
        "AnchorsZeroFeeHtlc" => lightning_signer::channel::CommitmentType::AnchorsZeroFeeHtlc,
        unknown => return Err(anyhow!("unsupported commitment type {}", unknown)),
    };
    if commitment_type == lightning_signer::channel::CommitmentType::Legacy {
        return Ok(Vec::new());
    }

    Ok(vls_protocol_signer::util::commitment_type_to_channel_type(
        commitment_type,
    ))
}

fn encode_scb_tlvs(
    channel: &RecoverableChannel,
    old_secrets: Option<&[CounterpartySecret]>,
) -> Result<(Vec<u8>, Vec<String>)> {
    let mut tlvs = Vec::new();
    let mut warnings = Vec::new();

    match old_secrets {
        Some(old_secrets) => {
            if let Some(shachain) = encode_shachain(old_secrets)? {
                put_tlv(&mut tlvs, 1, &shachain);
            }
        }
        None => warnings.push(SHACHAIN_MISSING_WARNING.to_string()),
    }

    let mut basepoints = Vec::new();
    basepoints.extend(decode_hex_array::<PUBKEY_LEN>(
        "revocation basepoint",
        &channel.remote_basepoints.revocation_basepoint,
    )?);
    basepoints.extend(decode_hex_array::<PUBKEY_LEN>(
        "payment basepoint",
        &channel.remote_basepoints.payment_point,
    )?);
    basepoints.extend(decode_hex_array::<PUBKEY_LEN>(
        "htlc basepoint",
        &channel.remote_basepoints.htlc_basepoint,
    )?);
    basepoints.extend(decode_hex_array::<PUBKEY_LEN>(
        "delayed payment basepoint",
        &channel.remote_basepoints.delayed_payment_basepoint,
    )?);
    put_tlv(&mut tlvs, 3, &basepoints);

    let opener = match channel.opener {
        RecoverableChannelOpener::Local => 0,
        RecoverableChannelOpener::Remote => 1,
    };
    put_tlv(&mut tlvs, 5, &[opener]);

    let remote_to_self_delay: u16 = channel.remote_to_self_delay.try_into().with_context(|| {
        format!(
            "remote_to_self_delay {} does not fit in CLN SCB u16",
            channel.remote_to_self_delay
        )
    })?;
    let mut delay = Vec::new();
    put_u16(&mut delay, remote_to_self_delay);
    put_tlv(&mut tlvs, 7, &delay);

    Ok((tlvs, warnings))
}

fn encode_shachain(old_secrets: &[CounterpartySecret]) -> Result<Option<Vec<u8>>> {
    if old_secrets.len() > SHACHAIN_MAX_ENTRIES {
        return Err(anyhow!(
            "shachain has {} entries, maximum is {}",
            old_secrets.len(),
            SHACHAIN_MAX_ENTRIES
        ));
    }

    let mut known = Vec::new();
    let mut trailing_dummy_start = None;

    for (position, secret) in old_secrets.iter().enumerate() {
        if is_dummy_shachain_secret(secret) {
            trailing_dummy_start = Some(position);
            break;
        }

        if secret.0.len() != SHACHAIN_SECRET_LEN {
            return Err(anyhow!(
                "shachain secret at position {} must be {} bytes, got {}",
                position,
                SHACHAIN_SECRET_LEN,
                secret.0.len()
            ));
        }

        let expected_position = shachain_position(secret.1)?;
        if expected_position != position {
            return Err(anyhow!(
                "shachain secret index {} belongs at position {}, found at position {}",
                secret.1,
                expected_position,
                position
            ));
        }

        known.push(secret);
    }

    if let Some(start) = trailing_dummy_start {
        for (position, secret) in old_secrets.iter().enumerate().skip(start) {
            if !is_dummy_shachain_secret(secret) {
                return Err(anyhow!(
                    "missing shachain position {} before real secret at position {}",
                    start,
                    position
                ));
            }
        }
    }

    if known.is_empty() {
        return Ok(None);
    }

    let min_index = known
        .iter()
        .map(|secret| secret.1)
        .min()
        .expect("known is not empty");
    let mut shachain = Vec::new();
    put_u64(&mut shachain, min_index);
    put_u32(&mut shachain, known.len().try_into()?);
    for secret in known {
        put_u64(&mut shachain, secret.1);
        shachain.extend(&secret.0);
    }

    Ok(Some(shachain))
}

fn is_dummy_shachain_secret(secret: &CounterpartySecret) -> bool {
    secret.1 == SHACHAIN_EMPTY_INDEX && secret.0.iter().all(|byte| *byte == 0)
}

fn shachain_position(index: u64) -> Result<usize> {
    if index >= SHACHAIN_EMPTY_INDEX {
        return Err(anyhow!("invalid shachain index {}", index));
    }

    for position in 0..48 {
        if index & (1u64 << position) == (1u64 << position) {
            return Ok(position);
        }
    }

    Ok(48)
}

fn encode_wireaddr(addr: &str) -> Result<Vec<u8>> {
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return Ok(encode_socket_addr(socket_addr));
    }

    let (host, port) = addr
        .rsplit_once(':')
        .ok_or_else(|| anyhow!("peer address {} is missing port", addr))?;
    let port = port
        .parse::<u16>()
        .with_context(|| format!("invalid peer address port in {}", addr))?;
    if host.contains(':') {
        return Err(anyhow!(
            "IPv6 peer address {} must use [addr]:port form",
            addr
        ));
    }

    if let Some(onion) = host.strip_suffix(".onion") {
        let onion = decode_tor_v3_onion(onion)
            .with_context(|| format!("invalid Tor v3 peer address {}", addr))?;
        let mut wire = vec![4];
        wire.extend(onion);
        put_u16(&mut wire, port);
        return Ok(wire);
    }

    let host = host.as_bytes();
    if host.is_empty() || host.len() > u8::MAX as usize {
        return Err(anyhow!(
            "DNS peer address host length is invalid in {}",
            addr
        ));
    }

    let mut wire = vec![5, host.len() as u8];
    wire.extend(host);
    put_u16(&mut wire, port);
    Ok(wire)
}

fn encode_socket_addr(addr: SocketAddr) -> Vec<u8> {
    let mut wire = Vec::new();
    match addr.ip() {
        IpAddr::V4(ip) => {
            wire.push(1);
            wire.extend(ip.octets());
        }
        IpAddr::V6(ip) => {
            wire.push(2);
            wire.extend(ip.octets());
        }
    }
    put_u16(&mut wire, addr.port());
    wire
}

fn decode_tor_v3_onion(host: &str) -> Result<[u8; 35]> {
    if host.len() != 56 {
        return Err(anyhow!(
            "Tor v3 onion host must be 56 base32 characters, got {}",
            host.len()
        ));
    }

    let mut bits: u16 = 0;
    let mut bit_count: u8 = 0;
    let mut out = Vec::with_capacity(35);

    for byte in host.bytes() {
        let value = match byte {
            b'a'..=b'z' => byte - b'a',
            b'A'..=b'Z' => byte - b'A',
            b'2'..=b'7' => byte - b'2' + 26,
            _ => return Err(anyhow!("invalid base32 character {}", byte as char)),
        };
        bits = (bits << 5) | u16::from(value);
        bit_count += 5;
        while bit_count >= 8 {
            bit_count -= 8;
            out.push((bits >> bit_count) as u8);
            bits &= (1u16 << bit_count) - 1;
        }
    }

    if bit_count != 0 || out.len() != 35 {
        return Err(anyhow!("invalid Tor v3 onion base32 length"));
    }

    Ok(out.try_into().expect("length checked"))
}

fn decode_hex_array<const N: usize>(label: &str, value: &str) -> Result<[u8; N]> {
    let bytes = hex::decode(value).with_context(|| format!("decoding {}", label))?;
    if bytes.len() != N {
        return Err(anyhow!(
            "{} must be {} bytes, got {}",
            label,
            N,
            bytes.len()
        ));
    }

    Ok(bytes.try_into().expect("length checked"))
}

fn put_tlv(out: &mut Vec<u8>, typ: u64, value: &[u8]) {
    put_bigsize(out, typ);
    put_bigsize(out, value.len() as u64);
    out.extend(value);
}

fn put_bigsize(out: &mut Vec<u8>, value: u64) {
    if value < 0xfd {
        out.push(value as u8);
    } else if value <= 0xffff {
        out.push(0xfd);
        put_u16(out, value as u16);
    } else if value <= 0xffff_ffff {
        out.push(0xfe);
        put_u32(out, value as u32);
    } else {
        out.push(0xff);
        put_u64(out, value);
    }
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend(value.to_be_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_be_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend(value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn state(entries: serde_json::Value) -> State {
        serde_json::from_value(json!({ "values": entries })).unwrap()
    }

    fn channel(setup: serde_json::Value) -> serde_json::Value {
        channel_with_enforcement(setup, json!({}))
    }

    fn channel_with_enforcement(
        setup: serde_json::Value,
        enforcement_state: serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "channel_setup": setup,
            "channel_value_satoshis": 1000,
            "id": null,
            "enforcement_state": enforcement_state,
            "blockheight": null
        })
    }

    fn enforcement_with_old_secrets(old_secrets: serde_json::Value) -> serde_json::Value {
        json!({
            "counterparty_secrets": {
                "old_secrets": old_secrets
            }
        })
    }

    fn old_secret(byte: u8, index: u64) -> serde_json::Value {
        json!([vec![byte; SHACHAIN_SECRET_LEN], index])
    }

    fn malformed_old_secret(secret: Vec<u8>, index: u64) -> serde_json::Value {
        json!([secret, index])
    }

    fn dummy_old_secret() -> serde_json::Value {
        json!([vec![0u8; SHACHAIN_SECRET_LEN], SHACHAIN_EMPTY_INDEX])
    }

    fn peer_entry(peer_id: &str, addr: &str) -> serde_json::Value {
        json!({
            "peer_id": peer_id,
            "addr": addr,
            "direction": "out",
            "features": ""
        })
    }

    fn state_with_peers(mut entries: serde_json::Value, peers: &[(&str, &str)]) -> State {
        let entries = entries.as_object_mut().expect("state entries object");
        for (peer_id, addr) in peers {
            entries.insert(format!("peers/{peer_id}"), json!([0, peer_entry(peer_id, addr)]));
        }
        state(serde_json::Value::Object(entries.clone()))
    }

    fn backup_snapshot(state: State) -> SignerBackupSnapshot {
        SignerBackupSnapshot {
            version: BACKUP_VERSION,
            created_at: "2026-04-29T00:00:00Z".to_string(),
            node_id: hex::encode([2u8; 33]),
            strategy: SignerBackupStrategy::NewChannelsOnly,
            state,
        }
    }

    fn peer_id(byte: u8) -> String {
        let mut bytes = vec![byte; 33];
        bytes[0] = 2;
        hex::encode(bytes)
    }

    fn channel_key(peer_id: &str, oid: u64) -> String {
        let mut raw = vec![3u8; 33];
        raw.extend(hex::decode(peer_id).unwrap());
        raw.extend(oid.to_le_bytes());
        format!("channels/{}", hex::encode(raw))
    }

    fn recovery_setup(txid: &str, vout: u32, sats: u64, is_outbound: bool) -> serde_json::Value {
        json!({
            "channel_value_sat": sats,
            "commitment_type": "AnchorsZeroFeeHtlc",
            "counterparty_points": {
                "delayed_payment_basepoint": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "funding_pubkey": "02bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "htlc_basepoint": "02cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "payment_point": "02dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                "revocation_basepoint": "02eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            },
            "counterparty_selected_contest_delay": 144,
            "counterparty_shutdown_script": null,
            "funding_outpoint": {
                "txid": txid,
                "vout": vout
            },
            "holder_selected_contest_delay": 144,
            "holder_shutdown_script": null,
            "is_outbound": is_outbound,
            "push_value_msat": 0
        })
    }

    fn full_txid() -> &'static str {
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    }

    fn expected_cln_txid(txid: &str) -> [u8; 32] {
        hex::decode(txid).unwrap().try_into().unwrap()
    }

    fn scb_tlvs(scb: &[u8]) -> &[u8] {
        let channel_type_len = u16::from_be_bytes(scb[124..126].try_into().unwrap()) as usize;
        let tlv_len_offset = 126 + channel_type_len;
        let tlv_len =
            u32::from_be_bytes(scb[tlv_len_offset..tlv_len_offset + 4].try_into().unwrap())
                as usize;
        let tlvs = &scb[tlv_len_offset + 4..];
        assert_eq!(tlvs.len(), tlv_len);
        tlvs
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    fn read_backup_err(path: &Path) -> String {
        SignerBackupSnapshot::read(path).err().unwrap().to_string()
    }

    #[test]
    fn new_channel_trigger_requires_channel_setup() {
        let empty = state(json!({}));
        let stub = state(json!({
            "channels/a": [0, channel(serde_json::Value::Null)]
        }));
        let ready = state(json!({
            "channels/a": [1, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let ready_updated = state(json!({
            "channels/a": [2, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let second_ready = state(json!({
            "channels/a": [2, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))],
            "channels/b": [0, channel(json!({ "funding_outpoint": { "txid": "11", "vout": 1 } }))]
        }));

        assert!(!should_snapshot_new_channels(&empty, &empty));
        assert!(!should_snapshot_new_channels(&empty, &stub));
        assert!(should_snapshot_new_channels(&stub, &ready));
        assert!(!should_snapshot_new_channels(&ready, &ready_updated));
        assert!(should_snapshot_new_channels(&ready_updated, &second_ready));
    }

    #[test]
    fn runtime_retries_new_channel_snapshot_until_success() {
        let stub = state(json!({
            "channels/a": [0, channel(serde_json::Value::Null)]
        }));
        let ready = state(json!({
            "channels/a": [1, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let mut runtime = BackupRuntime::default();

        assert!(runtime.observe(SignerBackupStrategy::NewChannelsOnly, &stub, &ready));
        assert!(runtime.observe(SignerBackupStrategy::NewChannelsOnly, &ready, &ready));

        runtime.snapshot_succeeded(&ready);

        assert!(!runtime.observe(SignerBackupStrategy::NewChannelsOnly, &ready, &ready));
    }

    #[test]
    fn periodic_trigger_counts_recoverable_channel_updates() {
        let ready = state(json!({
            "channels/a": [1, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let updated_once = state(json!({
            "channels/a": [2, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let updated_twice = state(json!({
            "channels/a": [3, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let mut runtime = BackupRuntime::default();
        let strategy = SignerBackupStrategy::Periodic { updates: 2 };
        runtime.snapshot_succeeded(&ready);

        assert!(!runtime.observe(strategy, &ready, &updated_once));
        assert!(runtime.observe(strategy, &updated_once, &updated_twice));
        assert!(runtime.observe(strategy, &updated_twice, &updated_twice));

        runtime.snapshot_succeeded(&updated_twice);

        assert!(!runtime.observe(strategy, &updated_twice, &updated_twice));
    }

    #[test]
    fn periodic_trigger_writes_new_channels_immediately() {
        let ready = state(json!({
            "channels/a": [1, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))]
        }));
        let second_ready = state(json!({
            "channels/a": [1, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))],
            "channels/b": [1, channel(json!({ "funding_outpoint": { "txid": "11", "vout": 1 } }))]
        }));
        let mut runtime = BackupRuntime::default();
        runtime.snapshot_succeeded(&ready);

        assert!(runtime.observe(
            SignerBackupStrategy::Periodic { updates: 100 },
            &ready,
            &second_ready
        ));
    }

    #[test]
    fn write_snapshot_includes_state_peers_and_omits_tombstones() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let config = SignerBackupConfig::new(path.clone());
        let state = state_with_peers(json!({
            "channels/a": [0, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))],
            "channels/deleted": [u64::MAX, null]
        }), &[("02aa", "127.0.0.1:9735")])
        .omit_tombstones();

        write_snapshot(&config, &[2u8; 33], state).unwrap();

        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(written["version"], 1);
        assert_eq!(written["node_id"], hex::encode([2u8; 33]));
        assert_eq!(written["strategy"], "new_channels_only");
        assert!(written["state"]["values"]["channels/a"].is_array());
        assert!(written["state"]["values"]
            .as_object()
            .unwrap()
            .get("channels/deleted")
            .is_none());
        assert_eq!(written["state"]["values"]["peers/02aa"][1]["peer_id"], "02aa");
        assert_eq!(
            written["state"]["values"]["peers/02aa"][1]["addr"],
            "127.0.0.1:9735"
        );
        assert!(written.get("peers").is_none());
        assert!(written.get("peerlist").is_none());
    }

    #[test]
    fn read_snapshot_accepts_v1_new_channels_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let config = SignerBackupConfig::new(path.clone());

        write_snapshot(&config, &[2u8; 33], state(json!({}))).unwrap();

        let snapshot = SignerBackupSnapshot::read(&path).unwrap();
        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.strategy, SignerBackupStrategy::NewChannelsOnly);
        assert_eq!(snapshot.node_id, hex::encode([2u8; 33]));
    }

    #[test]
    fn read_snapshot_rejects_unsupported_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        write_json(
            &path,
            json!({
                "version": 2,
                "created_at": "2026-04-29T00:00:00Z",
                "node_id": hex::encode([2u8; 33]),
                "strategy": "new_channels_only",
                "state": { "values": {} }
            }),
        );

        let err = read_backup_err(&path);
        assert!(err.contains("unsupported signer backup version 2"));
    }

    #[test]
    fn read_snapshot_rejects_malformed_json_and_state() {
        let dir = tempfile::tempdir().unwrap();
        let malformed_json = dir.path().join("malformed.json");
        let malformed_state = dir.path().join("malformed-state.json");
        std::fs::write(&malformed_json, b"not-json").unwrap();
        write_json(
            &malformed_state,
            json!({
                "version": 1,
                "created_at": "2026-04-29T00:00:00Z",
                "node_id": hex::encode([2u8; 33]),
                "strategy": "new_channels_only",
                "state": { "values": { "channels/a": "not-a-state-entry" } }
            }),
        );

        assert!(read_backup_err(&malformed_json).contains("parsing signer backup"));
        assert!(read_backup_err(&malformed_state).contains("parsing signer backup"));
    }

    #[test]
    fn read_snapshot_rejects_top_level_peers_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        write_json(
            &path,
            json!({
                "version": 1,
                "created_at": "2026-04-29T00:00:00Z",
                "node_id": hex::encode([2u8; 33]),
                "strategy": "new_channels_only",
                "state": { "values": {} },
                "peers": []
            }),
        );

        assert!(read_backup_err(&path).contains("parsing signer backup"));
    }

    #[test]
    fn recovery_data_extracts_channels_and_joins_peer_addresses() {
        let peer_a = peer_id(0xaa);
        let peer_b = peer_id(0xbb);
        let channel_a = channel_key(&peer_a, 1);
        let channel_b = channel_key(&peer_a, 2);
        let channel_missing_addr = channel_key(&peer_b, 3);
        let stub = channel_key(&peer_a, 4);
        let tombstone = channel_key(&peer_a, 5);
        let state = state_with_peers(
            json!({
                channel_a.clone(): [1, channel(recovery_setup("00", 0, 1000, true))],
                channel_b.clone(): [1, channel(recovery_setup("11", 1, 2000, false))],
                channel_missing_addr.clone(): [1, channel(recovery_setup("22", 2, 3000, false))],
                stub: [1, channel(serde_json::Value::Null)],
                tombstone: [u64::MAX, null]
            }),
            &[(&peer_a, "127.0.0.1:9735")],
        );
        let snapshot = backup_snapshot(state);

        let inventory = snapshot.recovery_data().unwrap();

        assert_eq!(inventory.len(), 3);
        let first = inventory
            .iter()
            .find(|channel| channel.channel_key == channel_a)
            .unwrap();
        assert!(first.complete);
        assert_eq!(first.peer_id, peer_a);
        assert_eq!(first.peer_addr.as_deref(), Some("127.0.0.1:9735"));
        assert_eq!(first.funding_outpoint.txid, "00");
        assert_eq!(first.funding_outpoint.vout, 0);
        assert_eq!(first.funding_sats, 1000);
        assert_eq!(first.opener, RecoverableChannelOpener::Local);
        assert_eq!(first.remote_to_self_delay, 144);
        assert_eq!(first.commitment_type, "AnchorsZeroFeeHtlc");
        assert_eq!(
            first.remote_basepoints.funding_pubkey,
            "02bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );

        let second = inventory
            .iter()
            .find(|channel| channel.channel_key == channel_b)
            .unwrap();
        assert_eq!(second.peer_addr.as_deref(), Some("127.0.0.1:9735"));
        assert_eq!(second.opener, RecoverableChannelOpener::Remote);

        let missing_addr = inventory
            .iter()
            .find(|channel| channel.channel_key == channel_missing_addr)
            .unwrap();
        assert!(!missing_addr.complete);
        assert_eq!(missing_addr.peer_addr, None);
        assert_eq!(missing_addr.warnings, vec!["missing_peer_addr".to_string()]);
    }

    #[test]
    fn recovery_data_marks_missing_peer_state_incomplete() {
        let peer = peer_id(0xaa);
        let key = channel_key(&peer, 1);
        let snapshot = backup_snapshot(state(json!({
            key: [1, channel(recovery_setup("00", 0, 1000, true))]
        })));

        let inventory = snapshot.recovery_data().unwrap();

        assert_eq!(inventory.len(), 1);
        assert!(!inventory[0].complete);
        assert_eq!(inventory[0].peer_addr, None);
        assert_eq!(inventory[0].warnings, vec!["missing_peer_addr"]);
    }

    #[test]
    fn recovery_data_rejects_malformed_peer_state() {
        let peer = peer_id(0xaa);
        let key = channel_key(&peer, 1);
        let snapshot = SignerBackupSnapshot {
            version: BACKUP_VERSION,
            created_at: "2026-04-29T00:00:00Z".to_string(),
            node_id: hex::encode([2u8; 33]),
            strategy: SignerBackupStrategy::NewChannelsOnly,
            state: state(json!({
                key: [1, channel(recovery_setup("00", 0, 1000, true))],
                format!("peers/{peer}"): [0, json!({ "peer_id": peer })]
            })),
        };

        assert!(snapshot
            .recovery_data()
            .unwrap_err()
            .to_string()
            .contains("parsing peer state"));
    }

    #[test]
    fn recovery_data_rejects_malformed_channel_json() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 1);
        let snapshot = backup_snapshot(state(json!({
            channel_key: [1, channel(json!({ "channel_value_sat": 1000 }))]
        })));

        assert!(snapshot
            .recovery_data()
            .unwrap_err()
            .to_string()
            .contains("parsing recoverable channel"));
    }

    #[test]
    fn to_cln_backup_rejects_incomplete_channels_by_default() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 1);
        let snapshot = backup_snapshot(state(json!({
            channel_key: [1, channel(recovery_setup(full_txid(), 0, 1000, true))]
        })));

        let err = snapshot
            .to_cln_backup(CLNBackupOptions::default())
            .unwrap_err()
            .to_string();

        assert!(err.contains("missing_peer_addr"));
    }

    #[test]
    fn to_cln_backup_skips_incomplete_channels_when_requested() {
        let peer_a = peer_id(0xaa);
        let peer_b = peer_id(0xbb);
        let channel_a = channel_key(&peer_a, 1);
        let channel_b = channel_key(&peer_b, 2);
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_a.clone(): [1, channel(recovery_setup(full_txid(), 0, 1000, true))],
                    channel_b.clone(): [1, channel(recovery_setup(full_txid(), 1, 2000, false))]
                }),
                &[(&peer_a, "127.0.0.1:9735")],
            ),
        );

        let export = snapshot
            .to_cln_backup(CLNBackupOptions {
                skip_incomplete: true,
            })
            .unwrap();

        assert_eq!(export.request.scb.len(), 1);
        assert_eq!(export.total_channels, 2);
        assert_eq!(export.exported_channels, 1);
        assert_eq!(export.skipped_channels, 1);
        assert_eq!(export.channels[0].channel_key, channel_a);
        assert_eq!(export.channels[0].cln_dbid, 1);
        assert_eq!(
            export.channels[0].warnings,
            vec![SHACHAIN_MISSING_WARNING.to_string()]
        );
        assert_eq!(export.skipped[0].channel_key, channel_b);
        assert_eq!(export.skipped[0].warnings, vec!["missing_peer_addr"]);
    }

    #[test]
    fn to_cln_backup_fails_when_every_channel_is_skipped() {
        let peer = peer_id(0xaa);
        let snapshot = backup_snapshot(state(json!({
            channel_key(&peer, 1): [1, channel(recovery_setup(full_txid(), 0, 1000, true))]
        })));

        let err = snapshot
            .to_cln_backup(CLNBackupOptions {
                skip_incomplete: true,
            })
            .unwrap_err()
            .to_string();

        assert!(err.contains("no complete recoverable channels"));
    }

    #[test]
    fn to_cln_backup_encodes_modern_scb_for_ipv4_peer() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 42);
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_key.clone(): [1, channel(recovery_setup(full_txid(), 0x0102, 1_000_000, true))]
                }),
                &[(&peer, "127.0.0.1:9735")],
            ),
        );

        let export = snapshot.to_cln_backup(CLNBackupOptions::default()).unwrap();
        let scb = hex::decode(&export.request.scb[0]).unwrap();
        let txid = expected_cln_txid(full_txid());
        let mut channel_id = txid;
        channel_id[30] ^= 0x01;
        channel_id[31] ^= 0x02;

        assert_eq!(&scb[0..8], &42u64.to_be_bytes());
        assert_eq!(&scb[8..40], &channel_id);
        assert_eq!(export.channels[0].channel_id, hex::encode(channel_id));
        assert_eq!(&scb[40..73], hex::decode(&peer).unwrap());
        assert_eq!(&scb[73..80], hex::decode("017f0000012607").unwrap());
        assert_eq!(&scb[80..112], &txid);
        assert_eq!(&scb[112..116], &0x0102u32.to_be_bytes());
        assert_eq!(&scb[116..124], &1_000_000u64.to_be_bytes());

        let channel_type_len = u16::from_be_bytes(scb[124..126].try_into().unwrap()) as usize;
        assert!(channel_type_len > 0);
        let tlvs = scb_tlvs(&scb);
        assert_eq!(tlvs[0], 3);
        assert_eq!(tlvs[1], 132);
        assert_eq!(
            &tlvs[2..35],
            hex::decode("02eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
                .unwrap()
        );
        assert!(tlvs.windows(3).any(|window| window == [5, 1, 0]));
        assert!(tlvs.windows(4).any(|window| window == [7, 2, 0, 144]));
    }

    #[test]
    fn to_cln_backup_encodes_shachain_tlv_when_secrets_are_present() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 42);
        let first_index = SHACHAIN_EMPTY_INDEX - 1;
        let second_index = SHACHAIN_EMPTY_INDEX - 2;
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_key.clone(): [1, channel_with_enforcement(
                        recovery_setup(full_txid(), 0, 1_000_000, true),
                        enforcement_with_old_secrets(json!([
                            old_secret(0x11, first_index),
                            old_secret(0x22, second_index)
                        ]))
                    )]
                }),
                &[(&peer, "127.0.0.1:9735")],
            ),
        );

        let export = snapshot.to_cln_backup(CLNBackupOptions::default()).unwrap();
        let scb = hex::decode(&export.request.scb[0]).unwrap();
        let tlvs = scb_tlvs(&scb);

        assert!(export.channels[0].warnings.is_empty());
        assert_eq!(tlvs[0], 1);
        assert_eq!(tlvs[1], 92);
        let shachain = &tlvs[2..94];
        assert_eq!(
            u64::from_be_bytes(shachain[0..8].try_into().unwrap()),
            second_index
        );
        assert_eq!(u32::from_be_bytes(shachain[8..12].try_into().unwrap()), 2);
        assert_eq!(
            u64::from_be_bytes(shachain[12..20].try_into().unwrap()),
            first_index
        );
        assert_eq!(&shachain[20..52], &[0x11; SHACHAIN_SECRET_LEN]);
        assert_eq!(
            u64::from_be_bytes(shachain[52..60].try_into().unwrap()),
            second_index
        );
        assert_eq!(&shachain[60..92], &[0x22; SHACHAIN_SECRET_LEN]);
        assert_eq!(tlvs[94], 3);
    }

    #[test]
    fn to_cln_backup_omits_empty_shachain_without_warning() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 42);
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_key: [1, channel_with_enforcement(
                        recovery_setup(full_txid(), 0, 1_000_000, true),
                        enforcement_with_old_secrets(json!([]))
                    )]
                }),
                &[(&peer, "127.0.0.1:9735")],
            ),
        );

        let export = snapshot.to_cln_backup(CLNBackupOptions::default()).unwrap();
        let scb = hex::decode(&export.request.scb[0]).unwrap();
        let tlvs = scb_tlvs(&scb);

        assert!(export.channels[0].warnings.is_empty());
        assert_eq!(tlvs[0], 3);
    }

    #[test]
    fn to_cln_backup_warns_when_counterparty_secrets_are_missing() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 42);
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_key: [1, channel_with_enforcement(
                        recovery_setup(full_txid(), 0, 1_000_000, true),
                        json!({ "counterparty_secrets": null })
                    )]
                }),
                &[(&peer, "127.0.0.1:9735")],
            ),
        );

        let export = snapshot.to_cln_backup(CLNBackupOptions::default()).unwrap();

        assert_eq!(
            export.channels[0].warnings,
            vec![SHACHAIN_MISSING_WARNING.to_string()]
        );
    }

    #[test]
    fn to_cln_backup_ignores_trailing_dummy_shachain_entries() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 42);
        let first_index = SHACHAIN_EMPTY_INDEX - 1;
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_key: [1, channel_with_enforcement(
                        recovery_setup(full_txid(), 0, 1_000_000, true),
                        enforcement_with_old_secrets(json!([
                            old_secret(0x11, first_index),
                            dummy_old_secret(),
                            dummy_old_secret()
                        ]))
                    )]
                }),
                &[(&peer, "127.0.0.1:9735")],
            ),
        );

        let export = snapshot.to_cln_backup(CLNBackupOptions::default()).unwrap();
        let scb = hex::decode(&export.request.scb[0]).unwrap();
        let tlvs = scb_tlvs(&scb);

        assert!(export.channels[0].warnings.is_empty());
        assert_eq!(tlvs[0], 1);
        assert_eq!(tlvs[1], 52);
        let shachain = &tlvs[2..54];
        assert_eq!(u32::from_be_bytes(shachain[8..12].try_into().unwrap()), 1);
        assert_eq!(tlvs[54], 3);
    }

    #[test]
    fn to_cln_backup_rejects_malformed_shachain_entries() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 42);

        for (old_secrets, expected) in [
            (
                json!([malformed_old_secret(
                    vec![0x11; SHACHAIN_SECRET_LEN - 1],
                    SHACHAIN_EMPTY_INDEX - 1
                )]),
                "must be 32 bytes",
            ),
            (
                json!([old_secret(0x22, SHACHAIN_EMPTY_INDEX - 2)]),
                "belongs at position",
            ),
            (
                json!([
                    old_secret(0x11, SHACHAIN_EMPTY_INDEX - 1),
                    dummy_old_secret(),
                    old_secret(0x22, SHACHAIN_EMPTY_INDEX - 2)
                ]),
                "missing shachain position",
            ),
            (
                json!([malformed_old_secret(
                    vec![0x11; SHACHAIN_SECRET_LEN],
                    SHACHAIN_EMPTY_INDEX
                )]),
                "invalid shachain index",
            ),
        ] {
            let snapshot = backup_snapshot(
                state_with_peers(
                    json!({
                        channel_key.clone(): [1, channel_with_enforcement(
                            recovery_setup(full_txid(), 0, 1_000_000, true),
                            enforcement_with_old_secrets(old_secrets)
                        )]
                    }),
                    &[(&peer, "127.0.0.1:9735")],
                ),
            );

            let err = snapshot
                .to_cln_backup(CLNBackupOptions::default())
                .unwrap_err()
                .to_string();
            assert!(err.contains(expected), "{err}");
        }
    }

    #[test]
    fn to_cln_backup_rejects_malformed_export_fields() {
        let peer = peer_id(0xaa);
        let mut setup = recovery_setup(full_txid(), 0, 1000, true);
        setup["commitment_type"] = json!("Unknown");
        let snapshot = backup_snapshot(
            state_with_peers(
                json!({
                    channel_key(&peer, 1): [1, channel(setup)]
                }),
                &[(&peer, "127.0.0.1:9735")],
            ),
        );

        let err = snapshot
            .to_cln_backup(CLNBackupOptions::default())
            .unwrap_err()
            .to_string();

        assert!(err.contains("unsupported commitment type"));
    }

    #[test]
    fn encode_wireaddr_supports_ipv4_ipv6_dns_and_tor_v3() {
        let tor = format!("{}.onion:9735", "a".repeat(56));

        assert_eq!(
            encode_wireaddr("127.0.0.1:9735").unwrap(),
            hex::decode("017f0000012607").unwrap()
        );
        assert_eq!(
            encode_wireaddr("[2001:db8::1]:9735").unwrap(),
            hex::decode("0220010db80000000000000000000000012607").unwrap()
        );
        assert_eq!(
            encode_wireaddr("example.com:9735").unwrap(),
            hex::decode("050b6578616d706c652e636f6d2607").unwrap()
        );

        let encoded_tor = encode_wireaddr(&tor).unwrap();
        assert_eq!(encoded_tor.len(), 38);
        assert_eq!(encoded_tor[0], 4);
        assert_eq!(&encoded_tor[1..36], &[0u8; 35]);
        assert_eq!(&encoded_tor[36..38], &9735u16.to_be_bytes());
    }

    #[test]
    fn encode_channel_type_preserves_legacy_as_empty_features() {
        assert_eq!(encode_channel_type("Legacy").unwrap(), Vec::<u8>::new());
        assert!(!encode_channel_type("StaticRemoteKey").unwrap().is_empty());
    }

    #[test]
    fn write_snapshot_fails_when_parent_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = SignerBackupConfig::new(dir.path().join("missing").join("backup.json"));
        let state = state(json!({}));

        assert!(write_snapshot(&config, &[2u8; 33], state).is_err());
    }

    #[test]
    fn write_snapshot_serializes_periodic_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let config = SignerBackupConfig::periodic(path.clone(), 5).unwrap();

        write_snapshot(&config, &[2u8; 33], state(json!({}))).unwrap();

        let written: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(written["strategy"]["periodic"]["updates"], 5);

        let snapshot = SignerBackupSnapshot::read(dir.path().join("backup.json")).unwrap();
        assert_eq!(
            snapshot.strategy,
            SignerBackupStrategy::Periodic { updates: 5 }
        );
    }
}
