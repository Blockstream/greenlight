// Input parsing for BOLT11 invoices, Lightning node IDs, LNURL
// strings, and Lightning Addresses.
//
// `parse_input` is async and resolves LNURL endpoints over HTTP,
// returning a fully-typed `InputType` ready for the caller's next
// action (display the invoice, build a pay/withdraw request, etc.).

use crate::lnurl::{LnUrlAuthRequestData, LnUrlPayRequestData, LnUrlWithdrawRequestData};
use crate::Error;

/// Parsed BOLT11 invoice with extracted fields.
#[derive(Clone, uniffi::Record)]
pub struct ParsedInvoice {
    /// The original invoice string.
    pub bolt11: String,
    /// Recipient public key as lowercase hex (66 chars), recovered from the invoice signature.
    pub payee_pubkey: Option<String>,
    /// Payment hash as lowercase hex (64 chars) identifying this payment.
    pub payment_hash: String,
    /// Invoice description. None if the invoice uses a description hash.
    pub description: Option<String>,
    /// Requested amount in millisatoshis. None for "any amount" invoices.
    pub amount_msat: Option<u64>,
    /// Seconds from creation until the invoice expires.
    pub expiry: u64,
    /// Unix timestamp (seconds) when the invoice was created.
    pub timestamp: u64,
}

/// The result of `parse_input`: a fully-resolved input ready for the
/// caller's next action. LNURL bech32 strings and Lightning Addresses
/// are resolved over HTTP into typed pay or withdraw request data;
/// LNURL-auth endpoints are classified from the URL query string
/// without an HTTP fetch.
#[derive(Clone, uniffi::Enum)]
pub enum InputType {
    /// A BOLT11 Lightning invoice. No HTTP was performed.
    Bolt11 { invoice: ParsedInvoice },
    /// A Lightning node public key. No HTTP was performed.
    NodeId { node_id: String },
    /// An LNURL-pay endpoint with the service's parameters fetched.
    LnUrlPay { data: LnUrlPayRequestData },
    /// An LNURL-withdraw endpoint with the service's parameters fetched.
    LnUrlWithdraw { data: LnUrlWithdrawRequestData },
    /// An LNURL-auth (LUD-04) challenge. No HTTP was performed —
    /// classification comes from the `tag=login` URL query parameter.
    LnUrlAuth { data: LnUrlAuthRequestData },
}

/// Classify the input string offline before deciding whether to make
/// an HTTP request. Internal — `parse_input` is the public entry point.
enum Classification {
    Bolt11(ParsedInvoice),
    NodeId(String),
    LnUrl { decoded_url: String, original: String },
    LnUrlAddress { address: String },
}

/// Parse and resolve any supported input in one async call.
///
/// Identifies BOLT11 invoices, node IDs, LNURL bech32 strings, and
/// Lightning Addresses. For LNURL and Lightning Address inputs,
/// performs the HTTP GET and returns the typed pay or withdraw request
/// data. For BOLT11 and node IDs, returns immediately without I/O.
///
/// Strips `lightning:` / `LIGHTNING:` prefixes automatically.
pub async fn parse_input(input: String) -> Result<InputType, Error> {
    match classify(&input)? {
        Classification::Bolt11(invoice) => Ok(InputType::Bolt11 { invoice }),
        Classification::NodeId(node_id) => Ok(InputType::NodeId { node_id }),
        Classification::LnUrl { decoded_url, original } => {
            resolve_lnurl_endpoint(&decoded_url, &original).await
        }
        Classification::LnUrlAddress { address } => {
            let url = gl_client::lnurl::pay::parse_lightning_address(&address)
                .map_err(|e| Error::Other(e.to_string()))?;
            resolve_lnurl_endpoint(&url, &address).await
        }
    }
}

