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
struct Cli {
    /// The directory containing the seed and the credentials
    #[arg(short, long, global = true)]
    data_dir: Option<String>,
    /// Bitcoin network to use
    #[arg(short, long, default_value = "testnet", global = true, value_parser = clap::value_parser!(Network))]
    network: Network,
    #[arg(long, short)]
    verbose: bool,
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => (),
        Err(e) => {
            println!("{}", e);
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    if cli.verbose {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug")
        }
        env_logger::init();
    }

    let data_dir = match cli.data_dir {
        Some(d) => util::DataDir(PathBuf::from_str(&d).unwrap()),
        None => util::DataDir::default(),
    };

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
