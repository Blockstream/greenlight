use super::models::SuccessAction;
use super::utils::parse_lnurl;

use crate::lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescriptionRef};
use crate::lnurl::{
    models::{LnUrlHttpClient, PayRequestCallbackResponse, PayRequestResponse},
    utils::parse_invoice,
};

use anyhow::{anyhow, ensure, Result};
use log::debug;
use reqwest::Url;
use sha256;

impl PayRequestResponse {
    /// Extract the "text/plain" description from the metadata JSON.
    pub fn description(&self) -> Option<String> {
        super::utils::extract_description_from_metadata(&self.metadata)
    }

    /// Validate this pay request response for a given amount.
    ///
    /// Checks the tag, amount range, and — for lightning addresses —
    /// that the metadata contains a matching identifier.
    pub fn validate(&self, identifier: &str, amount_msats: u64) -> Result<()> {
        if self.tag != "payRequest" {
            return Err(anyhow!("Expected tag to say 'payRequest'"));
        }

        if amount_msats < self.min_sendable {
            return Err(anyhow!(
                "Amount must be {} or greater",
                self.min_sendable
            ));
        }
        if amount_msats > self.max_sendable {
            return Err(anyhow!(
                "Amount must be {} or less",
                self.max_sendable
            ));
        }

        debug!(
            "Accepted range (in millisatoshis): {} - {}",
            self.min_sendable, self.max_sendable
        );

        // For lightning addresses, verify the identifier appears in metadata
        if !is_lnurl(identifier) {
            let entries: Vec<Vec<String>> =
                serde_json::from_str(&self.metadata)
                    .map_err(|e| anyhow!("Failed to deserialize metadata: {}", e))?;

            let found = entries.iter().any(|entry| {
                entry.len() >= 2
                    && (entry[0] == "text/email" || entry[0] == "text/identifier")
                    && entry[1] == identifier
            });

            if !found {
                return Err(anyhow!(
                    "The lightning address specified in the original request \
                     does not match what was found in the metadata array"
                ));
            }
        }

        Ok(())
    }

    /// Fetch an invoice from this pay request's callback endpoint.
    ///
    /// Builds the callback URL with the given amount and optional comment,
    /// fetches the invoice, validates it against the metadata, and returns
    /// the invoice string along with any success action.
    pub async fn get_invoice<T: LnUrlHttpClient>(
        &self,
        http_client: &T,
        amount_msats: u64,
        comment: Option<&str>,
    ) -> Result<(String, Option<SuccessAction>)> {
        fetch_invoice(http_client, &self.callback, amount_msats, &self.metadata, comment).await
    }
}

/// Fetch an invoice from a pay-request callback URL.
///
/// This is the "phase 2" of the two-phase LNURL-pay flow: the caller
/// already has the callback URL and metadata from the initial
/// `payRequest` response, and now requests an invoice for a specific
/// amount.  The returned invoice is validated against the expected
/// amount and metadata hash before being returned.
pub async fn fetch_invoice<T: LnUrlHttpClient>(
    http_client: &T,
    callback: &str,
    amount_msats: u64,
    metadata: &str,
    comment: Option<&str>,
) -> Result<(String, Option<SuccessAction>)> {
    let callback_url = build_callback_url(callback, amount_msats, comment)?;
    let callback_response: PayRequestCallbackResponse = http_client
        .get_pay_request_callback_response(&callback_url)
        .await?;

    let invoice = parse_invoice(&callback_response.pr)?;
    validate_invoice(&invoice, amount_msats, metadata)?;
    Ok((invoice.to_string(), callback_response.success_action))
}

/// Build a callback URL with amount and optional comment query parameters.
fn build_callback_url(
    callback: &str,
    amount: u64,
    comment: Option<&str>,
) -> Result<String> {
    let mut url = Url::parse(callback)?;
    url.query_pairs_mut()
        .append_pair("amount", &amount.to_string());
    if let Some(c) = comment {
        url.query_pairs_mut().append_pair("comment", c);
    }
    Ok(url.to_string())
}

/// Validate a BOLT11 invoice against the expected amount and metadata.
fn validate_invoice(
    invoice: &Bolt11Invoice,
    amount_msats: u64,
    metadata: &str,
) -> Result<()> {
    ensure!(
        invoice.amount_milli_satoshis().unwrap_or_default() == amount_msats,
        "Amount found in invoice was not equal to the amount found in the original request\n\
         Request amount: {}\nInvoice amount: {:?}",
        amount_msats,
        invoice.amount_milli_satoshis()
    );

    let description_hash: String = match invoice.description() {
        Bolt11InvoiceDescriptionRef::Direct(d) => sha256::digest(d.to_string()),
        Bolt11InvoiceDescriptionRef::Hash(h) => h.0.to_string(),
    };

    ensure!(
        description_hash == sha256::digest(metadata),
        "description_hash {} does not match the hash of the metadata {}",
        description_hash,
        sha256::digest(metadata)
    );

    Ok(())
}

