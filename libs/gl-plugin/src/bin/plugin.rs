use anyhow::Error;
use log::info;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cwd = env::current_dir()?;
    info!("Running in {}", cwd.to_str().unwrap());
    let plugin = gl_plugin::init()?;
    if let Some(plugin) = plugin.start().await? {
        plugin.join().await
    } else {
        Ok(()) // This is just an invocation with `--help`, we're good to exit
    }
}
