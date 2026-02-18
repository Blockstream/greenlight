use crate::error::{Error, Result};
use crate::model;
use crate::util::{self, CREDENTIALS_FILE_NAME, SEED_FILE_NAME};
use clap::Subcommand;
use futures::stream::StreamExt;
use gl_client::pb::StreamLogRequest;
use gl_client::{bitcoin::Network, pb::cln};
use std::path::Path;

pub struct Config<P: AsRef<Path>> {
    pub data_dir: P,
    pub network: Network,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Creates a new node
    #[command(name = "init")]
    Init {
        #[arg(long)]
        mnemonic: Option<String>,
    },
    /// Stream logs to stdout
    Log,
    /// Returns some basic node info
    #[command(name = "getinfo")]
    GetInfo,
    /// Create a new invoice
    Invoice {
        #[arg(required = true)]
        label: String,
        #[arg(required = true)]
        description: String,
        #[arg(long, value_parser = clap::value_parser!(model::AmountOrAny))]
        amount_msat: Option<model::AmountOrAny>,
        #[arg(long)]
        expiry: Option<u64>,
        #[arg(long)]
        fallbacks: Option<Vec<String>>,
        #[arg(long)]
        preimage: Option<Vec<u8>>,
        #[arg(long)]
        cltv: Option<u32>,
        #[arg(long)]
        deschashonly: Option<bool>,
    },
    /// Pay a bolt11 invoice
    Pay {
        #[arg(required = true)]
        bolt11: String,
        #[arg(long)]
        amount_msat: Option<u64>,
        #[arg(long)]
        label: Option<String>,
        #[arg(long)]
        riskfactor: Option<f64>,
        #[arg(long)]
        maxfeepercent: Option<f64>,
        #[arg(long)]
        retry_for: Option<u32>,
        #[arg(long)]
        maxdelay: Option<u32>,
        #[arg(long)]
        exemptfee: Option<u64>,
        #[arg(long)]
        localinvreqid: Option<Vec<u8>>,
        #[arg(long)]
        exclude: Option<Vec<String>>,
        #[arg(long)]
        maxfee: Option<u64>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Establish a new connection with another lightning node.
    Connect {
        #[arg(
            required = true,
            help = "The targets nodes public key, can be of form id@host:port, host and port must be omitted in this case."
        )]
        id: String,
        #[arg(long, help = "The peer's hostname or IP address.")]
        host: Option<String>,
        #[arg(
            long,
            help = "The peer's port number defaults to the networks default ports if missing."
        )]
        port: Option<u32>,
    },
    /// List attempted payments
    Listpays {
        #[arg(long, help = "A Bolt11 string to get the payment details")]
        bolt11: Option<String>,
        #[arg(long, help = "A payment_hash to get the payment details")]
        payment_hash: Option<String>,
        #[arg(
            long,
            help = "Can be one of \"pending\", \"completed\", \"failed\", filters the payments that are returned"
        )]
        status: Option<String>,
    },
    /// Generates a new bitcoin address to receive funds
    Newaddr,
    /// List available funds, both on-chain and payment channels
    Listfunds,
    /// Send on-chain funds to external wallet
    Withdraw {
        #[arg(long, help = "Bitcoin address for the destination")]
        destination: String,
        #[arg(long, help = "Amount is sats or the string \"all\"")]
        amount_sat: model::AmountSatOrAll,
    },
    /// Open a channel with peer
    Fundchannel {
        #[arg(long, help = "Peer id")]
        id: String,
        #[arg(long, help = "Amount in sats or the string \"all\"")]
        amount_sat: model::AmountSatOrAll,
    },
    /// Close a channel with peer
    Close {
        #[arg(long, help = "Peer id, channel id or short channel id")]
        id: String,
    },
    /// Stop the node
    Stop,
}