/// Decrypt an AES-256-CBC encrypted success action payload (LUD-10).
///
/// - `preimage`: 32-byte payment preimage (used as the AES key)
/// - `ciphertext_b64`: base64-encoded ciphertext
/// - `iv_b64`: base64-encoded IV (decodes to 16 bytes)
pub fn decrypt_aes_success_action(
    preimage: &[u8],
    ciphertext_b64: &str,
    iv_b64: &str,
) -> Result<String> {
    use aes::Aes256;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};

    let ciphertext = STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| anyhow!("Invalid base64 ciphertext: {}", e))?;
    let iv = STANDARD
        .decode(iv_b64)
        .map_err(|e| anyhow!("Invalid base64 IV: {}", e))?;

    if preimage.len() != 32 {
        return Err(anyhow!(
            "Payment preimage must be 32 bytes, got {}",
            preimage.len()
        ));
    }
    if iv.len() != 16 {
        return Err(anyhow!("IV must be 16 bytes, got {}", iv.len()));
    }

    type Aes256CbcDec = cbc::Decryptor<Aes256>;
    let decryptor = Aes256CbcDec::new_from_slices(preimage, &iv)
        .map_err(|e| anyhow!("AES init failed: {}", e))?;

    let plaintext_bytes = decryptor
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| anyhow!("AES decryption failed: {}", e))?;

    String::from_utf8(plaintext_bytes)
        .map_err(|e| anyhow!("Decrypted data is not valid UTF-8: {}", e))
}

fn is_lnurl(lnurl_identifier: &str) -> bool {
    const LNURL_PREFIX: &str = "LNURL";
    lnurl_identifier
        .trim()
        .to_uppercase()
        .starts_with(LNURL_PREFIX)
}

/// Resolve an LNURL or lightning address to an invoice in one shot.
///
/// Convenience function that combines resolution + validation + invoice
/// fetching. For a two-phase flow, use `PayRequestResponse::get_invoice()`
/// directly after resolving.
pub async fn resolve_lnurl_to_invoice<T: LnUrlHttpClient>(
    http_client: &T,
    lnurl_identifier: &str,
    amount_msats: u64,
    comment: Option<&str>,
) -> Result<(String, Option<SuccessAction>)> {
    let url = match is_lnurl(lnurl_identifier) {
        true => parse_lnurl(lnurl_identifier)?,
        false => parse_lightning_address(lnurl_identifier)?,
    };

    debug!("Domain: {}", Url::parse(&url).unwrap().host().unwrap());

    let pay_request: PayRequestResponse =
        http_client.get_pay_request_response(&url).await?;

    pay_request.validate(lnurl_identifier, amount_msats)?;
    pay_request.get_invoice(http_client, amount_msats, comment).await
}

