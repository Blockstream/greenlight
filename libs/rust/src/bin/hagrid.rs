use anyhow::Result;
use gl_client::{signer::Signer, tls::NOBODY_CONFIG, Network};
use tokio::fs;
use tonic::transport::Identity;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();


    let identity = Identity::from_pem(
        fs::read("./device.crt").await?,
        fs::read("./device-key.pem").await?,
    );

    let hsm_secret = fs::read("hsm_secret").await?;

    let _signer = Signer::new(
        hsm_secret,
        Network::BITCOIN,
        NOBODY_CONFIG.clone(),
    )?
    .with_identity(identity);

    _signer.run_forever().await
}