fn classify(input: &str) -> Result<Classification, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::Other("Empty input".to_string()));
    }

    // Strip lightning: prefix (case-insensitive)
    let stripped = if let Some(rest) = trimmed.strip_prefix("lightning:") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("LIGHTNING:") {
        rest
    } else {
        trimmed
    };

    // Try LNURL bech32 (must come before BOLT11 since both start with "ln")
    if let Some(result) = try_parse_lnurl(stripped) {
        return result;
    }

    // Try BOLT11
    if let Some(c) = try_parse_bolt11(stripped) {
        return c;
    }

    // Try Lightning Address (user@domain)
    if let Some(c) = try_parse_lightning_address(stripped) {
        return Ok(c);
    }

    // Try Node ID
    if let Some(c) = try_parse_node_id(stripped) {
        return Ok(c);
    }

    Err(Error::Other("Unrecognized input".to_string()))
}

async fn resolve_lnurl_endpoint(url: &str, original: &str) -> Result<InputType, Error> {
    use gl_client::lnurl::models::LnUrlHttpClearnetClient;
    use gl_client::lnurl::{LnUrlResponse, LNURL};

    // LUD-04 endpoints carry tag=login in the URL query string. We can
    // classify them without an HTTP fetch — the callback is hit later
    // by Node::lnurl_auth once the user approves.
    if let Some(auth) = try_parse_lnurl_auth(url)? {
        return Ok(InputType::LnUrlAuth { data: auth });
    }
    let _ = original; // reserved for future provenance fields

    let client = LNURL::new(LnUrlHttpClearnetClient::new());
    let response = client
        .resolve(url)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok(match response {
        LnUrlResponse::Pay(d) => {
            let mut data: LnUrlPayRequestData = d.into();
            data.lnurl = original.to_string();
            InputType::LnUrlPay { data }
        }
        LnUrlResponse::Withdraw(d) => {
            let mut data: LnUrlWithdrawRequestData = d.into();
            data.lnurl = original.to_string();
            InputType::LnUrlWithdraw { data }
        }
    })
}

/// Detect and parse an LNURL-auth endpoint (LUD-04) from a URL.
///
/// Returns `Ok(Some(_))` when the URL has `tag=login`, `Ok(None)` when
/// the URL is for a different LNURL flow, and `Err` when the URL is
/// malformed or the LUD-04 fields fail validation.
fn try_parse_lnurl_auth(raw_url: &str) -> Result<Option<LnUrlAuthRequestData>, Error> {
    let parsed =
        url::Url::parse(raw_url).map_err(|e| Error::Other(format!("Invalid LNURL URL: {e}")))?;

    let mut tag = None;
    let mut k1 = None;
    let mut action = None;
    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "tag" => tag = Some(value.into_owned()),
            "k1" => k1 = Some(value.into_owned()),
            "action" => action = Some(value.into_owned()),
            _ => {}
        }
    }

    if tag.as_deref() != Some("login") {
        return Ok(None);
    }

    let k1 = k1.ok_or_else(|| Error::Other("LNURL-auth URL missing k1".to_string()))?;
    let k1_bytes =
        hex::decode(&k1).map_err(|e| Error::Other(format!("LNURL-auth k1 not hex: {e}")))?;
    if k1_bytes.len() != 32 {
        return Err(Error::Other(
            "LNURL-auth k1 must be 32 bytes".to_string(),
        ));
    }

    if let Some(a) = action.as_deref() {
        if !["register", "login", "link", "auth"].contains(&a) {
            return Err(Error::Other(format!(
                "LNURL-auth action '{a}' is not one of register/login/link/auth"
            )));
        }
    }

    let domain = parsed
        .domain()
        .ok_or_else(|| Error::Other("LNURL-auth URL has no domain".to_string()))?
        .to_string();

    Ok(Some(LnUrlAuthRequestData {
        k1,
        action,
        domain,
        url: raw_url.to_string(),
    }))
}

