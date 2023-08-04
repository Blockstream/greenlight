use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::debug;
use mockall::automock;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
    status: String,
    reason: String,
}

#[async_trait]
#[automock]
pub trait LnUrlHttpClient {
    async fn get_pay_request_response(&self, lnurl: &str) -> Result<PayRequestResponse>;
    async fn get_pay_request_callback_response(
        &self,
        callback_url: &str,
        amount: u64,
    ) -> Result<PayRequestCallbackResponse>;
}

pub struct LnUrlHttpClearnetClient {
    client: reqwest::Client,
}

impl LnUrlHttpClearnetClient {
    pub fn new() -> LnUrlHttpClearnetClient {
        LnUrlHttpClearnetClient {
            client: reqwest::Client::new(),
        }
    }

    async fn get<T: DeserializeOwned + 'static>(&self, url: &str) -> Result<T> {
        let response: Response = self.client.get(url).send().await?;
        match response.json::<T>().await {
            Ok(body) => Ok(body),
            Err(e) => {
                debug!("{}", e);
                Err(anyhow!("Unable to parse http response body as json"))
            }
        }
    }
}

#[async_trait]
impl LnUrlHttpClient for LnUrlHttpClearnetClient {
    async fn get_pay_request_response(&self, lnurl: &str) -> Result<PayRequestResponse> {
        self.get::<PayRequestResponse>(lnurl).await
    }

    async fn get_pay_request_callback_response(
        &self,
        callback_url: &str,
        amount: u64,
    ) -> Result<PayRequestCallbackResponse> {
        self.get::<PayRequestCallbackResponse>(callback_url).await
    }
}
