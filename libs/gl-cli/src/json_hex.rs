use gl_client::pb::cln;
use serde_json::json;

pub trait ToJsonHex {
    fn to_json_hex(&self) -> serde_json::Value;
}

impl ToJsonHex for cln::GetinfoResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "id": hex::encode(&self.id),
            "color": hex::encode(&self.color),
            "num_peers": self.num_peers,
            "num_pending_channels": self.num_pending_channels,
            "num_active_channels": self.num_active_channels,
            "num_inactive_channels": self.num_inactive_channels,
            "address": self.address.clone(),
            "binding": self.binding.clone(),
            "version": self.version.clone(),
            "blockheight": self.blockheight,
            "network": self.network,
            "lightning_dir": self.lightning_dir.clone(),
        });
        if let Some(alias) = &self.alias {
            j["alias"] = json!(alias);
        }
        if let Some(amt) = &self.fees_collected_msat {
            j["fees_collected_msat"] = amt.msat.into();
        }
        if let Some(feat) = &self.our_features {
            j["our_features"] = json!({
                "init": hex::encode(&feat.init),
                "node": hex::encode(&feat.node),
                "channel": hex::encode(&feat.channel),
                "invoice": hex::encode(&feat.invoice),
            });
        }
        if let Some(warn_bsync) = &self.warning_bitcoind_sync {
            j["warning_bitcoind_sync"] = json!(warn_bsync);
        }
        if let Some(warn_lsync) = &self.warning_lightningd_sync {
            j["warning_lightningd_sync"] = json!(warn_lsync);
        }
        j
    }
}
