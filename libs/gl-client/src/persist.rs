use lightning_signer::bitcoin::secp256k1::PublicKey;
use lightning_signer::chain::tracker::ChainTracker;
use lightning_signer::channel::ChannelId;
use lightning_signer::channel::ChannelStub;
use lightning_signer::node::NodeConfig;
use lightning_signer::node::NodeState;
use lightning_signer::persist::ChainTrackerListenerEntry;
use lightning_signer::persist::{Error, Persist, SignerId};
use lightning_signer::policy::validator::ValidatorFactory;
use lightning_signer::SendSync;
use log::{trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

const NODE_PREFIX: &str = "nodes";
const NODE_STATE_PREFIX: &str = "nodestates";
const CHANNEL_PREFIX: &str = "channels";
const ALLOWLIST_PREFIX: &str = "allowlists";
const TRACKER_PREFIX: &str = "trackers";
const TOMBSTONE_PREFIX: &str = "tombs";

/**
 * TODO: consider tombstone garbage collection strategy if we expect a large number of deletes.
 */

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    values: BTreeMap<String, (u64, serde_json::Value)>,
}

impl State {
    fn insert_node(
        &mut self,
        key: &str,
        node_entry: vls_persist::model::NodeEntry,
        node_state_entry: vls_persist::model::NodeStateEntry,
    ) -> Result<(), Error> {
        let node_key = format!("{NODE_PREFIX}/{key}");
        let state_key = format!("{NODE_STATE_PREFIX}/{key}");
        let node_version = self.next_version(&node_key);
        let state_version = self.next_version(&state_key);

        self.values
            .insert(node_key, (node_version, serde_json::to_value(node_entry).unwrap()));
        self.values.insert(
            state_key,
            (state_version, serde_json::to_value(node_state_entry).unwrap()),
        );
        Ok(())
    }

    fn update_node(
        &mut self,
        key: &str,
        node_state: vls_persist::model::NodeStateEntry,
    ) -> Result<(), Error> {
        trace!(
            "Update node: {}",
            serde_json::to_string(&node_state).unwrap()
        );
        let key = format!("{NODE_STATE_PREFIX}/{key}");
        let v = self
            .values
            .get_mut(&key)
            .expect("attempting to update non-existent node");
        *v = (v.0 + 1, serde_json::to_value(node_state).unwrap());
        Ok(())
    }

    fn delete_node(&mut self, key: &str) -> Result<(), Error> {
        let node_key = format!("{NODE_PREFIX}/{key}");
        let state_key = format!("{NODE_STATE_PREFIX}/{key}");
        let allowlist_key = format!("{ALLOWLIST_PREFIX}/{key}");
        let tracker_key = format!("{TRACKER_PREFIX}/{key}");
        let channel_prefix = format!("{CHANNEL_PREFIX}/{key}");
        let channel_keys: Vec<String> = self
            .values
            .keys()
            .filter(|k| k.starts_with(&channel_prefix))
            .cloned()
            .collect();

        self.put_tombstone(&node_key);
        self.put_tombstone(&state_key);
        self.put_tombstone(&allowlist_key);
        self.put_tombstone(&tracker_key);
        for channel_key in channel_keys {
            self.put_tombstone(&channel_key);
        }
        Ok(())
    }

    fn insert_channel(
        &mut self,
        key: &str,
        channel_entry: vls_persist::model::ChannelEntry,
    ) -> Result<(), Error> {
        let key = format!("{CHANNEL_PREFIX}/{key}");
        let version = self.next_version(&key);
        self.values
            .insert(key, (version, serde_json::to_value(channel_entry).unwrap()));
        Ok(())
    }

    fn delete_channel(&mut self, key: &str) {
        let live_key = format!("{CHANNEL_PREFIX}/{key}");
        self.put_tombstone(&live_key);
    }

    fn update_channel(
        &mut self,
        key: &str,
        channel_entry: vls_persist::model::ChannelEntry,
    ) -> Result<(), Error> {
        trace!("Updating channel {key}");
        let key = format!("{CHANNEL_PREFIX}/{key}");
        let v = self
            .values
            .get_mut(&key)
            .expect("attempting to update non-existent channel");
        *v = (v.0 + 1, serde_json::to_value(channel_entry).unwrap());
        Ok(())
    }

