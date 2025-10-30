use anyhow::{Result};
use bip39::{Language, Mnemonic};
use gl_client::{
    bitcoin::Network,
    credentials::{Device, Nobody},
    node::ClnClient,
    pb::{cln, cln::{amount_or_any, Amount, AmountOrAny}},
    scheduler::Scheduler,
    signer::Signer,
};
use rand::RngCore;
use std::{env, fs, path::PathBuf};
use tokio;

const NETWORK: Network = Network::Regtest;
const TEST_NODE_DATA_DIR: &str = "/tmp/gltests/node2";

fn save_to_file(file_name: &str, data: &[u8]) -> Result<()> {
    let path = PathBuf::from(TEST_NODE_DATA_DIR).join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, data)?;
    Ok(())
}

fn read_file(file_name: &str) -> Result<Vec<u8>> {
    let path = PathBuf::from(TEST_NODE_DATA_DIR).join(file_name);
    Ok(fs::read(path)?)
}

async fn upgrade_device_certs_to_creds(
    scheduler: &Scheduler<Nobody>,
    signer: &Signer,
    creds_path: &str,
) -> Result<Device> {
    // ---8<--- [start: upgrade_device_certs_to_creds]
    let device = Device::from_path(creds_path);
    let upgraded = device.upgrade(scheduler, signer).await?;
    save_to_file("credentials_upgraded.gfs", &upgraded.to_bytes())?;
    // ---8<--- [end: upgrade_device_certs_to_creds]
    Ok(upgraded)
}

fn create_seed() -> Result<Vec<u8>> {
    // ---8<--- [start: create_seed]
    let mut rng = rand::thread_rng();
    let mut entropy = [0u8; 32];
    rng.fill_bytes(&mut entropy);

    // Seed phrase for user
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    let _phrase = mnemonic.words().collect::<Vec<_>>().join(" ");

    const EMPTY_PASSPHRASE: &str = "";
    let seed = &mnemonic.to_seed(EMPTY_PASSPHRASE)[0..32]; // Only need the first 32 bytes

    // Store the seed on the filesystem, or secure configuration system
    save_to_file("hsm_secret", seed)?;
    // ---8<--- [end: create_seed]
    Ok(seed.to_vec())
}

fn load_developer_creds() -> Result<Nobody> {
    // ---8<--- [start: dev_creds]
    let developer_cert_path = env::var("GL_NOBODY_CRT")?;
    let developer_key_path = env::var("GL_NOBODY_KEY")?;
    let developer_cert = std::fs::read(developer_cert_path).unwrap_or_default();
    let developer_key = std::fs::read(developer_key_path).unwrap_or_default();
    let developer_creds = Nobody {
        cert: developer_cert,
        key: developer_key,
        ..Nobody::default()
    };
    // ---8<--- [end: dev_creds]
    Ok(developer_creds)
}

async fn register_node(seed: Vec<u8>, developer_creds: Nobody) -> Result<(Scheduler<Nobody>, Device, Signer)> {
    // ---8<--- [start: init_signer]
    let signer = Signer::new(seed.clone(), NETWORK, developer_creds.clone())?;
    // ---8<--- [end: init_signer]

    // ---8<--- [start: register_node]
    let scheduler = Scheduler::new(NETWORK, developer_creds).await?;

    // Passing in the signer is required because the client needs to prove
    // ownership of the `node_id`
    let registration_response = scheduler.register(&signer, None).await?;
    // ---8<--- [start: device_creds]
    let device_creds = Device::from_bytes(registration_response.creds);
    save_to_file("credentials.gfs", &device_creds.to_bytes())?;
    // ---8<--- [end: device_creds]
    // ---8<--- [end: register_node]
    Ok((scheduler, device_creds, signer))
}

async fn get_node(scheduler: &Scheduler<Device>) -> Result<ClnClient> {
    // ---8<--- [start: get_node]
    let node = scheduler.node().await?;
    // ---8<--- [end: get_node]
    Ok(node)
}

async fn start_node(device_creds_file_path: &str) -> Result<(cln::GetinfoResponse, cln::ListpeersResponse, cln::InvoiceResponse)> {
    // ---8<--- [start: start_node]
    let creds = Device::from_path(device_creds_file_path);
    let scheduler = Scheduler::new(NETWORK, creds.clone()).await?;
    let mut node: ClnClient = scheduler.node().await?;
    // ---8<--- [end: start_node]

    // ---8<--- [start: list_peers]
    let info = node.getinfo(cln::GetinfoRequest::default()).await?;
    let info = info.into_inner();
    let peers = node.list_peers(cln::ListpeersRequest::default()).await?;
    let peers = peers.into_inner();
    // ---8<--- [end: list_peers]

    // ---8<--- [start: start_signer]
    let seed = read_file("hsm_secret")?;
    let signer = Signer::new(seed, NETWORK, creds.clone())?;
    let (_tx, rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        signer.run_forever(rx).await.unwrap();
    });
    // ---8<--- [end: start_signer]

    // ---8<--- [start: create_invoice]
    let amount = AmountOrAny {
        value: Some(amount_or_any::Value::Amount(Amount { msat: 10_000 })),
    };
    let invoice = node
        .invoice(cln::InvoiceRequest {
            amount_msat: Some(amount),
            description: format!("desc_{}", rand::random::<u32>()),
            label: format!("label_{}", rand::random::<u32>()),
            ..Default::default()
        })
        .await?;
    let invoice = invoice.into_inner();
    // ---8<--- [end: create_invoice]
    Ok((info, peers, invoice))
}

async fn recover_node(dev_creds: Nobody) -> Result<(Scheduler<Nobody>, Device, Signer)> {
    // ---8<--- [start: recover_node]
    let seed = read_file("hsm_secret")?;
    let signer = Signer::new(seed.clone(), NETWORK, dev_creds.clone())?;
    let scheduler = Scheduler::new(NETWORK, dev_creds).await?;
    let recover_response = scheduler.recover(&signer).await?;
    // ---8<--- [end: recover_node]
    let device_creds = Device::from_bytes(recover_response.creds);
    save_to_file("credentials.gfs", &device_creds.to_bytes())?;
    Ok((scheduler, device_creds, signer))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Creating seed...");
    let seed = create_seed()?;

    println!("Loading developer credentials...");
    let developer_creds = load_developer_creds()?;

    println!("Registering node...");
    let (scheduler, device_creds, signer) = register_node(seed, developer_creds.clone()).await?;
    println!("Node Registered!");

    println!("Getting node information...");
    let device_scheduler = Scheduler::new(NETWORK, device_creds.clone()).await?;
    let _gl_node = get_node(&device_scheduler).await?;    

    let (info, peers, invoice) = start_node(&format!("{TEST_NODE_DATA_DIR}/credentials.gfs")).await?;
    println!("Node pubkey: {}", hex::encode(info.id));
    println!("Peers list: {:?}", peers.peers);
    println!("Invoice created: {}", invoice.bolt11);

    println!("Upgrading certs...");
    let _upgraded = upgrade_device_certs_to_creds(&scheduler, &signer, &format!("{TEST_NODE_DATA_DIR}/credentials.gfs")).await?;

    println!("Recovering node...");
    let (_scheduler2, _device_creds2, _signer2) = recover_node(developer_creds.clone()).await?;
    println!("Node Recovered!");

    let (info, _peers, _invoice) = start_node(&format!("{TEST_NODE_DATA_DIR}/credentials.gfs")).await?;
    println!("Node pubkey: {}", hex::encode(info.id));

    println!("All steps completed successfully!");

    Ok(())
}
