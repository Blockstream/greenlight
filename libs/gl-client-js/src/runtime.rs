use ::tokio::runtime::{Builder, Runtime};
use anyhow::Result;
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
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf)?;
    Ok(buf)
}