    fn get_channel(
        &self,
        key: &str,
    ) -> Result<lightning_signer::persist::model::ChannelEntry, Error> {
        let key = format!("{CHANNEL_PREFIX}/{key}");
        if self.is_tombstoned(&key) {
            return Err(Error::Internal(format!("channel {} has been deleted", key)));
        }
        let value = self
            .values
            .get(&key)
            .ok_or_else(|| Error::Internal(format!("missing channel state for key {}", key)))?;
        let entry: vls_persist::model::ChannelEntry =
            serde_json::from_value(value.1.clone()).unwrap();
        Ok(entry.into())
    }

    fn get_node_channels(
        &self,
        node_id: &PublicKey,
    ) -> Result<
        Vec<(
            lightning_signer::channel::ChannelId,
            lightning_signer::persist::model::ChannelEntry,
        )>,
        Error,
    > {
        let prefix = hex::encode(node_id.serialize());
        let prefix = format!("{CHANNEL_PREFIX}/{prefix}");
        Ok(self
            .values
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .filter(|(k, _)| !self.is_tombstoned(k))
            .map(|(k, v)| {
                let key = k.split('/').last().unwrap();
                let key = vls_persist::model::NodeChannelId(hex::decode(&key).unwrap());

                let value: vls_persist::model::ChannelEntry =
                    serde_json::from_value(v.1.clone()).unwrap();
                (key.channel_id(), value.into())
            })
            .collect())
    }

    fn new_chain_tracker(
        &mut self,
        node_id: &PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) -> Result<(), Error> {
        let key = hex::encode(node_id.serialize());
        let key = format!("{TRACKER_PREFIX}/{key}");
        let version = self.next_version(&key);

        let tracker: vls_persist::model::ChainTrackerEntry = tracker.into();

        self.values
            .insert(key, (version, serde_json::to_value(tracker).unwrap()));
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.values.clear();
        Ok(())
    }

    fn put_tombstone(&mut self, live_key: &str) {
        let tombstone_key = Self::tombstone_key(live_key);
        let version = self.next_version(live_key);
        self.values
            .insert(tombstone_key, (version, serde_json::Value::Null));
        self.values.remove(live_key);
    }

    fn tombstone_key(live_key: &str) -> String {
        format!("{TOMBSTONE_PREFIX}/{live_key}")
    }

    fn tombstone_target_key(key: &str) -> Option<&str> {
        key.strip_prefix(TOMBSTONE_PREFIX)
            .and_then(|rest| rest.strip_prefix('/'))
    }

    fn is_tombstone_key(key: &str) -> bool {
        Self::tombstone_target_key(key).is_some()
    }

    fn tombstone_version(&self, live_key: &str) -> Option<u64> {
        let tombstone_key = Self::tombstone_key(live_key);
        self.values.get(&tombstone_key).map(|v| v.0)
    }

    fn next_version(&self, live_key: &str) -> u64 {
        let live_ver = self.values.get(live_key).map(|v| v.0);
        let tombstone_ver = self.tombstone_version(live_key);
        live_ver
            .into_iter()
            .chain(tombstone_ver)
            .max()
            .map(|v| v.saturating_add(1))
            .unwrap_or(0)
    }

    fn is_tombstoned(&self, live_key: &str) -> bool {
        let live_ver = self.values.get(live_key).map(|v| v.0);
        let tombstone_ver = self.tombstone_version(live_key);

        match (live_ver, tombstone_ver) {
            (_, None) => false,
            (None, Some(_)) => true,
            (Some(live), Some(tomb)) => tomb >= live,
        }
    }

}

#[derive(Debug)]
pub struct StateChange {
    key: String,
    old: Option<(u64, serde_json::Value)>,
    new: (u64, serde_json::Value),
}

#[derive(Debug, Default)]
pub struct SafeMergeResult {
    pub changes: Vec<(String, Option<u64>, u64)>,
    pub conflict_count: usize,
}

impl SafeMergeResult {
    pub fn has_conflicts(&self) -> bool {
        self.conflict_count > 0
    }
}

use core::fmt::Display;

