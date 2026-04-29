use super::{SignerBackupConfig, SignerBackupStrategy};
use crate::persist::State;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

const BACKUP_VERSION: u32 = 1;
const PEERLIST_PREFIX: [&str; 2] = ["greenlight", "peerlist"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerlistEntry {
    pub peer_id: String,
    pub addr: String,
    pub direction: String,
    pub features: String,
    pub generation: Option<u64>,
    pub raw_datastore_string: String,
}

#[derive(Deserialize)]
struct PeerRecord {
    id: String,
    direction: String,
    addr: String,
    features: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignerBackupSnapshot {
    pub version: u32,
    pub created_at: String,
    pub node_id: String,
    pub strategy: SignerBackupStrategy,
    pub state: State,
    pub peerlist: Vec<PeerlistEntry>,
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

        let peers: BTreeMap<&str, &PeerlistEntry> = self
            .peerlist
            .iter()
            .map(|peer| (peer.peer_id.as_str(), peer))
            .collect();

        self.state
            .omit_tombstones()
            .recoverable_channel_values()
            .into_iter()
            .map(|(channel_key, value)| {
                let peer_id = peer_id_from_channel_key(&channel_key)?;
                let entry: ChannelEntry = serde_json::from_value(value)
                    .with_context(|| format!("parsing recoverable channel {}", channel_key))?;
                let setup = entry.channel_setup.ok_or_else(|| {
                    anyhow!("recoverable channel {} is missing channel_setup", channel_key)
                })?;
                let peer_addr = peers
                    .get(peer_id.as_str())
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

#[derive(Deserialize)]
struct ChannelEntry {
    channel_setup: Option<ChannelSetup>,
}

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
    let before_channels = before.recoverable_channel_keys();
    after
        .recoverable_channel_keys()
        .iter()
        .any(|key| !before_channels.contains(key))
}

fn has_recoverable_state_update(before: &State, after: &State) -> bool {
    !before.diff_state(after).recoverable_channel_keys().is_empty()
}

pub(crate) fn parse_peerlist(
    entries: &[crate::pb::cln::ListdatastoreDatastore],
) -> Result<Vec<PeerlistEntry>> {
    let mut peers = entries
        .iter()
        .map(parse_peerlist_entry)
        .collect::<Result<Vec<_>>>()?;
    peers.sort_by(|a, b| a.peer_id.cmp(&b.peer_id));
    Ok(peers)
}

pub(crate) fn write_snapshot(
    config: &SignerBackupConfig,
    node_id: &[u8],
    state: State,
    peerlist: Vec<PeerlistEntry>,
) -> Result<()> {
    let snapshot = SignerBackupSnapshot {
        version: BACKUP_VERSION,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        node_id: hex::encode(node_id),
        strategy: config.strategy,
        state,
        peerlist,
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

fn parse_peerlist_entry(entry: &crate::pb::cln::ListdatastoreDatastore) -> Result<PeerlistEntry> {
    if entry.key.len() != 3
        || entry.key[0] != PEERLIST_PREFIX[0]
        || entry.key[1] != PEERLIST_PREFIX[1]
    {
        return Err(anyhow!("invalid peerlist datastore key: {:?}", entry.key));
    }

    let key_peer_id = &entry.key[2];
    let raw = entry
        .string
        .as_ref()
        .ok_or_else(|| anyhow!("peerlist entry {} is missing string payload", key_peer_id))?;
    let peer = parse_peer_record(raw)
        .with_context(|| format!("parsing peerlist entry {}", key_peer_id))?;

    if peer.id != *key_peer_id {
        return Err(anyhow!(
            "peerlist key {} does not match payload id {}",
            key_peer_id,
            peer.id
        ));
    }

    Ok(PeerlistEntry {
        peer_id: peer.id,
        addr: peer.addr,
        direction: peer.direction,
        features: peer.features,
        generation: entry.generation,
        raw_datastore_string: raw.clone(),
    })
}

fn parse_peer_record(raw: &str) -> Result<PeerRecord> {
    serde_json::from_str(raw).or_else(|first_error| {
        let cleaned = raw.replace('\\', "");
        serde_json::from_str(&cleaned)
            .with_context(|| format!("invalid peer JSON; original parse error: {}", first_error))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn state(entries: serde_json::Value) -> State {
        serde_json::from_value(json!({ "values": entries })).unwrap()
    }

    fn channel(setup: serde_json::Value) -> serde_json::Value {
        json!({
            "channel_setup": setup,
            "channel_value_satoshis": 1000,
            "id": null,
            "enforcement_state": {},
            "blockheight": null
        })
    }

    fn peer_entry(
        key: Vec<&str>,
        generation: Option<u64>,
        string: Option<&str>,
    ) -> crate::pb::cln::ListdatastoreDatastore {
        crate::pb::cln::ListdatastoreDatastore {
            key: key.into_iter().map(str::to_owned).collect(),
            generation,
            hex: None,
            string: string.map(str::to_owned),
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
    fn parse_peerlist_normalizes_valid_entries() {
        let raw = r#"{"id":"02aa","direction":"out","addr":"127.0.0.1:9735","features":"abcd"}"#;
        let peers = parse_peerlist(&[peer_entry(
            vec!["greenlight", "peerlist", "02aa"],
            Some(7),
            Some(raw),
        )])
        .unwrap();

        assert_eq!(
            peers,
            vec![PeerlistEntry {
                peer_id: "02aa".to_string(),
                addr: "127.0.0.1:9735".to_string(),
                direction: "out".to_string(),
                features: "abcd".to_string(),
                generation: Some(7),
                raw_datastore_string: raw.to_string(),
            }]
        );
    }

    #[test]
    fn parse_peerlist_rejects_malformed_entries() {
        assert!(parse_peerlist(&[peer_entry(
            vec!["greenlight", "peerlist", "02aa"],
            None,
            Some("not-json"),
        )])
        .is_err());

        assert!(parse_peerlist(&[peer_entry(
            vec!["greenlight", "peerlist", "02aa"],
            None,
            Some(r#"{"id":"02aa","direction":"out","features":""}"#),
        )])
        .is_err());

        assert!(parse_peerlist(&[peer_entry(
            vec!["greenlight", "wrong", "02aa"],
            None,
            Some(r#"{"id":"02aa","direction":"out","addr":"127.0.0.1:9735","features":""}"#),
        )])
        .is_err());
    }

    #[test]
    fn write_snapshot_includes_state_peerlist_and_omits_tombstones() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let config = SignerBackupConfig::new(path.clone());
        let state = state(json!({
            "channels/a": [0, channel(json!({ "funding_outpoint": { "txid": "00", "vout": 0 } }))],
            "channels/deleted": [u64::MAX, null]
        }))
        .omit_tombstones();
        let peerlist = vec![PeerlistEntry {
            peer_id: "02aa".to_string(),
            addr: "127.0.0.1:9735".to_string(),
            direction: "out".to_string(),
            features: "".to_string(),
            generation: Some(3),
            raw_datastore_string:
                r#"{"id":"02aa","direction":"out","addr":"127.0.0.1:9735","features":""}"#
                    .to_string(),
        }];

        write_snapshot(&config, &[2u8; 33], state, peerlist).unwrap();

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
        assert_eq!(written["peerlist"][0]["peer_id"], "02aa");
        assert_eq!(written["peerlist"][0]["generation"], 3);
        assert!(written["peerlist"][0]["raw_datastore_string"]
            .as_str()
            .unwrap()
            .contains("\"addr\""));
    }

    #[test]
    fn read_snapshot_accepts_v1_new_channels_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let config = SignerBackupConfig::new(path.clone());

        write_snapshot(&config, &[2u8; 33], state(json!({})), vec![]).unwrap();

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
                "state": { "values": {} },
                "peerlist": []
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
                "state": { "values": { "channels/a": "not-a-state-entry" } },
                "peerlist": []
            }),
        );

        assert!(read_backup_err(&malformed_json).contains("parsing signer backup"));
        assert!(read_backup_err(&malformed_state).contains("parsing signer backup"));
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
        let state = state(json!({
            channel_a.clone(): [1, channel(recovery_setup("00", 0, 1000, true))],
            channel_b.clone(): [1, channel(recovery_setup("11", 1, 2000, false))],
            channel_missing_addr.clone(): [1, channel(recovery_setup("22", 2, 3000, false))],
            stub: [1, channel(serde_json::Value::Null)],
            tombstone: [u64::MAX, null]
        }));
        let snapshot = SignerBackupSnapshot {
            version: BACKUP_VERSION,
            created_at: "2026-04-29T00:00:00Z".to_string(),
            node_id: hex::encode([2u8; 33]),
            strategy: SignerBackupStrategy::NewChannelsOnly,
            state,
            peerlist: vec![PeerlistEntry {
                peer_id: peer_a.clone(),
                addr: "127.0.0.1:9735".to_string(),
                direction: "out".to_string(),
                features: "".to_string(),
                generation: Some(1),
                raw_datastore_string: "{}".to_string(),
            }],
        };

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
    fn recovery_data_rejects_malformed_channel_json() {
        let peer = peer_id(0xaa);
        let channel_key = channel_key(&peer, 1);
        let snapshot = SignerBackupSnapshot {
            version: BACKUP_VERSION,
            created_at: "2026-04-29T00:00:00Z".to_string(),
            node_id: hex::encode([2u8; 33]),
            strategy: SignerBackupStrategy::NewChannelsOnly,
            state: state(json!({
                channel_key: [1, channel(json!({ "channel_value_sat": 1000 }))]
            })),
            peerlist: vec![],
        };

        assert!(snapshot
            .recovery_data()
            .unwrap_err()
            .to_string()
            .contains("parsing recoverable channel"));
    }

    #[test]
    fn write_snapshot_fails_when_parent_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = SignerBackupConfig::new(dir.path().join("missing").join("backup.json"));
        let state = state(json!({}));

        assert!(write_snapshot(&config, &[2u8; 33], state, vec![]).is_err());
    }

    #[test]
    fn write_snapshot_serializes_periodic_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let config = SignerBackupConfig::periodic(path.clone(), 5).unwrap();

        write_snapshot(&config, &[2u8; 33], state(json!({})), vec![]).unwrap();

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
