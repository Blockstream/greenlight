use crate::{Credentials, Error};
use bip39::{Language, Mnemonic};
use std::str::FromStr;

#[derive(uniffi::Object, Clone)]
pub struct Signer {
    seed: Vec<u8>,
    pub(crate) inner: gl_client::signer::Signer,
    credentials: Option<Credentials>,
}

#[uniffi::export]
impl Signer {
    #[uniffi::constructor()]
    fn new(phrase: String) -> Result<Signer, Error> {
        let phrase = Mnemonic::from_str(phrase.as_str()).map_err(|e| Error::PhraseCorrupted())?;
        let seed = phrase.to_seed_normalized(&"").to_vec();

        // FIXME: We may need to give the signer real credentials to
        // talk to the node too.
        let credentials = gl_client::credentials::Nobody::new();

        let inner = gl_client::signer::Signer::new(
            seed.clone(),
            gl_client::bitcoin::Network::Bitcoin,
            credentials,
        )
        .map_err(|e| Error::Other(e.to_string()))?;
        let credentials = None;
        Ok(Signer {
            seed,
            inner,
            credentials,
        })
    }

    fn authenticate(&self, creds: &Credentials) -> Result<Signer, Error> {
        let credentials = Some(creds.clone());

        let inner = gl_client::signer::Signer::new(
            self.seed.clone(),
            gl_client::bitcoin::Network::Bitcoin,
            creds.inner.clone(),
        )
        .map_err(|e| Error::Other(e.to_string()))?;

        Ok(Signer {
            inner,
            credentials,
            ..self.clone()
        })
    }

    fn start(&self) -> Result<Handle, Error> {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel(1);

        let clone = self.clone();
        tokio::spawn(async move {
            clone.run(rx).await;
        });

        Ok(Handle { chan: tx })
    }

    fn node_id(&self) -> Vec<u8> {
        unimplemented!()
    }
}

// Not exported through uniffi, internal logic only.
impl Signer {
    async fn run(&self, signal: tokio::sync::mpsc::Receiver<()>) {
        self.inner.run_forever(signal).await.expect("Error running signer loop");
    }
}

/// A handle to interact with a signer loop running and processing
/// requests in the background. Used primarily to stop the loop and
/// exiting the signer.
#[derive(uniffi::Object, Clone)]
pub struct Handle {
    chan: tokio::sync::mpsc::Sender<()>,
}

#[uniffi::export]
impl Handle {
    pub fn stop(&self) {
        self.chan.try_send(()).expect("sending shutdown signal");
    }
}
