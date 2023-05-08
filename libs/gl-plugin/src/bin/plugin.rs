use anyhow::{Context, Error};
use gl_plugin::config::Config;
use gl_plugin::{
    hsm,
    node::PluginNodeServer,
    stager::Stage,
    storage::{SledStateStore, StateStore},
    Event,
};
use log::info;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cwd = env::current_dir()?;
    info!("Running in {}", cwd.to_str().unwrap());
    let config = Config::new().context("loading config")?;
    let stage = Arc::new(Stage::new());
    let (events, _) = tokio::sync::broadcast::channel(16);
    let state_store = get_signer_store().await?;

    start_hsm_server(config.clone(), stage.clone())?;
    start_node_server(config, stage.clone(), events.clone(), state_store).await?;

    let plugin = gl_plugin::init(stage, events).await?;
    if let Some(plugin) = plugin.start().await? {
        plugin.join().await
    } else {
        Ok(()) // This is just an invocation with `--help`, we're good to exit
    }
}

async fn start_node_server(
    config: Config,
    stage: Arc<Stage>,
    events: tokio::sync::broadcast::Sender<Event>,
    signer_state_store: Box<dyn StateStore>,
) -> Result<(), Error> {
    let addr: SocketAddr = config
        .node_grpc_binding
        .parse()
        .context("parsing the node_grpc_binding")?;

    let tls = tonic::transport::ServerTlsConfig::new()
        .identity(config.identity.id.clone())
        .client_ca_root(config.identity.ca.clone());

    let mut rpc_path = std::env::current_dir().unwrap();
    rpc_path.push("lightning-rpc");

    info!(
        "Starting grpc server on addr={} serving rpc={}",
        addr,
        rpc_path.display()
    );

    let node_server = PluginNodeServer::new(
        stage.clone(),
        config.clone(),
        events.clone(),
        signer_state_store,
    )
    .await?;

    let cln_node = gl_plugin::grpc::pb::node_server::NodeServer::new(
        gl_plugin::node::WrappedNodeServer::new(&rpc_path)
            .await
            .context("creating cln_grpc::pb::node_server::NodeServer instance")?,
    );
    let router = tonic::transport::Server::builder()
        .tls_config(tls)?
        .layer(gl_plugin::node::SignatureContextLayer::new(
            node_server.ctx.clone(),
        ))
        .add_service(gl_plugin::node::RpcWaitService::new(cln_node, rpc_path))
        .add_service(gl_plugin::pb::node_server::NodeServer::new(node_server));

    tokio::spawn(async move {
        router
            .serve(addr)
            .await
            .context("grpc interface exited with error")
    });
    Ok(())
}

async fn get_signer_store() -> Result<Box<dyn StateStore>, Error> {
    let mut state_dir = env::current_dir()?;
    state_dir.push("signer_state");
    Ok(Box::new(SledStateStore::new(state_dir)?))
}

fn start_hsm_server(config: Config, stage: Arc<Stage>) -> Result<(), Error> {
    // We run this already at startup, not at configuration because if
    // the signerproxy doesn't find the socket on the FS it'll exit.
    let hsm_server = hsm::StagingHsmServer::new(
        PathBuf::from_str(&config.hsmd_sock_path).context("hsmd_sock_path is not a valid path")?,
        stage.clone(),
        config.node_info.clone(),
        config.node_config.clone(),
    );
    tokio::spawn(hsm_server.run());
    Ok(())
}
