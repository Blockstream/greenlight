use crate::pb::RegistrationResponse;
use crate::runtime::exec;
use crate::Signer;
use crate::{pb, pb::scheduler_client::SchedulerClient};
use anyhow::{anyhow, Result};
use core::future::Future;
use gl_client::Network;
use prost::Message;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::convert::TryInto;
use tokio::runtime::Builder;
use tonic::transport::Channel;

type Client = SchedulerClient<Channel>;

#[pyclass]
#[derive(Clone)]
pub struct Scheduler {
    node_id: Vec<u8>,
    network: Network,
}

#[pymethods]
impl Scheduler {
    #[new]
    fn new(node_id: Vec<u8>, network: String) -> PyResult<Scheduler> {
        let network: Network = match network.try_into() {
            Ok(v) => v,
            Err(e) => return Err(PyValueError::new_err("Error parsing the network")),
        };
        warn!("Node ID {:?}", node_id);
        Ok(Scheduler { node_id, network })
    }

    fn register(&self, signer: &Signer) -> PyResult<()> {
        let secs = 10;
        exec(async move {
            println!("Sleeping {}", secs);
            tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
            println!("Done sleeping {}", secs);
            Ok(())
        })
    }

    fn get_node_info(&self) -> PyResult<Vec<u8>> {
        let res: Result<pb::NodeInfoResponse> = exec(async move {
            let mut client = self.connect().await.unwrap();

            println!("get_node_id Node id {:?}", self.node_id);
            let info = client
                .get_node_info(pb::NodeInfoRequest {
                    node_id: self.node_id.clone(),
                    wait: false,
                })
                .await;

            println!("INFO: {:?}", info);
            Ok(info?.into_inner())
        });

        let res = match res {
            Ok(v) => v,
            Err(e) => return Err(PyValueError::new_err("error calling get_node_info")),
        };
        let mut buf = Vec::new();
        buf.reserve(res.encoded_len());
        res.encode(&mut buf).unwrap();
        Ok(buf)
    }
}

impl Scheduler {
    async fn connect(&self) -> Result<Client> {
        let uri = gl_client::utils::scheduler_uri();
        let client_tls = gl_client::tls::NOBODY_CONFIG.clone();
        let channel = Channel::from_shared(uri)?
            .tls_config(client_tls.clone())?
            .connect()
            .await?;
        Ok(SchedulerClient::new(channel))
    }
}
