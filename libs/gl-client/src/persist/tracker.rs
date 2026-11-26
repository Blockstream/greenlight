use anyhow::{bail, Result};
use lightning_signer::bitcoin::block::Header;
use lightning_signer::bitcoin::consensus::deserialize;
use lightning_signer::bitcoin::OutPoint;
use lightning_signer::chain::tracker::Headers;
use serde::de::IgnoredAny;
use serde::Deserialize;
use std::collections::BTreeSet;

use super::StoredTxid;

#[derive(Deserialize)]
struct StoredOutPoint {
    txid: StoredTxid,
    vout: u32,
}

#[derive(Deserialize)]
struct CheckedListenSlot {
    #[serde(rename = "txid_watches")]
    _txid_watches: Vec<StoredTxid>,
    #[serde(rename = "watches")]
    _watches: Vec<StoredOutPoint>,
    #[serde(rename = "seen")]
    _seen: Vec<StoredOutPoint>,
}

/// Checks the fields that VLS assumes are valid during tracker deserialization and restore.
#[derive(Deserialize)]
pub(super) struct TrackerEntry {
    headers: Vec<String>,
    tip: String,
    #[serde(rename = "height")]
    _height: IgnoredAny,
    #[serde(rename = "network")]
    _network: IgnoredAny,
    listeners: Vec<(StoredOutPoint, (IgnoredAny, CheckedListenSlot))>,
}

impl TrackerEntry {
    pub(super) fn validate_headers(&self) -> Result<()> {
        let tip = hex::decode(&self.tip)?;
        // Older VLS versions stored the tip without a filter header.
        if deserialize::<Headers>(&tip).is_err() {
            deserialize::<Header>(&tip)?;
        }
        for header in &self.headers {
            deserialize::<Headers>(&hex::decode(header)?)?;
        }
        Ok(())
    }

    pub(super) fn validate_listeners(
        &self,
        ready_channels: impl IntoIterator<Item = OutPoint>,
    ) -> Result<()> {
        // VLS collects listeners by outpoint and consumes one for each ready channel.
        let mut outpoints: BTreeSet<_> = self
            .listeners
            .iter()
            .map(|(point, _)| OutPoint::new(point.txid.0, point.vout))
            .collect();
        for outpoint in ready_channels {
            if !outpoints.remove(&outpoint) {
                bail!("missing chain tracker listener for channel outpoint {outpoint}");
            }
        }
        if !outpoints.is_empty() {
            bail!("unmatched chain tracker listeners");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lightning_signer::bitcoin::blockdata::constants::genesis_block;
    use lightning_signer::bitcoin::consensus::serialize;
    use lightning_signer::bitcoin::hashes::Hash;
    use lightning_signer::bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
    use lightning_signer::bitcoin::{FilterHeader, Network};
    use lightning_signer::policy::simple_validator::SimpleValidatorFactory;
    use serde_json::json;
    use std::sync::Arc;
    use vls_persist::model::ChainTrackerEntry;

    #[test]
    fn restores_a_legacy_tracker_tip() {
        let network = Network::Regtest;
        let header = genesis_block(network).header;
        let value = json!({
            "headers": [],
            "tip": hex::encode(serialize(&header)),
            "height": 0,
            "network": network,
            "listeners": [],
        });
        let checks = TrackerEntry::deserialize(&value).unwrap();
        checks.validate_headers().unwrap();
        let node_id = PublicKey::from_secret_key(
            &Secp256k1::new(),
            &SecretKey::from_slice(&[1; 32]).unwrap(),
        );
        let entry = ChainTrackerEntry::deserialize(&value).unwrap();
        let (tracker, _) = entry.into_tracker(node_id, Arc::new(SimpleValidatorFactory::new()));
        assert_eq!(tracker.tip.0, header);
        assert_eq!(tracker.tip.1, FilterHeader::all_zeros());
    }
}
