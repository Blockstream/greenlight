mod canonical;

use anyhow::anyhow;
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
use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use self::canonical::{canonical_json_bytes, CanonicalJsonValue};

const NODE_PREFIX: &str = "nodes";
const NODE_STATE_PREFIX: &str = "nodestates";
const CHANNEL_PREFIX: &str = "channels";
const ALLOWLIST_PREFIX: &str = "allowlists";
const TRACKER_PREFIX: &str = "trackers";
const TOMBSTONE_VERSION: u64 = u64::MAX;

#[derive(Clone, Debug, PartialEq)]
struct StateEntry {
    version: u64,
    value: serde_json::Value,
    signature: Vec<u8>,
}

impl StateEntry {
    fn new(version: u64, value: serde_json::Value) -> Self {
        Self {
            version,
            value,
            signature: vec![],
        }
    }

    fn canonical_value_bytes(&self) -> anyhow::Result<Vec<u8>> {
        canonical_json_bytes(&self.value)
    }
}

impl Serialize for StateEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = if self.signature.is_empty() {
            serializer.serialize_seq(Some(2))?
        } else {
            serializer.serialize_seq(Some(3))?
        };

        seq.serialize_element(&self.version)?;
        seq.serialize_element(&CanonicalJsonValue(&self.value))?;
        if !self.signature.is_empty() {
            seq.serialize_element(&self.signature)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for StateEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StateEntryVisitor;

        impl<'de> Visitor<'de> for StateEntryVisitor {
            type Value = StateEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple [version, value] or [version, value, signature]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let version = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let value = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let signature = seq.next_element()?.unwrap_or_default();

                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(4, &self));
                }

                Ok(StateEntry {
                    version,
                    value,
                    signature,
                })
            }
        }

        deserializer.deserialize_seq(StateEntryVisitor)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct State {
    values: BTreeMap<String, StateEntry>,
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
        self.ensure_not_tombstone(&node_key)?;
        self.ensure_not_tombstone(&state_key)?;
        let node_version = self.next_version(&node_key);
        let state_version = self.next_version(&state_key);

        self.values.insert(
            node_key,
            StateEntry::new(node_version, serde_json::to_value(node_entry).unwrap()),
        );
        self.values.insert(
            state_key,
            StateEntry::new(
                state_version,
                serde_json::to_value(node_state_entry).unwrap(),
            ),
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
        self.ensure_not_tombstone(&key)?;
        let v = self
            .values
            .get_mut(&key)
            .expect("attempting to update non-existent node");
        *v = StateEntry::new(v.version + 1, serde_json::to_value(node_state).unwrap());
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
        self.ensure_not_tombstone(&key)?;
        let version = self.next_version(&key);
        self.values.insert(
            key,
            StateEntry::new(version, serde_json::to_value(channel_entry).unwrap()),
        );
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
        self.ensure_not_tombstone(&key)?;
        let v = self
            .values
            .get_mut(&key)
            .expect("attempting to update non-existent channel");
        *v = StateEntry::new(v.version + 1, serde_json::to_value(channel_entry).unwrap());
        Ok(())
    }

    fn get_channel(
        &self,
        key: &str,
    ) -> Result<lightning_signer::persist::model::ChannelEntry, Error> {
        let key = format!("{CHANNEL_PREFIX}/{key}");
        if self.is_tombstone(&key) {
            return Err(Error::Internal(format!("channel {} has been deleted", key)));
        }
        let value = self
            .values
            .get(&key)
            .ok_or_else(|| Error::Internal(format!("missing channel state for key {}", key)))?;
        let entry: vls_persist::model::ChannelEntry =
            serde_json::from_value(value.value.clone()).unwrap();
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
            .filter(|(k, _)| !self.is_tombstone(k))
            .map(|(k, v)| {
                let key = k.split('/').last().unwrap();
                let key = vls_persist::model::NodeChannelId(hex::decode(&key).unwrap());

                let value: vls_persist::model::ChannelEntry =
                    serde_json::from_value(v.value.clone()).unwrap();
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
        self.ensure_not_tombstone(&key)?;
        let version = self.next_version(&key);

        let tracker: vls_persist::model::ChainTrackerEntry = tracker.into();

        self.values.insert(
            key,
            StateEntry::new(version, serde_json::to_value(tracker).unwrap()),
        );
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.values.clear();
        Ok(())
    }

    fn put_tombstone(&mut self, live_key: &str) {
        self.values.insert(
            live_key.to_owned(),
            StateEntry::new(TOMBSTONE_VERSION, serde_json::Value::Null),
        );
    }

    fn next_version(&self, live_key: &str) -> u64 {
        self.values
            .get(live_key)
            .map(|v| v.version.saturating_add(1))
            .unwrap_or(0)
    }

    fn is_tombstone(&self, live_key: &str) -> bool {
        self.values
            .get(live_key)
            .map(|v| v.version == TOMBSTONE_VERSION)
            .unwrap_or(false)
    }

    fn ensure_not_tombstone(&self, key: &str) -> Result<(), Error> {
        if self.is_tombstone(key) {
            return Err(Error::Internal(format!("key {} has been deleted", key)));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StateChange {
    key: String,
    old: Option<StateEntry>,
    new: StateEntry,
}

#[derive(Debug, Default)]
pub struct MergeResult {
    pub changes: Vec<(String, Option<u64>, u64)>,
    pub conflict_count: usize,
}

impl MergeResult {
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
                o.version,
                self.new.version,
                serde_json::to_string(&o.value).unwrap(),
                serde_json::to_string(&self.new.value).unwrap()
            )),
            None => f.write_str(&format!(
                "StateChange[{}]: old_version=null, new_version={}, old_value=null, new_value={}",
                self.key,
                self.new.version,
                serde_json::to_string(&self.new.value).unwrap()
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
    pub fn merge(&mut self, other: &State) -> anyhow::Result<MergeResult> {
        let mut res = MergeResult::default();
        for (key, incoming) in other.values.iter() {
            let newver = incoming.version;
            let incoming_is_tombstone = newver == TOMBSTONE_VERSION;
            match self.values.get_mut(key) {
                None => {
                    trace!("Insert new key {}: version={}", key, newver);
                    res.changes.push((key.to_owned(), None, newver));
                    let mut inserted = incoming.clone();
                    if incoming_is_tombstone {
                        inserted.value = serde_json::Value::Null;
                    }
                    self.values.insert(key.clone(), inserted);
                }
                Some(v) => {
                    if incoming_is_tombstone {
                        if v.version == TOMBSTONE_VERSION {
                            continue;
                        }
                        trace!(
                            "Tombstoning key {}: version={} => version={}",
                            key,
                            v.version,
                            newver
                        );
                        res.changes.push((key.to_owned(), Some(v.version), newver));
                        *v = StateEntry {
                            version: newver,
                            value: serde_json::Value::Null,
                            signature: incoming.signature.clone(),
                        };
                        continue;
                    }

                    if v.version == TOMBSTONE_VERSION {
                        trace!(
                            "Ignoring live key {} version={} because local key is tombstoned",
                            key,
                            newver
                        );
                        res.conflict_count += 1;
                        continue;
                    }

                    if v.version == newver {
                        if v.value == incoming.value && v.signature != incoming.signature {
                            trace!("Updating signature for key {} at version={}", key, newver);
                            res.changes.push((key.to_owned(), Some(v.version), newver));
                            v.signature = incoming.signature.clone();
                        }
                        continue;
                    } else if v.version > newver {
                        warn!(
                            "Ignoring outdated state version newver={}, we have oldver={}: newval={:?} vs oldval={:?}",
                            newver,
                            v.version,
                            serde_json::to_string(&incoming.value),
                            serde_json::to_string(&v.value)
                        );
                        res.conflict_count += 1;
                        continue;
                    } else {
                        trace!(
                            "Updating key {}: version={} => version={}",
                            key,
                            v.version,
                            newver
                        );
                        res.changes.push((key.to_owned(), Some(v.version), newver));
                        *v = incoming.clone();
                    }
                }
            }
        }
        Ok(res)
    }

    pub fn diff(&self, other: &State) -> anyhow::Result<Vec<StateChange>> {
        Ok(other
            .values
            .iter()
            .map(|(key, new)| (key, self.values.get(key), new))
            .map(|(key, old, new)| StateChange {
                key: key.clone(),
                old: old.map(|o| o.clone()),
                new: new.clone(),
            })
            .filter(|c| match (&c.old, &c.new) {
                (None, _) => true,
                (Some(old), new) => old.version < new.version,
            })
            .collect())
    }

    /// Return a `State` containing only entries that are newer in `other` than in `self`.
    /// This is useful for sending compact state diffs.
    pub fn diff_state(&self, other: &State) -> State {
        let mut values = BTreeMap::new();
        for (key, new_entry) in other.values.iter() {
            match self.values.get(key) {
                None => {
                    values.insert(key.clone(), new_entry.clone());
                }
                Some(old_entry) if old_entry.version < new_entry.version => {
                    values.insert(key.clone(), new_entry.clone());
                }
                Some(old_entry)
                    if old_entry.version == new_entry.version
                        && old_entry.signature != new_entry.signature =>
                {
                    values.insert(key.clone(), new_entry.clone());
                }
                _ => {}
            }
        }
        State { values }
    }

    /// Sign entries missing signatures and return how many signatures were added.
    pub fn resign_signatures<F>(&mut self, mut signer: F) -> anyhow::Result<usize>
    where
        F: FnMut(&str, u64, &[u8]) -> anyhow::Result<Vec<u8>>,
    {
        let mut changed = 0usize;
        for (key, entry) in self.values.iter_mut() {
            if !entry.signature.is_empty() {
                continue;
            }
            let value = entry.canonical_value_bytes()?;
            let signature = signer(key, entry.version, &value)?;
            entry.signature = signature;
            changed += 1;
        }
        Ok(changed)
    }

    pub fn sketch(&self) -> StateSketch {
        StateSketch::from_state(self)
    }

    // Return a copy of the state with tombstoned entries omitted.
    pub fn omit_tombstones(&self) -> State {
        let values = self
            .values
            .iter()
            .filter(|(_, value)| value.version != TOMBSTONE_VERSION)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        State { values }
    }

    pub(crate) fn recoverable_channel_keys(&self) -> BTreeSet<String> {
        self.values
            .iter()
            .filter(|(_, value)| value.version != TOMBSTONE_VERSION)
            .filter(|(key, _)| key.starts_with(&format!("{CHANNEL_PREFIX}/")))
            .filter(|(_, value)| {
                value
                    .value
                    .get("channel_setup")
                    .map(|setup| !setup.is_null())
                    .unwrap_or(false)
            })
            .map(|(key, _)| key.clone())
            .collect()
    }

    pub(crate) fn recoverable_channel_values(&self) -> Vec<(String, serde_json::Value)> {
        self.values
            .iter()
            .filter(|(_, value)| value.version != TOMBSTONE_VERSION)
            .filter(|(key, _)| key.starts_with(&format!("{CHANNEL_PREFIX}/")))
            .filter(|(_, value)| {
                value
                    .value
                    .get("channel_setup")
                    .map(|setup| !setup.is_null())
                    .unwrap_or(false)
            })
            .map(|(key, value)| (key.clone(), value.value.clone()))
            .collect()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct SketchEntry {
    version: u64,
    signature: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct StateSketch {
    versions: BTreeMap<String, SketchEntry>,
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
        for (key, value) in state.values.iter() {
            self.versions.insert(
                key.clone(),
                SketchEntry {
                    version: value.version,
                    signature: value.signature.clone(),
                },
            );
        }
    }

    /// Build a `State` containing entries newer than those recorded in the sketch.
    pub fn diff_state(&self, state: &State) -> State {
        let mut values = BTreeMap::new();
        for (key, new_entry) in state.values.iter() {
            match self.versions.get(key) {
                None => {
                    values.insert(key.clone(), new_entry.clone());
                }
                Some(old) if old.version < new_entry.version => {
                    values.insert(key.clone(), new_entry.clone());
                }
                Some(old)
                    if old.version == new_entry.version && old.signature != new_entry.signature =>
                {
                    values.insert(key.clone(), new_entry.clone());
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
                value: v
                    .canonical_value_bytes()
                    .expect("canonical signer state value"),
                version: v.version,
                signature: v.signature.clone(),
            })
            .collect()
    }
}

impl TryFrom<&[crate::pb::SignerStateEntry]> for State {
    type Error = anyhow::Error;

    fn try_from(v: &[crate::pb::SignerStateEntry]) -> Result<State, Self::Error> {
        let values = v
            .iter()
            .map(|entry| -> anyhow::Result<(String, StateEntry)> {
                let value = serde_json::from_slice(&entry.value).map_err(|e| {
                    anyhow!(
                        "failed to decode signer state value for key {}: {}",
                        entry.key,
                        e
                    )
                })?;

                Ok((
                    entry.key.to_owned(),
                    StateEntry {
                        version: entry.version,
                        value,
                        signature: entry.signature.clone(),
                    },
                ))
            })
            .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

        Ok(State { values })
    }
}

impl From<Vec<crate::pb::SignerStateEntry>> for State {
    fn from(v: Vec<crate::pb::SignerStateEntry>) -> State {
        State::try_from(v.as_slice())
            .expect("signer state entries must contain valid JSON payloads")
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
        state.ensure_not_tombstone(&key)?;
        let v = state.values.get_mut(&key).unwrap();
        let tracker: vls_persist::model::ChainTrackerEntry = tracker.into();
        *v = StateEntry::new(v.version + 1, serde_json::to_value(tracker).unwrap());
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
        if state.is_tombstone(&key) {
            return Err(Error::Internal(format!("tracker {} has been deleted", key)));
        }
        let v: vls_persist::model::ChainTrackerEntry = serde_json::from_value(
            state
                .values
                .get(&key)
                .ok_or_else(|| Error::Internal(format!("missing tracker state {}", key)))?
                .value
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
        state.ensure_not_tombstone(&key)?;
        match state.values.get_mut(&key) {
            Some(v) => {
                *v = StateEntry::new(v.version + 1, serde_json::to_value(allowlist).unwrap());
            }
            None => {
                let version = state.next_version(&key);
                state.values.insert(
                    key,
                    StateEntry::new(version, serde_json::to_value(allowlist).unwrap()),
                );
            }
        }
        Ok(())
    }

    fn get_node_allowlist(&self, node_id: &PublicKey) -> Result<Vec<std::string::String>, Error> {
        let state = self.state.lock().unwrap();
        let key = hex::encode(node_id.serialize());
        let key = format!("{ALLOWLIST_PREFIX}/{key}");
        if state.is_tombstone(&key) {
            return Ok(Vec::new());
        }

        // If allowlist doesn't exist (e.g., node created before VLS 0.14), default to empty
        let allowlist: Vec<String> = match state.values.get(&key) {
            Some(value) => serde_json::from_value(value.value.clone()).unwrap_or_default(),
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
            .filter(|k| !state.is_tombstone(k))
            .filter_map(|k| k.split('/').last())
            .collect();

        let mut res = Vec::new();
        for node_id in node_ids.iter() {
            let node_key = format!("{NODE_PREFIX}/{node_id}");
            let state_key = format!("{NODE_STATE_PREFIX}/{node_id}");
            let allowlist_key = format!("{ALLOWLIST_PREFIX}/{node_id}");

            if state.is_tombstone(&node_key) || state.is_tombstone(&state_key) {
                continue;
            }

            let node: vls_persist::model::NodeEntry = match state.values.get(&node_key) {
                Some(value) => serde_json::from_value(value.value.clone()).unwrap(),
                None => continue,
            };
            let state_e: vls_persist::model::NodeStateEntry = match state.values.get(&state_key) {
                Some(value) => serde_json::from_value(value.value.clone()).unwrap(),
                None => continue,
            };

            // Load allowlist, defaulting to empty if not found (for nodes created before VLS 0.14)
            let allowlist_strings: Vec<String> = if state.is_tombstone(&allowlist_key) {
                Vec::new()
            } else {
                match state.values.get(&allowlist_key) {
                    Some(value) => serde_json::from_value(value.value.clone()).unwrap_or_default(),
                    None => Vec::new(),
                }
            };

            // Parse allowlist strings into Allowable objects
            use lightning_signer::node::Allowable;
            let network = lightning_signer::bitcoin::Network::from_str(&node.network)
                .map_err(|e| Error::Internal(format!("Invalid network: {}", e)))?;

            let allowlist: Vec<Allowable> = allowlist_strings
                .into_iter()
                .filter_map(|s| match Allowable::from_str(&s, network) {
                    Ok(a) => Some(a),
                    Err(e) => {
                        warn!("Failed to parse allowlist entry '{}': {}", s, e);
                        None
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
    use crate::persist::TOMBSTONE_VERSION;

    use super::{
        State, StateEntry, StateSketch, ALLOWLIST_PREFIX, CHANNEL_PREFIX, NODE_PREFIX,
        NODE_STATE_PREFIX, TRACKER_PREFIX,
    };
    use crate::pb::SignerStateEntry;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn mk_state(entries: Vec<(&str, u64, serde_json::Value)>) -> State {
        let mut values = BTreeMap::new();
        for (key, version, value) in entries {
            values.insert(key.to_string(), StateEntry::new(version, value));
        }
        State { values }
    }

    fn assert_entry(
        state: &State,
        key: &str,
        expected_version: u64,
        expected_value: serde_json::Value,
    ) {
        let actual = state
            .values
            .get(key)
            .unwrap_or_else(|| panic!("expected state to include key: {key}"));
        assert_eq!(actual.version, expected_version);
        assert_eq!(actual.value, expected_value);
    }

    fn assert_entry_absent(state: &State, key: &str) {
        assert!(
            state.values.get(key).is_none(),
            "expected state to omit key {key}"
        );
    }

    fn assert_tombstone(state: &State, key: &str) {
        assert_entry(state, key, TOMBSTONE_VERSION, serde_json::Value::Null);
    }

    #[test]
    fn state_deserialize_legacy_tuple_defaults_empty_signature() {
        let raw = r#"{"values":{"k":[1,{"v":1}]}}"#;
        let state: State = serde_json::from_str(raw).unwrap();
        let entry = state.values.get("k").unwrap();
        assert_eq!(entry.version, 1);
        assert_eq!(entry.value, json!({"v": 1}));
        assert!(entry.signature.is_empty());
    }

    #[test]
    fn state_deserialize_extended_tuple_preserves_signature() {
        let raw = r#"{"values":{"k":[2,{"v":2},[1,2,3]]}}"#;
        let state: State = serde_json::from_str(raw).unwrap();
        let entry = state.values.get("k").unwrap();
        assert_eq!(entry.version, 2);
        assert_eq!(entry.value, json!({"v": 2}));
        assert_eq!(entry.signature, vec![1, 2, 3]);
    }

    #[test]
    fn state_serialize_empty_signature_emits_legacy_tuple() {
        let state = mk_state(vec![("k", 3, json!({"v": 3}))]);
        let v: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        let values = v.get("values").unwrap().as_object().unwrap();
        let tuple = values.get("k").unwrap().as_array().unwrap();
        assert_eq!(tuple.len(), 2);
        assert_eq!(tuple[0], json!(3));
        assert_eq!(tuple[1], json!({"v": 3}));
    }

    #[test]
    fn state_serialize_non_empty_signature_emits_extended_tuple() {
        let mut values = BTreeMap::new();
        values.insert(
            "k".to_string(),
            StateEntry {
                version: 4,
                value: json!({"v": 4}),
                signature: vec![7, 8, 9],
            },
        );
        let state = State { values };
        let v: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        let values = v.get("values").unwrap().as_object().unwrap();
        let tuple = values.get("k").unwrap().as_array().unwrap();
        assert_eq!(tuple.len(), 3);
        assert_eq!(tuple[0], json!(4));
        assert_eq!(tuple[1], json!({"v": 4}));
        assert_eq!(tuple[2], json!([7, 8, 9]));
    }

    #[test]
    fn state_entry_canonical_value_bytes_sorts_nested_object_keys() {
        let entry = StateEntry::new(0, json!({
            "z": {"b": 1, "a": 2},
            "a": [{"d": 4, "c": 3}]
        }));

        let bytes = entry.canonical_value_bytes().unwrap();

        assert_eq!(bytes, br#"{"a":[{"c":3,"d":4}],"z":{"a":2,"b":1}}"#);
    }

    #[test]
    fn signer_state_entry_conversions_preserve_signature() {
        let entries = vec![SignerStateEntry {
            version: 5,
            key: "k".to_string(),
            value: serde_json::to_vec(&json!({"v": 5})).unwrap(),
            signature: vec![11, 12],
        }];

        let state: State = entries.clone().into();
        let entry = state.values.get("k").unwrap();
        assert_eq!(entry.version, 5);
        assert_eq!(entry.value, json!({"v": 5}));
        assert_eq!(entry.signature, vec![11, 12]);

        let roundtrip: Vec<SignerStateEntry> = state.into();
        assert_eq!(roundtrip, entries);
    }

    #[test]
    fn signer_state_entry_conversions_emit_canonical_value_bytes() {
        let entries = vec![SignerStateEntry {
            version: 6,
            key: "k".to_string(),
            value: br#"{ "b": 1, "a": 2 }"#.to_vec(),
            signature: vec![11, 12],
        }];

        let state: State = entries.into();
        let roundtrip: Vec<SignerStateEntry> = state.into();
        assert_eq!(roundtrip.len(), 1);
        assert_eq!(roundtrip[0].value, br#"{"a":2,"b":1}"#);
        assert_eq!(roundtrip[0].signature, vec![11, 12]);
    }

    #[test]
    fn merge_newer_entry_propagates_signature() {
        let mut base = mk_state(vec![("k", 1, json!({"v": 1}))]);
        let mut incoming_values = BTreeMap::new();
        incoming_values.insert(
            "k".to_string(),
            StateEntry {
                version: 2,
                value: json!({"v": 2}),
                signature: vec![21, 22, 23],
            },
        );
        let incoming = State {
            values: incoming_values,
        };

        let res = base.merge(&incoming).unwrap();
        assert_eq!(res.conflict_count, 0);
        let merged = base.values.get("k").unwrap();
        assert_eq!(merged.version, 2);
        assert_eq!(merged.value, json!({"v": 2}));
        assert_eq!(merged.signature, vec![21, 22, 23]);
    }

    #[test]
    fn merge_same_version_updates_signature_when_value_matches() {
        let mut base_values = BTreeMap::new();
        base_values.insert(
            "k".to_string(),
            StateEntry {
                version: 2,
                value: json!({"v": 1}),
                signature: vec![1],
            },
        );
        let mut base = State {
            values: base_values,
        };

        let mut incoming_values = BTreeMap::new();
        incoming_values.insert(
            "k".to_string(),
            StateEntry {
                version: 2,
                value: json!({"v": 1}),
                signature: vec![9, 9],
            },
        );
        let incoming = State {
            values: incoming_values,
        };

        let res = base.merge(&incoming).unwrap();
        assert_eq!(res.conflict_count, 0);
        let merged = base.values.get("k").unwrap();
        assert_eq!(merged.version, 2);
        assert_eq!(merged.value, json!({"v": 1}));
        assert_eq!(merged.signature, vec![9, 9]);
    }

    #[test]
    fn merge_same_version_does_not_overwrite_value() {
        let mut base_values = BTreeMap::new();
        base_values.insert(
            "k".to_string(),
            StateEntry {
                version: 2,
                value: json!({"v": 1}),
                signature: vec![1],
            },
        );
        let mut base = State {
            values: base_values,
        };

        let mut incoming_values = BTreeMap::new();
        incoming_values.insert(
            "k".to_string(),
            StateEntry {
                version: 2,
                value: json!({"v": 999}),
                signature: vec![9, 9],
            },
        );
        let incoming = State {
            values: incoming_values,
        };

        let _ = base.merge(&incoming).unwrap();
        let merged = base.values.get("k").unwrap();
        assert_eq!(merged.version, 2);
        assert_eq!(merged.value, json!({"v": 1}));
        assert_eq!(merged.signature, vec![1]);
    }

    #[test]
    fn omit_tombstones_omits_tombstoned_entries() {
        let state = mk_state(vec![
            ("k1", 1, json!({"v": 1})),
            ("k2", TOMBSTONE_VERSION, serde_json::Value::Null),
        ]);

        let filtered = state.omit_tombstones();

        assert_eq!(filtered.values.len(), 1);
        assert_entry(&filtered, "k1", 1, json!({"v": 1}));
        assert_entry_absent(&filtered, "k2");
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
    fn diff_state_includes_signature_only_changes() {
        let mut old_values = BTreeMap::new();
        old_values.insert(
            "k".to_string(),
            StateEntry {
                version: 5,
                value: json!({"v": 5}),
                signature: vec![1],
            },
        );
        let old = State { values: old_values };

        let mut new_values = BTreeMap::new();
        new_values.insert(
            "k".to_string(),
            StateEntry {
                version: 5,
                value: json!({"v": 5}),
                signature: vec![2, 3],
            },
        );
        let new = State { values: new_values };

        let diff = old.diff_state(&new);
        assert_eq!(diff.values.len(), 1);
        let entry = diff.values.get("k").unwrap();
        assert_eq!(entry.version, 5);
        assert_eq!(entry.signature, vec![2, 3]);
    }

    #[test]
    fn sate_diff_with_empty_old_state_includes_all_entries() {
        let old = State::new();
        let new = mk_state(vec![("k1", 1, json!({"v": 1})), ("k2", 2, json!({"v": 2}))]);

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
        let state = mk_state(vec![("a", 1, json!(1)), ("b", 2, json!(2))]);
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

        let next = mk_state(vec![
            ("a", 2, json!(10)),
            ("b", 2, json!(20)),
            ("c", 0, json!(30)),
        ]);
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
    fn sketch_diff_includes_signature_only_changes() {
        let mut old_values = BTreeMap::new();
        old_values.insert(
            "k".to_string(),
            StateEntry {
                version: 8,
                value: json!({"v": 8}),
                signature: vec![1],
            },
        );
        let old = State { values: old_values };

        let mut new_values = BTreeMap::new();
        new_values.insert(
            "k".to_string(),
            StateEntry {
                version: 8,
                value: json!({"v": 8}),
                signature: vec![4, 5, 6],
            },
        );
        let new = State { values: new_values };

        let diff = old.sketch().diff_state(&new);
        assert_eq!(diff.values.len(), 1);
        let entry = diff.values.get("k").unwrap();
        assert_eq!(entry.version, 8);
        assert_eq!(entry.signature, vec![4, 5, 6]);
    }

    #[test]
    fn merge_tombstone_deletes_older_live_entry() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let mut state = mk_state(vec![(live_key.as_str(), 2, json!({"v": 1}))]);
        let incoming = mk_state(vec![(live_key.as_str(), u64::MAX, serde_json::Value::Null)]);

        let res = state.merge(&incoming).unwrap();

        assert_tombstone(&state, &live_key);
        assert_eq!(res.conflict_count, 0);
    }

    #[test]
    fn merge_ignores_live_entry_if_key_is_tombstone() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let mut state = mk_state(vec![(live_key.as_str(), u64::MAX, serde_json::Value::Null)]);
        let incoming = mk_state(vec![(live_key.as_str(), 4, json!({"v": 1}))]);

        let res = state.merge(&incoming).unwrap();

        assert_tombstone(&state, &live_key);
        assert_eq!(res.conflict_count, 1);
    }

    #[test]
    fn merge_ignores_newer_live_entry_if_key_is_tombstone() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let mut state = mk_state(vec![(live_key.as_str(), u64::MAX, serde_json::Value::Null)]);
        let incoming = mk_state(vec![(live_key.as_str(), 6, json!({"v": 2}))]);

        let res = state.merge(&incoming).unwrap();

        assert_tombstone(&state, &live_key);
        assert_eq!(res.conflict_count, 1);
    }

    #[test]
    fn safe_merge_reports_conflict_for_outdated_live_version() {
        let mut state = mk_state(vec![("k1", 5, json!({"v": 5}))]);
        let incoming = mk_state(vec![("k1", 4, json!({"v": 4}))]);

        let res = state.merge(&incoming).unwrap();

        assert_eq!(res.conflict_count, 1);
        assert_entry(&state, "k1", 5, json!({"v": 5}));
    }

    #[test]
    fn signer_state_entry_try_from_rejects_invalid_json_value() {
        let entries = vec![SignerStateEntry {
            version: 1,
            key: "bad".to_string(),
            value: b"not-json".to_vec(),
            signature: vec![],
        }];

        let res = State::try_from(entries.as_slice());
        assert!(res.is_err());
    }

    #[test]
    fn resign_signatures_only_signs_missing_entries() {
        let mut values = BTreeMap::new();
        values.insert(
            "signed".to_string(),
            StateEntry {
                version: 1,
                value: json!({"v": 1}),
                signature: vec![9, 9],
            },
        );
        values.insert(
            "unsigned".to_string(),
            StateEntry {
                version: 2,
                value: json!({"v": 2}),
                signature: vec![],
            },
        );

        let mut state = State { values };
        let mut calls = 0usize;
        let changed = state
            .resign_signatures(|_, _, _| {
                calls += 1;
                Ok(vec![1, 2, 3, 4])
            })
            .unwrap();

        assert_eq!(calls, 1);
        assert_eq!(changed, 1);
        assert_eq!(state.values.get("signed").unwrap().signature, vec![9, 9]);
        assert_eq!(
            state.values.get("unsigned").unwrap().signature,
            vec![1, 2, 3, 4]
        );
    }

    #[test]
    fn delete_channel_marks_channel_with_tombstone_version() {
        let live_key = format!("{CHANNEL_PREFIX}/abc");
        let mut state = mk_state(vec![(live_key.as_str(), 7, json!({"v": 1}))]);

        state.delete_channel("abc");

        assert_tombstone(&state, &live_key);
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
            assert_tombstone(&state, &live_key);
            assert!(old_version < u64::MAX);
        }
    }
}
