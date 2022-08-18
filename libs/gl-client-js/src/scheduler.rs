use crate::node::Node;
use crate::runtime::{convert, exec, get_runtime};
use crate::tls::TlsConfig;
use crate::Signer;
use bitcoin::Network;
use neon::prelude::*;
use prost::Message;
use std::sync::Arc;

pub(crate) struct Scheduler {
    inner: gl_client::scheduler::Scheduler,
}

impl Scheduler {
    pub(crate) fn new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<Self>>> {
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

        Ok(cx.boxed(Arc::new(Self { inner })))
    }

    pub(crate) fn register(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = Arc::clone(&&cx.argument::<JsBox<Arc<Scheduler>>>(0)?);
        let signer = Arc::clone(&&cx.argument::<JsBox<Arc<Signer>>>(1)?);
        jsconvert(exec(this.inner.register(&signer.inner)), cx)
    }

    pub(crate) fn recover(mut cx: FunctionContext) -> JsResult<JsBuffer> {
        let this = Arc::clone(&&cx.argument::<JsBox<Arc<Scheduler>>>(0)?);
        let signer = Arc::clone(&&cx.argument::<JsBox<Arc<Signer>>>(1)?);
        jsconvert(exec(this.inner.recover(&signer.inner)), cx)
    }

    pub(crate) fn schedule(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let this = Arc::clone(&&cx.argument::<JsBox<Arc<Scheduler>>>(0)?);
        let tls = Arc::clone(&&cx.argument::<JsBox<Arc<TlsConfig>>>(1)?);

        // Callback and synchronous queue used to report the result in a promise
        let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);
        let chan = cx.channel();

        get_runtime().spawn(async move {
            let res = this.inner.schedule(tls.inner.clone()).await;
            chan.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                let node = match res {
                    Ok(client) => Node::new(client),
                    Err(e) => cx.throw_error(format!("{}", e))?,
                };
                let args = vec![cx.undefined().upcast::<JsValue>(), cx.boxed(node).upcast()];

                callback.call(&mut cx, this, args)?;
                Ok(())
            });
        });
        // Dummy result, the real one is returned above
        Ok(cx.undefined())
    }
}

impl Finalize for Scheduler {}

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