impl Display for StateChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.old {
            Some(o) => f.write_str(&format!(
                "StateChange[{}]: old_version={}, new_version={}, old_value={}, new_value={}",
                self.key,
                o.0,
                self.new.0,
                serde_json::to_string(&o.1).unwrap(),
                serde_json::to_string(&self.new.1).unwrap()
            )),
            None => f.write_str(&format!(
                "StateChange[{}]: old_version=null, new_version={}, old_value=null, new_value={}",
                self.key,
                self.new.0,
                serde_json::to_string(&self.new.1).unwrap()
            )),
        }
    }
}

impl State {
    pub fn new() -> Self {
        State {
            values: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Merge incoming state and track potential version conflicts.
    ///
    /// A conflict means the incoming state is stale or incompatible with local
    /// tombstone knowledge. Callers may use this signal to trigger a full sync.
    pub fn safe_merge(&mut self, other: &State) -> anyhow::Result<SafeMergeResult> {
        let mut res = SafeMergeResult::default();
        for (key, (newver, newval)) in other.values.iter() {
            if Self::is_tombstone_key(key) {
                let local = self.values.get_mut(key);
                match local {
                    None => {
                        trace!("Insert new tombstone key {}: version={}", key, newver);
                        res.changes.push((key.to_owned(), None, *newver));
                        self.values.insert(key.clone(), (*newver, newval.clone()));
                    }
                    Some(v) => {
                        if v.0 == *newver {
                            // Idempotent re-application, but still enforce live-key cleanup below.
                        } else if v.0 > *newver {
                            warn!("Ignoring outdated tombstone version newver={}, we have oldver={}: newval={:?} vs oldval={:?}", newver, v.0, serde_json::to_string(newval), serde_json::to_string(&v.1));
                            // Keep the newer local tombstone and still enforce live-key cleanup below.
                            res.conflict_count += 1;
                        } else {
                            trace!(
                                "Updating tombstone key {}: version={} => version={}",
                                key,
                                v.0,
                                *newver
                            );
                            res.changes.push((key.to_owned(), Some(v.0), *newver));
                            *v = (*newver, newval.clone());
                        }
                    }
                }

                if let Some(live_key) = Self::tombstone_target_key(key) {
                    if self.is_tombstoned(live_key) {
                        self.values.remove(live_key);
                    }
                }
                continue;
            }

            if self
                .tombstone_version(key)
                .map(|tomb| tomb >= *newver)
                .unwrap_or(false)
            {
                trace!(
                    "Ignoring live key {} version={} because a tombstone is newer or equal",
                    key, newver
                );
                res.conflict_count += 1;
                continue;
            }

            let local = self.values.get_mut(key);

            match local {
                None => {
                    trace!("Insert new key {}: version={}", key, newver);
                    res.changes.push((key.to_owned(), None, *newver));
                    self.values.insert(key.clone(), (*newver, newval.clone()));
                }
                Some(v) => {
                    if v.0 == *newver {
                        continue;
                    } else if v.0 > *newver {
                        warn!("Ignoring outdated state version newver={}, we have oldver={}: newval={:?} vs oldval={:?}", newver, v.0, serde_json::to_string(newval), serde_json::to_string(&v.1));
                        res.conflict_count += 1;
                        continue;
                    } else {
                        trace!(
                            "Updating key {}: version={} => version={}",
                            key,
                            v.0,
                            *newver
                        );
                        res.changes.push((key.to_owned(), Some(v.0), *newver));
                        *v = (*newver, newval.clone());
                    }
                }
            }
        }
        Ok(res)
    }

    /// Backward-compatible merge API.
    pub fn merge(&mut self, other: &State) -> anyhow::Result<Vec<(String, Option<u64>, u64)>> {
        Ok(self.safe_merge(other)?.changes)
    }

    pub fn diff(&self, other: &State) -> anyhow::Result<Vec<StateChange>> {
        Ok(other
            .values
            .iter()
            .map(|(key, (ver, val))| (key, self.values.get(key), (ver, val)))
            .map(|(key, old, new)| StateChange {
                key: key.clone(),
                old: old.map(|o| o.clone()),
                new: (*new.0, new.1.clone()),
            })
            .filter(|c| match (&c.old, &c.new) {
                (None, _) => true,
                (Some((oldver, _)), (newver, _)) => oldver < newver,
            })
            .collect())
    }

    /// Return a `State` containing only entries that are newer in `other` than in `self`.
    /// This is useful for sending compact state diffs.
    pub fn diff_state(&self, other: &State) -> State {
        let mut values = BTreeMap::new();
        for (key, (newver, newval)) in other.values.iter() {
            match self.values.get(key) {
                None => {
                    values.insert(key.clone(), (*newver, newval.clone()));
                }
                Some((oldver, _)) if oldver < newver => {
                    values.insert(key.clone(), (*newver, newval.clone()));
                }
                _ => {}
            }
        }
        State { values }
    }

    pub fn sketch(&self) -> StateSketch {
        StateSketch::from_state(self)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct StateSketch {
    versions: BTreeMap<String, u64>,
}

impl StateSketch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_state(state: &State) -> Self {
        let mut sketch = Self::new();
        sketch.apply_state(state);
        sketch
    }

    /// Apply versions from `state` without clearing existing entries.
    pub fn apply_state(&mut self, state: &State) {
        for (key, (ver, _)) in state.values.iter() {
            self.versions.insert(key.clone(), *ver);
        }
    }

    /// Build a `State` containing entries newer than those recorded in the sketch.
    pub fn diff_state(&self, state: &State) -> State {
        let mut values = BTreeMap::new();
        for (key, (newver, newval)) in state.values.iter() {
            match self.versions.get(key) {
                None => {
                    values.insert(key.clone(), (*newver, newval.clone()));
                }
                Some(oldver) if oldver < newver => {
                    values.insert(key.clone(), (*newver, newval.clone()));
                }
                _ => {}
            }
        }
        State { values }
    }
}

impl Into<Vec<crate::pb::SignerStateEntry>> for State {
    fn into(self) -> Vec<crate::pb::SignerStateEntry> {
        self.values
            .iter()
            .map(|(k, v)| crate::pb::SignerStateEntry {
                key: k.to_owned(),
                value: serde_json::to_vec(&v.1).unwrap(),
                version: v.0,
            })
            .collect()
    }
}

impl From<Vec<crate::pb::SignerStateEntry>> for State {
    fn from(v: Vec<crate::pb::SignerStateEntry>) -> State {
        use std::iter::FromIterator;
        let values = BTreeMap::from_iter(v.iter().map(|v| {
            (
                v.key.to_owned(),
                (v.version, serde_json::from_slice(&v.value).unwrap()),
            )
        }));

        State { values }
    }
}

pub(crate) struct MemoryPersister {
    state: Arc<Mutex<State>>,
}

impl MemoryPersister {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(State {
            values: BTreeMap::new(),
        }));
        MemoryPersister { state }
    }

    pub fn state(&self) -> Arc<Mutex<State>> {
        self.state.clone()
    }
}

impl SendSync for MemoryPersister {}

impl Persist for MemoryPersister {
    fn new_node(
        &self,
        node_id: &lightning_signer::bitcoin::secp256k1::PublicKey,
        config: &NodeConfig,
        state: &NodeState,
    ) -> Result<(), Error> {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().insert_node(
            &key,
            vls_persist::model::NodeEntry {
                key_derivation_style: config.key_derivation_style as u8,
                network: config.network.to_string(),
            },
            state.into(),
        )
    }

