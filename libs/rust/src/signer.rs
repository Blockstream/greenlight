/// The core signer system. It runs in a dedicated thread or using the
/// caller thread, streaming incoming requests, verifying them,
/// signing if ok, and then shipping the response to the node.
use crate::pb::{node_client::NodeClient, Empty, HsmRequest, HsmRequestContext, HsmResponse};
use crate::pb::{scheduler_client::SchedulerClient, NodeInfoRequest};
use anyhow::Result;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref instance: Arc<Mutex<Signer>> = Arc::new(Mutex::new(Signer::new()));
}

struct Signer {}

impl Signer {
    fn new() -> Signer {
        Signer {}
    }
}

pub fn start() -> Result<()> {
    unimplemented!()
}

pub fn stop() -> Result<()> {
    unimplemented!()
}
