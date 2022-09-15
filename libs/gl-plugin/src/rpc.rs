use crate::{requests, responses};
use clightningrpc::{error::Error, Response};
use cln_rpc::codec::JsonCodec;
use futures::{SinkExt, StreamExt};
use log::{debug, error, warn};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Deserializer, Value};
use std::path::{Path, PathBuf};
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

#[derive(Clone, Debug)]
pub struct LightningClient {
    sockpath: PathBuf,
}

impl LightningClient {
    pub fn new<P: AsRef<Path>>(sockpath: P) -> LightningClient {
        LightningClient {
            sockpath: sockpath.as_ref().to_path_buf(),
        }
    }

    pub async fn send_request<S: Serialize, D: DeserializeOwned>(
        &self,
        method: &str,
        params: S,
    ) -> Result<Response<D>, Error> {
        // Setup connection
        let stream = UnixStream::connect(&self.sockpath).await?;
        let mut codec = Framed::new(stream, JsonCodec::default());

        // TODO Re-enable timeout for the socket
        //stream.set_read_timeout(self.timeout)?;
        //stream.set_write_timeout(self.timeout)?;

        let request = json!({
            "method": method,
            "params": params,
            "id": 0, // we always open a new connection, so we don't have to care about the nonce
            "jsonrpc": "2.0",
        });

        debug!(
            "Sending request to JSON-RPC: {}",
            serde_json::to_string(&request).unwrap()
        );

        if let Err(e) = codec.send(request).await {
            warn!("Error sending request to RPC interface: {}", e);
            return Err(Error::NonceMismatch);
        }

        let response = match codec.next().await {
            Some(Ok(v)) => v,
            Some(Err(e)) => {
                warn!("Error from RPC: {:?}", e);
                return Err(Error::NonceMismatch);
            }
            None => {
                warn!("Error reading response from RPC interface, returned None");
                return Err(Error::NonceMismatch);
            }
        };

        debug!(
            "Read response from JSON-RPC: {}",
            serde_json::to_string(&response).unwrap()
        );

        // TODO (cdecker) inefficient: serialize just to re-serialize,
        // but it's how I got it working.
        let response: Response<D> = Deserializer::from_str(&response.to_string())
            .into_iter()
            .next()
            .map_or(Err(Error::NoErrorOrResult), |res| Ok(res?))?;
        Ok(response)
    }

    /// Generic call function for RPC calls.
    pub async fn call<T: Serialize, U: DeserializeOwned>(
        &self,
        method: &str,
        input: T,
    ) -> Result<U, Error> {
        self.send_request(method, input)
            .await
            .and_then(|res| res.into_result())
    }

    /// Show information about this node.
    pub async fn getinfo(&self) -> Result<crate::responses::GetInfo, Error> {
        self.call("getinfo", json!({})).await
    }

    pub async fn stop(&self) -> Result<(), Error> {
        match self.call::<Value, ()>("stop", json!({})).await {
            Ok(()) => Ok(()),
            Err(e) => {
                debug!("Ignoring error on `stop` call: {}", e);
                Ok(())
            }
        }
    }

    pub async fn connect<'a>(
        &self,
        req: &requests::Connect<'a>,
    ) -> Result<responses::Connect, Error> {
        self.call("connect", req).await
    }

    pub async fn listpeers(
        &self,
        node_id: Option<&str>,
    ) -> Result<crate::responses::ListPeers, Error> {
        self.call(
            "listpeers",
            requests::ListPeers {
                id: node_id,
                level: None,
            },
        )
        .await
    }

    pub async fn disconnect(&self, node_id: &str, force: bool) -> Result<(), Error> {
        if force {
            error!("Force-disconnects are currently not supported");
            return Err(Error::NonceMismatch);
        }

        self.call::<requests::Disconnect, responses::Disconnect>(
            "disconnect",
            requests::Disconnect { id: node_id },
        )
        .await?;
        Ok(())
    }

    pub async fn newaddr(&self, typ: crate::pb::BtcAddressType) -> Result<String, Error> {
        use crate::pb::BtcAddressType;
        let addresstype = match typ {
            BtcAddressType::Bech32 => "bech32",
            BtcAddressType::P2shSegwit => "p2sh-segwit",
        };
        let res: responses::NewAddr = self
            .call(
                "newaddr",
                requests::NewAddr {
                    addresstype: Some(addresstype),
                },
            )
            .await?;

        let addr = match typ {
            BtcAddressType::Bech32 => res.bech32.unwrap(),
            BtcAddressType::P2shSegwit => res.p2sh_segwit.unwrap(),
        };

        Ok(addr)
    }

    pub async fn listincoming(&self) -> Result<crate::responses::ListIncoming, Error> {
        self.call("listincoming", crate::requests::ListIncoming {})
            .await
    }
}
