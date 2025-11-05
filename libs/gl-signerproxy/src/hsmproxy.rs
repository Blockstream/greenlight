// Implementation of the server-side hsmd. It collects requests and passes
// them on to the clients which actually have access to the keys.
use crate::pb::{hsm_client::HsmClient, Empty, HsmRequest, HsmRequestContext};
use crate::wire::{DaemonConnection, Message};
use anyhow::{anyhow, Context};
use anyhow::{Error, Result};
use log::{debug, error, info, warn};
use std::convert::TryFrom;
use std::env;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::str;
use std::sync::atomic;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
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
    let ms = unsafe { UnixStream::from_raw_fd(3) };
    Ok(DaemonConnection::new(ms))
}

fn start_handler(
    local: NodeConnection,
    counter: Arc<atomic::AtomicUsize>,
    grpc: GrpcClient,
    runtime: Arc<Runtime>,
) {
    thread::spawn(move || {
        match process_requests(local, counter, grpc, runtime).context("processing requests") {
            Ok(()) => panic!("why did the hsmproxy stop processing requests without an error?"),
            Err(e) => warn!("hsmproxy stopped processing requests with error: {}", e),
        }
    });
}

fn process_requests(
    node_conn: NodeConnection,
    request_counter: Arc<atomic::AtomicUsize>,
    mut server: GrpcClient,
    runtime: Arc<Runtime>,
) -> Result<(), Error> {
    let conn = node_conn.conn;
    let context = node_conn.context;
    info!("Pinging server");
    runtime.block_on(server.ping(Empty::default()))?;
    loop {
        if let Ok(msg) = conn.read() {
            match msg.msgtype() {
                9 => {
                    eprintln!("Got a message from node: {:?}", &msg.body);
                    // This requests a new client fd with a given context,
                    // handle it locally, and defer the creation of the client
                    // fd on the server side until we need it.
                    let ctx = HsmRequestContext::from_client_hsmfd_msg(&msg)?;
                    eprintln!("Got a request for a new client fd. Context: {:?}", ctx);

                    let (local, remote) = UnixStream::pair()?;
                    let local = NodeConnection {
                        conn: DaemonConnection::new(local),
                        context: Some(ctx),
                    };
                    let remote = remote.as_raw_fd();
                    let msg = Message::new_with_fds(vec![0, 109], &vec![remote]);

                    let grpc = server.clone();
                    // Start new handler for the client
                    start_handler(local, request_counter.clone(), grpc, runtime.clone());
                    if let Err(e) = conn.write(msg) {
                        error!("error writing msg to node_connection: {:?}", e);
                        return Err(e);
                    }
                }
                _ => {
                    // By default we forward to the remote HSMd
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
                    let start_time = tokio::time::Instant::now();
                    debug!("Got a message from node: {:?}", &req);
                    let res = runtime.block_on(server.request(req))?.into_inner();
                    let delta = start_time.elapsed();
                    let msg = Message::from_raw(res.raw);
                    eprintln!(
                        "WIRE: plugin -> hsmd: Got respone from hsmd: {:?} after {}ms",
                        &msg,
                        delta.as_millis()
                    );
                    eprintln!("WIRE: hsmd -> lightningd: {:?}", &msg);
                    conn.write(msg)?
                }
            }
        } else {
            error!("Connection lost");
            return Err(anyhow!("Connection lost"));
        }
    }
}

fn grpc_connect(runtime: &Runtime) -> Result<GrpcClient, Error> {
    runtime.block_on(async {
        // We will ignore this uri because uds do not use it
        // if your connector does use the uri it will be provided
        // as the request to the `MakeConnection`.
        // Connect to a Uds socket
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
    })
}

pub fn run() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();

    // Start the counter at 1000 so we can inject some message before
    // real requests if we want to.
    let request_counter = Arc::new(atomic::AtomicUsize::new(1000));
    if args.len() == 2 && args[1] == "--version" {
        println!("{}", version());
        return Ok(());
    }

    info!("Starting hsmproxy");

    // Create a dedicated tokio runtime for gRPC operations
    let runtime = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to create tokio runtime")?,
    );

    let node = setup_node_stream()?;
    let grpc = grpc_connect(&runtime)?;

    process_requests(
        NodeConnection {
            conn: node,
            context: None,
        },
        request_counter,
        grpc,
        runtime,
    )
}
