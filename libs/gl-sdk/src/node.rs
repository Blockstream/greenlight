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

    /// Stop the node if it is currently running.
    pub fn stop(&self) -> Result<(), Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::StopRequest {};

        // It's ok, the error here is expected and should just be
        // telling us that we've lost the connection. This is to
        // be expected on shutdown, so we clamp this to success.
        let _ = exec(cln_client.stop(req));
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

    fn send(&self, invoice: String, amount_msat: Option<u64>) -> Result<SendResponse, Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();
        let req = clnpb::PayRequest {
            amount_msat: match amount_msat {
                Some(a) => Some(clnpb::Amount { msat: a }),
                None => None,
            },

            bolt11: invoice,
            description: None,
            exclude: vec![],
            exemptfee: None,
            label: None,
            localinvreqid: None,
            maxdelay: None,
            maxfee: None,
            maxfeepercent: None,
            partial_msat: None,
            retry_for: None,
            riskfactor: None,
        };
        exec(cln_client.pay(req))
            .map_err(|e| Error::Rpc(e.to_string()))
            .map(|r| r.into_inner().into())
    }

    fn onchain_send(
        &self,
        destination: String,
        amount_or_all: String,
    ) -> Result<OnchainSendResponse, Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();

        // Decode what the user intends to do. Either we have `all`,
        // or we have an amount that we can parse.
        let (num, suffix): (String, String) = amount_or_all.chars().partition(|c| c.is_digit(10));

        let num = if num.len() > 0 {
            num.parse::<u64>().unwrap()
        } else {
            0
        };
        let satoshi = match (num, suffix.as_ref()) {
            (n, "") | (n, "sat") => clnpb::AmountOrAll {
                // No value suffix, interpret as satoshis. This is an
                // onchain RPC method, hence the sat denomination by
                // default.
                value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount {
                    msat: n * 1000,
                })),
            },
            (n, "msat") => clnpb::AmountOrAll {
                // No value suffix, interpret as satoshis. This is an
                // onchain RPC method, hence the sat denomination by
                // default.
                value: Some(clnpb::amount_or_all::Value::Amount(clnpb::Amount {
                    msat: n * 1000,
                })),
            },
            (0, "all") => clnpb::AmountOrAll {
                value: Some(clnpb::amount_or_all::Value::All(true)),
            },
            (_, _) => return Err(Error::Argument("amount_or_all".to_owned(), amount_or_all)),
        };

        let req = clnpb::WithdrawRequest {
            destination: destination,
            minconf: None,
            feerate: None,
            satoshi: Some(satoshi),
            utxos: vec![],
        };

        exec(cln_client.withdraw(req))
            .map_err(|e| Error::Rpc(e.to_string()))
            .map(|r| r.into_inner().into())
    }

    fn onchain_receive(&self) -> Result<OnchainReceiveResponse, Error> {
        let mut cln_client = exec(self.get_cln_client())?.clone();

        let req = clnpb::NewaddrRequest {
            addresstype: Some(clnpb::newaddr_request::NewaddrAddresstype::All.into()),
        };

        let res = exec(cln_client.new_addr(req))
            .map_err(|e| Error::Rpc(e.to_string()))?
            .into_inner();
        Ok(res.into())
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

#[allow(unused)]
#[derive(uniffi::Object)]
struct OnchainSendResponse {
    tx: Vec<u8>,
    txid: Vec<u8>,
    psbt: String,
}

impl From<clnpb::WithdrawResponse> for OnchainSendResponse {
    fn from(other: clnpb::WithdrawResponse) -> Self {
        Self {
            tx: other.tx,
            txid: other.txid,
            psbt: other.psbt,
        }
    }
}

#[allow(unused)]
#[derive(uniffi::Object)]
struct OnchainReceiveResponse {
    bech32: String,
    p2tr: String,
}

impl From<clnpb::NewaddrResponse> for OnchainReceiveResponse {
    fn from(other: clnpb::NewaddrResponse) -> Self {
        OnchainReceiveResponse {
            bech32: other.bech32.unwrap_or_default(),
            p2tr: other.p2tr.unwrap_or_default(),
        }
    }
}

#[allow(unused)]
#[derive(uniffi::Object)]
struct SendResponse {
    status: PayStatus,
    preimage: Vec<u8>,
    amount_msat: u64,
    amount_sent_msat: u64,
    parts: u32,
}

impl From<clnpb::PayResponse> for SendResponse {
    fn from(other: clnpb::PayResponse) -> Self {
        Self {
            status: other.status.into(),
            preimage: other.payment_preimage,
            amount_msat: other.amount_msat.unwrap().msat,
            amount_sent_msat: other.amount_sent_msat.unwrap().msat,
            parts: other.parts,
        }
    }
}

#[allow(unused)]
#[derive(uniffi::Object)]
struct ReceiveResponse {
    bolt11: String,
}

#[derive(uniffi::Enum)]
enum PayStatus {
    COMPLETE = 0,
    PENDING = 1,
    FAILED = 2,
}

impl From<clnpb::pay_response::PayStatus> for PayStatus {
    fn from(other: clnpb::pay_response::PayStatus) -> Self {
        match other {
            clnpb::pay_response::PayStatus::Complete => PayStatus::COMPLETE,
            clnpb::pay_response::PayStatus::Failed => PayStatus::FAILED,
            clnpb::pay_response::PayStatus::Pending => PayStatus::PENDING,
        }
    }
}

impl From<i32> for PayStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => PayStatus::COMPLETE,
            1 => PayStatus::PENDING,
            2 => PayStatus::FAILED,
            o => panic!("Unknown pay_status {}", o),
        }
    }
}
