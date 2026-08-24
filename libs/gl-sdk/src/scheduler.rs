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

    /// Asks whether the Lightning account backed by `node_id` may be
    /// surfaced to the user.
    ///
    /// This is a feature gate for applications that offer Lightning
    /// alongside other account types. Greenlight relays the question
    /// to its Lightning Service Provider, which answers based on
    /// whether it has previously granted this node a slot, or has the
    /// capacity to grant one now.
    ///
    /// Deliberately callable before `register()`, since an
    /// application has to decide whether to offer Lightning at all
    /// before it creates a node. It needs no credentials, and the
    /// `node_id` is passed explicitly.
    ///
    /// The answer is advisory and may change over time. Treat an
    /// error as "unknown" rather than as a negative answer.
    pub fn lightning_available(&self, node_id: Vec<u8>) -> Result<bool, Error> {
        let nobody = self.nobody();
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(self.network, nobody)
                .await
                .map_err(|e| Error::other(e.to_string()))?;

            let res = inner
                .check_lightning_availability(node_id)
                .await
                .map_err(|e| Error::rpc(e.to_string()))?;

            Ok(res.available)
        })
    }

    pub fn register(&self, signer: &Signer, code: Option<String>) -> Result<Credentials, Error> {
        let nobody = self.nobody();
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(self.network, nobody)
                .await
                .map_err(|e| Error::other(e.to_string()))?;

            let res = inner
                .register(&signer.inner, code)
                .await
                .map_err(|e| Error::other(e.to_string()))?;

            Credentials::load(res.creds).map_err(|_e| Error::unparseable_creds())
        })
    }

    pub fn recover(&self, signer: &Signer) -> Result<Credentials, Error> {
        let nobody = self.nobody();
        exec(async move {
            let inner = gl_client::scheduler::Scheduler::new(self.network, nobody)
                .await
                .map_err(|e| Error::other(e.to_string()))?;

            let res = inner
                .recover(&signer.inner)
                .await
                .map_err(|e| Error::other(e.to_string()))?;

            Credentials::load(res.creds).map_err(|_e| Error::unparseable_creds())
        })
    }
}
