use ::tokio::runtime::{Builder, Runtime};
use once_cell::sync::OnceCell;
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

// Dedicated runtime for signer tasks. Separate from the RPC runtime
// because exec() uses block_on(), and calling block_on from within
// a runtime deadlocks.
static SIGNER_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub(crate) fn get_signer_runtime() -> &'static Runtime {
    SIGNER_RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Unable to build signer runtime")
    })
}
