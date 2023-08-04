use anyhow::{anyhow, Result};
use bech32::FromBase32;

// Function to decode and parse the lnurl into a URL
pub fn parse_lnurl(lnurl: &str) -> Result<String> {
    let (_hrp, data, _variant) =
        bech32::decode(lnurl).map_err(|e| anyhow!("Failed to decode lnurl: {}", e))?;

    let vec = Vec::<u8>::from_base32(&data)
        .map_err(|e| anyhow!("Failed to base32 decode data: {}", e))?;

    let url = String::from_utf8(vec).map_err(|e| anyhow!("Failed to convert to utf-8: {}", e))?;
    Ok(url)
}
