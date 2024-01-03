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
use std::sync::Arc;
use std::sync::Mutex;

const NODE_PREFIX: &str = "nodes";
const NODE_STATE_PREFIX: &str = "nodestates";
const CHANNEL_PREFIX: &str = "channels";
const ALLOWLIST_PREFIX: &str = "allowlists";
const TRACKER_PREFIX: &str = "trackers";

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
        assert!(!self.values.contains_key(&node_key), "inserting node twice");
        assert!(
            !self.values.contains_key(&state_key),
            "inserting node_state twice"
        );

        self.values
            .insert(node_key, (0u64, serde_json::to_value(node_entry).unwrap()));
        self.values.insert(
            state_key,
            (0u64, serde_json::to_value(node_state_entry).unwrap()),
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

        self.values.remove(&node_key);
        self.values.remove(&state_key);
        Ok(())
    }

    fn insert_channel(
        &mut self,
        key: &str,
        channel_entry: vls_persist::model::ChannelEntry,
    ) -> Result<(), Error> {
        let key = format!("{CHANNEL_PREFIX}/{key}");
        assert!(!self.values.contains_key(&key));
        self.values
            .insert(key, (0u64, serde_json::to_value(channel_entry).unwrap()));
        Ok(())
    }

    fn delete_channel(&mut self, key: &str) {
        self.values.remove(key);
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
        let value = self.values.get(&key).unwrap();
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
        assert!(!self.values.contains_key(&key));

        let tracker: vls_persist::model::ChainTrackerEntry = tracker.into();

        self.values
            .insert(key, (0u64, serde_json::to_value(tracker).unwrap()));
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.values.clear();
        Ok(())
    }
}

#[derive(Debug)]
pub struct StateChange {
    key: String,
    old: Option<(u64, serde_json::Value)>,
    new: (u64, serde_json::Value),
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

    /// Take another `State` and attempt to update ourselves with any
    /// entry that is newer than ours. This may fail if the other
    /// state includes states that are older than our own.
    pub fn merge(&mut self, other: &State) -> anyhow::Result<Vec<(String, Option<u64>, u64)>> {
        let mut res = Vec::new();
        for (key, (newver, newval)) in other.values.iter() {
            let local = self.values.get_mut(key);

            match local {
                None => {
                    trace!("Insert new key {}: version={}", key, newver);
                    res.push((key.to_owned(), None, *newver));
                    self.values.insert(key.clone(), (*newver, newval.clone()));
                }
                Some(v) => {
                    if v.0 == *newver {
                        continue;
                    } else if v.0 > *newver {
                        warn!("Ignoring outdated state version newver={}, we have oldver={}: newval={:?} vs oldval={:?}", newver, v.0, serde_json::to_string(newval), serde_json::to_string(&v.1));
                        continue;
                    } else {
                        trace!(
                            "Updating key {}: version={} => version={}",
                            key,
                            v.0,
                            *newver
                        );
                        res.push((key.to_owned(), Some(v.0), *newver));
                        *v = (*newver, newval.clone());
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
        let v: vls_persist::model::ChainTrackerEntry =
            serde_json::from_value(state.values.get(&key).unwrap().1.clone()).unwrap();

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
                state
                    .values
                    .insert(key, (0u64, serde_json::to_value(allowlist).unwrap()));
            }
        }
        Ok(())
    }

    fn get_node_allowlist(&self, node_id: &PublicKey) -> Result<Vec<std::string::String>, Error> {
        let state = self.state.lock().unwrap();
        let key = hex::encode(node_id.serialize());
        let key = format!("{ALLOWLIST_PREFIX}/{key}");
        let allowlist: Vec<String> =
            serde_json::from_value(state.values.get(&key).unwrap().1.clone()).unwrap();

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
            .map(|k| k.split('/'))
            .filter(|k| k.clone().next().unwrap() == NODE_PREFIX)
            .map(|k| k.clone().last().unwrap())
            .collect();

        let mut res = Vec::new();
        for node_id in node_ids.iter() {
            let node_key = format!("{NODE_PREFIX}/{node_id}");
            let state_key = format!("{NODE_STATE_PREFIX}/{node_id}");

            let node: vls_persist::model::NodeEntry =
                serde_json::from_value(state.values.get(&node_key).unwrap().1.clone()).unwrap();
            let state_e: vls_persist::model::NodeStateEntry =
                serde_json::from_value(state.values.get(&state_key).unwrap().1.clone()).unwrap();

            let state = CoreNodeState::restore(
                state_e.invoices,
                state_e.issued_invoices,
                state_e.preimages,
                0,
                state_e.velocity_control.into(),
                state_e.fee_velocity_control.into(),
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
