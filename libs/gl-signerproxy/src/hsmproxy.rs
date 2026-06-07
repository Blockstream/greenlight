// Implementation of the server-side hsmd. It collects requests and passes
// them on to the clients which actually have access to the keys.
use crate::pb::{hsm_client::HsmClient, Empty, HsmRequest, HsmRequestContext, HsmResponse};
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
use tokio::sync::{mpsc, oneshot};
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

/// Messages sent from blocking handler threads to the async gRPC dispatcher.
enum GrpcMessage {
    Ping {
        response_tx: oneshot::Sender<Result<(), tonic::Status>>,
    },
    Request {
        request: HsmRequest,
        response_tx: oneshot::Sender<Result<HsmResponse, tonic::Status>>,
    },
}

/// Async dispatcher running on the single Tokio runtime. Each message is
/// spawned as an independent task so a stuck gRPC call (e.g. type 143 waiting
/// for a signer that never connects) cannot block other concurrent requests.
async fn grpc_dispatcher(server: GrpcClient, mut rx: mpsc::Receiver<GrpcMessage>) {
    while let Some(msg) = rx.recv().await {
        let mut s = server.clone();
        tokio::spawn(async move {
            match msg {
                GrpcMessage::Ping { response_tx } => {
                    let res = s.ping(Empty::default()).await.map(|_| ());
                    let _ = response_tx.send(res);
                }
                GrpcMessage::Request {
                    request,
                    response_tx,
                } => {
                    let res = s
                        .request(tonic::Request::new(request))
                        .await
                        .map(|r| r.into_inner());
                    let _ = response_tx.send(res);
                }
            }
        });
    }
}

fn start_handler(
    local: NodeConnection,
    counter: Arc<atomic::AtomicUsize>,
    tx: mpsc::Sender<GrpcMessage>,
) {
    thread::spawn(move || {
        match process_requests(local, counter, tx).context("processing requests") {
            Ok(()) => panic!("why did the hsmproxy stop processing requests without an error?"),
            Err(e) => warn!("hsmproxy stopped processing requests with error: {}", e),
        }
    });
}

