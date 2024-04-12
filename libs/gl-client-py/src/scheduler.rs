use crate::credentials::{Credentials, PyCredentials};
use crate::runtime::exec;
use crate::Signer;
use anyhow::{anyhow, Result};
use gl_client::bitcoin::Network;
use gl_client::credentials::RuneProvider;
use gl_client::credentials::TlsConfigProvider;
use gl_client::pb;
use gl_client::scheduler;
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[derive(Clone)]
pub enum UnifiedScheduler<T, R>
where
    T: TlsConfigProvider,
    R: TlsConfigProvider + RuneProvider + Clone,
{
    Unauthenticated(scheduler::Scheduler<T>),
    Authenticated(scheduler::Scheduler<R>),
}

impl<T, R> UnifiedScheduler<T, R>
where
    T: TlsConfigProvider,
    R: TlsConfigProvider + RuneProvider + Clone,
{
    pub fn is_authenticated(&self) -> Result<()> {
        if let Self::Authenticated(_) = self {
            Ok(())
        } else {
            Err(anyhow!("scheduler is unauthenticated",))?
        }
    }

    async fn register(
        &self,
        signer: &gl_client::signer::Signer,
        invite_code: Option<String>,
    ) -> Result<pb::scheduler::RegistrationResponse> {
        match self {
            UnifiedScheduler::Unauthenticated(u) => u.register(&signer, invite_code).await,
            UnifiedScheduler::Authenticated(a) => a.register(&signer, invite_code).await,
        }
    }

    async fn recover(
        &self,
        signer: &gl_client::signer::Signer,
    ) -> Result<pb::scheduler::RecoveryResponse> {
        match self {
            UnifiedScheduler::Unauthenticated(u) => u.recover(&signer).await,
            UnifiedScheduler::Authenticated(a) => a.recover(&signer).await,
        }
    }

    async fn authenticate(self, creds: R) -> Result<Self> {
        match self {
            UnifiedScheduler::Unauthenticated(u) => {
                let inner = u.authenticate(creds).await?;
                Ok(Self::Authenticated(inner))
            }
            UnifiedScheduler::Authenticated(_) => {
                Err(anyhow!("scheduler is already authenticated"))
            }
        }
    }
}

/// The following implementations need an authenticated scheduler.
impl<T, R> UnifiedScheduler<T, R>
where
    T: TlsConfigProvider,
    R: TlsConfigProvider + RuneProvider + Clone,
{
    async fn export_node(&self) -> Result<pb::scheduler::ExportNodeResponse> {
        let s = self.authenticated_scheduler()?;
        s.export_node().await
    }

    async fn schedule(&self) -> Result<pb::scheduler::NodeInfoResponse> {
        let s = self.authenticated_scheduler()?;
        s.schedule().await
    }

    async fn node(&self) -> Result<pb::scheduler::NodeInfoResponse> {
        let s = self.authenticated_scheduler()?;
        s.schedule().await
    }

    async fn get_node_info(&self, wait: bool) -> Result<pb::scheduler::NodeInfoResponse> {
        let s = self.authenticated_scheduler()?;
        s.get_node_info(wait).await
    }

    async fn get_invite_codes(&self) -> Result<pb::scheduler::ListInviteCodesResponse> {
        let s = self.authenticated_scheduler()?;
        s.get_invite_codes().await
    }

    async fn add_outgoing_webhook(
        &self,
        uri: String,
    ) -> Result<pb::scheduler::AddOutgoingWebhookResponse> {
        let s = self.authenticated_scheduler()?;
        s.add_outgoing_webhook(uri).await
    }

    async fn list_outgoing_webhooks(&self) -> Result<pb::scheduler::ListOutgoingWebhooksResponse> {
        let s = self.authenticated_scheduler()?;
        s.list_outgoing_webhooks().await
    }

    async fn delete_outgoing_webhooks(&self, ids: Vec<i64>) -> Result<pb::Empty> {
        let s = self.authenticated_scheduler()?;
        s.delete_webhooks(ids).await
    }

    async fn rotate_outgoing_webhook_secret(
        &self,
        webhook_id: i64,
    ) -> Result<pb::scheduler::WebhookSecretResponse> {
        let s = self.authenticated_scheduler()?;
        s.rotate_outgoing_webhook_secret(webhook_id).await
    }

    fn authenticated_scheduler(&self) -> Result<&scheduler::Scheduler<R>> {
        match self {
            UnifiedScheduler::Unauthenticated(_) => {
                Err(anyhow!("scheduler needs to be authenticated"))
            }
            UnifiedScheduler::Authenticated(a) => Ok(a),
        }
    }
}

