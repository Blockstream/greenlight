use super::*;
use crate::{credentials, pb};
use ring::signature::{self, EcdsaKeyPair, KeyPair};
use vls_protocol::msgs::GetHeartbeat;

const ADDRESS: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
const CONFIGURE: &str = "/greenlight.Node/Configure";

fn mk_signer() -> Signer {
    Signer::new(
        vec![0; 32],
        Network::Bitcoin,
        credentials::Nobody::default(),
    )
    .unwrap()
}

fn pkcs8() -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
        .unwrap()
        .as_ref()
        .to_vec()
}

fn keypair(pkcs8: &[u8]) -> EcdsaKeyPair {
    EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8).unwrap()
}

fn sign(key: &EcdsaKeyPair, data: &[u8]) -> Vec<u8> {
    key.sign(&ring::rand::SystemRandom::new(), data)
        .unwrap()
        .as_ref()
        .to_vec()
}

fn frame(message: &impl prost::Message) -> Vec<u8> {
    let payload = message.encode_to_vec();
    let mut frame = vec![0];
    frame.put_u32(payload.len() as u32);
    frame.extend_from_slice(&payload);
    frame
}

fn rune(signer: &Signer, key: &EcdsaKeyPair) -> String {
    let pubkey = format!("pubkey={}", hex::encode(key.public_key()));
    signer.create_rune(None, vec![vec![&pubkey]]).unwrap()
}

fn signed_context(
    signer: &Signer,
    key: &EcdsaKeyPair,
    uri: &str,
    frame: Vec<u8>,
) -> PendingRequest {
    let timestamp = 1_700_000_000_000;
    PendingRequest {
        uri: uri.to_string(),
        signature: sign(
            key,
            &crate::request_context::payload(uri, &frame, timestamp),
        ),
        request: frame,
        pubkey: key.public_key().as_ref().to_vec(),
        timestamp,
        rune: general_purpose::URL_SAFE.decode(rune(signer, key)).unwrap(),
    }
}

fn configure(signer: &Signer, key: &EcdsaKeyPair, address: &str) -> PendingRequest {
    let frame = frame(&pb::GlConfig {
        close_to_addr: address.into(),
    });
    signed_context(signer, key, CONFIGURE, frame)
}

/// Run an operation that needs no context, which must succeed whatever the
/// contexts attached to it are.
async fn heartbeat(signer: &Signer, requests: Vec<PendingRequest>) -> HsmResponse {
    signer
        .process_request(HsmRequest {
            request_id: 1,
            raw: GetHeartbeat {}.as_vec(),
            requests,
            ..Default::default()
        })
        .await
        .unwrap()
}

fn allowlist(signer: &Signer) -> Vec<String> {
    signer
        .services
        .persister
        .get_node_allowlist(&PublicKey::from_slice(&signer.id).unwrap())
        .unwrap()
}

#[test]
fn rpc_context_signature_binds_uri_frame_and_timestamp() {
    let signer = mk_signer();
    let key = keypair(&pkcs8());
    let context = signed_context(
        &signer,
        &key,
        "/cln.Node/Pay",
        frame(&pb::cln::PayRequest {
            bolt11: "invoice".into(),
            ..Default::default()
        }),
    );
    let is_valid = |c: &PendingRequest| signer.check_request_auth(vec![c.clone()])[0].is_ok();
    assert!(is_valid(&context));

    let mut changed = context.clone();
    changed.uri = "/cln.Node/CheckMessage".into();
    assert!(!is_valid(&changed));

    let mut changed = context.clone();
    changed.request[5] ^= 1;
    assert!(!is_valid(&changed));

    let mut changed = context;
    changed.timestamp += 1;
    assert!(!is_valid(&changed));
}

#[tokio::test]
async fn rpc_context_relabeling_has_no_effect() {
    let signer = mk_signer();
    let key = keypair(&pkcs8());
    let check_message = signed_context(
        &signer,
        &key,
        "/cln.Node/CheckMessage",
        frame(&pb::cln::CheckmessageRequest {
            message: ADDRESS.into(),
            ..Default::default()
        }),
    );
    let getinfo = signed_context(&signer, &key, "/cln.Node/Getinfo", vec![0; 5]);
    for (request, target) in [
        (&check_message, "/cln.Node/Pay"),
        (&check_message, "/cln.Node/PreApproveInvoice"),
        (&check_message, CONFIGURE),
        (&getinfo, "/cln.Node/Invoice"),
        (&getinfo, CONFIGURE),
    ] {
        let mut relabeled = request.clone();
        relabeled.uri = target.into();
        // Protobuf decoding alone cannot distinguish these intents.
        assert!(decode_request(relabeled.clone()).is_ok());
        assert!(signer.check_request_auth(vec![relabeled.clone()])[0].is_err());
        heartbeat(&signer, vec![relabeled]).await;
        assert!(allowlist(&signer).is_empty());
    }
}

