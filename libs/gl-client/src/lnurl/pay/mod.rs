pub mod models;

use anyhow::{anyhow, ensure, Result};
use bech32::FromBase32;
use lightning_invoice::{Invoice, InvoiceDescription};
use log::debug;
use models::{
    LnUrlHttpClient, MockLnUrlHttpClient, PayRequestCallbackResponse, PayRequestResponse,
};
use reqwest::Url;
use sha256;
use std::str::FromStr;

pub fn resolve_to_invoice<T: LnUrlHttpClient>(
    http_client: T,
    lnurl: &str,
    amount: u64,
) -> Result<String> {
    let url = decode_and_parse_lnurl(lnurl)?;

    let lnurl_pay_request_response: PayRequestResponse =
        http_client.get_pay_request_response(&url)?;

    validate_pay_request_response(&lnurl_pay_request_response, amount)?;
    let description = extract_description(&lnurl_pay_request_response)?;

    debug!("Domain: {}", Url::parse(&url).unwrap().host().unwrap());
    debug!("Description: {}", description);
    debug!(
        "Accepted range (in millisatoshis): {} - {}",
        lnurl_pay_request_response.min_sendable, lnurl_pay_request_response.max_sendable
    );

    let callback_url = build_callback_url(&lnurl_pay_request_response, amount)?;
    let callback_response: PayRequestCallbackResponse =
        http_client.get_pay_request_callback_response(&callback_url, amount)?;

    let invoice = parse_invoice(&callback_response.pr)?;
    validate_invoice_from_callback_response(&invoice, amount, lnurl_pay_request_response)?;
    Ok(invoice.to_string())
}

// Get an Invoice from a Lightning Network URL pay request
fn parse_invoice(invoice_str: &str) -> Result<Invoice> {
    Invoice::from_str(&invoice_str).map_err(|e| anyhow!(format!("Failed to parse invoice: {}", e)))
}

// Function to decode and parse the lnurl into a URL
fn decode_and_parse_lnurl(lnurl: &str) -> Result<String> {
    let (_hrp, data, _variant) =
        bech32::decode(lnurl).map_err(|e| anyhow!("Failed to decode lnurl: {}", e))?;

    let vec = Vec::<u8>::from_base32(&data)
        .map_err(|e| anyhow!("Failed to base32 decode data: {}", e))?;

    let url = String::from_utf8(vec).map_err(|e| anyhow!("Failed to convert to utf-8: {}", e))?;
    Ok(url)
}

// Function to extract the description from the lnurl pay request response
fn extract_description(lnurl_pay_request_response: &PayRequestResponse) -> Result<String> {
    let mut description = String::new();

    let serialized_metadata = lnurl_pay_request_response.metadata.clone();

    let deserialized_metadata: Vec<Vec<String>> =
        serde_json::from_str(&serialized_metadata.to_owned())
            .map_err(|e| anyhow!("Failed to deserialize metadata: {}", e))?;

    for metadata in deserialized_metadata {
        if metadata[0] == "text/plain" {
            description = metadata[1].clone();
        }
    }

    ensure!(!description.is_empty(), "No description found");

    Ok(description)
}

// Function to build the callback URL based on lnurl pay request response and amount
fn build_callback_url(
    lnurl_pay_request_response: &models::PayRequestResponse,
    amount: u64,
) -> Result<String> {

    let mut url = Url::parse(&lnurl_pay_request_response.callback)?;
    url.query_pairs_mut().append_pair("amount", &amount.to_string());
    Ok(url.to_string())
}

// Validates the pay request response for expected values
fn validate_pay_request_response(
    lnurl_pay_request_response: &PayRequestResponse,
    amount: u64,
) -> Result<()> {
    if amount < lnurl_pay_request_response.min_sendable {
        return Err(anyhow!(
            "Amount must be {} or greater",
            lnurl_pay_request_response.min_sendable
        ));
    }

    if amount > lnurl_pay_request_response.max_sendable {
        return Err(anyhow!(
            "Amount must be {} or less",
            lnurl_pay_request_response.max_sendable
        ));
    }

    if lnurl_pay_request_response.tag != "payRequest" {
        return Err(anyhow!("Expected tag to say 'payRequest'"));
    }

    Ok(())
}