#[pyclass]
pub struct Scheduler {
    node_id: Vec<u8>,
    pub inner: UnifiedScheduler<PyCredentials, PyCredentials>,
}

#[pymethods]
impl Scheduler {
    #[new]
    fn new(node_id: Vec<u8>, network: &str, creds: Credentials) -> PyResult<Scheduler> {
        let network: Network = network
            .parse()
            .map_err(|_| PyValueError::new_err("Error parsing the network"))?;

        let id = node_id.clone();
        let uri = gl_client::utils::scheduler_uri();

        let inner = match creds.inner {
            crate::credentials::UnifiedCredentials::Nobody(_) => {
                let scheduler = exec(async move {
                    gl_client::scheduler::Scheduler::with(id, network, creds.inner.clone(), uri)
                        .await
                })
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
                UnifiedScheduler::Unauthenticated(scheduler)
            }
            crate::credentials::UnifiedCredentials::Device(_) => {
                let scheduler = exec(async move {
                    gl_client::scheduler::Scheduler::with(id, network, creds.inner.clone(), uri)
                        .await
                })
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
                UnifiedScheduler::Authenticated(scheduler)
            }
        };

        Ok(Scheduler { node_id, inner })
    }

    fn register(&self, signer: &Signer, invite_code: Option<String>) -> PyResult<Vec<u8>> {
        convert(exec(async {
            self.inner.register(&signer.inner, invite_code).await
        }))
    }

    fn recover(&self, signer: &Signer) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.recover(&signer.inner).await }))
    }

    fn authenticate(&self, creds: Credentials) -> PyResult<Self> {
        creds.ensure_device().map_err(|_| {
            PyValueError::new_err(
                "can not authenticate scheduler, need device credentials".to_string(),
            )
        })?;
        let s =
            exec(async { self.inner.clone().authenticate(creds.inner).await }).map_err(|e| {
                PyValueError::new_err(format!(
                    "could not authenticate scheduler {}",
                    e.to_string()
                ))
            })?;
        Ok(Scheduler {
            node_id: self.node_id.clone(),
            inner: s,
        })
    }

    fn export_node(&self) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.export_node().await }))
    }

    fn schedule(&self) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.schedule().await }))
    }

    fn node(&self) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.node().await }))
    }

    fn get_invite_codes(&self) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.get_invite_codes().await }))
    }

    fn get_node_info(&self, wait: bool) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.get_node_info(wait).await }))
    }

    fn add_outgoing_webhook(&self, uri: String) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.add_outgoing_webhook(uri).await }))
    }

    fn list_outgoing_webhooks(&self) -> PyResult<Vec<u8>> {
        convert(exec(async { self.inner.list_outgoing_webhooks().await }))
    }

    fn delete_outgoing_webhooks(&self, webhook_ids: Vec<i64>) -> PyResult<Vec<u8>> {
        convert(exec(async {
            self.inner.delete_outgoing_webhooks(webhook_ids).await
        }))
    }

    fn rotate_outgoing_webhook_secret(&self, webhook_id: i64) -> PyResult<Vec<u8>> {
        convert(exec(async {
            self.inner.rotate_outgoing_webhook_secret(webhook_id).await
        }))
    }
}

pub fn convert<T: Message>(r: Result<T>) -> PyResult<Vec<u8>> {
    let res = r.map_err(crate::node::error_calling_remote_method)?;
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(buf)
}
