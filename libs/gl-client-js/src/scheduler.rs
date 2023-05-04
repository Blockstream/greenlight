use crate::node::Node;
use crate::runtime::{convert, exec};
use crate::tls::TlsConfig;
use crate::Signer;
use gl_client::bitcoin::Network;
use neon::prelude::*;
use prost::Message;

pub(crate) struct Scheduler {
    inner: gl_client::scheduler::Scheduler,
}

impl Scheduler {
    pub(crate) fn new(mut cx: FunctionContext) -> JsResult<JsBox<Self>> {
        let buf = cx.argument::<JsBuffer>(0)?;
        let node_id: Vec<u8> = cx.borrow(&buf, |data| data.as_slice().to_vec());
        let network = cx.argument::<JsString>(1)?.value(&mut cx);

        let network: Network = match network.parse() {
            Ok(v) => v,
            Err(_) => cx.throw_error("Error parsing the network")?,
        };

        let inner = match exec(gl_client::scheduler::Scheduler::new(node_id, network)) {
            Ok(i) => i,
            Err(e) => cx.throw_error(format!("Error contacting scheduler: {}", e))?,
        };

        Ok(cx.boxed(Self { inner }))
    }

    pub(crate) fn register(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<Scheduler>>(0)?;
        let signer = cx.argument::<JsBox<Signer>>(1)?;

        // Check if an invite code is set. If so, pass it on to the register
        // method, else pass on `None`.
        let mut invite_code = None;
        _ = cx
            .argument::<JsString>(2)
            .map(|ic_arg| invite_code = Some(ic_arg.value(&mut cx)));
        jsconvert(exec(this.inner.register(&signer.inner, invite_code)), cx)
    }

    pub(crate) fn recover(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = cx.argument::<JsBox<Scheduler>>(0)?;
        let signer = cx.argument::<JsBox<Signer>>(1)?;
        jsconvert(exec(this.inner.recover(&signer.inner)), cx)
    }

    pub(crate) fn schedule(mut cx: FunctionContext) -> JsResult<JsBox<Node>> {
        let this = cx.argument::<JsBox<Scheduler>>(0)?;
        let tls = cx.argument::<JsBox<TlsConfig>>(1)?;

        match exec(this.inner.schedule(tls.inner.clone())) {
            Ok(client) => Ok(cx.boxed(Node::new(client))),
            Err(e) => cx.throw_error(format!("{}", e))?,
        }
    }

    pub(crate) fn get_invite_codes(mut cx: FunctionContext) -> JsResult<JsBox<Vec<InviteCode>>> {
        let this = cx.argument::<JsBox<Scheduler>>(0)?;

        match exec(this.inner.get_invite_codes()) {
            Ok(r) => Ok(cx.boxed(convert_invite_codes(r))),
            Err(e) => cx.throw_error(format!("{}", e)),
        }
    }
}

impl Finalize for Scheduler {}

pub struct InviteCode {
    pub code: String,
    pub is_redeemed: bool,
}

impl InviteCode {
    fn from_proto(msg: gl_client::pb::scheduler::InviteCode) -> Self {
        Self {
            code: msg.code,
            is_redeemed: msg.is_redeemed,
        }
    }
}

impl Finalize for InviteCode {}

pub fn convert_invite_codes(msg: gl_client::pb::scheduler::ListInviteCodesResponse) -> Vec<InviteCode> {
    let mut icodes = Vec::with_capacity(msg.invite_code_list.len());
    for c in msg.invite_code_list {
        icodes.push(InviteCode::from_proto(c));
    }
    icodes
}

pub fn jsconvert<T, E>(r: Result<T, E>, mut cx: FunctionContext) -> JsResult<JsBuffer>
where
    T: Message,
    E: std::fmt::Display + std::fmt::Debug,
{
    let r = match r {
        Ok(v) => v,
        Err(e) => cx.throw_error(format!("{}", e))?,
    };

    let buf = match convert(r) {
        Ok(v) => v,
        Err(e) => cx.throw_error(format!("{}", e))?,
    };

    let jsbuf = JsBuffer::new(&mut cx, buf.len() as u32)?;
    cx.borrow(&jsbuf, |jsbuf| jsbuf.as_mut_slice().copy_from_slice(&buf));

    Ok(jsbuf)
}
