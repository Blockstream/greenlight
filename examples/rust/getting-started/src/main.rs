use gl_client::credentials::{Device, Nobody};
use gl_client::scheduler::Scheduler;
use gl_client::{bitcoin::Network, signer::Signer};
use tokio;
use gl_client::node::ClnClient;
const CA_RAW: &[u8] = include_str!("../../tls/ca.pem").as_bytes();

#[tokio::main]
async fn main() {
}

async fn starting_a_node(device_cert: Vec<u8>, device_key: Vec<u8>, seed: Vec<u8>) {
    let node_id =
        hex::decode("02058e8b6c2ad363ec59aa136429256d745164c2bdc87f98f0a68690ec2c5c9b0b").unwrap();
    let network = gl_client::bitcoin::Network::Testnet;

    let tls = gl_client::tls::TlsConfig::new()
        .identity(device_cert.clone(), device_key.clone());

    let scheduler = gl_client::scheduler::Scheduler::new(node_id, network)
        .await
        .unwrap();
    let mut node: gl_client::node::ClnClient = scheduler.schedule(tls).await.unwrap();

    //p2
    use gl_client::pb::cln;
    let info = node.getinfo(cln::GetinfoRequest::default()).await.unwrap();
    let peers = node
        .list_peers(gl_client::pb::cln::ListpeersRequest::default())
        .await
        .unwrap();

    //p3
    node.invoice(cln::InvoiceRequest {
        label: "label".to_string(),
        description: "description".to_string(),
        ..Default::default()
    })
    .await
    .unwrap();

    //p4
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let tls = gl_client::tls::TlsConfig::new()
        .unwrap()
        .identity(device_cert, device_key);
    let signer =
        gl_client::signer::Signer::new(seed, gl_client::bitcoin::Network::Bitcoin, tls).unwrap();
    signer.run_forever(rx).await.unwrap();
}

async fn recover(device_cert: Vec<u8>, device_key: Vec<u8>, seed: Vec<u8>) {
    let tls = gl_client::tls::TlsConfig::new()
        .unwrap()
        .identity(device_cert, device_key);
    let signer =
        gl_client::signer::Signer::new(seed, gl_client::bitcoin::Network::Bitcoin, tls).unwrap();
    let scheduler = gl_client::scheduler::Scheduler::new(
        signer.node_id(),
        gl_client::bitcoin::Network::Bitcoin,
    )
    .await
    .unwrap();

    let res = scheduler.recover(&signer).await.unwrap();
}

async fn make_seed(cert: Vec<u8>, key: Vec<u8>, _secret: Vec<u8>) {
    use bip39::{Language, Mnemonic};

    let mut rng = rand::thread_rng();
    let m = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();
    let phrase = m.word_iter().fold("".to_string(), |c, n| c + " " + n);

    // Prompt user to safely store the phrase

    const EMPTY_PASSPHRASE: &str = "";
    let seed = &m.to_seed(EMPTY_PASSPHRASE)[0..32]; // Only need the first 32 bytes

    let secret = seed[0..32].to_vec();

    // Store the seed on the filesystem, or secure configuration system

    //-------
    //Registering the node / Initializing nobody credentials
    //---------

    // Creating a new `TlsConfig` object using your developer certificate
    // cert: contains the content of `client.crt`
    // key: contains the content of `client-key.pem`

    let creds = Nobody::with(cert, key, key);



     //-------
    //Creating a signer
    //---------

    let signer = Signer::new(secret, Network::Bitcoin, creds.clone()).unwrap();



    //Registering a new node

    let scheduler = Scheduler::new(signer.node_id(), Network::Bitcoin, creds)
        .await
        .unwrap();

    // Passing in the signer is required because the client needs to prove
    // ownership of the `node_id`
    let registration_response = scheduler.register(&signer, None).await.unwrap();


    //Authenticating the scheduler
    let creds = Device::from_bytes(registration_response.creds);

    
    let scheduler = scheduler.authenticate(creds).await.unwrap();
    let mut node: ClnClient = scheduler.node().await.unwrap();
}
