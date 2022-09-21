use lightning_signer::chain::tracker::ChainTracker;
use lightning_signer::channel::ChannelId;
use lightning_signer::channel::ChannelStub;
use lightning_signer::node::NodeConfig;
use lightning_signer::node::NodeState;
use lightning_signer::persist::Persist;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

const NODE_PREFIX: &str = "nodes";
const NODE_STATE_PREFIX: &str = "nodestates";
const CHANNEL_PREFIX: &str = "channels";
const ALLOWLIST_PREFIX: &str = "allowlists";
const TRACKER_PREFIX: &str = "trackers";

#[derive(Serialize, Deserialize)]
pub(crate) struct State {
    values: BTreeMap<String, (u64, serde_json::Value)>,
}

impl State {
    fn insert_node(
        &mut self,
        key: &str,
        node_entry: vls_persist::model::NodeEntry,
        node_state_entry: vls_persist::model::NodeStateEntry,
    ) {
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
    }

    fn update_node(&mut self, key: &str, node_state: vls_persist::model::NodeStateEntry) {
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
    }

    fn delete_node(&mut self, key: &str) {
        let node_key = format!("{NODE_PREFIX}/{key}");
        let state_key = format!("{NODE_STATE_PREFIX}/{key}");

        self.values.remove(&node_key);
        self.values.remove(&state_key);
    }

    fn insert_channel(
        &mut self,
        key: &str,
        channel_entry: vls_persist::model::ChannelEntry,
    ) -> Result<(), ()> {
        let key = format!("{CHANNEL_PREFIX}/{key}");
        assert!(!self.values.contains_key(&key));
        self.values
            .insert(key, (0u64, serde_json::to_value(channel_entry).unwrap()));
        Ok(())
    }

    fn update_channel(
        &mut self,
        key: &str,
        channel_entry: vls_persist::model::ChannelEntry,
    ) -> Result<(), ()> {
        trace!("Updating channel {key}");
        let key = format!("{CHANNEL_PREFIX}/{key}");
        let v = self
            .values
            .get_mut(&key)
            .expect("attempting to update non-existent channel");
        *v = (v.0 + 1, serde_json::to_value(channel_entry).unwrap());
        Ok(())
    }

    fn get_channel(&self, key: &str) -> Result<lightning_signer::persist::model::ChannelEntry, ()> {
        let key = format!("{CHANNEL_PREFIX}/{key}");
        let value = self.values.get(&key).unwrap();
        let entry: vls_persist::model::ChannelEntry =
            serde_json::from_value(value.1.clone()).unwrap();
        Ok(entry.into())
    }

    fn get_node_channels(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Vec<(
        lightning_signer::channel::ChannelId,
        lightning_signer::persist::model::ChannelEntry,
    )> {
        let prefix = hex::encode(node_id.serialize());
        let prefix = format!("{CHANNEL_PREFIX}/{prefix}");
        self.values
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| {
                let key = k.split('/').last().unwrap();
                let key = vls_persist::model::NodeChannelId(hex::decode(&key).unwrap());

                let value: vls_persist::model::ChannelEntry =
                    serde_json::from_value(v.1.clone()).unwrap();
                (key.channel_id(), value.into())
            })
            .collect()
    }

    fn new_chain_tracker(
        &mut self,
        node_id: &bitcoin::secp256k1::PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) {
        let key = hex::encode(node_id.serialize());
        let key = format!("{TRACKER_PREFIX}/{key}");
        assert!(!self.values.contains_key(&key));

        let tracker: vls_persist::model::ChainTrackerEntry = tracker.into();

        self.values
            .insert(key, (0u64, serde_json::to_value(tracker).unwrap()));
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub(crate) fn dump(&self) {
        eprintln!(
            "SIGNERSTATE: {}",
            serde_json::to_string_pretty(&self.values).unwrap()
        );
        eprintln!("Size: {}", serde_json::to_vec(&self.values).unwrap().len());
    }
}

pub(crate) struct WrappingPersister {
    state: Arc<Mutex<State>>,
}

impl WrappingPersister {
    pub fn new(_path: &str) -> Self {
        let state = Arc::new(Mutex::new(State {
            values: BTreeMap::new(),
        }));
        WrappingPersister { state }
    }

