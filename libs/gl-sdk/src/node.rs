use crate::{credentials::Credentials, util::exec, Error};
use gl_client::credentials::NodeIdProvider;
use gl_client::node::{Client as GlClient, ClnClient, Node as ClientNode};

use gl_client::pb::cln as clnpb;
use tokio::sync::OnceCell;

/// The `Node` is an RPC stub representing the node running in the
/// cloud. It is the main entrypoint to interact with the node.
#[derive(uniffi::Object, Clone)]
#[allow(unused)]
pub struct Node {
    inner: ClientNode,
    cln_client: OnceCell<ClnClient>,
    gl_client: OnceCell<GlClient>,
}

#[uniffi::export]
impl Node {
    #[uniffi::constructor()]
    pub fn new(credentials: &Credentials) -> Result<Self, Error> {
        let node_id = credentials
            .inner
            .node_id()
            .map_err(|_e| Error::UnparseableCreds())?;
        let inner = ClientNode::new(node_id, credentials.inner.clone())
            .expect("infallible client instantiation");

        let cln_client = OnceCell::const_new();
        let gl_client = OnceCell::const_new();
        Ok(Node {
            inner,
            cln_client,
            gl_client,
        })
    }

    pub fn stop(&self) -> Result<(), Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::StopRequest {};

        // It's ok, the error here is expected and should just be
        // telling us that we've lost the connection. This is to
        // be expected on shutdown, so we clamp this to success.
        exec(cln_client.stop(req));
        Ok(())
    }

    /// Receive an off-chain payment.
    ///
    /// This method generates a request for a payment, also called an
    /// invoice, that encodes all the information, including amount
    /// and destination, for a prospective sender to send a lightning
    /// payment. The invoice includes negotiation of an LSPS2 / JIT
    /// channel, meaning that if there is no channel sufficient to
    /// receive the requested funds, the node will negotiate an
    /// opening, and when/if executed the payment will cause a channel
    /// to be created, and the incoming payment to be forwarded.
    fn receive(
        &self,
        label: String,
        description: String,
        amount_msat: Option<u64>,
    ) -> Result<ReceiveResponse, Error> {
        let mut gl_client = exec(self.get_gl_client())?.clone();

        let req = gl_client::pb::LspInvoiceRequest {
            amount_msat: amount_msat.unwrap_or_default(),
            description: description,
            label: label,
            lsp_id: "".to_owned(),
            token: "".to_owned(),
        };
        let res = exec(gl_client.lsp_invoice(req))
            .map_err(|s| Error::Rpc(s.to_string()))?
            .into_inner();
        Ok(ReceiveResponse { bolt11: res.bolt11 })
    }

    fn onchain_send(&self, req: &OnchainSendRequest) -> Result<OnchainSendResponse, Error> {
        unimplemented!()
    }
    fn onchain_receive(
        &self,
        addresstype: Option<AddressType>,
    ) -> Result<OnchainReceiveResponse, Error> {
        unimplemented!()
    }

    fn send(&self, invoice: String) -> SendResponse {
        unimplemented!()
    }
}

// Not exported through uniffi
impl Node {
    async fn get_gl_client<'a>(&'a self) -> Result<&'a GlClient, Error> {
        let inner = self.inner.clone();
        self.gl_client
            .get_or_try_init(|| async { inner.schedule::<GlClient>().await })
            .await
            .map_err(|e| Error::Rpc(e.to_string()))
    }

    async fn get_cln_client<'a>(&'a self) -> Result<&'a ClnClient, Error> {
        let inner = self.inner.clone();

        self.cln_client
            .get_or_try_init(|| async { inner.schedule::<ClnClient>().await })
            .await
            .map_err(|e| Error::Rpc(e.to_string()))
    }
}

#[derive(uniffi::Object)]
struct OnchainSendResponse {}

#[derive(uniffi::Object)]
struct OnchainSendRequest {
    label: String,
    description: String,
    amount_msat: Option<u64>,
}

#[derive(uniffi::Enum)]
enum AddressType {
    BECH32,
    P2TR,
}

#[derive(uniffi::Object)]
struct OnchainReceiveResponse {
    address: String,
}

#[derive(uniffi::Object)]
struct SendResponse {}

#[derive(uniffi::Object)]
struct ReceiveResponse {
    bolt11: String,
}
