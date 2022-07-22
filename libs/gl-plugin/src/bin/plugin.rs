use std::env;
use log::info;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::builder()
        .target(env_logger::Target::Stderr)
        .init();
    // Setup the TLS configuration
    let cwd = env::current_dir()?;
    info!("Running in {}", cwd.to_str().unwrap());
Ok(())
}