    fn delete_channel(&self, node_id: &PublicKey, channel: &ChannelId) -> Result<(), Error> {
        let node_channel_id = vls_persist::model::NodeChannelId::new(node_id, &channel);
        let id = hex::encode(node_channel_id.0);
        self.state.lock().unwrap().delete_channel(&id);
        Ok(())
    }

    fn update_node(
        &self,
        node_id: &lightning_signer::bitcoin::secp256k1::PublicKey,
        state: &NodeState,
    ) -> Result<(), Error> {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().update_node(&key, state.into())
    }

    fn delete_node(
        &self,
        node_id: &lightning_signer::bitcoin::secp256k1::PublicKey,
    ) -> Result<(), Error> {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().delete_node(&key)
    }

    fn new_channel(
        &self,
        node_id: &lightning_signer::bitcoin::secp256k1::PublicKey,
        stub: &ChannelStub,
    ) -> Result<(), Error> {
        let id = vls_persist::model::NodeChannelId::new(node_id, &stub.id0);
        let channel_value_satoshis = 0;
        let enforcement_state = lightning_signer::policy::validator::EnforcementState::new(0);
        let entry = vls_persist::model::ChannelEntry {
            channel_value_satoshis,
            channel_setup: None,
            id: None,
            enforcement_state,
            // birth blockheight for stub, None for channel
            blockheight: Some(stub.blockheight),
        };
        let id = hex::encode(id.0);

        self.state.lock().unwrap().insert_channel(&id, entry)
    }

