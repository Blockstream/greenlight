/// A simple staging mechanism for incoming requests so we can invert from
/// pull to push. Used by `hsmproxy` to stage requests that can then
/// asynchronously be retrieved and processed by one or more client
/// devices.
use crate::pb;
use anyhow::{anyhow, Error};
use log::{debug, trace};
use std::collections;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{broadcast, mpsc, Mutex};

#[derive(Debug)]
pub struct Stage {
    requests: Mutex<collections::HashMap<u32, Request>>,
    notify: broadcast::Sender<Request>,
    hsm_connections: Arc<AtomicUsize>,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub request: pb::HsmRequest,
    pub response: mpsc::Sender<pb::HsmResponse>,
}

impl Stage {
    pub fn new() -> Self {
        let (notify, _) = broadcast::channel(1000);
        Stage {
            requests: Mutex::new(collections::HashMap::new()),
            notify: notify,
            hsm_connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn send(
        &self,
        request: pb::HsmRequest,
    ) -> Result<mpsc::Receiver<pb::HsmResponse>, Error> {
        let mut requests = self.requests.lock().await;
        let (sender, receiver): (
            mpsc::Sender<pb::HsmResponse>,
            mpsc::Receiver<pb::HsmResponse>,
        ) = mpsc::channel(1);
        let r = Request {
            request: request,
            response: sender,
        };

        requests.insert(r.request.request_id, r.clone());
        if let Err(e) = self.notify.send(r) {
            eprintln!("Error notifying hsmd request streams {:?}", e);
        }
        Ok(receiver)
    }

    pub async fn mystream(&self) -> StageStream {
        let requests = self.requests.lock().await;
        self.hsm_connections.fetch_add(1, Ordering::Relaxed);
        StageStream {
            backlog: requests.values().map(|e| e.clone()).collect(),
            bcast: self.notify.subscribe(),
            hsm_connections: self.hsm_connections.clone(),
        }
    }

    pub async fn respond(&self, response: pb::HsmResponse) -> Result<(), Error> {
        let mut requests = self.requests.lock().await;
        match requests.remove(&response.request_id) {
            Some(req) => {
                debug!(
                    "Response for request_id={}, outstanding requests count={}",
                    response.request_id,
                    requests.len()
                );
                if let Err(e) = req.response.send(response).await {
                    Err(anyhow!("Error sending request to requester: {:?}", e))
                } else {
                    Ok(())
                }
            }
            None => {
                trace!(
                    "Request {} not found, is this a duplicate result?",
                    response.request_id
                );
                Ok(())
            }
        }
    }

    pub async fn is_stuck(&self) -> bool {
        let sticky = self
            .requests
            .lock()
            .await
            .values()
            .filter(|r| r.request.raw[0..2] == [0u8, 5])
            .count();

        trace!("Found {sticky} sticky requests.");
        sticky != 0
    }
}

pub struct StageStream {
    backlog: Vec<Request>,
    bcast: broadcast::Receiver<Request>,
    hsm_connections: Arc<AtomicUsize>,
}

impl StageStream {
    pub async fn next(&mut self) -> Result<Request, Error> {
        if self.backlog.len() > 0 {
            let req = self.backlog[0].clone();
            self.backlog.remove(0);
            Ok(req)
        } else {
            match self.bcast.recv().await {
                Ok(r) => Ok(r),
                Err(e) => Err(anyhow!("error waiting for more requests: {:?}", e)),
            }
        }
    }
}

impl Drop for StageStream {
    fn drop(&mut self) {
        self.hsm_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep as delay_for;

    #[tokio::test]
    async fn test_live_stream() {
        let stage = Stage::new();

        let mut responses = vec![];

        for i in 0..10 {
            responses.push(
                stage
                    .send(pb::HsmRequest {
                        request_id: i,
                        context: None,
                        raw: vec![],
			signer_state: vec![],
                    })
                    .await
                    .unwrap(),
            );
        }

        let mut s1 = stage.mystream().await;
        let mut s2 = stage.mystream().await;
        let f1 = tokio::spawn(async move {
            while let Ok(r) = s1.next().await {
                eprintln!("hsmd {} is handling request {}", 1, r.request.request_id);
                match r
                    .response
                    .send(pb::HsmResponse {
                        request_id: r.request.request_id,
                        raw: vec![],
			signer_state: vec![],
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!("errror {:?}", e),
                }
                delay_for(Duration::from_millis(10)).await;
            }
        });
        let f2 = tokio::spawn(async move {
            while let Ok(r) = s2.next().await {
                eprintln!("hsmd {} is handling request {}", 2, r.request.request_id);
                match r
                    .response
                    .send(pb::HsmResponse {
                        request_id: r.request.request_id,
                        raw: vec![],
			signer_state: vec![],
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!("errror {:?}", e),
                }
                delay_for(Duration::from_millis(10)).await;
            }
        });

        for i in 10..100 {
            responses.push(
                stage
                    .send(pb::HsmRequest {
                        request_id: i,
                        context: None,
                        raw: vec![],
			signer_state: vec![],
                    })
                    .await
                    .unwrap(),
            );
        }

        for mut r in responses {
            let resp = r.recv().await.unwrap();
            eprintln!("Got response {:?}", resp);
        }

        drop(stage);
        f1.await.unwrap();
        f2.await.unwrap();
    }
}
