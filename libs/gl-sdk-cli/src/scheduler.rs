use crate::error::{Error, Result};
use crate::util::{self, DataDir};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Register a new greenlight node
    Register {
        /// An invite code for greenlight
        #[arg(short, long)]
        invite_code: Option<String>,
    },
    /// Recover credentials for an existing node
    Recover,
}

pub fn handle(cmd: Command, data_dir: &DataDir, network: glsdk::Network) -> Result<()> {
    match cmd {
        Command::Register { invite_code } => register(data_dir, network, invite_code),
        Command::Recover => recover(data_dir, network),
    }
}

fn register(
    data_dir: &DataDir,
    network: glsdk::Network,
    invite_code: Option<String>,
) -> Result<()> {
    // Generate or load secret
    let signer = match util::read_secret(data_dir) {
        Ok(secret) => {
            eprintln!("Secret already exists, using it");
            util::signer_from_secret(secret)?
        }
        Err(_) => {
            let mnemonic = bip39::Mnemonic::generate(12)?;
            let phrase = mnemonic.to_string();
            util::write_phrase(data_dir, &phrase)?;
            eprintln!("New mnemonic generated and saved");
            eprintln!("Recovery phrase: {phrase}");
            glsdk::Signer::new(phrase).map_err(Error::Sdk)?
        }
    };
    let scheduler = glsdk::Scheduler::new(network)?;
    let creds = scheduler.register(&signer, invite_code)?;

    util::write_credentials(data_dir, &creds)?;
    eprintln!("Credentials saved");

    Ok(())
}

fn recover(data_dir: &DataDir, network: glsdk::Network) -> Result<()> {
    let signer = util::make_signer(data_dir)?;
    let scheduler = glsdk::Scheduler::new(network)?;
    let creds = scheduler.recover(&signer)?;

    util::write_credentials(data_dir, &creds)?;
    eprintln!("Credentials recovered and saved");

    Ok(())
}