    fn update_channel(
        &self,
        node_id: &lightning_signer::bitcoin::secp256k1::PublicKey,
        channel: &lightning_signer::channel::Channel,
    ) -> Result<(), Error> {
        let node_channel_id = vls_persist::model::NodeChannelId::new(node_id, &channel.id0);
        let id = hex::encode(node_channel_id.0);
        let channel_value_satoshis = channel.setup.channel_value_sat;
        let entry = vls_persist::model::ChannelEntry {
            channel_value_satoshis,
            channel_setup: Some(channel.setup.clone()),
            id: channel.id.clone(),
            enforcement_state: channel.enforcement_state.clone(),
            blockheight: None,
        };
        self.state.lock().unwrap().update_channel(&id, entry)
    }

    fn get_channel(
        &self,
        node_id: &PublicKey,
        channel_id: &ChannelId,
    ) -> Result<lightning_signer::persist::model::ChannelEntry, Error> {
        let id = vls_persist::model::NodeChannelId::new(node_id, channel_id);
        let id = hex::encode(id.0);
        self.state.lock().unwrap().get_channel(&id)
    }

    fn new_tracker(
        &self,
        node_id: &PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) -> Result<(), Error> {
        self.state
            .lock()
            .unwrap()
            .new_chain_tracker(node_id, tracker)
    }

    fn update_tracker(
        &self,
        node_id: &PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) -> Result<(), Error> {
        let key = hex::encode(node_id.serialize());
        let key = format!("{TRACKER_PREFIX}/{key}");

        let mut state = self.state.lock().unwrap();
        let v = state.values.get_mut(&key).unwrap();
        let tracker: vls_persist::model::ChainTrackerEntry = tracker.into();
        *v = (v.0 + 1, serde_json::to_value(tracker).unwrap());
        Ok(())
    }

    fn get_tracker(
        &self,
        node_id: PublicKey,
        validator_factory: Arc<dyn ValidatorFactory>,
    ) -> Result<
        (
            ChainTracker<lightning_signer::monitor::ChainMonitor>,
            Vec<ChainTrackerListenerEntry>,
        ),
        Error,
    > {
        let key = hex::encode(node_id.serialize());
        let key = format!("{TRACKER_PREFIX}/{key}");

        let state = self.state.lock().unwrap();
        if state.is_tombstoned(&key) {
            return Err(Error::Internal(format!("tracker {} has been deleted", key)));
        }
        let v: vls_persist::model::ChainTrackerEntry =
            serde_json::from_value(
                state
                    .values
                    .get(&key)
                    .ok_or_else(|| Error::Internal(format!("missing tracker state {}", key)))?
                    .1
                    .clone(),
            )
            .unwrap();

        Ok(v.into_tracker(node_id, validator_factory))
    }

    fn get_node_channels(
        &self,
        node_id: &PublicKey,
    ) -> Result<Vec<(ChannelId, lightning_signer::persist::model::ChannelEntry)>, Error> {
        self.state.lock().unwrap().get_node_channels(node_id)
    }

    fn update_node_allowlist(
        &self,
        node_id: &PublicKey,
        allowlist: Vec<std::string::String>,
    ) -> Result<(), Error> {
        let key = hex::encode(node_id.serialize());
        let key = format!("{ALLOWLIST_PREFIX}/{key}");

        let mut state = self.state.lock().unwrap();
        match state.values.get_mut(&key) {
            Some(v) => {
                *v = (v.0 + 1, serde_json::to_value(allowlist).unwrap());
            }
            None => {
                let version = state.next_version(&key);
                state
                    .values
                    .insert(key, (version, serde_json::to_value(allowlist).unwrap()));
            }
        }
        Ok(())
    }

