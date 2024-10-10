tonic::include_proto!("greenlight");
use crate::{messages, requests, responses};
use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use cln_grpc::pb::RouteHop;
use cln_rpc::primitives::{self};

impl HsmRequestContext {
    pub fn to_client_hsmfd_msg(&self) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(2 + 33 + 8 + 8);

        buf.put_u16(9); // client_hsmfd type
        buf.put_slice(&self.node_id[..]);
        buf.put_u64(self.dbid);
        buf.put_u64(self.capabilities);

        Ok(buf.freeze())
    }

    pub fn from_client_hsmfd_msg(msg: &mut Bytes) -> Result<HsmRequestContext> {
        let typ = msg.get_u16();

        if typ != 9 {
            return Err(anyhow!("message is not an init"));
        }

        let mut node_id = [0u8; 33];
        msg.copy_to_slice(&mut node_id);
        let dbid = msg.get_u64();
        let caps = msg.get_u64();

        Ok(HsmRequestContext {
            dbid,
            node_id: node_id.to_vec(),
            capabilities: caps,
        })
    }
}

impl HsmRequest {
    pub fn get_type(&self) -> u16 {
        (self.raw[0] as u16) << 8 | (self.raw[1] as u16)
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

impl From<RouteHop> for requests::RoutehintHopDev {
    fn from(r: RouteHop) -> requests::RoutehintHopDev {
        requests::RoutehintHopDev {
            id: hex::encode(r.id),
            short_channel_id: r.short_channel_id,
            fee_base_msat: r.feebase.map(|f| f.msat).unwrap(),
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
