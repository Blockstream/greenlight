use crate::pb::{hsm_client::HsmClient, Empty, HsmRequest, HsmRequestContext};
use crate::wire::{DaemonConnection, Message};
use anyhow::{Context, Error, Result};
use log::{info, warn};
use std::convert::TryFrom;
use std::env;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::sync::atomic;
use std::sync::Arc;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use which::which;

type GrpcClient = HsmClient<tonic::transport::Channel>;

fn get_sock_path() -> Result<String> {
    Ok(env::var("HSMD_SOCK_PATH").unwrap_or("hsmd.sock".to_string()))
}

struct NodeConnection {
    conn: DaemonConnection,
    context: Option<HsmRequestContext>,
}

fn version() -> String {
    let path = which("lightning_hsmd").expect("could not find HSM executable in PATH");
    let version = Command::new(path)
        .args(&["--version"])
        .output()
        .expect("failed to execute process");
    str::from_utf8(&version.stdout).unwrap().trim().to_string()
}

fn setup_node_stream() -> Result<DaemonConnection, Error> {
    let std_stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(3) };
    std_stream.set_nonblocking(true)?;
    let tok_stream = UnixStream::from_std(std_stream)?;
    Ok(DaemonConnection::new(tok_stream))
}

fn start_handler(
    local: NodeConnection,
    counter: Arc<atomic::AtomicUsize>,
    grpc: GrpcClient,
) {
    tokio::spawn(async move {
        match process_requests(local, counter, grpc).await {
            Ok(()) => warn!("hsmproxy child handler exited without error"),
            Err(e) => warn!("hsmproxy child handler exited: {}", e),
        }
    });
}

async fn process_requests(
    node_conn: NodeConnection,
    request_counter: Arc<atomic::AtomicUsize>,
    mut server: GrpcClient,
) -> Result<(), Error> {
    let NodeConnection { mut conn, context } = node_conn;

    info!("Pinging server");
    server.ping(Empty::default()).await?;

    loop {
        let msg = conn.read().await?;
        match msg.msgtype() {
            9 => {
                eprintln!("Got a message from node: {:?}", &msg.body);
                let ctx = HsmRequestContext::from_client_hsmfd_msg(&msg)?;
                eprintln!("Got a request for a new client fd. Context: {:?}", ctx);

                // Use std pair so remote is a blocking socket — CLN's C subdaemons
                // use blocking I/O; O_NONBLOCK (set by tokio::pair) causes EAGAIN.
                let (std_local, std_remote) = std::os::unix::net::UnixStream::pair()?;
                let remote_fd = std_remote.as_raw_fd();
                std_local.set_nonblocking(true)?;
                let local = UnixStream::from_std(std_local)?;
                let reply = Message::new_with_fds(vec![0, 109], &[remote_fd]);

                let local_nc = NodeConnection {
                    conn: DaemonConnection::new(local),
                    context: Some(ctx),
                };
                start_handler(local_nc, request_counter.clone(), server.clone());

                conn.write(reply).await?;
                drop(std_remote);
            }
            28 => {
                eprintln!("Locally handling the `hsmd_check_pubkey` call");
                let reply = Message::new(vec![0, 128, 1]);
                conn.write(reply).await?;
            }
            _ => {
                let req = tonic::Request::new(HsmRequest {
                    context: context.clone(),
                    raw: msg.body.clone(),
                    request_id: request_counter.fetch_add(1, atomic::Ordering::Relaxed) as u32,
                    requests: Vec::new(),
                    signer_state: Vec::new(),
                });

                eprintln!(
                    "WIRE: lightningd -> hsmd: Got a message from node: {:?}",
                    &req
                );
                let start = std::time::Instant::now();
                let res = server.request(req).await?.into_inner();
                let reply = Message::from_raw(res.raw);
                eprintln!(
                    "WIRE: plugin -> hsmd: Got response after {}ms: {:?}",
                    start.elapsed().as_millis(),
                    &reply,
                );
                conn.write(reply).await?;
            }
        }
    }
}

async fn grpc_connect() -> Result<GrpcClient, Error> {
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(|_: Uri| {
            let sock_path = get_sock_path().unwrap();
            let mut path = PathBuf::new();
            if !sock_path.starts_with('/') {
                path.push(env::current_dir().unwrap());
            }
            path.push(&sock_path);
            let path = path.to_str().unwrap().to_string();
            info!("Connecting to hsmserver at {}", path);
            tokio::net::UnixStream::connect(path)
        }))
        .await
        .context("could not connect to the socket file")?;

    Ok(HsmClient::new(channel))
}

pub fn run() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    let request_counter = Arc::new(atomic::AtomicUsize::new(1000));
    if args.len() == 2 && args[1] == "--version" {
        println!("{}", version());
        return Ok(());
    }

    info!("Starting hsmproxy");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to create tokio runtime")?
        .block_on(async {
            let node = setup_node_stream()?;
            let grpc = grpc_connect().await?;
            process_requests(
                NodeConnection {
                    conn: node,
                    context: None,
                },
                request_counter,
                grpc,
            )
            .await
        })
}
