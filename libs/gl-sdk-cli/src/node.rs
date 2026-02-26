use crate::error::{Error, Result};
use crate::output::{self, *};
use crate::util::{self, DataDir};
use clap::Subcommand;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Get basic node information
    GetInfo,
    /// List connected peers
    ListPeers,
    /// List channels with peers
    ListPeerChannels,
    /// List available funds
    ListFunds,
    /// Create a lightning invoice
    Receive {
        /// Invoice label
        label: String,
        /// Invoice description
        description: String,
        /// Amount in millisatoshis (omit for any-amount invoice)
        #[arg(long)]
        amount_msat: Option<u64>,
    },
    /// Pay a lightning invoice
    Send {
        /// BOLT11 invoice to pay
        invoice: String,
        /// Amount in millisatoshis (for amount-less invoices)
        #[arg(long)]
        amount_msat: Option<u64>,
    },
    /// Get a new on-chain address
    OnchainReceive,
    /// Send funds on-chain
    OnchainSend {
        /// Destination bitcoin address
        destination: String,
        /// Amount in satoshis, or "all"
        amount_or_all: String,
    },
    /// Stream real-time node events (blocks until Ctrl+C)
    StreamEvents,
    /// Stop the node
    Stop,
}

pub fn handle(cmd: Command, data_dir: &DataDir) -> Result<()> {
    let creds = util::read_credentials(data_dir)?;
    let node = glsdk::Node::new(&creds).map_err(|e| Error::Other(e.to_string()))?;

    match cmd {
        Command::GetInfo => get_info(&node),
        Command::ListPeers => list_peers(&node),
        Command::ListPeerChannels => list_peer_channels(&node),
        Command::ListFunds => list_funds(&node),
        Command::Receive {
            label,
            description,
            amount_msat,
        } => receive(&node, label, description, amount_msat),
        Command::Send {
            invoice,
            amount_msat,
        } => send(&node, invoice, amount_msat),
        Command::OnchainReceive => onchain_receive(&node),
        Command::OnchainSend {
            destination,
            amount_or_all,
        } => onchain_send(&node, destination, amount_or_all),
        Command::StreamEvents => stream_events(&node),
        Command::Stop => stop(&node),
    }
}

fn get_info(node: &glsdk::Node) -> Result<()> {
    let res = node.get_info()?;
    output::print_json(&GetInfoOutput::from(res));
    Ok(())
}

fn list_peers(node: &glsdk::Node) -> Result<()> {
    let res = node.list_peers()?;
    output::print_json(&ListPeersOutput::from(res));
    Ok(())
}

fn list_peer_channels(node: &glsdk::Node) -> Result<()> {
    let res = node.list_peer_channels()?;
    output::print_json(&ListPeerChannelsOutput::from(res));
    Ok(())
}

fn list_funds(node: &glsdk::Node) -> Result<()> {
    let res = node.list_funds()?;
    output::print_json(&ListFundsOutput::from(res));
    Ok(())
}

fn receive(
    node: &glsdk::Node,
    label: String,
    description: String,
    amount_msat: Option<u64>,
) -> Result<()> {
    let res = node.receive(label, description, amount_msat)?;
    output::print_json(&ReceiveOutput::from(res));
    Ok(())
}

fn send(node: &glsdk::Node, invoice: String, amount_msat: Option<u64>) -> Result<()> {
    let res = node.send(invoice, amount_msat)?;
    output::print_json(&SendOutput::from(res));
    Ok(())
}

fn onchain_receive(node: &glsdk::Node) -> Result<()> {
    let res = node.onchain_receive()?;
    output::print_json(&OnchainReceiveOutput::from(res));
    Ok(())
}

fn onchain_send(node: &glsdk::Node, destination: String, amount_or_all: String) -> Result<()> {
    let res = node.onchain_send(destination, amount_or_all)?;
    output::print_json(&OnchainSendOutput::from(res));
    Ok(())
}

fn stream_events(node: &glsdk::Node) -> Result<()> {
    let stream = node.stream_node_events()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| Error::Other(e.to_string()))?;

    eprintln!("Streaming events, press Ctrl+C to stop");

    while running.load(Ordering::SeqCst) {
        match stream.next() {
            Ok(Some(event)) => {
                output::print_json(&NodeEventOutput::from(event));
            }
            Ok(None) => {
                eprintln!("Event stream ended");
                break;
            }
            Err(e) => {
                if running.load(Ordering::SeqCst) {
                    return Err(e.into());
                }
                break;
            }
        }
    }

    Ok(())
}

fn stop(node: &glsdk::Node) -> Result<()> {
    node.stop()?;
    eprintln!("Node stopped");
    Ok(())
}
