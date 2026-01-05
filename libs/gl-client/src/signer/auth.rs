//! Utilities used to authorize a signature request based on pending RPCs
use std::str::FromStr;
use lightning_signer::invoice::Invoice;
use vls_protocol_signer::approver::Approval;
use crate::signer::model::Request;
use crate::Error;

pub trait Authorizer {
    fn authorize(
        &self,
        requests: &Vec<Request>,
    ) -> Result<Vec<Approval>, Error>;
}

pub struct GreenlightAuthorizer {}

impl Authorizer for GreenlightAuthorizer {
    fn authorize(
        &self,
        requests: &Vec<Request>,
    ) -> Result<Vec<Approval>, Error> {
        let mut approvals = Vec::new();
        for request in requests.iter() {
            match request {
                Request::Pay(req) => {
                    match Invoice::from_str(&req.bolt11) {
                        Ok(invoice) => {
                            approvals.push(Approval::Invoice(invoice));
                        }
                        Err(e) => {
                            return Err(crate::Error::IllegalArgument(
                                format!("Failed to parse invoice from Pay request: {:?}", e)
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(approvals)
    }
}
