//! Utilities used to authorize a signature request based on pending RPCs
use crate::signer::model::Request;
use crate::Error;

pub trait Authorizer {
    fn authorize(
        &self,
        sigreq: vls_protocol::msgs::Message,
        requests: Vec<Request>,
    ) -> Result<(), Error>;
}

pub struct DummyAuthorizer {}

impl Authorizer for DummyAuthorizer {
    fn authorize(
        &self,
        _sigreq: vls_protocol::msgs::Message,
        _requests: Vec<Request>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
