mod models;
mod pay;
mod utils;

use crate::node::Client;
use crate::pb::{PayRequest, Payment};
use anyhow::{anyhow, Result};
use models::LnUrlHttpClearnetClient;
use pay::resolve_lnurl_to_invoice;
use url::Url;

use self::models::{LnUrlHttpClient, PayRequestCallbackResponse, PayRequestResponse};
use self::pay::validate_invoice_from_callback_response;
use self::utils::{parse_invoice, parse_lnurl};

pub struct LNURL<T: LnUrlHttpClient> {
    http_client: T,
}

impl<T: LnUrlHttpClient> LNURL<T> {
    pub fn new(http_client: T) -> Self {
        LNURL { http_client }
    }

    pub fn new_with_clearnet_client() -> LNURL<LnUrlHttpClearnetClient> {
        let http_client = LnUrlHttpClearnetClient::new();
        LNURL { http_client }
    }

    pub async fn get_pay_request_response(&self, lnurl: &str) -> Result<PayRequestResponse> {
        let url = parse_lnurl(lnurl)?;

        let lnurl_pay_request_response: PayRequestResponse =
            self.http_client.get_pay_request_response(&url).await?;

        if lnurl_pay_request_response.tag != "payRequest" {
            return Err(anyhow!("Expected tag to say 'payRequest'"));
        }

        Ok(lnurl_pay_request_response)
    }

    pub async fn get_pay_request_callback_response(
        &self,
        base_callback_url: &str,
        amount_msats: u64,
        metadata: &str,
    ) -> Result<PayRequestCallbackResponse> {
        let mut url = Url::parse(base_callback_url)?;
        url.query_pairs_mut()
            .append_pair("amount", &amount_msats.to_string());

        let callback_response: PayRequestCallbackResponse = self
            .http_client
            .get_pay_request_callback_response(&url.to_string())
            .await?;

        let invoice = parse_invoice(&callback_response.pr)?;
        validate_invoice_from_callback_response(&invoice, amount_msats, metadata)?;
        Ok(callback_response)
    }

    pub async fn pay(
        &self,
        lnurl: &str,
        amount_msats: u64,
        node: &mut Client,
    ) -> Result<tonic::Response<Payment>> {
        let invoice = resolve_lnurl_to_invoice(&self.http_client, lnurl, amount_msats).await?;

        node.pay(PayRequest {
            bolt11: invoice.to_string(),
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow!(e))
    }
}
