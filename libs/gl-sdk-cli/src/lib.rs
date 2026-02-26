use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod error;
pub mod node;
pub mod output;
pub mod scheduler;
pub mod signer;
pub mod util;

use error::Result;

#[derive(Parser, Debug)]
#[command(name = "glsdk", about = "CLI for gl-sdk", version)]
pub struct Cli {
    /// Data directory for phrase and credentials
    #[arg(short, long, global = true, help_heading = "Global options")]
    data_dir: Option<String>,

    /// Bitcoin network (bitcoin or regtest)
    #[arg(short, long, default_value = "bitcoin", global = true, help_heading = "Global options")]
    network: String,

    /// Enable debug logging
    #[arg(short, long, global = true, help_heading = "Global options")]
    verbose: bool,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interact with the scheduler (register, recover)
    #[command(subcommand)]
    Scheduler(scheduler::Command),

    /// Interact with the local signer
    #[command(subcommand)]
    Signer(signer::Command),

    /// Interact with the node
    #[command(subcommand)]
    Node(node::Command),
}

pub fn run(cli: Cli) -> Result<()> {
    if cli.verbose {
        if std::env::var("RUST_LOG").is_err() {
            unsafe { std::env::set_var("RUST_LOG", "debug") };
        }
        env_logger::init();
    }

    let data_dir = cli
        .data_dir
        .map(|d| util::DataDir(PathBuf::from(d)))
        .unwrap_or_default();

    let network = util::parse_network(&cli.network)?;

    match cli.cmd {
        Commands::Scheduler(cmd) => scheduler::handle(cmd, &data_dir, network),
        Commands::Signer(cmd) => signer::handle(cmd, &data_dir),
        Commands::Node(cmd) => node::handle(cmd, &data_dir),
    }
}