// Validates the invoice on the pay request's callback response
fn validate_invoice_from_callback_response(
    invoice: &Invoice,
    amount: u64,
    lnurl_pay_request: PayRequestResponse,
) -> Result<()> {
    ensure!(invoice.amount_milli_satoshis().unwrap_or_default() == amount ,
        format!("Amount found in invoice was not equal to the amount found in the original request\nRequest amount: {}\nInvoice amount:{:?}", amount, invoice.amount_milli_satoshis().unwrap())
    );

    let description_hash: String = match invoice.description() {
        InvoiceDescription::Direct(d) => sha256::digest(d.clone().into_inner()),
        InvoiceDescription::Hash(h) => h.0.to_string(),
    };

    ensure!(
        description_hash == sha256::digest(lnurl_pay_request.metadata),
        "description_hash does not match the hash of the metadata"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invoice() {
        let invoice_str = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";

        let result = parse_invoice(invoice_str);
        assert!(result.is_ok());

        let invoice = result.unwrap();
        assert_eq!(invoice.amount_milli_satoshis().unwrap(), 10);
    }

    #[test]
    fn test_lnurl_pay() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            Ok(x)
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url, _amount| {
            let invoice = "lnbc1u1pjv9qrvsp5e5wwexctzp9yklcrzx448c68q2a7kma55cm67ruajjwfkrswnqvqpp55x6mmz8ch6nahrcuxjsjvs23xkgt8eu748nukq463zhjcjk4s65shp5dd6hc533r655wtyz63jpf6ja08srn6rz6cjhwsjuyckrqwanhjtsxqzjccqpjrzjqw6lfdpjecp4d5t0gxk5khkrzfejjxyxtxg5exqsd95py6rhwwh72rpgrgqq3hcqqgqqqqlgqqqqqqgq9q9qxpqysgq95njz4sz6h7r2qh7txnevcrvg0jdsfpe72cecmjfka8mw5nvm7tydd0j34ps2u9q9h6v5u8h3vxs8jqq5fwehdda6a8qmpn93fm290cquhuc6r";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice).to_string();
            Ok(serde_json::from_str(&callback_response_json).unwrap())
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 100000;

        resolve_to_invoice(mock_http_client, lnurl, amount);
    }

    #[test]
    fn test_lnurl_pay_returns_error_on_invalid_lnurl() {
        let mock_http_client = MockLnUrlHttpClient::new();

        let lnurl = "LNURL1111111111111111111111111111111111111111111111111111111111111111111";
        let amount = 100000;

        let result = resolve_to_invoice(mock_http_client, lnurl, amount);

        match result {
            Err(err) => {
                assert!(err
                    .to_string()
                    .contains("Failed to decode lnurl: invalid length"));
            }
            _ => panic!("Expected an error, but got Ok"),
        }
    }

    #[test]
    fn test_lnurl_pay_returns_error_on_amount_less_than_min_sendable() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        // Set up expectations for the first two calls
        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100000, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            Ok(x)
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url, _amount| {
            let invoice = "lnbc1u1pjv9qrvsp5e5wwexctzp9yklcrzx448c68q2a7kma55cm67ruajjwfkrswnqvqpp55x6mmz8ch6nahrcuxjsjvs23xkgt8eu748nukq463zhjcjk4s65shp5dd6hc533r655wtyz63jpf6ja08srn6rz6cjhwsjuyckrqwanhjtsxqzjccqpjrzjqw6lfdpjecp4d5t0gxk5khkrzfejjxyxtxg5exqsd95py6rhwwh72rpgrgqq3hcqqgqqqqlgqqqqqqgq9q9qxpqysgq95njz4sz6h7r2qh7txnevcrvg0jdsfpe72cecmjfka8mw5nvm7tydd0j34ps2u9q9h6v5u8h3vxs8jqq5fwehdda6a8qmpn93fm290cquhuc6r";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice).to_string();
            Ok(serde_json::from_str(&callback_response_json).unwrap())
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 1;

        let result = resolve_to_invoice(mock_http_client, lnurl, amount);

        match result {
            Err(err) => {
                assert!(err.to_string().contains("Amount must be"));
            }
            _ => panic!("Expected an error, but got Ok"),
        }
    }

    #[test]
    fn test_lnurl_pay_returns_error_on_amount_greater_than_max_sendable() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100000, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            Ok(x)
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url, _amount| {
            let invoice = "lnbc1u1pjv9qrvsp5e5wwexctzp9yklcrzx448c68q2a7kma55cm67ruajjwfkrswnqvqpp55x6mmz8ch6nahrcuxjsjvs23xkgt8eu748nukq463zhjcjk4s65shp5dd6hc533r655wtyz63jpf6ja08srn6rz6cjhwsjuyckrqwanhjtsxqzjccqpjrzjqw6lfdpjecp4d5t0gxk5khkrzfejjxyxtxg5exqsd95py6rhwwh72rpgrgqq3hcqqgqqqqlgqqqqqqgq9q9qxpqysgq95njz4sz6h7r2qh7txnevcrvg0jdsfpe72cecmjfka8mw5nvm7tydd0j34ps2u9q9h6v5u8h3vxs8jqq5fwehdda6a8qmpn93fm290cquhuc6r";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice).to_string();
            Ok(serde_json::from_str(&callback_response_json).unwrap())
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 1;

        let result = resolve_to_invoice(mock_http_client, lnurl, amount);

        match result {
            Err(err) => {
                assert!(err.to_string().contains("Amount must be"));
            }
            _ => panic!("Expected an error, amount specified is greater than maxSendable"),
        }
    }
}
