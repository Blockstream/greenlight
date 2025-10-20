mod models;
mod pay;
mod utils;
mod withdraw;

use self::models::{
    LnUrlHttpClient, PayRequestCallbackResponse, PayRequestResponse, WithdrawRequestResponse,
};
use self::utils::{parse_invoice, parse_lnurl};
use crate::node::ClnClient;
use crate::pb::cln::{amount_or_any, Amount, AmountOrAny};
use anyhow::{anyhow, Result};
use models::LnUrlHttpClearnetClient;
use pay::{resolve_lnurl_to_invoice, validate_invoice_from_callback_response};
use url::Url;
use withdraw::{build_withdraw_request_callback_url, parse_withdraw_request_response_from_url};

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
        node: &mut ClnClient,
    ) -> Result<tonic::Response<crate::pb::cln::PayResponse>> {
        let invoice = resolve_lnurl_to_invoice(&self.http_client, lnurl, amount_msats).await?;

        node.pay(crate::pb::cln::PayRequest {
            bolt11: invoice.to_string(),
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow!(e))
    }

    pub async fn get_withdraw_request_response(
        &self,
        lnurl: &str,
    ) -> Result<WithdrawRequestResponse> {
        let url = parse_lnurl(lnurl)?;
        let withdrawal_request_response = parse_withdraw_request_response_from_url(&url);

        //If it's not a quick withdraw, then get the withdrawal_request_response from the web.
        let withdrawal_request_response = match withdrawal_request_response {
            Some(w) => w,
            None => {
                self.http_client
                    .get_withdrawal_request_response(&url)
                    .await?
            }
        };

        Ok(withdrawal_request_response)
    }

    pub async fn withdraw(
        &self,
        lnurl: &str,
        amount_msats: u64,
        node: &mut ClnClient,
    ) -> Result<()> {
        let withdraw_request_response = self.get_withdraw_request_response(lnurl).await?;

        let amount = AmountOrAny {
            value: Some(amount_or_any::Value::Amount(Amount { msat: amount_msats })),
        };
        let invoice = node
            .invoice(crate::pb::cln::InvoiceRequest {
                amount_msat: Some(amount),
                description: withdraw_request_response.default_description.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow!(e))?
            .into_inner();

        let callback_url =
            build_withdraw_request_callback_url(&withdraw_request_response, invoice.bolt11)?;

        let _ = self
            .http_client
            .send_invoice_for_withdraw_request(&callback_url);

        Ok(())
    }
}
