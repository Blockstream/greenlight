use crate::error::{Error, Result};
use crate::util;
use clap::Subcommand;
use core::fmt::Debug;
use gl_client::{credentials, signer::Signer};
use lightning_signer::bitcoin::Network;
use std::path::Path;
use tokio::{join, signal};
use util::SEED_FILE_NAME;

pub struct Config<P: AsRef<Path>> {
    pub data_dir: P,
    pub network: Network,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Starts a signer that connects to greenlight
    Run,
    /// Prints the version of the signer used
    Version,
}

pub async fn command_handler<P: AsRef<Path>>(cmd: Command, config: Config<P>) -> Result<()> {
    match cmd {
        Command::Run => run_handler(config).await,
        Command::Version => version(config).await,
    }
}

async fn run_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    // Check if we can find a seed file, if we can not find one, we need to register first.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_none() {
        println!("Seed not found");
        return Err(Error::SeedNotFoundError(format!(
            "could not read from {}",
            seed_path.display()
        )));
    }

    let seed = seed.unwrap(); // we checked if it is none before.

    // Initialize a signer and scheduler with default credentials.
    let creds = credentials::Nobody::new();
    let signer = Signer::new(seed, config.network, creds.clone())
        .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;

    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let handle = tokio::spawn(async move {
        let _ = signer.run_forever(rx).await;
    });

    _ = signal::ctrl_c().await.map_err(|e| Error::custom(e))?;

    _ = tx.send(()).await;
    _ = join!(handle);

    Ok(())
}

async fn version<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    // Check if we can find a seed file, if we can not find one, we need to register first.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_none() {
        println!("Seed not found");
        return Err(Error::SeedNotFoundError(format!(
            "could not read from {}",
            seed_path.display()
        )));
    }

    let seed = seed.unwrap(); // we checked if it is none before.

    // Initialize a signer and scheduler with default credentials.
    let creds = gl_client::credentials::Nobody::new();
    let signer = gl_client::signer::Signer::new(seed, config.network, creds.clone())
        .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;
    println!("{}", signer.version());
    Ok(())
}
