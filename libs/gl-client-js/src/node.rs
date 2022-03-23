use crate::runtime::{convert, exec};
use anyhow::{anyhow, Result};
use gl_client::{node::Client, pb};
use neon::prelude::*;
use prost::Message;
use tonic::Status;

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
            "connect_peer" => convert(
                client
                    .connect_peer(pb::ConnectRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "disconnect" => convert(
                client
                    .disconnect(pb::DisconnectRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "newaddr" => convert(
                client
                    .new_addr(pb::NewAddrRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "fund_channel" => convert(
                client
                    .fund_channel(pb::FundChannelRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "create_invoice" => convert(
                client
                    .create_invoice(pb::InvoiceRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "pay" => convert(client.pay(pb::PayRequest::decode(req)?).await?.into_inner()),
            "withdraw" => convert(
                client
                    .withdraw(pb::WithdrawRequest::decode(req)?)
                    .await?
                    .into_inner(),
            ),
            "keysend" => convert(
                client
                    .keysend(pb::KeysendRequest::decode(req)?)
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

        match exec(this.dispatch(method.as_ref(), &args)) {
            Ok(res) => {
                let jsbuf = JsBuffer::new(&mut cx, res.len() as u32)?;
                cx.borrow(&jsbuf, |jsbuf| jsbuf.as_mut_slice().copy_from_slice(&res));
                Ok(jsbuf)
            }
            Err(e) => cx.throw_error(format!("error calling {}: {}", method, e))?,
        }
    }

    pub(crate) fn call_stream_log(mut cx: FunctionContext) -> JsResult<JsBox<LogStream>> {
        let this = cx.argument::<JsBox<Node>>(0)?;
        let req = pb::StreamLogRequest {};
        let stream = match exec(this.client.clone().stream_log(req)).map(|x| x.into_inner()) {
            Ok(s) => s,
            Err(e) => cx.throw_error(format!("error calling stream_log: {}", e))?,
        };

        Ok(cx.boxed(LogStream {
            inner: Arc::new(Mutex::new(stream)),
        }))
    }
    pub(crate) fn call_stream_incoming(mut cx: FunctionContext) -> JsResult<JsBox<IncomingStream>> {
        let this = cx.argument::<JsBox<Node>>(0)?;
        let req = pb::StreamIncomingFilter {};
        let stream = match exec(this.client.clone().stream_incoming(req)).map(|x| x.into_inner()) {
            Ok(s) => s,
            Err(e) => cx.throw_error(format!("error calling stream_incoming: {}", e))?,
        };

        Ok(cx.boxed(IncomingStream {
            inner: Arc::new(Mutex::new(stream)),
        }))
    }
}

impl Finalize for Node {}

use std::sync::{Arc, Mutex};

pub(crate) struct LogStream {
    inner: Arc<Mutex<tonic::codec::Streaming<pb::LogEntry>>>,
}

impl LogStream {
    pub(crate) fn next(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<LogStream>>(0)?;
        let stream = this.inner.clone();
        let x = convert_stream_entry(exec(stream.lock().unwrap().message()), cx);
        x
    }
}

impl Finalize for LogStream {}

pub(crate) struct IncomingStream {
    inner: Arc<Mutex<tonic::codec::Streaming<pb::IncomingPayment>>>,
}

impl IncomingStream {
    pub(crate) fn next(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<IncomingStream>>(0)?;
        let stream = this.inner.clone();
        let x = convert_stream_entry(exec(stream.lock().unwrap().message()), cx);
        x
    }
}

impl Finalize for IncomingStream {}

fn convert_stream_entry<T: Message>(
    r: Result<Option<T>, Status>,
    mut cx: FunctionContext,
) -> JsResult<JsBuffer> {
    let res = match r {
        Ok(Some(v)) => v,

        // Empty result means we're done with the stream.
        Ok(None) => return JsBuffer::new(&mut cx, 0),
        Err(e) => cx.throw_error(format!("error retrieving stream item: {}", e))?,
    };
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf).unwrap();

    let jsbuf = JsBuffer::new(&mut cx, buf.len() as u32)?;
    cx.borrow(&jsbuf, |jsbuf| jsbuf.as_mut_slice().copy_from_slice(&buf));

    Ok(jsbuf)
}
