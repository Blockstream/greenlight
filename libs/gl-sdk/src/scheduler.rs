use crate::{
    credentials::{Credentials, DeveloperCert},
    signer::Signer,
    util::exec,
    Error,
};

#[derive(uniffi::Object, Clone)]
pub struct Scheduler {
    credentials: Option<Credentials>,
    network: gl_client::bitcoin::Network,
    developer_cert: Option<gl_client::credentials::Nobody>,
}

impl Scheduler {
    /// Resolve the credentials to use for unauthenticated scheduler
    /// calls (register, recover). Uses the developer certificate if
    /// one was provided via `with_developer_cert()`, otherwise falls
    /// back to the compiled-in default.
    fn nobody(&self) -> gl_client::credentials::Nobody {
        self.developer_cert
            .clone()
            .unwrap_or_else(gl_client::credentials::Nobody::new)
    }
}

#[uniffi::export]
impl Scheduler {
    /// Create a `Scheduler` instance configured with the Greenlight
    /// production service pre-configured.
    #[uniffi::constructor()]
    pub fn new(network: crate::Network) -> Result<Scheduler, Error> {
        let network: gl_client::bitcoin::Network = network.into();

        Ok(Scheduler {
            credentials: None,
            network,
            developer_cert: None,
        })
    }

    /// Configure a developer certificate obtained from the Greenlight
    /// Developer Console. Nodes registered through this scheduler
    /// will be associated with the developer's account.
    ///
    /// Returns a new `Scheduler` instance with the developer
    /// certificate configured.
    pub fn with_developer_cert(&self, cert: &DeveloperCert) -> Scheduler {
        Scheduler {
            developer_cert: Some(cert.inner.clone()),
            ..self.clone()
        }
    }

    pub fn register(&self, signer: &Signer, code: Option<String>) -> Result<Credentials, Error> {
        let nobody = self.nobody();
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(self.network, nobody)
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
        let nobody = self.nobody();
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(self.network, nobody)
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
