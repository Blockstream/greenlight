use super::models;
use super::utils::parse_lnurl;
use crate::lnurl::{
    models::{LnUrlHttpClient, PayRequestCallbackResponse, PayRequestResponse},
    utils::parse_invoice,
};
use anyhow::{anyhow, ensure, Result};
use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription};
use log::debug;
use reqwest::Url;
use sha256;

pub async fn resolve_lnurl_to_invoice<T: LnUrlHttpClient>(
    http_client: &T,
    lnurl: &str,
    amount_msats: u64,
) -> Result<String> {
    let url = parse_lnurl(lnurl)?;

    let lnurl_pay_request_response: PayRequestResponse =
        http_client.get_pay_request_response(&url).await?;

    if lnurl_pay_request_response.tag != "payRequest" {
        return Err(anyhow!("Expected tag to say 'payRequest'"));
    }

    ensure_amount_is_within_range(&lnurl_pay_request_response, amount_msats)?;
    let description = extract_description(&lnurl_pay_request_response)?;

    debug!("Domain: {}", Url::parse(&url).unwrap().host().unwrap());
    debug!("Description: {}", description);
    debug!(
        "Accepted range (in millisatoshis): {} - {}",
        lnurl_pay_request_response.min_sendable, lnurl_pay_request_response.max_sendable
    );

    let callback_url = build_callback_url(&lnurl_pay_request_response, amount_msats)?;
    let callback_response: PayRequestCallbackResponse = http_client
        .get_pay_request_callback_response(&callback_url)
        .await?;

    let invoice = parse_invoice(&callback_response.pr)?;
    validate_invoice_from_callback_response(
        &invoice,
        amount_msats,
        &lnurl_pay_request_response.metadata,
    )?;
    Ok(invoice.to_string())
}

// Validates the invoice on the pay request's callback response
pub fn validate_invoice_from_callback_response(
    invoice: &Bolt11Invoice,
    amount_msats: u64,
    metadata: &str,
) -> Result<()> {
    ensure!(invoice.amount_milli_satoshis().unwrap_or_default() == amount_msats ,
        "Amount found in invoice was not equal to the amount found in the original request\nRequest amount: {}\nInvoice amount:{:?}", amount_msats, invoice.amount_milli_satoshis().unwrap()
    );

    let description_hash: String = match invoice.description() {
        Bolt11InvoiceDescription::Direct(d) => sha256::digest(d.clone().into_inner()),
        Bolt11InvoiceDescription::Hash(h) => h.0.to_string(),
    };

    ensure!(
        description_hash == sha256::digest(metadata),
        "description_hash does not match the hash of the metadata"
    );

    Ok(())
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

    Ok(description)
}

// Function to build the callback URL based on lnurl pay request response and amount
fn build_callback_url(
    lnurl_pay_request_response: &models::PayRequestResponse,
    amount: u64,
) -> Result<String> {
    let mut url = Url::parse(&lnurl_pay_request_response.callback)?;
    url.query_pairs_mut()
        .append_pair("amount", &amount.to_string());
    Ok(url.to_string())
}

