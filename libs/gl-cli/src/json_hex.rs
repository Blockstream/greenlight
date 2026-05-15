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
            "fees_collected_msat": self.fees_collected_msat.clone().map_or(0, |amt| amt.msat),
        });
        if let Some(alias) = &self.alias {
            j["alias"] = json!(alias);
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

impl ToJsonHex for cln::InvoiceResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "bolt11": self.bolt11.clone(),
            "payment_hash": hex::encode(&self.payment_hash),
            "payment_secret": hex::encode(&self.payment_secret),
            "expires_at": self.expires_at,
        });
        if let Some(x) = self.created_index {
            j["created_index"] = json!(x);
        }
        if let Some(x) = &self.warning_capacity {
            j["warning_capacity"] = json!(x);
        }
        if let Some(x) = &self.warning_offline {
            j["warning_offline"] = json!(x);
        }
        if let Some(x) = &self.warning_deadends {
            j["warning_deadends"] = json!(x);
        }
        if let Some(x) = &self.warning_private_unused {
            j["warning_private_unused"] = json!(x);
        }
        if let Some(x) = &self.warning_mpp {
            j["warning_mpp"] = json!(x);
        }
        j
    }
}

impl ToJsonHex for cln::PayResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "status": self.status,
            "payment_preimage": hex::encode(&self.payment_preimage),
            "payment_hash": hex::encode(&self.payment_hash),
            "created_at": self.created_at,
            "parts": self.parts,
            "amount_msat": self.amount_msat.clone().map_or(0, |amt| amt.msat),
            "amount_sent_msat": self.amount_sent_msat.clone().map_or(0, |amt| amt.msat),
        });
        if let Some(x) = &self.destination {
            j["destination"] = json!(hex::encode(x));
        }
        if let Some(x) = &self.warning_partial_completion {
            j["warning_partial_completion"] = json!(x);
        }
        j
    }
}

impl ToJsonHex for cln::ListpaysPays {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "payment_hash": hex::encode(&self.payment_hash),
            "status": self.status,
            "created_at": self.created_at,
        });
        if let Some(x) = &self.destination {
            j["destination"] = json!(hex::encode(x));
        }
        if let Some(x) = self.completed_at {
            j["completed_at"] = json!(x);
        }
        if let Some(x) = &self.label {
            j["label"] = json!(x);
        }
        if let Some(x) = &self.bolt11 {
            j["bolt11"] = json!(x);
        }
        if let Some(x) = &self.description {
            j["description"] = json!(x);
        }
        if let Some(x) = &self.bolt12 {
            j["bolt12"] = json!(x);
        }
        if let Some(x) = &self.amount_msat {
            j["amount_msat"] = x.msat.into();
        }
        if let Some(x) = &self.amount_sent_msat {
            j["amount_sent_msat"] = x.msat.into();
        }
        if let Some(x) = &self.preimage {
            j["preimage"] = json!(hex::encode(x));
        }
        if let Some(x) = self.number_of_parts {
            j["number_of_parts"] = json!(x);
        }
        if let Some(x) = &self.erroronion {
            j["erroronion"] = json!(hex::encode(x));
        }
        j
    }
}

impl ToJsonHex for cln::ListpaysResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        json!({
            "pays": json!(self.pays.iter().map(|x| x.to_json_hex()).collect::<Vec<_>>())
        })
    }
}

impl ToJsonHex for cln::ConnectAddress {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "item_type": self.item_type,
        });
        if let Some(x) = &self.socket {
            j["socket"] = json!(x);
        }
        if let Some(x) = &self.address {
            j["address"] = json!(x);
        }
        if let Some(x) = &self.port {
            j["port"] = json!(x);
        }
        j
    }
}

impl ToJsonHex for cln::ConnectResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "id": hex::encode(&self.id),
            "features": hex::encode(&self.features),
            "direction": self.direction,
        });
        if let Some(x) = &self.address {
            j["address"] = x.to_json_hex();
        }
        j
    }
}

impl ToJsonHex for cln::StopResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        json!({})
    }
}

impl ToJsonHex for cln::CloseResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "item_type": self.item_type,
        });
        if let Some(x) = &self.tx {
            j["tx"] = json!(hex::encode(x));
        }
        if let Some(x) = &self.txid {
            j["txid"] = json!(hex::encode(x));
        }

        j
    }
}

impl ToJsonHex for cln::FundchannelResponse {
    fn to_json_hex(&self) -> serde_json::Value {
        let mut j = json!({
            "tx": hex::encode(&self.tx),
            "txid": hex::encode(&self.txid),
            "outnum": self.outnum,
            "channel_id": hex::encode(&self.channel_id),
        });
        if let Some(x) = &self.close_to {
            j["close_to"] = json!(hex::encode(x));
        }
        if let Some(x) = &self.mindepth {
            j["mindepth"] = json!(x);
        }

        j
    }
}