/// Parse a lightning address into its well-known LNURL-pay URL (LUD-16).
pub fn parse_lightning_address(lightning_address: &str) -> Result<String> {
    let parts: Vec<&str> = lightning_address.split('@').collect();

    if parts.len() != 2 {
        return Err(anyhow!(
            "The provided lightning address is improperly formatted"
        ));
    }

    let username = parts[0];
    let domain = parts[1];

    if username.is_empty() {
        return Err(anyhow!("Username can not be empty"));
    }
    if domain.is_empty() {
        return Err(anyhow!("Domain can not be empty"));
    }

    Ok(format!(
        "https://{}/.well-known/lnurlp/{}",
        domain, username
    ))
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
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice);
            let x = serde_json::from_str(&callback_response_json).unwrap();
            convert_to_async_return_value(Ok(x))
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let amount = 100000;

        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, amount, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lnurl_pay_with_lightning_address() {
        let mut mock_http_client = MockLnUrlHttpClient::new();
        let lightning_address_username = "satoshi";
        let lightning_address_domain = "cipherpunk.com";
        let lnurl = format!(
            "{}@{}",
            lightning_address_username, lightning_address_domain
        );

        let lnurl_clone = lnurl.clone();
        mock_http_client.expect_get_pay_request_response().returning(move |url| {
            let expected_url = format!("https://{}/.well-known/lnurlp/{}", lightning_address_domain, lightning_address_username);
            assert_eq!(expected_url, url);

            let pay_request_json = format!("{{\"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"], [\\\"text/identifier\\\", \\\"{}\\\"]]\" }}", lnurl_clone);

            let x: PayRequestResponse = serde_json::from_str(&pay_request_json).unwrap();
            convert_to_async_return_value(Ok(x))
        });

        mock_http_client.expect_get_pay_request_callback_response().returning(|_url| {
            let invoice = "lnbcrt1u1pj0ypx6sp5hzczugdw9eyw3fcsjkssux7awjlt68vpj7uhmen7sup0hdlrqxaqpp5gp5fm2sn5rua2jlzftkf5h22rxppwgszs7ncm73pmwhvjcttqp3qdy2tddjyar90p6z7urvv95kug3vyq39xarpwf6zqargv5syxmmfde28yctfdc396tpqtv38getcwshkjer9de6xjenfv4ezytpqyfekzar0wd5xjsrrd9cxsetjwp6ku6ewvdhk6gjat5xqyjw5qcqp29qxpqysgqujuf5zavazln2q9gks7nqwdgjypg2qlvv7aqwfmwg7xmjt8hy4hx2ctr5fcspjvmz9x5wvmur8vh6nkynsvateafm73zwg5hkf7xszsqajqwcf";
            let callback_response_json = format!("{{\"pr\":\"{}\",\"routes\":[]}}", invoice);
            let x = serde_json::from_str(&callback_response_json).unwrap();
            convert_to_async_return_value(Ok(x))
        });

        let amount = 100000;

        let result = resolve_lnurl_to_invoice(&mock_http_client, &lnurl, amount, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_lnurl_pay_with_lightning_address_fails_with_empty_username() {
        let mock_http_client = MockLnUrlHttpClient::new();
        let lnurl = "@cipherpunk.com";
        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, 100000, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Username can not be empty"));
    }

    #[tokio::test]
    async fn test_lnurl_pay_with_lightning_address_fails_with_empty_domain() {
        let mock_http_client = MockLnUrlHttpClient::new();
        let lnurl = "satoshi@";
        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, 100000, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Domain can not be empty"));
    }

    #[tokio::test]
    async fn test_lnurl_pay_returns_error_on_invalid_lnurl() {
        let mock_http_client = MockLnUrlHttpClient::new();
        let lnurl = "LNURL1111111111111111111111111111111111111111111111111111111111111111111";

        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, 100000, None).await;
        assert!(result.unwrap_err().to_string().contains("Failed to decode lnurl: invalid length"));
    }

    #[tokio::test]
    async fn test_lnurl_pay_returns_error_on_amount_less_than_min_sendable() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100000, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            convert_to_async_return_value(Ok(x))
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, 1, None).await;
        assert!(result.unwrap_err().to_string().contains("Amount must be"));
    }

    #[tokio::test]
    async fn test_lnurl_pay_returns_error_on_amount_greater_than_max_sendable() {
        let mut mock_http_client = MockLnUrlHttpClient::new();

        mock_http_client.expect_get_pay_request_response().returning(|_url| {
            let x: PayRequestResponse = serde_json::from_str("{ \"callback\": \"https://cipherpunk.com/lnurlp/api/v1/lnurl/cb/1\", \"maxSendable\": 100000, \"minSendable\": 100000, \"tag\": \"payRequest\", \"metadata\": \"[[\\\"text/plain\\\", \\\"Start the CoinTrain\\\"]]\" }").unwrap();
            convert_to_async_return_value(Ok(x))
        });

        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        let result = resolve_lnurl_to_invoice(&mock_http_client, lnurl, 200000, None).await;
        assert!(result.unwrap_err().to_string().contains("Amount must be"));
    }

    #[test]
    fn test_aes_decrypt_known_vector() {
        use aes::Aes256;
        use base64::{engine::general_purpose::STANDARD, Engine};
        use cbc::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};

        let key = [0x42u8; 32];
        let iv = [0x24u8; 16];
        let plaintext = b"hello world";

        // Encrypt
        type Aes256CbcEnc = cbc::Encryptor<Aes256>;
        let ciphertext = Aes256CbcEnc::new_from_slices(&key, &iv)
            .unwrap()
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext);

        let ciphertext_b64 = STANDARD.encode(&ciphertext);
        let iv_b64 = STANDARD.encode(&iv);

        // Decrypt
        let result = decrypt_aes_success_action(&key, &ciphertext_b64, &iv_b64).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_aes_decrypt_wrong_preimage_length() {
        let result = decrypt_aes_success_action(&[0u8; 16], "YWJj", "MTIzNDU2Nzg5MDEyMzQ1Ng==");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
    }

    #[test]
    fn test_pay_request_description() {
        let resp: PayRequestResponse = serde_json::from_str(
            r#"{"callback":"https://x.com/cb","maxSendable":1000,"minSendable":1,"tag":"payRequest","metadata":"[[\"text/plain\",\"Buy coffee\"]]"}"#
        ).unwrap();
        assert_eq!(resp.description(), Some("Buy coffee".to_string()));
    }

    #[test]
    fn test_pay_request_validate_amount_range() {
        let resp: PayRequestResponse = serde_json::from_str(
            r#"{"callback":"https://x.com/cb","maxSendable":10000,"minSendable":1000,"tag":"payRequest","metadata":"[[\"text/plain\",\"test\"]]"}"#
        ).unwrap();

        // In range
        assert!(resp.validate("LNURL1TEST", 5000).is_ok());
        // Below min
        assert!(resp.validate("LNURL1TEST", 500).is_err());
        // Above max
        assert!(resp.validate("LNURL1TEST", 20000).is_err());
    }
}
