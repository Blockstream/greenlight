use crate::error::{Error, Result};
use crate::output::{self, NodeIdOutput};
use crate::util::{self, DataDir};
use clap::Subcommand;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the signer (blocks until Ctrl+C)
    Run,
    /// Print the node's public key
    NodeId,
}

pub fn handle(cmd: Command, data_dir: &DataDir) -> Result<()> {
    match cmd {
        Command::Run => run(data_dir),
        Command::NodeId => node_id(data_dir),
    }
}

fn run(data_dir: &DataDir) -> Result<()> {
    let creds = util::read_credentials(data_dir)?;
    let signer = util::make_signer(data_dir)?;
    let signer = signer
        .authenticate(&creds)
        .map_err(|e| Error::Other(e.to_string()))?;
    let handle = signer.start().map_err(|e| Error::Other(e.to_string()))?;

    eprintln!("Signer running, press Ctrl+C to stop");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| Error::Other(e.to_string()))?;

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    eprintln!("\nStopping signer...");
    handle.stop();
    Ok(())
}

fn node_id(data_dir: &DataDir) -> Result<()> {
    let signer = util::make_signer(data_dir)?;
    let id = signer.node_id();
    output::print_json(&NodeIdOutput {
        node_id: hex::encode(&id),
    });
    Ok(())
}
