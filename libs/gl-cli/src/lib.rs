use crate::error::Result;
use clap::{Parser, Subcommand};
use gl_client::bitcoin::Network;
use std::{path::PathBuf, str::FromStr};
mod error;
pub mod model;
mod node;
mod scheduler;
mod signer;
mod util;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The directory containing the seed and the credentials
    #[arg(short, long, global = true, help_heading = "Global options")]
    data_dir: Option<String>,
    /// Bitcoin network to use. Supported networks are "signet" and "bitcoin"
    #[arg(short, long, default_value = "bitcoin", value_parser = clap::value_parser!(Network), global = true, help_heading = "Global options")]
    network: Network,
    #[arg(long, short, global = true, help_heading = "Global options")]
    verbose: bool,
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interact with the scheduler that is the brain of most operations
    #[command(subcommand)]
    Scheduler(scheduler::Command),
    /// Interact with a local signer
    #[command(subcommand)]
    Signer(signer::Command),
    /// Interact with the node
    #[command(subcommand)]
    Node(node::Command),
}

pub async fn run(cli: Cli) -> Result<()> {
    if cli.verbose {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug")
        }
        env_logger::init();
    }

    let data_dir = cli
        .data_dir
        .map(|d| util::DataDir(PathBuf::from_str(&d).expect("is not a valid path")))
        .unwrap_or_default();

    Ok(match cli.cmd {
        Commands::Scheduler(cmd) => {
            scheduler::command_handler(
                cmd,
                scheduler::Config {
                    data_dir,
                    network: cli.network,
                },
            )
            .await?
        }

        Commands::Signer(cmd) => {
            signer::command_handler(
                cmd,
                signer::Config {
                    data_dir,
                    network: cli.network,
                },
            )
            .await?
        }
        Commands::Node(cmd) => {
            node::command_handler(
                cmd,
                node::Config {
                    data_dir,
                    network: cli.network,
                },
            )
            .await?
        }
    })
}