    pub fn state(&self) -> Arc<Mutex<State>> {
        self.state.clone()
    }
}

impl Persist for WrappingPersister {
    fn new_node(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        config: &NodeConfig,
        state: &NodeState,
        seed: &[u8],
    ) {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().insert_node(
            &key,
            vls_persist::model::NodeEntry {
                seed: seed.to_vec(),
                key_derivation_style: config.key_derivation_style as u8,
                network: config.network.to_string(),
            },
            state.into(),
        );
    }
    fn update_node(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        state: &NodeState,
    ) -> Result<(), ()> {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().update_node(&key, state.into());
        Ok(())
    }
    fn delete_node(&self, node_id: &bitcoin::secp256k1::PublicKey) {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().delete_node(&key);
    }

    fn new_channel(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        stub: &ChannelStub,
    ) -> Result<(), ()> {
        let id = vls_persist::model::NodeChannelId::new(node_id, &stub.id0);
        let channel_value_satoshis = 0;
        let enforcement_state = lightning_signer::policy::validator::EnforcementState::new(0);
        let entry = vls_persist::model::ChannelEntry {
            channel_value_satoshis,
            channel_setup: None,
            id: None,
            enforcement_state,
        };
        let id = hex::encode(id.0);

        self.state.lock().unwrap().insert_channel(&id, entry)
    }

    fn update_channel(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        channel: &lightning_signer::channel::Channel,
    ) -> Result<(), ()> {
        let node_channel_id = vls_persist::model::NodeChannelId::new(node_id, &channel.id0);
        let id = hex::encode(node_channel_id.0);
        let channel_value_satoshis = channel.setup.channel_value_sat;
        let entry = vls_persist::model::ChannelEntry {
            channel_value_satoshis,
            channel_setup: Some(channel.setup.clone()),
            id: channel.id.clone(),
            enforcement_state: channel.enforcement_state.clone(),
        };
        self.state.lock().unwrap().update_channel(&id, entry)
    }

    fn get_channel(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        channel_id: &ChannelId,
    ) -> Result<lightning_signer::persist::model::ChannelEntry, ()> {
        let id = vls_persist::model::NodeChannelId::new(node_id, channel_id);
        let id = hex::encode(id.0);
        self.state.lock().unwrap().get_channel(&id)
    }

    fn new_chain_tracker(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) {
        self.state
            .lock()
            .unwrap()
            .new_chain_tracker(node_id, tracker);
    }

    fn update_tracker(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) -> Result<(), ()> {
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
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Result<ChainTracker<lightning_signer::monitor::ChainMonitor>, ()> {
        let key = hex::encode(node_id.serialize());
        let key = format!("{TRACKER_PREFIX}/{key}");

        let state = self.state.lock().unwrap();
        let v: vls_persist::model::ChainTrackerEntry =
            serde_json::from_value(state.values.get(&key).unwrap().1.clone()).unwrap();

        Ok(v.into())
    }

    fn get_node_channels(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Vec<(ChannelId, lightning_signer::persist::model::ChannelEntry)> {
        self.state.lock().unwrap().get_node_channels(node_id)
    }

    fn update_node_allowlist(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        allowlist: Vec<std::string::String>,
    ) -> Result<(), ()> {
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

    fn get_node_allowlist(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Vec<std::string::String> {
        let state = self.state.lock().unwrap();
        let key = hex::encode(node_id.serialize());
        let key = format!("{ALLOWLIST_PREFIX}/{key}");
        let allowlist: Vec<String> =
            serde_json::from_value(state.values.get(&key).unwrap().1.clone()).unwrap();

        allowlist
    }

    fn get_nodes(
        &self,
    ) -> Vec<(
        bitcoin::secp256k1::PublicKey,
        lightning_signer::persist::model::NodeEntry,
    )> {
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
            let node_state: vls_persist::model::NodeStateEntry =
                serde_json::from_value(state.values.get(&state_key).unwrap().1.clone()).unwrap();

            let state = lightning_signer::node::NodeState {
                invoices: Default::default(),
                issued_invoices: Default::default(),
                payments: Default::default(),
                excess_amount: 0,
                log_prefix: "".to_string(),
                velocity_control: node_state.velocity_control.into(),
            };
            let entry = lightning_signer::persist::model::NodeEntry {
                seed: node.seed,
                key_derivation_style: node.key_derivation_style,
                network: node.network,
                state,
            };

            let key: Vec<u8> = hex::decode(node_id).unwrap();
            res.push((
                bitcoin::secp256k1::PublicKey::from_slice(key.as_slice()).unwrap(),
                entry,
            ));
        }

        let nodes = res;
        nodes
    }
    fn clear_database(&self) {
        self.state.lock().unwrap().clear();
    }
}
