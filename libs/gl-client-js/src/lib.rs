use neon::prelude::*;

mod node;
mod scheduler;
mod signer;
use node::{IncomingStream, LogStream, Node};
use scheduler::Scheduler;
use signer::{Signer, SignerHandle};
mod tls;
use tls::TlsConfig;
mod runtime;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    env_logger::init();
    cx.export_function("signerNew", Signer::new)?;
    cx.export_function("signerRunInThread", Signer::run_in_thread)?;
    cx.export_function("signerRunInForeground", Signer::run_forever)?;
    cx.export_function("signerNodeId", Signer::node_id)?;
    cx.export_function("signerVersion", Signer::version)?;

    cx.export_function("schedulerNew", Scheduler::new)?;
    cx.export_function("schedulerRegister", Scheduler::register)?;
    cx.export_function("schedulerRecover", Scheduler::recover)?;
    cx.export_function("schedulerSchedule", Scheduler::schedule)?;

    cx.export_function("nodeCall", Node::call)?;
    cx.export_function("nodeCallStreamLog", Node::call_stream_log)?;
    cx.export_function("logStreamNext", LogStream::next)?;

    cx.export_function("nodeCallStreamIncoming", Node::call_stream_incoming)?;
    cx.export_function("incomingStreamNext", IncomingStream::next)?;

    cx.export_function("tlsConfigNew", TlsConfig::new)?;
    cx.export_function("tlsConfigIdentity", TlsConfig::identity)?;

    cx.export_function("signerHandleShutdown", SignerHandle::shutdown)?;

    Ok(())
}
