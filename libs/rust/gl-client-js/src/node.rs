use crate::runtime::{convert, exec};
use anyhow::{anyhow, Result};
use gl_client::{node::Client, pb};
use neon::prelude::*;
use prost::Message;

pub(crate) struct Node {
    client: gl_client::node::Client,
}

impl Node {
    pub(crate) fn new(client: Client) -> Self {
        Node { client }
    }

    async fn dispatch(&self, method: &str, req: &[u8]) -> Result<Vec<u8>> {
        let mut client = self.client.clone();
        match method {
            "get_info" => convert(
                client
                    .get_info(pb::GetInfoRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "stop" => convert(
                client
                    .stop(pb::StopRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "listfunds" => convert(
                client
                    .list_funds(pb::ListFundsRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "listpeers" => convert(
                client
                    .list_peers(pb::ListPeersRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "listinvoices" => convert(
                client
                    .list_invoices(pb::ListInvoicesRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "listpayments" => convert(
                client
                    .list_payments(pb::ListPaymentsRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            o => Err(anyhow!(
                "method \"{}\" is unknown to the glclient library",
                o
            )),
        }
    }

    pub(crate) fn call(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<Node>>(0)?;
        let method = cx.argument::<JsString>(1)?.value(&mut cx);
        let buf = cx.argument::<JsBuffer>(2)?;
        let args: Vec<u8> = cx.borrow(&buf, |data| data.as_slice().to_vec());
        let res = exec(this.dispatch(method.as_ref(), &args)).unwrap();

        let jsbuf = JsBuffer::new(&mut cx, res.len() as u32)?;
        cx.borrow(&jsbuf, |jsbuf| jsbuf.as_mut_slice().copy_from_slice(&res));
        Ok(jsbuf)
    }
}

impl Finalize for Node {}