    fn get_node_allowlist(&self, node_id: &PublicKey) -> Result<Vec<std::string::String>, Error> {
        let state = self.state.lock().unwrap();
        let key = hex::encode(node_id.serialize());
        let key = format!("{ALLOWLIST_PREFIX}/{key}");
        if state.is_tombstoned(&key) {
            return Ok(Vec::new());
        }

        // If allowlist doesn't exist (e.g., node created before VLS 0.14), default to empty
        let allowlist: Vec<String> = match state.values.get(&key) {
            Some(value) => serde_json::from_value(value.1.clone()).unwrap_or_default(),
            None => Vec::new(),
        };

        Ok(allowlist)
    }

    fn get_nodes(
        &self,
    ) -> Result<Vec<(PublicKey, lightning_signer::persist::model::NodeEntry)>, Error> {
        use lightning_signer::node::NodeState as CoreNodeState;

        let state = self.state.lock().unwrap();
        let node_ids: Vec<&str> = state
            .values
            .keys()
            .filter(|k| k.starts_with(&format!("{NODE_PREFIX}/")))
            .filter(|k| !state.is_tombstoned(k))
            .filter_map(|k| k.split('/').last())
            .collect();

        let mut res = Vec::new();
        for node_id in node_ids.iter() {
            let node_key = format!("{NODE_PREFIX}/{node_id}");
            let state_key = format!("{NODE_STATE_PREFIX}/{node_id}");
            let allowlist_key = format!("{ALLOWLIST_PREFIX}/{node_id}");

            if state.is_tombstoned(&node_key) || state.is_tombstoned(&state_key) {
                continue;
            }

            let node: vls_persist::model::NodeEntry = match state.values.get(&node_key) {
                Some(value) => serde_json::from_value(value.1.clone()).unwrap(),
                None => continue,
            };
            let state_e: vls_persist::model::NodeStateEntry = match state.values.get(&state_key) {
                Some(value) => serde_json::from_value(value.1.clone()).unwrap(),
                None => continue,
            };

            // Load allowlist, defaulting to empty if not found (for nodes created before VLS 0.14)
            let allowlist_strings: Vec<String> = if state.is_tombstoned(&allowlist_key) {
                Vec::new()
            } else {
                match state.values.get(&allowlist_key) {
                    Some(value) => serde_json::from_value(value.1.clone()).unwrap_or_default(),
                    None => Vec::new(),
                }
            };

            // Parse allowlist strings into Allowable objects
            use lightning_signer::node::Allowable;
            let network = lightning_signer::bitcoin::Network::from_str(&node.network)
                .map_err(|e| Error::Internal(format!("Invalid network: {}", e)))?;

            let allowlist: Vec<Allowable> = allowlist_strings
                .into_iter()
                .filter_map(|s| {
                    match Allowable::from_str(&s, network) {
                        Ok(a) => Some(a),
                        Err(e) => {
                            warn!("Failed to parse allowlist entry '{}': {}", s, e);
                            None
                        }
                    }
                })
                .collect();

            let state = CoreNodeState::restore(
                state_e.invoices,
                state_e.issued_invoices,
                state_e.preimages,
                0,
                state_e.velocity_control.into(),
                state_e.fee_velocity_control.into(),
                0u64,
                /* dbid_high_water_mark: prevents reuse of
                 * channel dbid, 0 disables enforcement. */
                allowlist,
            );

            let entry = lightning_signer::persist::model::NodeEntry {
                key_derivation_style: node.key_derivation_style,
                network: node.network,
                state,
            };

            let key: Vec<u8> = hex::decode(node_id).unwrap();
            res.push((PublicKey::from_slice(key.as_slice()).unwrap(), entry));
        }

        let nodes = res;
        Ok(nodes)
    }
    fn clear_database(&self) -> Result<(), Error> {
        self.state.lock().unwrap().clear()
    }

