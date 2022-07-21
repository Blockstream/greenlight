use anyhow::Result;
use gl_signerproxy::Proxy;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .target(env_logger::Target::Stderr)
        .init();

    Proxy::new().run().await
}
