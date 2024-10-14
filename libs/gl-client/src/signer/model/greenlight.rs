// Decoding support for the legacy `greenlight.proto` models and
// methods. This will be mostly deprecated as we go.

use super::Request;
pub use crate::pb::*;
use anyhow::anyhow;
use prost::Message;

pub fn decode_request(uri: &str, p: &[u8]) -> anyhow::Result<Request> {
    Ok(match uri {
        "/greenlight.Node/Configure" => Request::GlConfig(crate::pb::GlConfig::decode(p)?),
        "/greenlight.Node/TrampolinePay" => {
            Request::TrampolinePay(crate::pb::TrampolinePayRequest::decode(p)?)
        }
        uri => return Err(anyhow!("Unknown URI {}, can't decode payload", uri)),
    })
}