/// Try parsing as an LNURL bech32 string (LUD-01).
/// Returns None if the input doesn't look like an LNURL.
fn try_parse_lnurl(input: &str) -> Option<Result<Classification, Error>> {
    if !input.to_uppercase().starts_with("LNURL1") {
        return None;
    }
    match gl_client::lnurl::utils::parse_lnurl(input) {
        Ok(url) => Some(Ok(Classification::LnUrl {
            decoded_url: url,
            original: input.to_string(),
        })),
        Err(e) => Some(Err(Error::Other(format!("Invalid LNURL: {}", e)))),
    }
}

/// Try parsing as a Lightning Address (LUD-16): `user@domain.tld`.
fn try_parse_lightning_address(input: &str) -> Option<Classification> {
    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() != 2 {
        return None;
    }
    let (username, domain) = (parts[0], parts[1]);
    if username.is_empty() || domain.is_empty() {
        return None;
    }
    // Domain must contain a dot (rules out bare hostnames and emails
    // to local domains which aren't valid Lightning Addresses).
    if !domain.contains('.') {
        return None;
    }
    // Username: alphanumeric + limited symbols per LUD-16.
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return None;
    }
    Some(Classification::LnUrlAddress {
        address: input.to_string(),
    })
}

/// Try parsing as a BOLT11 invoice. Returns None if the input doesn't
/// look like an invoice, or Some(Result) if it does (even if malformed).
fn try_parse_bolt11(input: &str) -> Option<Result<Classification, Error>> {
    let lower = input.to_lowercase();
    if !lower.starts_with("lnbc") && !lower.starts_with("lntb") && !lower.starts_with("lnbcrt") {
        return None;
    }

    let parsed: lightning_invoice::Bolt11Invoice = match input.parse() {
        Ok(inv) => inv,
        Err(e) => return Some(Err(Error::Other(format!("Invalid BOLT11 invoice: {e}")))),
    };

    if parsed.check_signature().is_err() {
        return Some(Err(Error::Other(
            "BOLT11 invoice has invalid signature".to_string(),
        )));
    }

    let payee_pubkey = hex::encode(parsed.recover_payee_pub_key().serialize());

    let payment_hash = format!("{}", parsed.payment_hash());

    let description = match parsed.description() {
        lightning_invoice::Bolt11InvoiceDescriptionRef::Direct(d) => Some(d.to_string()),
        lightning_invoice::Bolt11InvoiceDescriptionRef::Hash(_) => None,
    };

    let amount_msat = parsed.amount_milli_satoshis();
    let expiry = parsed.expiry_time().as_secs();
    let timestamp = parsed
        .timestamp()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Some(Ok(Classification::Bolt11(ParsedInvoice {
        bolt11: input.to_string(),
        payee_pubkey: Some(payee_pubkey),
        payment_hash,
        description,
        amount_msat,
        expiry,
        timestamp,
    })))
}

