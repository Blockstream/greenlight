use std::str::FromStr;

use anyhow::{anyhow, Result};
use bech32::{FromBase32, ToBase32};

use crate::lightning_invoice::Bolt11Invoice;

/// Decode an LNURL bech32 string into the underlying URL (LUD-01).
pub fn parse_lnurl(lnurl: &str) -> Result<String> {
    let (_hrp, data, _variant) =
        bech32::decode(lnurl).map_err(|e| anyhow!("Failed to decode lnurl: {}", e))?;

    let vec = Vec::<u8>::from_base32(&data)
        .map_err(|e| anyhow!("Failed to base32 decode data: {}", e))?;

    let url = String::from_utf8(vec).map_err(|e| anyhow!("Failed to convert to utf-8: {}", e))?;
    Ok(url)
}

/// Encode a URL as an LNURL bech32 string (LUD-01).
///
/// Returns uppercase by convention (for QR code compatibility).
pub fn lnurl_encode(url: &str) -> Result<String> {
    let data = url.as_bytes().to_base32();
    bech32::encode("lnurl", data, bech32::Variant::Bech32)
        .map(|s| s.to_uppercase())
        .map_err(|e| anyhow!("Failed to encode lnurl: {}", e))
}

/// Extract the "text/plain" description from LNURL metadata JSON.
///
/// Metadata is a JSON array of `["mime", "content"]` pairs.
/// Returns the content of the first "text/plain" entry, or None.
pub fn extract_description_from_metadata(metadata: &str) -> Option<String> {
    let entries: Vec<Vec<String>> = serde_json::from_str(metadata).ok()?;
    for entry in entries {
        if entry.len() >= 2 && entry[0] == "text/plain" {
            return Some(entry[1].clone());
        }
    }
    None
}

/// Parse a BOLT11 invoice string.
pub fn parse_invoice(invoice_str: &str) -> Result<Bolt11Invoice> {
    Bolt11Invoice::from_str(invoice_str)
        .map_err(|e| anyhow!(format!("Failed to parse invoice: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lnurl_encode_decode_roundtrip() {
        let url = "https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b7e86a0850";
        let encoded = lnurl_encode(url).unwrap();
        assert!(encoded.starts_with("LNURL1"));
        let decoded = parse_lnurl(&encoded).unwrap();
        assert_eq!(decoded, url);
    }

    #[test]
    fn test_lnurl_decode_is_case_insensitive() {
        let url = "https://example.com/lnurl";
        let encoded = lnurl_encode(url).unwrap();
        // Uppercase (default) should work
        let decoded = parse_lnurl(&encoded).unwrap();
        assert_eq!(decoded, url);
        // Lowercase should also work
        let decoded = parse_lnurl(&encoded.to_lowercase()).unwrap();
        assert_eq!(decoded, url);
    }

    #[test]
    fn test_extract_description_from_metadata() {
        let metadata = r#"[["text/plain", "Pay to example"]]"#;
        assert_eq!(
            extract_description_from_metadata(metadata),
            Some("Pay to example".to_string())
        );
    }

    #[test]
    fn test_extract_description_from_metadata_with_multiple_entries() {
        let metadata =
            r#"[["text/identifier", "user@example.com"], ["text/plain", "Pay user"]]"#;
        assert_eq!(
            extract_description_from_metadata(metadata),
            Some("Pay user".to_string())
        );
    }

    #[test]
    fn test_extract_description_from_metadata_missing() {
        let metadata = r#"[["text/identifier", "user@example.com"]]"#;
        assert_eq!(extract_description_from_metadata(metadata), None);
    }

    #[test]
    fn test_extract_description_from_metadata_invalid_json() {
        assert_eq!(extract_description_from_metadata("not json"), None);
    }
}
