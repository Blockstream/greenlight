use crate::error::{Error, Result};
use crate::util;
use bip39::Mnemonic;
use clap::Subcommand;
use core::fmt::Debug;
use gl_client::signer::Signer;
use lightning_signer::bitcoin::Network;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::{join, signal};
use util::{CREDENTIALS_FILE_NAME, SEED_FILE_NAME};

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
    /// Generate a new hsm_secret file from a 12 word seed phrase
    GenerateSecret {
        #[arg(long)]
        mnemonic: Option<String>,
        #[arg(long)]
        passphrase: Option<String>,
    },
}

pub async fn command_handler<P: AsRef<Path>>(cmd: Command, config: Config<P>) -> Result<()> {
    match cmd {
        Command::Run => run_handler(config).await,
        Command::Version => version(config).await,
        Command::GenerateSecret {
            mnemonic,
            passphrase,
        } => generate_secret(config, mnemonic, passphrase).await,
    }
}

async fn generate_secret<P: AsRef<Path>>(
    config: Config<P>,
    mnemonic: Option<String>,
    passphrase: Option<String>,
) -> Result<()> {
    // Check if we can find a seed file, refuse to generate a new one if exists
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_some() {
        return Err(Error::custom(format!(
            "Seed already exists: {}",
            seed_path.display()
        )));
    }
    let mnemonic = match mnemonic {
        Some(sentence) => {
            Mnemonic::parse(sentence).map_err(|e| Error::custom(format!("Bad mnemonic: {e}")))?
        }
        None => Mnemonic::generate(12)
            .map_err(|e| Error::custom(format!("Failed to generate mnemonic: {e}")))?,
    };
    let passphrase = passphrase.unwrap_or(String::new());
    println!(
        "Mnemonic sentence: \"{}\"",
        mnemonic.words().collect::<Vec<_>>().join(" ")
    );
    println!("Passphrase: \"{passphrase}\"",);
    let seed = &mnemonic.to_seed(passphrase)[0..32];
    let mut file = File::create(seed_path)
        .map_err(|e| Error::custom(format!("Failed to create seed: {e}")))?;
    file.write_all(seed)
        .map_err(|e| Error::custom(format!("Failed to write seed to file: {e}")))?;
    Ok(())
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
    let creds_path = config.data_dir.as_ref().join(CREDENTIALS_FILE_NAME);
    let creds = match util::read_credentials(&creds_path) {
        Some(c) => c,
        None => {
            return Err(Error::CredentialsNotFoundError(format!(
                "could not read from {}",
                creds_path.display()
            )))
        }
    };
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
