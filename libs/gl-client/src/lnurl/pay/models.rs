use crate::lnurl::tor_http_client::TorHttpClient;
use anyhow::{anyhow, Result};
use mockall::automock;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Debug)]
pub struct PayRequestResponse {
    pub callback: String,
    #[serde(rename = "maxSendable")]
    pub max_sendable: u64,
    #[serde(rename = "minSendable")]
    pub min_sendable: u64,
    pub tag: String,
    pub metadata: String,
}

#[derive(Deserialize)]
pub struct PayRequestCallbackResponse {
    pub pr: String,
    pub routes: Vec<String>,
}

#[automock]
pub trait LnUrlHttpClient {
    fn get_pay_request_response(&self, lnurl: &str) -> Result<PayRequestResponse>;
    fn get_pay_request_callback_response(
        &self,
        callback_url: &str,
        amount: u64,
    ) -> Result<PayRequestCallbackResponse>;
}

pub struct LnUrlHttpClientImpl {
    clearnet_client: reqwest::Client,
    tor_client: TorHttpClient,
}

impl LnUrlHttpClientImpl {
    /// Constructs a new `LnUrlHttpClient`.
    ///
    /// # Panics
    ///
    /// This method panics if a http or tor client cannot be initialized.
    pub fn new() -> LnUrlHttpClientImpl {
        let tokio_runtime = Runtime::new().expect("Failed to create Tokio runtime");
        let tor_client = tokio_runtime.block_on(TorHttpClient::new()).unwrap();
        LnUrlHttpClientImpl {
            clearnet_client: reqwest::Client::new(),
            tor_client,
        }
    }

    fn get<T: DeserializeOwned + 'static>(&self, url: &str) -> Result<T> {
        // Create a new Tokio runtime
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        // Use the block_on function to synchronously wait for the future
        let parsed_json_response = runtime
            .block_on(self.clearnet_client.get(url).send())?
            .json::<T>();

        match runtime.block_on(parsed_json_response) {
            Err(e) => Err(anyhow!(
                "Could not decode PEM string into PKCS#8 format: {}",
                e
            )),
            Ok(e) => Ok(e),
        }
    }
}

impl LnUrlHttpClient for LnUrlHttpClientImpl {
    fn get_pay_request_response(&self, lnurl: &str) -> Result<PayRequestResponse> {
        Ok(self.get::<PayRequestResponse>(lnurl).unwrap())
    }

    fn get_pay_request_callback_response(
        &self,
        callback_url: &str,
        amount: u64,
    ) -> Result<PayRequestCallbackResponse> {
        Ok(self
            .get::<PayRequestCallbackResponse>(callback_url)
            .unwrap())
    }
}
