use crate::error::{Error, Result};
use crate::util;
use clap::Subcommand;
use core::fmt::Debug;
use gl_client::{credentials, scheduler::Scheduler, signer::Signer};
use lightning_signer::bitcoin::Network;
use std::fs;
use std::path::Path;
use util::{CREDENTIALS_FILE_NAME, SEED_FILE_NAME};

pub struct Config<P: AsRef<Path>> {
    pub data_dir: P,
    pub network: Network,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Register a new greenlight node
    Register {
        // An invite code for greenlight, format is xxxx-xxxx
        #[arg(short, long)]
        invite_code: Option<String>,
    },
    /// Recover the credentials for a greenlight node, still needs to access the seed
    Recover,
    /// Schedule the node on greenlight services
    Schedule,
    /// Upgrades from using certificate and key to using credentials blob
    UpgradeCredentials,
}

pub async fn command_handler<P: AsRef<Path>>(cmd: Command, config: Config<P>) -> Result<()> {
    match cmd {
        Command::Register { invite_code } => register_handler(config, invite_code).await,
        Command::Recover => recover_handler(config).await,
        Command::Schedule => schedule_handler(config).await,
        Command::UpgradeCredentials => upgrade_credentials_handler(config).await,
    }
}

async fn register_handler<P: AsRef<Path>>(
    config: Config<P>,
    invite_code: Option<String>,
) -> Result<()> {
    // Check if a node is already registered for the given seed.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = match util::read_seed(&seed_path) {
        Some(seed) => {
            println!("Seed already exists at {}, usign it", seed_path.display());
            seed
        }
        None => {
            // Generate a new seed and save it.
            let seed = util::generate_seed()?;
            util::write_seed(&seed_path, &seed)?;
            println!("Seed saved to {}", seed_path.display());
            seed
        }
    };

    // Initialize a signer and scheduler with default credentials.
    let creds = credentials::Nobody::new();
    let signer = Signer::new(seed, config.network, creds.clone())
        .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;
    let scheduler = Scheduler::new(config.network, creds)
        .await
        .map_err(|e| Error::custom(format!("Failed to create scheduler: {}", e)))?;

    // Attempt to register a new node.
    let res = scheduler
        .register(&signer, invite_code)
        .await
        .map_err(|e| Error::custom(format!("Failed to register node: {}", e)))?;

    if res.creds.is_empty() {
        println!("No credentials found. Please recover the node.");
    }

    // Save the device credentials to file.
    let creds_path = config.data_dir.as_ref().join(CREDENTIALS_FILE_NAME);
    let device_creds = credentials::Device::from_bytes(res.creds);
    util::write_credentials(&creds_path, &device_creds.to_bytes())?;
    println!("Credentials saved at {}", creds_path.display());

    Ok(())
}

async fn recover_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    // Check if we can find a seed file, if we can not find one, we need to register first.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_none() {
        println!("No seed found. Need to register first.");
        return Err(Error::seed_not_found(format!(
            "could not read from {}",
            seed_path.display()
        )));
    }

    let seed = seed.unwrap(); // we checked if it is none before.

    // Initialize a signer and scheduler with default credentials.
    let creds = credentials::Nobody::new();
    let signer = Signer::new(seed, config.network, creds.clone())
        .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;
    let scheduler = Scheduler::new(config.network, creds)
        .await
        .map_err(|e| Error::custom(format!("Failed to create scheduler: {}", e)))?;

    // Attempt to recover a new node.
    let res = scheduler
        .recover(&signer)
        .await
        .map_err(|e| Error::custom(format!("Failed to register node: {}", e)))?;

    if res.creds.is_empty() {
        println!("No credentials found. Please recover the node.");
    }

    // Save the device credentials to file.
    let creds_path = config.data_dir.as_ref().join(CREDENTIALS_FILE_NAME);
    let device_creds = credentials::Device::from_bytes(res.creds);
    util::write_credentials(&creds_path, &device_creds.to_bytes())?;
    println!("Credentials saved at {}", creds_path.display());

    Ok(())
}

async fn schedule_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    let creds_path = config.data_dir.as_ref().join(CREDENTIALS_FILE_NAME);
    let creds = util::read_credentials(&creds_path);
    if creds.is_none() {
        println!("Could not find credentials at {}", creds_path.display());
        return Err(Error::credentials_not_found(format!(
            "could not read from {}",
            creds_path.display()
        )));
    }

    let creds = creds.unwrap(); // we checked if it is none before.
    let scheduler = Scheduler::new(config.network, creds)
        .await
        .map_err(|e| Error::custom(format!("Failed to create scheduler: {}", e)))?;

    // Attempt to recover a new node.
    let res = scheduler
        .schedule()
        .await
        .map_err(|e| Error::custom(format!("Failed to register node: {}", e)))?;

    println!("{:?}", res);
    Ok(())
}

async fn upgrade_credentials_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    // Check if we can find a seed file, if we can not find one, we need to register first.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_none() {
        println!("No seed found. Need to register first.");
        return Err(Error::seed_not_found(format!(
            "could not read from {}",
            seed_path.display()
        )));
    }

    let seed = seed.unwrap(); // we checked if it is none before.

    // We are trying to upgrade credentials, load the ones we want to replace.
    let cert_path = config.data_dir.as_ref().join("device.crt");
    let cert = fs::read(cert_path).map_err(Error::custom)?;
    let key_path = config.data_dir.as_ref().join("device-key.pem");
    let key = fs::read(key_path).map_err(Error::custom)?;

    let device = credentials::Device {
        cert,
        key,
        ..Default::default()
    };

    // Initialize a signer and scheduler with default credentials.
    let nobody = credentials::Nobody::new();
    let signer = Signer::new(seed, config.network, nobody.clone())
        .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;
    let scheduler = Scheduler::new(config.network, nobody)
        .await
        .map_err(|e| Error::custom(format!("Failed to create scheduler: {}", e)))?;

    let creds = device
        .upgrade(&scheduler, &signer)
        .await
        .map_err(Error::custom)?;

    let creds_path = config.data_dir.as_ref().join(CREDENTIALS_FILE_NAME);
    util::write_credentials(&creds_path, &creds.to_bytes())?;
    println!("Credentials saved at {}", creds_path.display());
    Ok(())
}
