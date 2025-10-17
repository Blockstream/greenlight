use crate::{credentials::Credentials, signer::Signer, util::exec, Error};

#[derive(uniffi::Object, Clone)]
pub struct Scheduler {
    credentials: Option<Credentials>,
    network: gl_client::bitcoin::Network,
}

#[uniffi::export]
impl Scheduler {
    /// Create a `Scheduler` instance configured with the Greenlight
    /// production service pre-configured.
    #[uniffi::constructor()]
    pub fn new(network: crate::Network) -> Result<Scheduler, Error> {
        // We use the nobody credentials since there is no
        // authenticated method we expose at the moment.
        let creds = None;
        let network: gl_client::bitcoin::Network = network.into();

        Ok(Scheduler {
            credentials: creds,
            network,
        })
    }

    pub fn register(&self, signer: &Signer, code: Option<String>) -> Result<Credentials, Error> {
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(
                self.network,
                gl_client::credentials::Nobody::new(),
            )
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

            let res = inner
                .register(&signer.inner, code)
                .await
                .map_err(|e| Error::Other(e.to_string().clone()))?;

            Credentials::load(res.creds).map_err(|_e| Error::UnparseableCreds())
        })
    }

    pub fn recover(&self, signer: &Signer) -> Result<Credentials, Error> {
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(
                self.network,
                gl_client::credentials::Nobody::new(),
            )
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

            let res = inner
                .recover(&signer.inner)
                .await
                .map_err(|e| Error::Other(e.to_string()))?;

            Credentials::load(res.creds).map_err(|_e| Error::UnparseableCreds())
        })
    }
}