/// Older clients sign only the frame and timestamp. Such a context authorizes
/// nothing, and must not stop the signer serving the node.
#[tokio::test]
async fn rpc_context_legacy_signature_is_ignored() {
    let signer = mk_signer();
    let key = keypair(&pkcs8());
    let valid = configure(&signer, &key, ADDRESS);

    let mut legacy = valid.clone();
    let mut data = legacy.request.clone();
    data.put_u64(legacy.timestamp);
    legacy.signature = sign(&key, &data);

    heartbeat(&signer, vec![legacy.clone()]).await;
    assert!(allowlist(&signer).is_empty());

    // An invalid sibling must not suppress an authentic command either.
    heartbeat(&signer, vec![legacy, valid]).await;
    assert_eq!(allowlist(&signer), [ADDRESS]);
}

#[tokio::test]
async fn rpc_context_configure_only_allowlists_valid_addresses() {
    let signer = mk_signer();
    let key = keypair(&pkcs8());
    heartbeat(&signer, vec![configure(&signer, &key, ADDRESS)]).await;
    assert_eq!(allowlist(&signer), [ADDRESS]);

    for invalid in [
        "",
        "not-an-address",
        "payee:02abcdef",
        "mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn", // Testnet
    ] {
        let context = configure(&signer, &key, invalid);
        heartbeat(&signer, vec![context]).await;
        assert_eq!(allowlist(&signer), [ADDRESS]);
    }
}

/// A signature made by the real client, over a real connection, must verify
/// against the URI and frame a node sees. Permissive signers would hide a
/// mismatch here, since they proceed without the contexts they ignore.
#[tokio::test]
async fn rpc_context_client_signature_verifies_over_the_wire() {
    use std::convert::Infallible;
    use std::task::{Context, Poll};
    use tokio::sync::mpsc;
    use tonic::codegen::Body as _;
    use tonic::transport::{server::TcpIncoming, Body, Channel, Server};
    use tower::Layer;

    /// Captures a request the way the plugin builds a context from it.
    #[derive(Clone)]
    struct Node(mpsc::UnboundedSender<(http::request::Parts, Vec<u8>)>);

    impl tower::Service<http::Request<Body>> for Node {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Infallible;
        type Future = futures::future::BoxFuture<'static, Result<Self::Response, Infallible>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: http::Request<Body>) -> Self::Future {
            let captured = self.0.clone();
            Box::pin(async move {
                let (parts, mut body) = request.into_parts();
                let mut frame = vec![];
                while let Some(chunk) = body.data().await {
                    frame.extend_from_slice(&chunk.unwrap());
                }
                captured.send((parts, frame)).unwrap();
                Ok(tonic::Status::ok("").to_http())
            })
        }
    }

    impl tonic::server::NamedService for Node {
        const NAME: &'static str = "cln.Node";
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let incoming = TcpIncoming::from_listener(listener, true, None).unwrap();
    let (captured, mut requests) = mpsc::unbounded_channel();
    tokio::spawn(
        Server::builder()
            .add_service(Node(captured))
            .serve_with_incoming(incoming),
    );

    let signer = mk_signer();
    let pkcs8 = pkcs8();
    let key = keypair(&pkcs8);
    let pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        general_purpose::STANDARD.encode(&pkcs8)
    );
    let channel = Channel::from_shared(format!("http://{address}"))
        .unwrap()
        .connect_lazy();
    let service = crate::node::service::AuthLayer::new(pem.into_bytes(), rune(&signer, &key))
        .unwrap()
        .layer(channel);
    // The node does not reply, only the request it captures matters here.
    let _ = pb::cln::node_client::NodeClient::new(service)
        .pay(pb::cln::PayRequest {
            bolt11: "invoice".into(),
            ..Default::default()
        })
        .await;

    let (parts, request) = requests.recv().await.unwrap();
    let header = |name: &str| {
        general_purpose::STANDARD_NO_PAD
            .decode(&parts.headers[name])
            .unwrap()
    };
    let context = PendingRequest {
        uri: parts.uri.path_and_query().unwrap().to_string(),
        request,
        signature: header("glauthsig"),
        pubkey: header("glauthpubkey"),
        timestamp: u64::from_be_bytes(header("glts").try_into().unwrap()),
        rune: general_purpose::URL_SAFE
            .decode(&parts.headers["glrune"])
            .unwrap(),
    };
    assert_eq!(context.uri, "/cln.Node/Pay");
    assert!(signer.check_request_auth(vec![context])[0].is_ok());
}