/// Try parsing as a node ID (66-char hex → 33-byte compressed pubkey).
fn try_parse_node_id(input: &str) -> Option<Classification> {
    if input.len() != 66 {
        return None;
    }
    let bytes = hex::decode(input).ok()?;
    if bytes.len() != 33 {
        return None;
    }
    // Compressed pubkeys start with 0x02 or 0x03
    if bytes[0] != 0x02 && bytes[0] != 0x03 {
        return None;
    }
    Some(Classification::NodeId(input.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant_name(t: &InputType) -> &'static str {
        match t {
            InputType::Bolt11 { .. } => "Bolt11",
            InputType::NodeId { .. } => "NodeId",
            InputType::LnUrlPay { .. } => "LnUrlPay",
            InputType::LnUrlWithdraw { .. } => "LnUrlWithdraw",
            InputType::LnUrlAuth { .. } => "LnUrlAuth",
        }
    }

    #[test]
    fn test_parse_input_bolt11_no_http() {
        let invoice = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";
        let result = crate::util::exec(parse_input(invoice.to_string())).unwrap();
        match result {
            InputType::Bolt11 { invoice: parsed } => assert_eq!(parsed.amount_msat, Some(10)),
            other => panic!("Expected Bolt11, got {}", variant_name(&other)),
        }
    }

    #[test]
    fn test_parse_input_bolt11_with_lightning_prefix() {
        let invoice = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";
        let result = crate::util::exec(parse_input(format!("lightning:{}", invoice))).unwrap();
        assert!(matches!(result, InputType::Bolt11 { .. }));
    }

    #[test]
    fn test_parse_input_node_id_no_http() {
        let node_id = "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619";
        let result = crate::util::exec(parse_input(node_id.to_string())).unwrap();
        match result {
            InputType::NodeId { node_id: id } => assert_eq!(id, node_id),
            other => panic!("Expected NodeId, got {}", variant_name(&other)),
        }
    }

    #[test]
    fn test_parse_input_invalid_lnurl_errors_before_http() {
        let result = crate::util::exec(parse_input("LNURL1INVALIDDATA".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_address_with_no_dot_in_domain_errors() {
        let result = crate::util::exec(parse_input("user@localhost".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_empty_address_parts_errors() {
        assert!(crate::util::exec(parse_input("@example.com".to_string())).is_err());
        assert!(crate::util::exec(parse_input("user@".to_string())).is_err());
    }

    #[test]
    fn test_parse_input_unrecognized_errors() {
        assert!(crate::util::exec(parse_input("hello world".to_string())).is_err());
        assert!(crate::util::exec(parse_input("".to_string())).is_err());
        assert!(crate::util::exec(parse_input("   ".to_string())).is_err());
        assert!(crate::util::exec(parse_input(
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string()
        ))
        .is_err());
    }

    // 32 zero bytes hex-encoded — a syntactically valid k1.
    const ZERO_K1: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn test_try_parse_lnurl_auth_classifies_login_url_without_http() {
        let url = format!("https://service.example.com/auth?tag=login&k1={ZERO_K1}");
        let parsed = try_parse_lnurl_auth(&url).unwrap().expect("expected Some");
        assert_eq!(parsed.k1, ZERO_K1);
        assert_eq!(parsed.domain, "service.example.com");
        assert!(parsed.action.is_none());
        assert_eq!(parsed.url, url);
    }

    #[test]
    fn test_try_parse_lnurl_auth_captures_action() {
        let url = format!("https://x.com/a?tag=login&k1={ZERO_K1}&action=register");
        let parsed = try_parse_lnurl_auth(&url).unwrap().expect("expected Some");
        assert_eq!(parsed.action.as_deref(), Some("register"));
    }

    #[test]
    fn test_try_parse_lnurl_auth_returns_none_for_non_login_tag() {
        let url = format!("https://x.com/p?tag=payRequest&k1={ZERO_K1}");
        assert!(try_parse_lnurl_auth(&url).unwrap().is_none());
    }

    #[test]
    fn test_try_parse_lnurl_auth_returns_none_when_no_tag() {
        assert!(try_parse_lnurl_auth("https://x.com/something")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_try_parse_lnurl_auth_rejects_missing_k1() {
        assert!(try_parse_lnurl_auth("https://x.com/a?tag=login").is_err());
    }

    #[test]
    fn test_try_parse_lnurl_auth_rejects_short_k1() {
        assert!(try_parse_lnurl_auth("https://x.com/a?tag=login&k1=deadbeef").is_err());
    }

    #[test]
    fn test_try_parse_lnurl_auth_rejects_unknown_action() {
        let url = format!("https://x.com/a?tag=login&k1={ZERO_K1}&action=bogus");
        assert!(try_parse_lnurl_auth(&url).is_err());
    }

    #[test]
    fn test_parse_input_invalid_node_id_errors() {
        // 66 chars but starts with 0x04 (uncompressed pubkey prefix)
        assert!(crate::util::exec(parse_input(
            "04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619".to_string()
        ))
        .is_err());
        // 66 non-hex chars
        assert!(crate::util::exec(parse_input(
            "not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string()
        ))
        .is_err());
    }
}
