// Decoding support for the legacy `greenlight.proto` models and
// methods. This will be mostly deprecated as we go.

use super::Request;
pub use crate::pb::*;
use anyhow::anyhow;
use prost::Message;

pub fn decode_request(uri: &str, p: &[u8]) -> anyhow::Result<Request> {
    Ok(match uri {
        "/greenlight.Node/GetInfo" => Request::GlGetinfo(crate::pb::GetInfoRequest::decode(p)?),
        "/greenlight.Node/Stop" => Request::GlStop(crate::pb::StopRequest::decode(p)?),
        "/greenlight.Node/ListPeers" => {
            Request::GlListPeers(crate::pb::ListPeersRequest::decode(p)?)
        }
        "/greenlight.Node/Disconnect" => {
            Request::GlDisconnect(crate::pb::DisconnectRequest::decode(p)?)
        }
        "/greenlight.Node/NewAddr" => Request::GlNewAddr(crate::pb::NewAddrRequest::decode(p)?),
        "/greenlight.Node/ListFunds" => {
            Request::GlListFunds(crate::pb::ListFundsRequest::decode(p)?)
        }
        "/greenlight.Node/Withdraw" => Request::GlWithdraw(crate::pb::WithdrawRequest::decode(p)?),
        "/greenlight.Node/FundChannel" => {
            Request::GlFundChannel(crate::pb::FundChannelRequest::decode(p)?)
        }
        "/greenlight.Node/CloseChannel" => {
            Request::GlCloseChannel(crate::pb::CloseChannelRequest::decode(p)?)
        }
        "/greenlight.Node/CreateInvoice" => {
            Request::GlCreateInvoice(crate::pb::InvoiceRequest::decode(p)?)
        }
        "/greenlight.Node/Pay" => Request::GlPay(crate::pb::PayRequest::decode(p)?),
        "/greenlight.Node/Keysend" => Request::GlKeysend(crate::pb::KeysendRequest::decode(p)?),
        "/greenlight.Node/ListPayments" => {
            Request::GlListPayments(crate::pb::ListPaymentsRequest::decode(p)?)
        }
        "/greenlight.Node/ListInvoices" => {
            Request::GlListInvoices(crate::pb::ListInvoicesRequest::decode(p)?)
        }
        "/greenlight.Node/ConnectPeer" => {
            Request::GlConnectPeer(crate::pb::ConnectRequest::decode(p)?)
        }
        "/greenlight.Node/Configure" => {
            Request::GlConfig(crate::pb::GlConfig::decode(p)?)
        }
        uri => return Err(anyhow!("Unknown URI {}, can't decode payload", uri)),
    })
}