fn process_requests(
    node_conn: NodeConnection,
    request_counter: Arc<atomic::AtomicUsize>,
    tx: mpsc::Sender<GrpcMessage>,
) -> Result<(), Error> {
    let conn = node_conn.conn;
    let context = node_conn.context;

    info!("Pinging server");
    let (ping_tx, ping_rx) = oneshot::channel();
    tx.blocking_send(GrpcMessage::Ping { response_tx: ping_tx })
        .context("dispatcher gone")?;
    ping_rx
        .blocking_recv()
        .context("dispatcher dropped before ping response")?
        .map_err(|e| anyhow!("ping failed: {}", e))?;

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

                    start_handler(local, request_counter.clone(), tx.clone());
                    if let Err(e) = conn.write(msg) {
                        error!("error writing msg to node_connection: {:?}", e);
                        return Err(e);
                    }
                }
		28 => {
		    eprintln!("Locally handling the `hsmd_check_pubkey` call");
		    let msg = Message::new(vec![0, 128, 1]);
		    conn.write(msg)?
		},
                _ => {
                    // By default we forward to the remote HSMd
                    let req = HsmRequest {
                        context: context.clone(),
                        raw: msg.body.clone(),
                        request_id: request_counter.fetch_add(1, atomic::Ordering::Relaxed) as u32,
                        requests: Vec::new(),
                        signer_state: Vec::new(),
                    };

                    eprintln!(
                        "WIRE: lightningd -> hsmd: Got a message from node: {:?}",
                        &req
                    );
                    let start_time = std::time::Instant::now();
                    debug!("Got a message from node: {:?}", &req);

                    let (resp_tx, resp_rx) = oneshot::channel();
                    tx.blocking_send(GrpcMessage::Request {
                        request: req,
                        response_tx: resp_tx,
                    })
                    .context("dispatcher gone")?;
                    let res = resp_rx
                        .blocking_recv()
                        .context("dispatcher dropped before response")?
                        .map_err(|e| anyhow!("gRPC error: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::HsmResponse;
    use byteorder::{BigEndian, ByteOrder};
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream as StdUnixStream;
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    /// Write a CLN wire message: 4-byte big-endian length prefix followed by body.
    fn write_cln_msg(stream: &mut StdUnixStream, body: &[u8]) {
        let mut len_buf = [0u8; 4];
        BigEndian::write_u32(&mut len_buf, body.len() as u32);
        stream.write_all(&len_buf).unwrap();
        stream.write_all(body).unwrap();
    }

    /// Read a CLN wire message, returning the body (without the length prefix).
    fn read_cln_msg(stream: &mut StdUnixStream) -> Vec<u8> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).unwrap();
        let len = BigEndian::read_u32(&len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).unwrap();
        buf
    }

    /// Starts a mock gRPC dispatcher on a background thread.
    ///
    /// Pings are acked immediately. Type 143 requests are held for 60 s to
    /// simulate a signer that never connects. All other requests respond at once.
    fn start_mock_dispatcher(mut rx: mpsc::Receiver<GrpcMessage>) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                while let Some(msg) = rx.recv().await {
                    tokio::spawn(async move {
                        match msg {
                            GrpcMessage::Ping { response_tx } => {
                                let _ = response_tx.send(Ok(()));
                            }
                            GrpcMessage::Request {
                                request,
                                response_tx,
                            } => {
                                let type_id = ((request.raw[0] as u16) << 8)
                                    | request.raw[1] as u16;
                                if type_id == 143 {
                                    // Simulate a signer that never arrives.
                                    tokio::time::sleep(Duration::from_secs(60)).await;
                                }
                                let _ = response_tx.send(Ok(HsmResponse {
                                    request_id: request.request_id,
                                    raw: vec![0, 0],
                                    signer_state: vec![],
                                    error: "".into(),
                                }));
                            }
                        }
                    });
                }
            });
        });
    }

    /// Spawns a `process_requests` loop in a background thread using the given
    /// socket end and channel sender.
    fn spawn_proxy_connection(
        proxy_end: StdUnixStream,
        counter: Arc<AtomicUsize>,
        tx: mpsc::Sender<GrpcMessage>,
    ) {
        std::thread::spawn(move || {
            let conn = NodeConnection {
                conn: DaemonConnection::new(proxy_end),
                context: None,
            };
            let _ = process_requests(conn, counter, tx);
        });
    }

    /// A type 143 request that blocks indefinitely (no signer) must not
    /// prevent a concurrent type 27 request on a different connection from
    /// completing in a timely fashion.
    #[test]
    fn type_143_lockup_does_not_block_other_requests() {
        let (tx, rx) = mpsc::channel::<GrpcMessage>(64);
        start_mock_dispatcher(rx);

        let counter = Arc::new(AtomicUsize::new(1000));

        // Connection 1 — onchaind analogue; will send the blocking type 143.
        let (mut cln_1, proxy_1) = StdUnixStream::pair().unwrap();
        spawn_proxy_connection(proxy_1, counter.clone(), tx.clone());

        // Connection 2 — any other subdaemon; should be unaffected.
        let (mut cln_2, proxy_2) = StdUnixStream::pair().unwrap();
        spawn_proxy_connection(proxy_2, counter.clone(), tx.clone());

        // Allow both connections to complete the startup ping.
        std::thread::sleep(Duration::from_millis(100));

        // Send a blocking type 143 on connection 1.
        write_cln_msg(&mut cln_1, &[0, 143, 0, 0, 0, 0]);

        // Brief pause so the type 143 task is in-flight before we send the
        // next request — this is the race condition the fix resolves.
        std::thread::sleep(Duration::from_millis(50));

        // Send a fast type 27 on connection 2.
        write_cln_msg(&mut cln_2, &[0, 27, 0, 0]);

        // The type 27 response must arrive well before the 60-second type 143
        // timeout; 5 seconds is a generous bound for CI.
        cln_2
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let start = Instant::now();
        let response = read_cln_msg(&mut cln_2);
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(5),
            "type 27 response took {:?} — it was blocked by the type 143 request",
            elapsed
        );
        assert!(
            !response.is_empty(),
            "expected a non-empty response to the type 27 request"
        );
    }
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

    let (tx, rx) = mpsc::channel::<GrpcMessage>(64);
    let grpc = grpc_connect(&runtime)?;

    // Dedicated thread drives the runtime and the async dispatcher.
    // Each incoming GrpcMessage is spawned as an independent task so
    // concurrent requests cannot block each other.
    let rt = runtime.clone();
    thread::spawn(move || {
        rt.block_on(grpc_dispatcher(grpc, rx));
    });

    let node = setup_node_stream()?;
    process_requests(
        NodeConnection {
            conn: node,
            context: None,
        },
        request_counter,
        tx,
    )
}
