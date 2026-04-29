use super::{SignerBackupConfig, SignerBackupStrategy};
use crate::persist::State;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

const BACKUP_VERSION: u32 = 1;
const PEERLIST_PREFIX: [&str; 2] = ["greenlight", "peerlist"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PeerlistEntry {
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

#[derive(Serialize)]
struct BackupSnapshot {
    version: u32,
    created_at: String,
    node_id: String,
    strategy: SignerBackupStrategy,
    state: State,
    peerlist: Vec<PeerlistEntry>,
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
    let snapshot = BackupSnapshot {
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
    }
}
