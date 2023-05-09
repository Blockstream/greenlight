//! Utilities used to authorize a signature request based on pending RPCs
use std::str::FromStr;
use lightning_signer::invoice::Invoice;
use vls_protocol_signer::approver::Approval;
use crate::signer::model::Request;
use crate::Error;

pub trait Authorizer {
    fn authorize(
        &self,
        requests: Vec<Request>,
    ) -> Result<Vec<Approval>, Error>;
}

pub struct DummyAuthorizer {}

impl Authorizer for DummyAuthorizer {
    fn authorize(
        &self,
        _requests: Vec<Request>,
    ) -> Result<Vec<Approval>, Error> {
        Ok(vec![])
    }
}

pub struct GreenlightAuthorizer {}

impl Authorizer for GreenlightAuthorizer {
    fn authorize(
        &self,
        requests: Vec<Request>,
    ) -> Result<Vec<Approval>, Error> {
        let approvals : Vec<_> = requests.iter().flat_map(|request| {
            match request {
                Request::GlPay(req) => {
                    // TODO error handling
                    Some(Approval::Invoice(Invoice::from_str(&req.bolt11)
                        .expect("")))
                }
                Request::GlKeysend(_req) => {
                    // TODO missing payment hash
                    // Some(Approval::KeySend(req.payment_hash, req.amount))
                    unimplemented!("keysend")
                }
                Request::GlWithdraw(_req) => {
                    // TODO missing tx (it's in the response rather than the request)
                    // - record the destination and amount?
                    // if we use destination and amount, we have to prevent replay attacks
                    // Some(Approval::Onchain(req.tx))
                    unimplemented!("withdraw")
                }
                _ => None,
            }
        }).collect();
        Ok(approvals)
    }
}
