use anyhow::{anyhow, Result};
use http::{Request, Response};
use log::{debug, trace};
use rustls_pemfile as pemfile;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tower::{Layer, Service};

use ring::signature::KeyPair;
use ring::{
    rand,
    signature::{self, EcdsaKeyPair},
};

pub struct AuthLayer {
    key: Vec<u8>,
    rune: String,
}

impl AuthLayer {
    pub fn new(pem: Vec<u8>, rune: String) -> Result<Self> {
        // Try to convert the key into a keypair to make sure it works later
        // when we need it.
        let key = {
            let mut key = std::io::Cursor::new(&pem[..]);
            match pemfile::pkcs8_private_keys(&mut key) {
                Ok(v) => v,
                Err(e) => {
                    return Err(anyhow!(
                        "Could not decode PEM string into PKCS#8 format: {}",
                        e
                    ))
                }
            }
            .remove(0)
        };

        match EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, key.as_ref()) {
            Ok(_) => trace!("Successfully decoded keypair from PEM string"),
            Err(e) => return Err(anyhow!("Could not decide keypair from PEM string: {}", e)),
        };

        Ok(AuthLayer { key, rune })
    }
}

impl Layer<Channel> for AuthLayer {
    type Service = AuthService;

    fn layer(&self, inner: Channel) -> Self::Service {
        AuthService {
            key: self.key.clone(),
            inner,
            rune: self.rune.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService {
    // PKCS#8 formatted private key
    key: Vec<u8>,
    inner: Channel,
    rune: String,
}
impl Service<Request<BoxBody>> for AuthService {
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }
    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD_NO_PAD;

        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let keypair = EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            self.key.as_ref(),
        )
        .unwrap();

        let rune = self.rune.clone();

        Box::pin(async move {
            use bytes::BufMut;
            use std::convert::TryInto;
            use tonic::codegen::Body;

            // Returns UTC on all platforms, no need to handle
            // timezones.
            let time = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)?
                .as_millis();

            let (mut parts, mut body) = request.into_parts();

            let data = body.data().await.unwrap().unwrap();

            // Copy used to create the signature (payload + associated data)
            let mut buf = data.to_vec();

            // Associated data that is covered by the signature
            let mut ts = vec![];
            ts.put_u64(time.try_into()?);
            buf.put_u64(time.try_into()?);

            let rng = rand::SystemRandom::new();
            let pubkey = keypair.public_key().as_ref();
            let sig = keypair.sign(&rng, &buf).unwrap();

            // We use base64 encoding simply because it is
            // slightly more compact and we already have it as
            // a dependency from rustls. Sizes are as follows:
            //
            // - Pubkey: raw=65, hex=130, base64=88
            // - Signature: raw=64, hex=128, base64=88
            //
            // For an overall saving of 82 bytes per request,
            // and a total overhead of 199 bytes per request.
            parts
                .headers
                .insert("glauthpubkey", engine.encode(&pubkey).parse().unwrap());
            parts
                .headers
                .insert("glauthsig", engine.encode(sig).parse().unwrap());

            parts
                .headers
                .insert("glts", engine.encode(ts).parse().unwrap());

            // Runes already come base64 URL_SAFE encoded.
            parts
                .headers
                .insert("glrune", rune.parse().expect("Could not parse rune"));

            trace!("Payload size: {} (timestamp {})", data.len(), time);

            let body = crate::node::stasher::StashBody::new(data).into();
            let request = Request::from_parts(parts, body);
            debug!("Sending request {:?}", request);
            let response = inner.call(request).await?;
            Ok(response)
        })
    }
}
