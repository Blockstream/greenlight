mod pay;
mod tor_http_client;

use anyhow::{Result};
use pay::resolve_to_invoice;
use pay::models::{LnUrlHttpClientImpl};

pub struct LNURL;

impl LNURL {
  pub fn resolve_lnurl_to_invoice(&self, lnurl: &str, amount: u64) -> Result<String> {
    let http_client = LnUrlHttpClientImpl::new();
    resolve_to_invoice(http_client, lnurl, amount)
  }
}