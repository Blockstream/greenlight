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

//async fn getinfo(app: &App) -> Result<()> {
//    let mut node = getnode(app).await?;
//
//    let r = node
//        .getinfo(cln::GetinfoRequest {})
//        .await
//        .map_err(|e| Error::Unknown(e.to_string()))?
//        .into_inner();
//
//    println!(
//        "{}",
//        serde_json::to_string_pretty(&serde_json::json!(
//            model::GetinfoResponse::try_from(r).map_err(|e| Error::Unknown(e.to_string()))?
//        ))
//        .unwrap()
//    );
//
//    Ok(())
//}
//
//async fn list_funds(app: &App, spent: Option<bool>) -> Result<()> {
//    let mut node = getnode(app).await?;
//
//    println!("Starting signer in the background");
//    let (shutdown, handle) = bg_signer(app).await?;
//
//    let r = node
//        .list_funds(cln::ListfundsRequest { spent })
//        .await
//        .map_err(|e| Error::Unknown(e.to_string()))?
//        .into_inner();
//
//    println!(
//        "{}",
//        serde_json::to_string_pretty(&serde_json::json!(r)).unwrap()
//    );
//
//    _ = shutdown.send(()).await;
//    _ = tokio::join!(handle);
//
//    Ok(())
//}
//
//async fn invoice(
//    app: &App,
//    amount_msat: Option<String>,
//    description: String,
//    label: String,
//    expiry: Option<u64>,
//    fallbacks: Option<Vec<String>>,
//    preimage: Option<String>,
//    cltv: Option<u32>,
//    deschashonly: Option<bool>,
//) -> Result<()> {
//    let amount_msat = match amount_msat {
//        Some(amt) => {
//            if amt == "any" {
//                cln::AmountOrAny {
//                    value: Some(amount_or_any::Value::Any(true)),
//                }
//            } else {
//                cln::AmountOrAny {
//                    value: Some(amount_or_any::Value::Amount(cln::Amount {
//                        msat: amt.parse().expect("Invalid amount"),
//                    })),
//                }
//            }
//        }
//        None => cln::AmountOrAny {
//            value: Some(amount_or_any::Value::Any(true)),
//        },
//    };
//
//    let preimage = match preimage {
//        Some(pre) => Some(hex::decode(pre).expect("Invalid preimage")),
//        None => None,
//    };
//
//    let mut node = getnode(app).await?;
//
//    let (shutdown, handle) = bg_signer(app).await?;
//
//    let r = node
//        .invoice(cln::InvoiceRequest {
//            amount_msat: Some(amount_msat),
//            description,
//            label,
//            expiry,
//            fallbacks: fallbacks.unwrap_or_default(),
//            preimage,
//            cltv,
//            deschashonly,
//            exposeprivatechannels: false,
//        })
//        .await
//        .map_err(|e| Error::Unknown(e.to_string()))?
//        .into_inner();
//
//    println!(
//        "{}",
//        serde_json::to_string_pretty(&serde_json::json!(r)).unwrap()
//    );
//
//    _ = shutdown.send(()).await;
//    _ = tokio::join!(handle);
//
//    Ok(())
//}
//
//async fn pay(app: &App, bolt11: String) -> Result<()> {
//    let mut node = getnode(app).await?;
//
//    let r = node
//        .pay(cln::PayRequest {
//            bolt11,
//            amount_msat: None,
//            label: None,
//            riskfactor: None,
//            maxfeepercent: None,
//            retry_for: None,
//            maxdelay: None,
//            exemptfee: None,
//            localinvreqid: None,
//            exclude: vec![],
//            maxfee: None,
//            description: None,
//            partial_msat: None,
//        })
//        .await
//        .map_err(|e| Error::Unknown(e.to_string()))?
//        .into_inner();
//
//    println!(
//        "{}",
//        serde_json::to_string_pretty(&serde_json::json!(r)).unwrap()
//    );
//
//    Ok(())
//}
//
//async fn list_peer_channels(app: &App, id: Option<String>) -> Result<()> {
//    let mut node = getnode(app).await?;
//
//    let id = id
//        .map(|id| hex::decode(id))
//        .transpose()
//        .map_err(|e| Error::Unknown(e.to_string()))?;
//
//    let r = node
//        .list_peer_channels(cln::ListpeerchannelsRequest { id })
//        .await
//        .map_err(|e| Error::Unknown(e.to_string()))?
//        .into_inner();
//
//    println!(
//        "{}",
//        serde_json::to_string_pretty(&serde_json::json!(r)).unwrap()
//    );
//
//    Ok(())
//}
//
//async fn bg_signer(app: &App) -> Result<(Sender<()>, JoinHandle<()>)> {
//    let seed = fs::read(app.cli.path().join(SEEDFILE))
//        .map_err(|e| Error::Unknown(format!("Could not read seed file: {}", e)))?;
//
//    let creds = gl_client::credentials::Device::from_path(app.cli.path().join(CREDSFILE));
//
//    let signer = gl_client::signer::Signer::new(seed, app.cli.network(), creds.clone())
//        .map_err(|e| Error::Unknown(format!("Failed to create signer: {}", e)))?;
//
//    let (tx, rx) = tokio::sync::mpsc::channel(1);
//    let handle = tokio::spawn(async move {
//        let _ = signer.run_forever(rx).await;
//    });
//
//    Ok((tx, handle))
//}
//
//async fn getnode(app: &App) -> Result<gl_client::node::ClnClient> {
//    let seed = fs::read(app.cli.path().join(SEEDFILE))
//        .map_err(|e| Error::Unknown(format!("Could not read seed file: {}", e)))?;
//
//    let creds = gl_client::credentials::Device::from_path(app.cli.path().join(CREDSFILE));
//
//    let signer = gl_client::signer::Signer::new(seed, app.cli.network(), creds.clone())
//        .map_err(|e| Error::Unknown(format!("Failed to create signer: {}", e)))?;
//
//    let scheduler = gl_client::scheduler::Scheduler::new(signer.node_id(), creds)
//        .await
//        .map_err(|e| Error::Unknown(format!("Failed to create scheduler: {}", e)))?;
//
//    let _ = scheduler.schedule().await;
//
//    let node: gl_client::node::ClnClient = scheduler
//        .node()
//        .await
//        .map_err(|e| Error::Unknown(format!("Failed to create node client: {}", e)))?;
//
//    Ok(node)
//}

#[cfg(test)]
mod tests {}
