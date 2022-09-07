use crate::runtime::exec;
use crate::Signer;
use crate::{pb, pb::scheduler_client::SchedulerClient};
use anyhow::Result;
use bitcoin::Network;
use gl_client::tls::TlsConfig;
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tonic::transport::Channel;

type Client = SchedulerClient<Channel>;

#[pyclass]
#[derive(Clone)]
pub struct Scheduler {
    node_id: Vec<u8>,
    inner: gl_client::scheduler::Scheduler,
}

#[pymethods]
impl Scheduler {
    #[new]
    fn new(node_id: Vec<u8>, network: String) -> PyResult<Scheduler> {
        let network: Network = match network.parse() {
            Ok(v) => v,
            Err(_) => return Err(PyValueError::new_err("Error parsing the network")),
        };
        debug!("Node ID {}", hex::encode(&node_id));

        let id = node_id.clone();
        let res = exec(async move { gl_client::scheduler::Scheduler::new(id, network).await });

        let inner = match res {
            Ok(v) => v,
            Err(_) => return Err(PyValueError::new_err("could not connect to the scheduler")),
        };

        Ok(Scheduler { node_id, inner })
    }

    fn register(&self, signer: &Signer) -> PyResult<Vec<u8>> {
        convert(exec(self.inner.register(&signer.inner)))
    }

    fn recover(&self, signer: &Signer) -> PyResult<Vec<u8>> {
        convert(exec(async move { self.inner.recover(&signer.inner).await }))
    }

    fn get_node_info(&self) -> PyResult<Vec<u8>> {
        let res: Result<pb::NodeInfoResponse> = exec(async move {
            let mut client = self.connect().await.unwrap();

            let info = client
                .get_node_info(pb::NodeInfoRequest {
                    node_id: self.node_id.clone(),
                    wait: false,
                })
                .await;

            Ok(info?.into_inner())
        });

        let res = match res {
            Ok(v) => v,
            Err(_) => return Err(PyValueError::new_err("error calling get_node_info")),
        };
        let mut buf = Vec::with_capacity(res.encoded_len());
        res.encode(&mut buf).unwrap();
        Ok(buf)
    }

    fn schedule(&self) -> PyResult<Vec<u8>> {
        convert(exec(async move {
            Ok(self
                .connect()
                .await?
                .schedule(pb::ScheduleRequest {
                    node_id: self.node_id.clone(),
                })
                .await?
                .into_inner())
        }))
    }
}

pub fn convert<T: Message>(r: Result<T>) -> PyResult<Vec<u8>> {
    let res = r.map_err(crate::node::error_calling_remote_method)?;
    let mut buf = Vec::with_capacity(res.encoded_len());
    res.encode(&mut buf).unwrap();
    Ok(buf)
}

impl Scheduler {
    async fn connect_with(&self, uri: String, tls: &TlsConfig) -> Result<Client> {
        let client_tls = tls.client_tls_config();
        let channel = Channel::from_shared(uri)?
            .tls_config(client_tls)?
            .connect()
            .await?;
        Ok(SchedulerClient::new(channel))
    }

    async fn connect(&self) -> Result<Client> {
        let uri = gl_client::utils::scheduler_uri();
        let tls = TlsConfig::new()?;
        self.connect_with(uri, &tls).await
    }
}
