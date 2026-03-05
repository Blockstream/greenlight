use crate::error::{Error, Result};
use crate::util;
use clap::{Subcommand, ValueEnum};
use core::fmt::Debug;
use gl_client::signer::{
    Signer, SignerConfig, StateSignatureMode, StateSignatureOverrideConfig,
};
use lightning_signer::bitcoin::Network;
use std::path::Path;
use tokio::{join, signal};
use util::{CREDENTIALS_FILE_NAME, SEED_FILE_NAME};

pub struct Config<P: AsRef<Path>> {
    pub data_dir: P,
    pub network: Network,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StateSignatureModeArg {
    Off,
    Soft,
    Hard,
}

impl Default for StateSignatureModeArg {
    fn default() -> Self {
        Self::Soft
    }
}

impl From<StateSignatureModeArg> for StateSignatureMode {
    fn from(value: StateSignatureModeArg) -> Self {
        match value {
            StateSignatureModeArg::Off => StateSignatureMode::Off,
            StateSignatureModeArg::Soft => StateSignatureMode::Soft,
            StateSignatureModeArg::Hard => StateSignatureMode::Hard,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Starts a signer that connects to greenlight
    Run {
        #[arg(long, value_enum, default_value_t = StateSignatureModeArg::Soft)]
        state_signature_mode: StateSignatureModeArg,
        #[arg(long = "state-override")]
        state_override: Option<String>,
        #[arg(long = "state-override-note")]
        state_override_note: Option<String>,
    },
    /// Prints the version of the signer used
    Version,
}

pub async fn command_handler<P: AsRef<Path>>(cmd: Command, config: Config<P>) -> Result<()> {
    match cmd {
        Command::Run {
            state_signature_mode,
            state_override,
            state_override_note,
        } => {
            run_handler(
                config,
                state_signature_mode,
                state_override,
                state_override_note,
            )
            .await
        }
        Command::Version => version(config).await,
    }
}

async fn run_handler<P: AsRef<Path>>(
    config: Config<P>,
    state_signature_mode: StateSignatureModeArg,
    state_override: Option<String>,
    state_override_note: Option<String>,
) -> Result<()> {
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

    if state_override.is_none() && state_override_note.is_some() {
        return Err(Error::custom(
            "--state-override-note requires --state-override",
        ));
    }

    let state_signature_override = state_override.map(|ack| {
        StateSignatureOverrideConfig {
            ack,
            note: state_override_note,
        }
    });

    let signer = Signer::new_with_config(
        seed,
        config.network,
        creds.clone(),
        SignerConfig {
            state_signature_mode: state_signature_mode.into(),
            state_signature_override,
        },
    )
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

#[cfg(test)]
mod tests {
    use super::{Command, StateSignatureModeArg};
    use clap::{Parser, Subcommand};

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(subcommand)]
        cmd: Command,
    }

    #[derive(Subcommand, Debug)]
    enum RootCommand {
        #[command(subcommand)]
        Signer(Command),
    }

    #[test]
    fn parse_run_mode_flag() {
        let cli = TestCli::parse_from(["test", "run", "--state-signature-mode", "hard"]);
        match cli.cmd {
            Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            } => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Hard);
                assert!(state_override.is_none());
                assert!(state_override_note.is_none());
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn run_mode_defaults_to_soft() {
        let cli = TestCli::parse_from(["test", "run"]);
        match cli.cmd {
            Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            } => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Soft);
                assert!(state_override.is_none());
                assert!(state_override_note.is_none());
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn signer_subcommand_parses_mode_flag() {
        #[derive(Parser, Debug)]
        struct WrapperCli {
            #[command(subcommand)]
            cmd: RootCommand,
        }

        let cli =
            WrapperCli::parse_from(["test", "signer", "run", "--state-signature-mode", "off"]);
        match cli.cmd {
            RootCommand::Signer(Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            }) => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Off);
                assert!(state_override.is_none());
                assert!(state_override_note.is_none());
            }
            _ => panic!("expected signer run"),
        }
    }

    #[test]
    fn parse_override_flags() {
        let cli = TestCli::parse_from([
            "test",
            "run",
            "--state-signature-mode",
            "hard",
            "--state-override",
            "I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE",
            "--state-override-note",
            "debug session",
        ]);
        match cli.cmd {
            Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            } => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Hard);
                assert_eq!(
                    state_override.as_deref(),
                    Some("I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE")
                );
                assert_eq!(state_override_note.as_deref(), Some("debug session"));
            }
            _ => panic!("expected run command"),
        }
    }
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
