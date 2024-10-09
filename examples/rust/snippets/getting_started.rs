use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};
use gl_client::credentials::{Device, Nobody};
use gl_client::node::ClnClient;
use gl_client::pb::cln::{amount_or_any, Amount, AmountOrAny};
use gl_client::pb::{self, cln};
use gl_client::scheduler::Scheduler;
use gl_client::{bitcoin::Network, signer::Signer};
use std::fs::{self};
use tokio;

#[allow(unused)]
// ---8<--- [start: upgrade_device_certs_to_creds]
async fn upgrade_device_certs_to_creds(
    scheduler: &Scheduler<Nobody>,
    signer: &Signer,
    device_cert: Vec<u8>,
    device_key: Vec<u8>,
) -> Result<Device> {
    Device {
        cert: device_cert,
        key: device_key,
        ..Default::default()
    }
    .upgrade(scheduler, signer)
    .await
    .map_err(|e| anyhow!("{}", e.to_string()))
}
// ---8<--- [end: upgrade_device_certs_to_creds]

#[allow(unused)]
fn save_to_file(file_name: &str, data: Vec<u8>) {
    fs::write(file_name, data).unwrap();
}

#[allow(unused)]
fn read_file(file_name: &str) -> Vec<u8> {
    fs::read(file_name).unwrap()
}

#[allow(unused)]
async fn create_seed() -> Vec<u8> {
    // ---8<--- [start: create_seed]
    let mut rng = rand::thread_rng();
    let m = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();

    //Show seed phrase to user
    let _phrase = m.word_iter().fold("".to_string(), |c, n| c + " " + n);

    const EMPTY_PASSPHRASE: &str = "";
    let seed = &m.to_seed(EMPTY_PASSPHRASE)[0..32]; // Only need the first 32 bytes

    // Store the seed on the filesystem, or secure configuration system
    save_to_file("seed", seed.to_vec());

    // ---8<--- [end: create_seed]
    seed.to_vec()
}

#[allow(unused)]
async fn register_node(seed: Vec<u8>, developer_cert_path: String, developer_key_path: String) {
    // ---8<--- [start: dev_creds]
    let developer_cert = std::fs::read(developer_cert_path).unwrap_or_default();
    let developer_key = std::fs::read(developer_key_path).unwrap_or_default();
    let developer_creds = Nobody {
        cert: developer_cert,
        key: developer_key,
        ..Nobody::default()
    };
    // ---8<--- [end: dev_creds]

    // ---8<--- [start: init_signer]
    let network = Network::Bitcoin;
    let signer = Signer::new(seed, network, developer_creds.clone()).unwrap();
    // ---8<--- [end: init_signer]

    // ---8<--- [start: register_node]
    let scheduler = Scheduler::new(network, developer_creds).await.unwrap();

    // Passing in the signer is required because the client needs to prove
    // ownership of the `node_id`
    let registration_response = scheduler.register(&signer, None).await.unwrap();

    // ---8<--- [start: device_creds]
    let device_creds = Device::from_bytes(registration_response.creds);
    save_to_file("creds", device_creds.to_bytes());
    // ---8<--- [end: device_creds]

    // ---8<--- [end: register_node]

    // ---8<--- [start: get_node]
    let scheduler = scheduler.authenticate(device_creds).await.unwrap();
    let _node: ClnClient = scheduler.node().await.unwrap();
    // ---8<--- [end: get_node]
}

#[allow(unused)]
async fn start_node(device_creds_path: String) {
    // ---8<--- [start: start_node]
    let network = Network::Bitcoin;
    let device_creds = Device::from_path(device_creds_path);
    let scheduler = gl_client::scheduler::Scheduler::new(network, device_creds.clone())
        .await
        .unwrap();

    let mut node: gl_client::node::ClnClient = scheduler.node().await.unwrap();
    // ---8<--- [end: start_node]

    // ---8<--- [start: list_peers]
    let _info = node.getinfo(cln::GetinfoRequest::default()).await.unwrap();
    let _peers = node
        .list_peers(gl_client::pb::cln::ListpeersRequest::default())
        .await
        .unwrap();
    // ---8<--- [end: list_peers]

    // ---8<--- [start: start_signer]
    let seed = read_file("seed");
    let signer = Signer::new(seed, network, device_creds.clone()).unwrap();

    let (_tx, rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        signer.run_forever(rx).await.unwrap();
    });
    // ---8<--- [end: start_signer]

    // ---8<--- [start: create_invoice]
    let amount = AmountOrAny {
        value: Some(amount_or_any::Value::Amount(Amount { msat: 10000 })),
    };

    node.invoice(cln::InvoiceRequest {
        amount_msat: Some(amount),
        description: "description".to_string(),
        label: "label".to_string(),
        ..Default::default()
    })
    .await
    .unwrap();
    // ---8<--- [end: create_invoice]
}

#[allow(unused)]
async fn recover_node(
    nobody_cert: Vec<u8>,
    nobody_key: Vec<u8>,
) -> Result<pb::scheduler::RecoveryResponse> {
    // ---8<--- [start: recover_node]
    let seed = read_file("seed");
    let network = gl_client::bitcoin::Network::Bitcoin;
    let creds = Nobody {
        cert: nobody_cert,
        key: nobody_key,
        ..Nobody::default()
    };

    let signer = gl_client::signer::Signer::new(seed, network, creds.clone()).unwrap();

    let scheduler =
        gl_client::scheduler::Scheduler::new(gl_client::bitcoin::Network::Bitcoin, creds)
            .await
            .unwrap();

    scheduler.recover(&signer).await
    // ---8<--- [end: recover_node]
}
