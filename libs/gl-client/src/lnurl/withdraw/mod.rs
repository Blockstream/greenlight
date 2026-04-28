use super::models::WithdrawRequestResponse;
use anyhow::Result;
use log::debug;
use reqwest::Url;
use serde_json::{to_value, Map, Value};

impl WithdrawRequestResponse {
    /// Build the callback URL for submitting an invoice to the service.
    ///
    /// Appends `k1` and `pr` (the BOLT11 invoice) as query parameters.
    pub fn build_callback_url(&self, invoice: &str) -> Result<String> {
        build_withdraw_callback_url(&self.callback, &self.k1, invoice)
    }
}

/// Build a withdraw callback URL from its individual components.
///
/// Appends `k1` and `pr` (the BOLT11 invoice) as query parameters
/// to the callback base URL.
pub fn build_withdraw_callback_url(callback: &str, k1: &str, invoice: &str) -> Result<String> {
    let mut url = Url::parse(callback)?;
    url.query_pairs_mut()
        .append_pair("k1", k1)
        .append_pair("pr", invoice);
    Ok(url.to_string())
}

fn convert_value_field_from_str_to_u64(
    value: &mut Map<String, Value>,
    field_name: &str,
) -> Result<()> {
    match value.get(field_name) {
        Some(field_value) => match field_value.as_str() {
            Some(field_value_str) => {
                let converted_field_value = field_value_str.parse::<u64>()?;
                value.insert(
                    String::from(field_name),
                    to_value(converted_field_value).unwrap(),
                );
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Failed to convert {} into a str",
                field_name
            )),
        },
        None => Err(anyhow::anyhow!("Failed to find {} in map", field_name)),
    }
}

pub fn parse_withdraw_request_response_from_url(url: &str) -> Option<WithdrawRequestResponse> {
    let url = Url::parse(url).unwrap();
    let query_params: Value = url.query_pairs().clone().collect();

    if let Some(mut query_params) = query_params.as_object().cloned() {
        if convert_value_field_from_str_to_u64(&mut query_params, "minWithdrawable").is_err() {
            debug!("minWithdrawable could not be parsed into a number");
            return None;
        };

        if convert_value_field_from_str_to_u64(&mut query_params, "maxWithdrawable").is_err() {
            debug!("maxWithdrawable could not be parsed into a number");
            return None;
        };

        match serde_json::from_value(Value::Object(query_params)) {
            Ok(w) => {
                return w;
            }
            Err(e) => {
                debug!("{:?}", e);
                return None;
            }
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_withdraw_request_callback_url() -> Result<()> {
        let resp = WithdrawRequestResponse {
            tag: String::from("withdraw"),
            callback: String::from("https://cipherpunk.com/"),
            k1: String::from("unique"),
            default_description: String::from(""),
            min_withdrawable: 2,
            max_withdrawable: 300,
        };

        let url_str = resp.build_callback_url("invoice")?;
        let url = Url::parse(&url_str)?;
        let query_pairs = url.query_pairs().collect::<Value>();
        let query_params: &Map<String, Value> = query_pairs.as_object().unwrap();

        assert_eq!(query_params.get("k1").unwrap().as_str().unwrap(), "unique");
        assert_eq!(
            query_params.get("pr").unwrap().as_str().unwrap(),
            "invoice"
        );

        Ok(())
    }

    #[test]
    fn test_parse_withdraw_request_response_from_url() {
        let withdraw_request = parse_withdraw_request_response_from_url("https://cipherpunk.com?tag=withdraw&callback=cipherpunk.com&k1=42&minWithdrawable=1&maxWithdrawable=100&defaultDescription=");
        assert!(withdraw_request.is_some());
    }

    #[test]
    fn test_parse_withdraw_request_response_from_url_fails_when_field_is_missing() {
        let withdraw_request = parse_withdraw_request_response_from_url("https://cipherpunk.com?tag=withdraw&callback=cipherpunk.com&k1=42&minWithdrawable=1&maxWithdrawable=100");
        assert!(withdraw_request.is_none());
    }

    #[test]
    fn test_parse_withdraw_request_response_from_url_fails_when_min_withdrawable_is_wrong_type() {
        let withdraw_request = parse_withdraw_request_response_from_url("https://cipherpunk.com?tag=withdraw&callback=cipherpunk.com&k1=42&minWithdrawable=one&maxWithdrawable=100&defaultDescription=");
        assert!(withdraw_request.is_none());
    }
}
