use anyhow::Error;
use log::info;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cwd = env::current_dir()?;

    // We use the SledStatestore since we own the directory and we can
    // rely on the state being persisted correctly here.

    let mut state_dir = cwd.clone();
    state_dir.push("signer_state");

    let state_store = gl_plugin::storage::SledStateStore::new(state_dir)?;

    info!("Running in {}", cwd.to_str().unwrap());
    let plugin = gl_plugin::init(Box::new(state_store)).await?;
    if let Some(plugin) = plugin.start().await? {
        plugin.join().await
    } else {
        Ok(()) // This is just an invocation with `--help`, we're good to exit
    }
}
