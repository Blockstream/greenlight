use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::debug;
use mockall::automock;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PayRequestResponse {
    pub callback: String,
    #[serde(rename = "maxSendable")]
    pub max_sendable: u64,
    #[serde(rename = "minSendable")]
    pub min_sendable: u64,
    pub tag: String,
    pub metadata: String,
    /// Maximum comment length the service accepts (LUD-12).
    /// None or 0 means comments are not supported.
    #[serde(rename = "commentAllowed")]
    #[serde(default)]
    pub comment_allowed: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PayRequestCallbackResponse {
    pub pr: String,
    pub routes: Vec<String>,
    /// Optional success action returned by the service (LUD-09).
    #[serde(rename = "successAction")]
    #[serde(default)]
    pub success_action: Option<SuccessAction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OkResponse {
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WithdrawRequestResponse {
    pub tag: String,
    pub callback: String,
    pub k1: String,
    #[serde(rename = "defaultDescription")]
    pub default_description: String,
    #[serde(rename = "minWithdrawable")]
    pub min_withdrawable: u64,
    #[serde(rename = "maxWithdrawable")]
    pub max_withdrawable: u64,
}

/// Raw success action from an LNURL-pay callback response (LUD-09/10).
///
/// Deserialized directly from the service's JSON. For the AES variant,
/// the ciphertext has not yet been decrypted -- use
/// [`process_success_action`] with the payment preimage to produce a
/// [`ProcessedSuccessAction`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum SuccessAction {
    #[serde(rename = "message")]
    Message { message: String },
    #[serde(rename = "url")]
    Url { description: String, url: String },
    #[serde(rename = "aes")]
    Aes {
        description: String,
        /// Base64-encoded ciphertext (max 4096 chars).
        ciphertext: String,
        /// Base64-encoded IV (24 chars = 16 bytes).
        iv: String,
    },
}

/// A success action after client-side processing.
///
/// For the Message and Url variants this is identical to the raw
/// [`SuccessAction`]. For AES the ciphertext has been decrypted into
/// plaintext using the payment preimage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessedSuccessAction {
    Message { message: String },
    Url { description: String, url: String },
    Aes { description: String, plaintext: String },
}

impl SuccessAction {
    /// Process this success action, decrypting AES content if needed.
    ///
    /// `preimage` is the 32-byte payment preimage from the PayResponse.
    /// For Message and Url variants this is a simple conversion; for Aes
    /// it decrypts the ciphertext using the preimage as the AES-256 key.
    pub fn process(self, preimage: &[u8]) -> Result<ProcessedSuccessAction> {
        match self {
            SuccessAction::Message { message } => {
                Ok(ProcessedSuccessAction::Message { message })
            }
            SuccessAction::Url { description, url } => {
                Ok(ProcessedSuccessAction::Url { description, url })
            }
            SuccessAction::Aes {
                description,
                ciphertext,
                iv,
            } => {
                let plaintext =
                    super::pay::decrypt_aes_success_action(preimage, &ciphertext, &iv)?;
                Ok(ProcessedSuccessAction::Aes {
                    description,
                    plaintext,
                })
            }
        }
    }
}

#[async_trait]
#[automock]
pub trait LnUrlHttpClient {
    async fn get_pay_request_response(&self, lnurl: &str) -> Result<PayRequestResponse>;
    async fn get_pay_request_callback_response(
        &self,
        callback_url: &str,
    ) -> Result<PayRequestCallbackResponse>;
    async fn get_withdrawal_request_response(&self, url: &str) -> Result<WithdrawRequestResponse>;
    async fn send_invoice_for_withdraw_request(&self, url: &str) -> Result<OkResponse>;
    async fn get_json(&self, url: &str) -> Result<serde_json::Value>;
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
    ) -> Result<PayRequestCallbackResponse> {
        self.get::<PayRequestCallbackResponse>(callback_url).await
    }

    async fn get_withdrawal_request_response(&self, url: &str) -> Result<WithdrawRequestResponse> {
        self.get::<WithdrawRequestResponse>(url).await
    }

    async fn send_invoice_for_withdraw_request(&self, url: &str) -> Result<OkResponse> {
        self.get::<OkResponse>(url).await
    }

    async fn get_json(&self, url: &str) -> Result<serde_json::Value> {
        self.get::<serde_json::Value>(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_action_message_serde() {
        let json = r#"{"tag":"message","message":"Thank you!"}"#;
        let action: SuccessAction = serde_json::from_str(json).unwrap();
        match action {
            SuccessAction::Message { message } => assert_eq!(message, "Thank you!"),
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_success_action_url_serde() {
        let json = r#"{"tag":"url","description":"View order","url":"https://example.com/order/123"}"#;
        let action: SuccessAction = serde_json::from_str(json).unwrap();
        match action {
            SuccessAction::Url { description, url } => {
                assert_eq!(description, "View order");
                assert_eq!(url, "https://example.com/order/123");
            }
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_success_action_aes_serde() {
        let json = r#"{"tag":"aes","description":"Secret","ciphertext":"YWJj","iv":"MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0"}"#;
        let action: SuccessAction = serde_json::from_str(json).unwrap();
        match action {
            SuccessAction::Aes {
                description,
                ciphertext,
                iv,
            } => {
                assert_eq!(description, "Secret");
                assert_eq!(ciphertext, "YWJj");
                assert_eq!(iv, "MTIzNDU2Nzg5MDEyMzQ1Njc4OTAxMjM0");
            }
            _ => panic!("Expected Aes variant"),
        }
    }

    #[test]
    fn test_callback_response_without_success_action() {
        let json = r#"{"pr":"lnbc1...","routes":[]}"#;
        let resp: PayRequestCallbackResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success_action.is_none());
    }

    #[test]
    fn test_callback_response_with_success_action() {
        let json =
            r#"{"pr":"lnbc1...","routes":[],"successAction":{"tag":"message","message":"Done"}}"#;
        let resp: PayRequestCallbackResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success_action.is_some());
    }

    #[test]
    fn test_pay_request_response_with_comment_allowed() {
        let json = r#"{"callback":"https://example.com/cb","maxSendable":100000,"minSendable":1000,"tag":"payRequest","metadata":"[]","commentAllowed":140}"#;
        let resp: PayRequestResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.comment_allowed, Some(140));
    }

    #[test]
    fn test_pay_request_response_without_comment_allowed() {
        let json = r#"{"callback":"https://example.com/cb","maxSendable":100000,"minSendable":1000,"tag":"payRequest","metadata":"[]"}"#;
        let resp: PayRequestResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.comment_allowed, None);
    }
}