// Validates the pay request response for expected values
fn ensure_amount_is_within_range(
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lnurl::models::MockLnUrlHttpClient;
    use futures::future;
    use futures::future::Ready;
    use std::pin::Pin;

    use super::*;

    fn convert_to_async_return_value<T: Send + 'static>(
        value: T,
    ) -> Pin<Box<dyn std::future::Future<Output = T> + Send>> {
        let ready_future: Ready<_> = future::ready(value);
        Pin::new(Box::new(ready_future)) as Pin<Box<dyn std::future::Future<Output = T> + Send>>
    }

    #[test]
    fn test_parse_invoice() {
        let invoice_str = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";

        let result = parse_invoice(invoice_str);
        assert!(result.is_ok());

        let invoice = result.unwrap();
        assert_eq!(invoice.amount_milli_satoshis().unwrap(), 10);
    }

    #[tokio::test]
    async fn test_lnurl_pay() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            convert_to_async_return_value(Ok(x))
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url| {
            let invoice = "lnbc1u1pjv9qrvsp5e5wwexctzp9yklcrzx448c68q2a7kma55cm67ruajjwfkrswnqvqpp55x6mmz8ch6nahrcuxjsjvs23xkgt8eu748nukq463zhjcjk4s65shp5dd6hc533r655wtyz63jpf6ja08srn6rz6cjhwsjuyckrqwanhjtsxqzjccqpjrzjqw6lfdpjecp4d5t0gxk5khkrzfejjxyxtxg5exqsd95py6rhwwh72rpgrgqq3hcqqgqqqqlgqqqqqqgq9q9qxpqysgq95njz4sz6h7r2qh7txnevcrvg0jdsfpe72cecmjfka8mw5nvm7tydd0j34ps2u9q9h6v5u8h3vxs8jqq5fwehdda6a8qmpn93fm290cquhuc6r";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice).to_string();
            let x = serde_json::from_str(&callback_response_json).unwrap();
            convert_to_async_return_value(Ok(x))
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 100000;

        let _ = resolve_lnurl_to_invoice(&mock_http_client, lnurl, amount).await;
    }

    #[tokio::test]
    async fn test_lnurl_pay_returns_error_on_invalid_lnurl() {
        let mock_http_client = MockLnUrlHttpClient::new();

        let lnurl = "LNURL1111111111111111111111111111111111111111111111111111111111111111111";
        let amount = 100000;

        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, amount).await;

        match result {
            Err(err) => {
                assert!(err
                    .to_string()
                    .contains("Failed to decode lnurl: invalid length"));
            }
            _ => panic!("Expected an error, but got Ok"),
        }
    }

    #[tokio::test]
    async fn test_lnurl_pay_returns_error_on_amount_less_than_min_sendable() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        // Set up expectations for the first two calls
        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100000, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            convert_to_async_return_value(Ok(x))
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url| {
            let invoice = "lnbc1u1pjv9qrvsp5e5wwexctzp9yklcrzx448c68q2a7kma55cm67ruajjwfkrswnqvqpp55x6mmz8ch6nahrcuxjsjvs23xkgt8eu748nukq463zhjcjk4s65shp5dd6hc533r655wtyz63jpf6ja08srn6rz6cjhwsjuyckrqwanhjtsxqzjccqpjrzjqw6lfdpjecp4d5t0gxk5khkrzfejjxyxtxg5exqsd95py6rhwwh72rpgrgqq3hcqqgqqqqlgqqqqqqgq9q9qxpqysgq95njz4sz6h7r2qh7txnevcrvg0jdsfpe72cecmjfka8mw5nvm7tydd0j34ps2u9q9h6v5u8h3vxs8jqq5fwehdda6a8qmpn93fm290cquhuc6r";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice).to_string();
            let callback_response = serde_json::from_str(&callback_response_json).unwrap();
            convert_to_async_return_value(Ok(callback_response))
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 1;

        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, amount).await;

        match result {
            Err(err) => {
                assert!(err.to_string().contains("Amount must be"));
            }
            _ => panic!("Expected an error, but got Ok"),
        }
    }

    #[tokio::test]
    async fn test_lnurl_pay_returns_error_on_amount_greater_than_max_sendable() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100000, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            convert_to_async_return_value(Ok(x))
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url| {
            let invoice = "lnbc1u1pjv9qrvsp5e5wwexctzp9yklcrzx448c68q2a7kma55cm67ruajjwfkrswnqvqpp55x6mmz8ch6nahrcuxjsjvs23xkgt8eu748nukq463zhjcjk4s65shp5dd6hc533r655wtyz63jpf6ja08srn6rz6cjhwsjuyckrqwanhjtsxqzjccqpjrzjqw6lfdpjecp4d5t0gxk5khkrzfejjxyxtxg5exqsd95py6rhwwh72rpgrgqq3hcqqgqqqqlgqqqqqqgq9q9qxpqysgq95njz4sz6h7r2qh7txnevcrvg0jdsfpe72cecmjfka8mw5nvm7tydd0j34ps2u9q9h6v5u8h3vxs8jqq5fwehdda6a8qmpn93fm290cquhuc6r";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice).to_string();
            let value = serde_json::from_str(&callback_response_json).unwrap();
            convert_to_async_return_value(Ok(value))
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 1;

        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, amount).await;

        match result {
            Err(err) => {
                assert!(err.to_string().contains("Amount must be"));
            }
            _ => panic!("Expected an error, amount specified is greater than maxSendable"),
        }
    }
}
