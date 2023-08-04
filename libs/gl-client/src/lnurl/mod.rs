mod models;
mod pay;
mod utils;

use anyhow::Result;
use models::LnUrlHttpClearnetClient;
use pay::resolve_to_invoice;

pub struct LNURL;

impl LNURL {
    pub async fn resolve_lnurl_to_invoice(&self, lnurl: &str, amount: u64) -> Result<String> {
        let http_client = LnUrlHttpClearnetClient::new();
        resolve_to_invoice(http_client, lnurl, amount).await
    }
}
