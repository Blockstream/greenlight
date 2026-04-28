pub mod models;
pub mod pay;
pub mod utils;
pub mod withdraw;

use self::models::{LnUrlHttpClient, PayRequestResponse, WithdrawRequestResponse};
use self::utils::parse_lnurl;
use crate::node::ClnClient;
use crate::pb::cln::{amount_or_any, Amount, AmountOrAny};
use anyhow::{anyhow, Result};
use models::LnUrlHttpClearnetClient;

/// Result of resolving an LNURL endpoint via HTTP.
pub enum LnUrlResponse {
    Pay(PayRequestResponse),
    Withdraw(WithdrawRequestResponse),
}

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

    /// Resolve an LNURL to its endpoint data with a single HTTP GET.
    ///
    /// Decodes the bech32, fetches the URL, inspects the `tag` field,
    /// and returns the appropriate typed response.
    pub async fn resolve(&self, url: &str) -> Result<LnUrlResponse> {
        let json = self.http_client.get_json(url).await?;

        let tag = json
            .get("tag")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        match tag {
            "payRequest" => {
                let response: PayRequestResponse = serde_json::from_value(json)
                    .map_err(|e| anyhow!("Failed to parse payRequest response: {}", e))?;
                Ok(LnUrlResponse::Pay(response))
            }
            "withdrawRequest" => {
                let response: WithdrawRequestResponse = serde_json::from_value(json)
                    .map_err(|e| anyhow!("Failed to parse withdrawRequest response: {}", e))?;
                Ok(LnUrlResponse::Withdraw(response))
            }
            _ => Err(anyhow!(
                "Unknown LNURL tag: '{}'. Expected 'payRequest' or 'withdrawRequest'.",
                tag
            )),
        }
    }

    pub async fn pay(
        &self,
        lnurl: &str,
        amount_msats: u64,
        node: &mut ClnClient,
    ) -> Result<tonic::Response<crate::pb::cln::PayResponse>> {
        let (invoice, _success_action) =
            pay::resolve_lnurl_to_invoice(&self.http_client, lnurl, amount_msats, None).await?;

        node.pay(crate::pb::cln::PayRequest {
            bolt11: invoice,
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow!(e))
    }

    pub async fn withdraw(
        &self,
        lnurl: &str,
        amount_msats: u64,
        node: &mut ClnClient,
    ) -> Result<()> {
        let url = parse_lnurl(lnurl)?;
        let withdrawal_request_response =
            withdraw::parse_withdraw_request_response_from_url(&url);

        let withdrawal_request_response = match withdrawal_request_response {
            Some(w) => w,
            None => {
                self.http_client
                    .get_withdrawal_request_response(&url)
                    .await?
            }
        };

        let amount = AmountOrAny {
            value: Some(amount_or_any::Value::Amount(Amount { msat: amount_msats })),
        };
        let invoice = node
            .invoice(crate::pb::cln::InvoiceRequest {
                amount_msat: Some(amount),
                description: withdrawal_request_response.default_description.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| anyhow!(e))?
            .into_inner();

        let callback_url =
            withdrawal_request_response.build_callback_url(&invoice.bolt11)?;

        let _ = self
            .http_client
            .send_invoice_for_withdraw_request(&callback_url);

        Ok(())
    }
}
