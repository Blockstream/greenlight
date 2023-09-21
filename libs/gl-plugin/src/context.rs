//! Manage a signature request context.
//!
//! The signature request context is composed of any currently pending grpc request (serialized as byte string), along with a public key (corresponding to the caller's mTLS certificate), an attestation (signature) by the signer about the authenticity of this public key, as well as a signature from the caller's public key over the serialized payload.
//!
//! It is used by the signer to verify that:
//!
//! a) The caller is authenticated and authorized to initiate the
//!    action with the grpc call.
//! b) Verify that the changes that the signer is being asked to
//!    sign off actually match the authentic commands by a valid
//!    caller.

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    // The caller's mTLS public key
    pubkey: Vec<u8>,

    // A signature by the caller's public key, authenticating the
    // payload.
    signature: Vec<u8>,

    // The serialized grpc call, transferred as serialized String to
    // avoid breaking the signature.
    payload: bytes::Bytes,

    // The URI that the request asked for
    uri: String,

    // Timestamp in millis
    timestamp: Option<u64>,
}

impl Request {
    pub fn new(
        uri: String,
        payload: bytes::Bytes,
        pubkey: Vec<u8>,
        signature: Vec<u8>,
        timestamp: Option<u64>,
    ) -> Self {
        Request {
            uri,
            payload,
            signature,
            timestamp,
            pubkey,
        }
    }
}

impl From<Request> for crate::pb::PendingRequest {
    fn from(r: crate::context::Request) -> Self {
        crate::pb::PendingRequest {
            pubkey: r.pubkey,
            signature: r.signature,
            request: r.payload.to_vec(),
            uri: r.uri,
	    timestamp: r.timestamp.unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    // List of currently pending requests.
    requests: Arc<Mutex<Vec<Request>>>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn snapshot(&self) -> Vec<Request> {
        let r = self.requests.lock().await;
        r.clone()
    }

    pub async fn add_request(&self, r: Request) {
        let mut reqs = self.requests.lock().await;
        reqs.push(r);
    }

    pub async fn remove_request(&self, r: Request) {
        let mut reqs = self.requests.lock().await;
        reqs.retain(|a| a.signature != r.signature)
    }
}
