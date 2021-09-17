use ::tokio::runtime::{Builder, Handle, Runtime};
use once_cell::{sync::OnceCell, unsync::OnceCell as UnsyncOnceCell};
use std::future::Future;
use tokio::time::{sleep, Duration};
use pyo3::prelude::*;

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

async fn run_loop() {
    loop {
        sleep(Duration::from_secs(10)).await;
    }
}

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
    Python::with_gil(|py| py.allow_threads(move || get_runtime().block_on(f)))
}