pub async fn command_handler<P: AsRef<Path>>(cmd: Command, config: Config<P>) -> Result<()> {
    match cmd {
        Command::Init { mnemonic } => init_handler(config, mnemonic).await,
        Command::Log => log(config).await,
        Command::GetInfo => getinfo_handler(config).await,
        Command::Invoice {
            label,
            description,
            amount_msat,
            expiry,
            fallbacks,
            preimage,
            cltv,
            deschashonly,
        } => {
            invoice_handler(
                config,
                label,
                description,
                amount_msat,
                expiry,
                fallbacks,
                preimage,
                cltv,
                deschashonly,
            )
            .await
        }
        Command::Pay {
            bolt11,
            amount_msat,
            label,
            riskfactor,
            maxfeepercent,
            retry_for,
            maxdelay,
            exemptfee,
            localinvreqid,
            exclude,
            maxfee,
            description,
        } => {
            pay_handler(
                config,
                bolt11,
                amount_msat,
                label,
                riskfactor,
                maxfeepercent,
                retry_for,
                maxdelay,
                exemptfee,
                localinvreqid,
                exclude,
                maxfee,
                description,
            )
            .await
        }
        Command::Connect { id, host, port } => connect_handler(config, id, host, port).await,
        Command::Listpays {
            bolt11,
            payment_hash,
            status,
        } => {
            let payment_hash = if let Some(v) = payment_hash {
                match hex::decode(v) {
                    Ok(decoded) => Some(decoded),
                    Err(e) => {
                        println!("Payment hash is not a valid hex string: {}", e);
                        return Ok(()); // Exit the function early if hex decoding fails
                    }
                }
            } else {
                None
            };
            let status = if let Some(status_str) = status {
                match status_str.as_str() {
                    "pending" => Some(0),
                    "complete" => Some(1),
                    "failed" => Some(2),
                    _ => {
                        println!("Invalid status: {}, expected one of \"pending\", \"completed\", \"failed\"", status_str);
                        return Ok(()); // Exit the function early if the status is invalid
                    }
                }
            } else {
                None
            };

            listpays_handler(config, bolt11, payment_hash, status).await
        }
        Command::Newaddr => newaddr_handler(config).await,
        Command::Listfunds => listfunds_handler(config).await,
        Command::Withdraw {
            destination,
            amount_sat,
        } => withdraw_handler(config, destination, amount_sat).await,
        Command::Fundchannel { id, amount_sat } => {
            fundchannel_handler(config, id, amount_sat).await
        }
        Command::Close { id } => close_handler(config, id).await,
        Command::Stop => stop(config).await,
    }
}

async fn init_handler<P: AsRef<Path>>(config: Config<P>, mnemonic: Option<String>) -> Result<()> {
    // Check if seed already exists in the configuration path
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    if let Some(_) = util::read_seed(&seed_path) {
        return Err(Error::custom(format!(
            "Seed already exists at {}",
            seed_path.to_string_lossy()
        )));
    } else {
        std::fs::create_dir_all(config.data_dir.as_ref())
            .map_err(|e| Error::custom(format!("Failed to create data directory: {e}")))?;
        println!(
            "Local greenlight directory created at {}",
            config.data_dir.as_ref().to_string_lossy()
        );
    }

    let message = match mnemonic {
        Some(_) => "Secret seed derived from user provided mnemonic",
        None => "Your recovery mnemonic is",
    };
    let (seed, mnemonic) = util::generate_seed(mnemonic)?;
    util::write_seed(&seed_path, &seed)?;

    // report after success
    println!("{message}: {mnemonic}");
    Ok(())
}

async fn get_node<P: AsRef<Path>>(config: Config<P>) -> Result<gl_client::node::ClnClient> {
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

    let scheduler = gl_client::scheduler::Scheduler::new(config.network, creds)
        .await
        .map_err(Error::custom)?;

    scheduler.node().await.map_err(Error::custom)
}

async fn log<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
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

    let scheduler = gl_client::scheduler::Scheduler::new(config.network, creds)
        .await
        .map_err(Error::custom)?;

    let mut node: gl_client::node::Client = scheduler.node().await.map_err(Error::custom)?;
    let mut stream = node
        .stream_log(StreamLogRequest {})
        .await
        .map_err(Error::custom)?
        .into_inner();

    loop {
        tokio::select! {
            biased;
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            maybe_line = stream.next() => {
                match maybe_line {
                    Some(line) => {
                        println!("{}", line.map_err(Error::custom)?.line);
                    },
                    None => {
                        break;
                    },
                }
            }
        }
    }
    Ok(())
}

