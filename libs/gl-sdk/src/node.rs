use crate::{Error, credentials::Credentials};
use gl_client::node::Node as ClientNode;
use gl_client::credentials::NodeIdProvider;

/// The `Node` is an RPC stub representing the node running in the
/// cloud. It is the main entrypoint to interact with the node.
#[derive(uniffi::Object)]
#[allow(unused)]
pub struct Node {
    inner: ClientNode,
}

#[uniffi::export]
impl Node {
    #[uniffi::constructor()]
    pub fn new(credentials: &Credentials) -> Result<Self, Error> {
        let node_id = credentials
            .inner
            .node_id()
            .map_err(|_e| Error::UnparseableCreds())?;
        let client = ClientNode::new(node_id, credentials.inner.clone())
            .expect("infallible client instantiation");

        Ok(Node { inner: client })
    }
}