    fn signer_id(&self) -> SignerId {
        // The signers are clones of each other in Greenlight, and as
        // such we should not need to differentiate them. We therefore
        // just return a static dummy ID.
        [0u8; 16]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        State, StateSketch, ALLOWLIST_PREFIX, CHANNEL_PREFIX, NODE_PREFIX, NODE_STATE_PREFIX,
        TOMBSTONE_PREFIX, TRACKER_PREFIX,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    fn mk_state(entries: Vec<(&str, u64, serde_json::Value)>) -> State {
        let mut values = BTreeMap::new();
        for (key, version, value) in entries {
            values.insert(key.to_string(), (version, value));
        }
        State { values }
    }

    fn assert_entry(state: &State, key: &str, expected_version: u64, expected_value: serde_json::Value) {
        let (actual_version, actual_value) = state.values.get(key).unwrap_or_else(|| {
            panic!("expected state to include key: {key}")
        });
        assert_eq!(*actual_version, expected_version);
        assert_eq!(actual_value, &expected_value);
    }

    fn assert_entry_absent(state: &State, key: &str) {
        assert!(state.values.get(key).is_none(), "expected state to omit key {key}");
    }

    #[test]
    fn diff_state_only_includes_new_or_newer_entries() {
        let old = mk_state(vec![
            ("k1", 1, json!({"v": 1})),
            ("k2", 2, json!({"v": 2})),
            ("k3", 3, json!({"v": 3})),
        ]);
        let new = mk_state(vec![
            // unchanged version, changed value should still be ignored
            ("k1", 1, json!({"v": 999})),
            // newer version should be included
            ("k2", 3, json!({"v": 22})),
            // older version should be ignored
            ("k3", 2, json!({"v": 33})),
            // brand new key should be included
            ("k4", 0, json!({"v": 4})),
        ]);

        let diff = old.diff_state(&new);

        assert_eq!(diff.values.len(), 2);
        assert_entry(&diff, "k2", 3, json!({"v": 22}));
        assert_entry(&diff, "k4", 0, json!({"v": 4}));
        assert_entry_absent(&diff, "k1");
        assert_entry_absent(&diff, "k3");
    }

    #[test]
    fn sate_diff_with_empty_old_state_includes_all_entries() {
        let old = State::new();
        let new = mk_state(vec![
            ("k1", 1, json!({"v": 1})),
            ("k2", 2, json!({"v": 2})),
        ]);

        let diff = old.diff_state(&new);

        assert_eq!(diff.values.len(), 2);
        assert_entry(&diff, "k1", 1, json!({"v": 1}));
        assert_entry(&diff, "k2", 2, json!({"v": 2}));
    }

    #[test]
    fn sketch_diff_matches_state_diff() {
        let old = mk_state(vec![
            ("a", 5, json!(1)),
            ("b", 2, json!(2)),
            ("c", 7, json!(3)),
        ]);
        let new = mk_state(vec![
            ("a", 5, json!(10)),
            ("b", 3, json!(20)),
            ("c", 6, json!(30)),
            ("d", 1, json!(40)),
        ]);

        let state_diff = old.diff_state(&new);
        let sketch_diff = old.sketch().diff_state(&new);

        assert_eq!(state_diff.values, sketch_diff.values);
    }

    #[test]
    fn sketch_diff_with_empty_sketch_includes_all_entries() {
        let state = mk_state(vec![
            ("a", 1, json!(1)),
            ("b", 2, json!(2)),
        ]);
        let sketch = StateSketch::new();

        let diff = sketch.diff_state(&state);

        assert_eq!(diff.values.len(), 2);
        assert_entry(&diff, "a", 1, json!(1));
        assert_entry(&diff, "b", 2, json!(2));
    }

    #[test]
    fn sketch_apply_follow_version_updates() {
        let base = mk_state(vec![("a", 1, json!(1)), ("b", 2, json!(2))]);
        let mut sketch = StateSketch::new();
        sketch.apply_state(&base);

        let next = mk_state(vec![("a", 2, json!(10)), ("b", 2, json!(20)), ("c", 0, json!(30))]);
        let first_diff = sketch.diff_state(&next);
        assert_eq!(first_diff.values.len(), 2);
        assert_entry(&first_diff, "a", 2, json!(10));
        assert_entry(&first_diff, "c", 0, json!(30));
        assert_entry_absent(&first_diff, "b");

        sketch.apply_state(&first_diff);
        let second_diff = sketch.diff_state(&next);
        assert!(second_diff.values.is_empty());
    }

    #[test]
    fn merge_tombstone_deletes_older_live_entry() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let tombstone_key = format!("{TOMBSTONE_PREFIX}/{live_key}");
        let mut state = mk_state(vec![(live_key.as_str(), 2, json!({"v": 1}))]);
        let incoming = mk_state(vec![(tombstone_key.as_str(), 3, serde_json::Value::Null)]);

        let res = state.safe_merge(&incoming).unwrap();

        assert_entry_absent(&state, &live_key);
        assert_entry(&state, &tombstone_key, 3, serde_json::Value::Null);
        assert_eq!(res.conflict_count, 0);
    }

