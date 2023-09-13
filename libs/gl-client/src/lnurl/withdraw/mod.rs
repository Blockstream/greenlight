use super::models::WithdrawRequestResponse;
use anyhow::{anyhow, Result};
use log::debug;
use reqwest::Url;
use serde_json::{to_value, Map, Value};

pub fn build_withdraw_request_callback_url(
    lnurl_pay_request_response: &WithdrawRequestResponse,
    invoice: String,
) -> Result<String> {
    let mut url = Url::parse(&lnurl_pay_request_response.callback)?;
    url.query_pairs_mut()
        .append_pair("k1", &lnurl_pay_request_response.k1)
        .append_pair("pr", &invoice);

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

                //overwrites old type value
                value.insert(
                    String::from(field_name),
                    to_value(converted_field_value).unwrap(),
                );
                return Ok(());
            }
            None => return Err(anyhow!("Failed to convert {} into a str", field_name)),
        },
        None => return Err(anyhow!("Failed to find {} in map", field_name)),
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
            },
            Err(e) => {
                debug!("{:?}", e);
                return None;
            }
        }
    }

    None
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_build_withdraw_request_callback_url() -> Result<()> {

        let k1 =  String::from("unique");
        let invoice = String::from("invoice");

        let built_withdraw_request_callback_url = build_withdraw_request_callback_url(&WithdrawRequestResponse { 
            tag: String::from("withdraw"), 
            callback: String::from("https://cipherpunk.com/"), 
            k1: k1.clone(), 
            default_description: String::from(""), 
            min_withdrawable: 2, 
            max_withdrawable: 300, 
        }, invoice.clone());

        let url = Url::parse(&built_withdraw_request_callback_url.unwrap())?;
        let query_pairs = url.query_pairs().collect::<Value>();
        let query_params: &Map<String, Value> = query_pairs.as_object().unwrap();
        
       assert_eq!(query_params.get("k1").unwrap().as_str().unwrap(), k1);
       assert_eq!(query_params.get("pr").unwrap().as_str().unwrap(), invoice);

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
