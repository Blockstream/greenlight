use crate::{runtime::exec, Error};
use anyhow::Context;
use gl_client::node::GClient;
use std::str::FromStr;

pub struct RawClient {
    stub: GClient,
}
impl RawClient {
    pub fn new(
        node_id: Vec<u8>,
        network: String,
        cert_pem: Vec<u8>,
        key_pem: Vec<u8>,
        node_uri: String,
    ) -> Result<Self, Error> {
        let tls = gl_client::tls::TlsConfig::new()?.identity(cert_pem, key_pem);
        let network = gl_client::bitcoin::Network::from_str(&network)
            .map_err(|e| Error::InvalidArgument(Box::new(e)))?;

        let stub = exec(gl_client::node::Node::new(node_id, network, tls).connect(node_uri))
            .context("error connecting")?;
        Ok(Self { stub })
    }

    /// Used by the [crate::scheduler::Scheduler] to use the
    /// [gl_client::scheduler::Scheduler::schedule] method to start
    /// the node, and then wrap it in the binding representation.
    pub(crate) fn with_node(stub: GClient) -> Self {
        RawClient { stub }
    }

    pub fn call(&self, method: String, payload: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut stub = self.stub.clone();
        crate::runtime::exec(stub.call(&method, payload))
            .map_err(|e| Error::Call {
                method,
                error: e.to_string(),
            })
            .map(|r| r.into_inner().to_vec())
    }
}
