use anyhow::{anyhow, Result};
use gl_client::credentials::{Device, Nobody, RuneProvider, TlsConfigProvider};
use gl_client::node::ClnClient;
use gl_client::pb::{self, cln};
use gl_client::scheduler::Scheduler;
use gl_client::{bitcoin::Network, signer::Signer};
use tokio;

mod extensions;
use extensions::*;

#[tokio::main]
async fn main() {
    let seed = create_seed();
}

async fn create_seed() -> Vec<u8> {
    use bip39::{Language, Mnemonic};

    let mut rng = rand::thread_rng();
    let m = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();
    let phrase = m.word_iter().fold("".to_string(), |c, n| c + " " + n);

    // Prompt user to safely store the phrase

    const EMPTY_PASSPHRASE: &str = "";
    let seed = &m.to_seed(EMPTY_PASSPHRASE)[0..32]; // Only need the first 32 bytes

    // Store the seed on the filesystem, or secure configuration system
    seed[0..32].to_vec()
}

async fn register_node(seed: Vec<u8>, developer_cert: Vec<u8>, developer_key: Vec<u8>) {
    // Creating a new `TlsConfig` object using your developer certificate
    // cert: contains the content of `client.crt`
    // key: contains the content of `client-key.pem`

    let developer_creds = Nobody::with_identity(developer_cert, developer_key);
    let signer = Signer::new(seed, Network::Bitcoin, developer_creds.clone()).unwrap();

    let scheduler = Scheduler::new(signer.node_id(), Network::Bitcoin, developer_creds)
        .await
        .unwrap();

    // Passing in the signer is required because the client needs to prove
    // ownership of the `node_id`
    let registration_response = scheduler.register(&signer, None).await.unwrap();

    // Authenticating the scheduler
    let device_creds = Device::from_bytes(registration_response.creds);

    // Save the credentials somewhere safe

    let scheduler = scheduler.authenticate(device_creds).await.unwrap();
    let mut node: ClnClient = scheduler.node().await.unwrap();
}

async fn start_node(signer: Signer, device_creds: Device) {
    let scheduler = gl_client::scheduler::Scheduler::new(
        signer.node_id(),
        gl_client::bitcoin::Network::Bitcoin,
        device_creds.clone(),
    )
    .await
    .unwrap();

    let mut node: gl_client::node::ClnClient = scheduler.node().await.unwrap();

    let (_tx, rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        signer.run_forever(rx).await.unwrap();
    });

    node.invoice(cln::InvoiceRequest {
        label: "label".to_string(),
        description: "description".to_string(),
        ..Default::default()
    })
    .await
    .unwrap();
}

async fn recover_node(
    device_cert: Vec<u8>,
    device_key: Vec<u8>,
    seed: Vec<u8>,
) -> Result<pb::scheduler::RecoveryResponse> {
    let network = gl_client::bitcoin::Network::Bitcoin;
    let signer_creds = Device::with_identity(device_cert.clone(), device_key.clone());
    let signer = gl_client::signer::Signer::new(seed, network, signer_creds.clone()).unwrap();

    let scheduler_creds = signer
        .add_base_rune_to_device_credentials(signer_creds)
        .unwrap();
    let scheduler = gl_client::scheduler::Scheduler::new(
        signer.node_id(),
        gl_client::bitcoin::Network::Bitcoin,
        scheduler_creds,
    )
    .await
    .unwrap();

    scheduler.recover(&signer).await
}