async fn newaddr_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .new_addr(cln::NewaddrRequest { addresstype: None })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn listfunds_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .list_funds(cln::ListfundsRequest { spent: None })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn withdraw_handler<P: AsRef<Path>>(
    config: Config<P>,
    destination: String,
    amount_sat: model::AmountSatOrAll,
) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .withdraw(cln::WithdrawRequest {
            destination: destination,
            satoshi: Some(amount_sat.into()),
            feerate: None,
            minconf: Some(0),
            utxos: vec![],
        })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn fundchannel_handler<P: AsRef<Path>>(
    config: Config<P>,
    id: String,
    amount_sat: model::AmountSatOrAll,
) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let id_bytes = hex::FromHex::from_hex(&id)
        .map_err(|e| Error::custom(format!("Invalid hex string: {id}. {e}")))?;
    let res = node
        .fund_channel(cln::FundchannelRequest {
            id: id_bytes,
            amount: Some(amount_sat.into()),
            feerate: None,
            announce: None,
            minconf: None,
            push_msat: None,
            close_to: None,
            request_amt: None,
            compact_lease: None,
            utxos: vec![],
            mindepth: None,
            reserve: None,
            channel_type: vec![],
        })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn close_handler<P: AsRef<Path>>(config: Config<P>, id: String) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .close(cln::CloseRequest {
            id: id,
            unilateraltimeout: None,
            destination: None,
            fee_negotiation_step: None,
            wrong_funding: None,
            force_lease_closed: None,
            feerange: vec![],
        })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn stop<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .stop(cln::StopRequest {})
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn getinfo_handler<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .getinfo(cln::GetinfoRequest {})
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn invoice_handler<P: AsRef<Path>>(
    config: Config<P>,
    label: String,
    description: String,
    amount_msat: Option<model::AmountOrAny>,
    expiry: Option<u64>,
    fallbacks: Option<Vec<String>>,
    preimage: Option<Vec<u8>>,
    cltv: Option<u32>,
    deschashonly: Option<bool>,
) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .invoice(cln::InvoiceRequest {
            exposeprivatechannels: vec![],
            amount_msat: amount_msat.map(|v| v.into()),
            description,
            label,
            expiry,
            fallbacks: fallbacks.unwrap_or_default(),
            preimage,
            cltv,
            deschashonly,
        })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn connect_handler<P: AsRef<Path>>(
    config: Config<P>,
    id: String,
    host: Option<String>,
    port: Option<u32>,
) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .connect_peer(cln::ConnectRequest { id, host, port })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn listpays_handler<P: AsRef<Path>>(
    config: Config<P>,
    bolt11: Option<String>,
    payment_hash: Option<Vec<u8>>,
    status: Option<i32>,
) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .list_pays(cln::ListpaysRequest {
            index: None,
            start: None,
            limit: None,
            bolt11,
            payment_hash,
            status,
        })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}

async fn pay_handler<P: AsRef<Path>>(
    config: Config<P>,
    bolt11: String,
    amount_msat: Option<u64>,
    label: Option<String>,
    riskfactor: Option<f64>,
    maxfeepercent: Option<f64>,
    retry_for: Option<u32>,
    maxdelay: Option<u32>,
    exemptfee: Option<u64>,
    localinvreqid: Option<Vec<u8>>,
    exclude: Option<Vec<String>>,
    maxfee: Option<u64>,
    description: Option<String>,
) -> Result<()> {
    let mut node: gl_client::node::ClnClient = get_node(config).await?;
    let res = node
        .pay(cln::PayRequest {
            bolt11,
            amount_msat: amount_msat.map(|msat| cln::Amount { msat }),
            label,
            riskfactor,
            maxfeepercent,
            retry_for,
            maxdelay,
            exemptfee: exemptfee.map(|msat| cln::Amount { msat }),
            localinvreqid,
            exclude: exclude.unwrap_or_default(),
            maxfee: maxfee.map(|msat| cln::Amount { msat }),
            description,
            partial_msat: None,
        })
        .await
        .map_err(|e| Error::custom(e.message()))?
        .into_inner();
    println!("{:?}", res);
    Ok(())
}
