use std::str::FromStr;

use anyhow::{anyhow, Result};
use bech32::FromBase32;

use crate::lightning_invoice::Bolt11Invoice;

// Function to decode and parse the lnurl into a URL
pub fn parse_lnurl(lnurl: &str) -> Result<String> {
    let (_hrp, data, _variant) =
        bech32::decode(lnurl).map_err(|e| anyhow!("Failed to decode lnurl: {}", e))?;

    let vec = Vec::<u8>::from_base32(&data)
        .map_err(|e| anyhow!("Failed to base32 decode data: {}", e))?;

    let url = String::from_utf8(vec).map_err(|e| anyhow!("Failed to convert to utf-8: {}", e))?;
    Ok(url)
}

// Get an Invoice from a Lightning Network URL pay request
pub fn parse_invoice(invoice_str: &str) -> Result<Bolt11Invoice> {
    Bolt11Invoice::from_str(&invoice_str).map_err(|e| anyhow!(format!("Failed to parse invoice: {}", e)))
}
