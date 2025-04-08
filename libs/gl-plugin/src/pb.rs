tonic::include_proto!("greenlight");
use crate::{messages, requests, responses};
use cln_rpc::primitives;

impl HsmRequest {
    pub fn get_type(&self) -> u16 {
        (self.raw[0] as u16) << 8 | (self.raw[1] as u16)
    }
}

impl From<u64> for Amount {
    fn from(i: u64) -> Self {
        Amount {
            unit: Some(amount::Unit::Millisatoshi(i)),
        }
    }
}

impl From<messages::TlvField> for TlvField {
    fn from(f: messages::TlvField) -> TlvField {
        TlvField {
            r#type: f.typ,
            value: hex::decode(f.value).unwrap(),
        }
    }
}

impl From<cln_grpc::pb::RouteHop> for requests::RoutehintHopDev {
    fn from(r: cln_grpc::pb::RouteHop) -> requests::RoutehintHopDev {
        requests::RoutehintHopDev {
            id: hex::encode(r.id),
            short_channel_id: r.scid,
            fee_base_msat: r.feebase.map(|f| f.msat),
            fee_proportional_millionths: r.feeprop,
            cltv_expiry_delta: r.expirydelta as u16,
        }
    }
}

impl From<cln_grpc::pb::InvoiceRequest> for requests::Invoice {
    fn from(ir: cln_grpc::pb::InvoiceRequest) -> Self {
        let fallbacks = (!ir.fallbacks.is_empty()).then(|| ir.fallbacks);
        Self {
            amount_msat: ir
                .amount_msat
                .map(|a| a.into())
                .unwrap_or(primitives::AmountOrAny::Any),
            description: ir.description,
            dev_routes: None,
            label: ir.label,
            exposeprivatechannels: None,
            preimage: ir.preimage.map(|p| hex::encode(p)),
            expiry: ir.expiry,
            fallbacks,
            cltv: ir.cltv,
            deschashonly: ir.deschashonly,
        }
    }
}

impl From<responses::Invoice> for cln_grpc::pb::InvoiceResponse {
    fn from(i: responses::Invoice) -> Self {
        cln_grpc::pb::InvoiceResponse {
            bolt11: i.bolt11,
            expires_at: i.expiry_time as u64,
            payment_hash: hex::decode(i.payment_hash).unwrap(),
            payment_secret: i
                .payment_secret
                .map(|s| hex::decode(s).unwrap())
                .unwrap_or_default(),
            warning_capacity: None,
            warning_mpp: None,
            warning_deadends: None,
            warning_offline: None,
            warning_private_unused: None,
            created_index: None,
        }
    }
}
