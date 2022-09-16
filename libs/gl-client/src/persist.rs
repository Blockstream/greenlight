use lightning_signer::chain::tracker::ChainTracker;
use lightning_signer::channel::ChannelId;
use lightning_signer::channel::ChannelStub;
use lightning_signer::node::NodeConfig;
use lightning_signer::node::NodeState;
use lightning_signer::persist::Persist;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use vls_persist::kv_json::KVJsonPersister;

#[derive(Serialize, Deserialize)]
pub struct State {
    node_states: Vec<(String, u64, vls_persist::model::NodeStateEntry)>,
    nodes: Vec<(String, u64, vls_persist::model::NodeEntry)>,
    channels: Vec<(String, u64, vls_persist::model::ChannelEntry)>,
}

impl State {
    fn find_node_index(&self, key: &str) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, v)| &v.0 == &key)
            .map(|(i, _)| i)
            .nth(0)
    }

    fn contains_node(&self, key: &str) -> bool {
        self.nodes.iter().filter(|i| &i.0 == key).nth(0).is_none()
    }

    fn insert_node(
        &mut self,
        key: &str,
        node_entry: vls_persist::model::NodeEntry,
        node_state_entry: vls_persist::model::NodeStateEntry,
    ) {
        trace!(
            "Insert node: {} {}",
            serde_json::to_string(&node_entry).unwrap(),
            serde_json::to_string(&node_state_entry).unwrap()
        );
        assert!(self.contains_node(key), "can't insert a node twice");
        self.nodes.push((key.to_owned(), 0u64, node_entry));
        self.node_states
            .push((key.to_owned(), 0u64, node_state_entry));
        self.dump()
    }

    fn update_node(&mut self, key: &str, node_state: vls_persist::model::NodeStateEntry) {
        trace!(
            "Update node: {}",
            serde_json::to_string(&node_state).unwrap()
        );
        let idx = self
            .find_node_index(key)
            .expect("updating non-existent node");
        self.node_states[idx].2 = node_state;
        self.node_states[idx].1 += 1;
        self.dump();
    }

    fn delete_node(&mut self, key: &str) {
        let idx = self
            .find_node_index(key)
            .expect("deleting non-existent node");
        self.node_states.remove(idx);
        self.nodes.remove(idx);
        self.dump();
    }

    fn dump(&self) {
        eprintln!("SIGNERSTATE: {}", serde_json::to_string_pretty(self).unwrap());
    }
}

pub(crate) struct WrappingPersister<'a> {
    inner: KVJsonPersister<'a>,
    state: Mutex<State>,
}

impl WrappingPersister<'_> {
    pub fn new(path: &str) -> Self {
        let state = Mutex::new(State {
            node_states: vec![],
            nodes: vec![],
            channels: vec![],
        });
        WrappingPersister {
            inner: KVJsonPersister::new(path),
            state,
        }
    }
}

impl Persist for WrappingPersister<'_> {
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

        self.inner.new_node(node_id, config, state, seed)
    }
    fn update_node(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        state: &NodeState,
    ) -> Result<(), ()> {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().update_node(&key, state.into());

        self.inner.update_node(node_id, state)
    }
    fn delete_node(&self, node_id: &bitcoin::secp256k1::PublicKey) {
        let key = hex::encode(node_id.serialize());
        self.state.lock().unwrap().delete_node(&key);
        self.inner.delete_node(node_id)
    }
    fn new_channel(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        stub: &ChannelStub,
    ) -> Result<(), ()> {
        self.inner.new_channel(node_id, stub)
    }
    fn new_chain_tracker(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) {
        self.inner.new_chain_tracker(node_id, tracker)
    }
    fn update_tracker(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        tracker: &ChainTracker<lightning_signer::monitor::ChainMonitor>,
    ) -> Result<(), ()> {
        self.inner.update_tracker(node_id, tracker)
    }
    fn get_tracker(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Result<ChainTracker<lightning_signer::monitor::ChainMonitor>, ()> {
        self.inner.get_tracker(node_id)
    }

    fn update_channel(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        channel: &lightning_signer::channel::Channel,
    ) -> Result<(), ()> {
        self.inner.update_channel(node_id, channel)
    }

    fn get_channel(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        channel_id: &ChannelId,
    ) -> Result<lightning_signer::persist::model::ChannelEntry, ()> {
        self.inner.get_channel(node_id, channel_id)
    }

    fn get_node_channels(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Vec<(ChannelId, lightning_signer::persist::model::ChannelEntry)> {
        self.inner.get_node_channels(node_id)
    }

    fn update_node_allowlist(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
        allowlist: Vec<std::string::String>,
    ) -> Result<(), ()> {
        self.inner.update_node_allowlist(node_id, allowlist)
    }
    fn get_node_allowlist(
        &self,
        node_id: &bitcoin::secp256k1::PublicKey,
    ) -> Vec<std::string::String> {
        self.inner.get_node_allowlist(node_id)
    }
    fn get_nodes(
        &self,
    ) -> Vec<(
        bitcoin::secp256k1::PublicKey,
        lightning_signer::persist::model::NodeEntry,
    )> {
        let nodes = self.inner.get_nodes();

        eprintln!(
            "{:?}",
            nodes
                .iter()
                .map(|i| hex::encode(i.0.serialize()))
                .collect::<Vec<String>>()
        );
        eprintln!(
            "{:?}",
            nodes
                .iter()
                .map(|i| {
                    serde_json::to_string(&vls_persist::model::NodeEntry {
                        key_derivation_style: i.1.key_derivation_style,
                        network: i.1.network.clone(),
                        seed: i.1.seed.clone(),
                    })
                    .unwrap()
                })
                .collect::<Vec<String>>()
        );

        nodes
    }
    fn clear_database(&self) {
        self.inner.clear_database()
    }
}
