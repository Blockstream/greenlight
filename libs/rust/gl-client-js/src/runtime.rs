use ::tokio::runtime::{Builder, Runtime};
use anyhow::Result;
use neon::prelude::*;
use once_cell::sync::OnceCell;
use prost::Message;
use std::future::Future;

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub(crate) fn get_runtime<'a>() -> &'a Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        let mut builder = Builder::new_multi_thread();
        builder.enable_all();
        builder.build().expect("Unable to build Tokio runtime")
    })
}

pub(crate) fn exec<F, T>(f: F) -> T
where
    F: Future<Output = T> + Sized + Send,
    T: Send,
{
    get_runtime().block_on(f)
}

pub fn convert<T>(r: T) -> Result<Vec<u8>>
where
    T: Message,
{
    let res = r;
    let mut buf = Vec::new();
    buf.reserve(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(buf)
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
