use crate::runtime::exec;
use crate::{Error, RawClient};
use anyhow::Context;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct Scheduler {
    inner: gl_client::scheduler::Scheduler,
    tls: Arc<Mutex<gl_client::tls::TlsConfig>>,
}

impl Scheduler {
    pub fn new(node_id: Vec<u8>, network: String) -> Result<Self, Error> {
        let network = gl_client::bitcoin::Network::from_str(&network)
            .map_err(|e| Error::InvalidArgument(Box::new(e)))?;
        let tls = Arc::new(Mutex::new(
            gl_client::tls::TlsConfig::new().context("creating TlsConfig")?,
        ));

        let inner = exec(gl_client::scheduler::Scheduler::new(node_id, network))
            .context("creating scheduler")?;

        Ok(Scheduler { inner, tls })
    }
    pub fn authenticate(&self, cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Result<(), Error> {
        let tls = gl_client::tls::TlsConfig::new()
            .unwrap()
            .identity(cert_pem, key_pem);
        *self.tls.lock().unwrap() = tls;
        Ok(())
    }

    pub fn schedule_raw(&self) -> Result<RawClient, Error> {
        let tls = self.tls.lock().unwrap().clone();

        Ok(RawClient::with_node(
            crate::runtime::exec(self.inner.schedule(tls)).map_err(|e| Error::Call {
                method: "scheduler.Scheduler/schedule".into(),
                error: e.to_string(),
            })?,
        ))
    }
}