    #[test]
    fn merge_ignores_live_entry_if_tombstone_is_newer_or_equal() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let tombstone_key = format!("{TOMBSTONE_PREFIX}/{live_key}");
        let mut state = mk_state(vec![(tombstone_key.as_str(), 5, serde_json::Value::Null)]);
        let incoming = mk_state(vec![(live_key.as_str(), 4, json!({"v": 1}))]);

        let res = state.safe_merge(&incoming).unwrap();

        assert_entry_absent(&state, &live_key);
        assert_entry(&state, &tombstone_key, 5, serde_json::Value::Null);
        assert_eq!(res.conflict_count, 1);
    }

    #[test]
    fn merge_accepts_live_entry_if_newer_than_tombstone() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let tombstone_key = format!("{TOMBSTONE_PREFIX}/{live_key}");
        let mut state = mk_state(vec![(tombstone_key.as_str(), 5, serde_json::Value::Null)]);
        let incoming = mk_state(vec![(live_key.as_str(), 6, json!({"v": 2}))]);

        let res = state.safe_merge(&incoming).unwrap();

        assert_entry(&state, &live_key, 6, json!({"v": 2}));
        assert_entry(&state, &tombstone_key, 5, serde_json::Value::Null);
        assert_eq!(res.conflict_count, 0);
    }

    #[test]
    fn safe_merge_reports_conflict_for_outdated_live_version() {
        let mut state = mk_state(vec![("k1", 5, json!({"v": 5}))]);
        let incoming = mk_state(vec![("k1", 4, json!({"v": 4}))]);

        let res = state.safe_merge(&incoming).unwrap();

        assert_eq!(res.conflict_count, 1);
        assert_entry(&state, "k1", 5, json!({"v": 5}));
    }

    #[test]
    fn delete_channel_creates_tombstone_and_bumps_version() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let tombstone_key = format!("{TOMBSTONE_PREFIX}/{live_key}");
        let mut state = mk_state(vec![(live_key.as_str(), 7, json!({"v": 1}))]);

        state.delete_channel("abc");

        assert_entry_absent(&state, &live_key);
        assert_entry(&state, &tombstone_key, 8, serde_json::Value::Null);
    }

    #[test]
    fn delete_node_creates_tombstones_for_node_related_keys() {
        let node_id = "deadbeef";
        let node_key = format!("{NODE_PREFIX}/{node_id}");
        let node_state_key = format!("{NODE_STATE_PREFIX}/{node_id}");
        let allowlist_key = format!("{ALLOWLIST_PREFIX}/{node_id}");
        let tracker_key = format!("{TRACKER_PREFIX}/{node_id}");
        let channel_key = format!("{CHANNEL_PREFIX}/{node_id}cafebabe");

        let mut state = mk_state(vec![
            (node_key.as_str(), 1, json!({"n": 1})),
            (node_state_key.as_str(), 2, json!({"s": 1})),
            (allowlist_key.as_str(), 3, json!(["127.0.0.1"])),
            (tracker_key.as_str(), 4, json!({"t": 1})),
            (channel_key.as_str(), 5, json!({"c": 1})),
        ]);

        state.delete_node(node_id).unwrap();

        for (live_key, old_version) in vec![
            (node_key, 1u64),
            (node_state_key, 2u64),
            (allowlist_key, 3u64),
            (tracker_key, 4u64),
            (channel_key, 5u64),
        ] {
            assert_entry_absent(&state, &live_key);
            let tombstone_key = format!("{TOMBSTONE_PREFIX}/{live_key}");
            assert_entry(&state, &tombstone_key, old_version + 1, serde_json::Value::Null);
        }
    }
}
