use neon::prelude::*;

mod node;
mod scheduler;
mod signer;
use node::Node;
use scheduler::Scheduler;
use signer::Signer;
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

    cx.export_function("schedulerNew", Scheduler::new)?;
    cx.export_function("schedulerRegister", Scheduler::register)?;
    cx.export_function("schedulerRecover", Scheduler::recover)?;
    cx.export_function("schedulerSchedule", Scheduler::schedule)?;

    cx.export_function("nodeCall", Node::call)?;

    cx.export_function("tlsConfigNew", TlsConfig::new)?;
    cx.export_function("tlsConfigIdentity", TlsConfig::identity)?;

    Ok(())
}
