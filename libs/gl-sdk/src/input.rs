// Input parsing for BOLT11 invoices, Lightning node IDs, LNURL
// strings, and Lightning Addresses.
//
// Two entry points with explicit cost contracts:
//
//   * `parse_input(input)` — synchronous, offline, no I/O. Returns
//     `ParsedInput` identifying *what* the input is. LNURL bech32
//     strings are decoded to their underlying URL; Lightning
//     Addresses are returned as the unparsed `user@host` form. The
//     caller decides whether to resolve further.
//
//   * `resolve_input(input)` — asynchronous, network-touching. Calls
//     `parse_input` internally, then for the LNURL / Lightning
//     Address branches fetches the endpoint to produce typed pay or
//     withdraw request data. BOLT11 and node IDs pass through
//     without I/O.
//
// Wallets that want offline classification (clipboard checks,
// invoice sanity-checks on the send screen) call `parse_input`.
// Wallets handling a QR scan that should proceed straight to the
// pay/withdraw screen call `resolve_input`.

use crate::lnurl::{LnUrlPayRequestData, LnUrlWithdrawRequestData};
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

/// The result of `parse_input`: an offline classification of the
/// input. No HTTP, no I/O. LNURL bech32 strings are returned as their
/// decoded URL; Lightning Addresses as the unparsed `user@host` form.
#[derive(Clone, uniffi::Enum)]
pub enum ParsedInput {
    /// A BOLT11 Lightning invoice.
    Bolt11 { invoice: ParsedInvoice },
    /// A Lightning node public key.
    NodeId { node_id: String },
    /// An LNURL bech32 string (LUD-01) decoded to its underlying URL.
    /// Pass to `resolve_input` (or fetch yourself) to determine
    /// whether it's a pay, withdraw, or auth endpoint.
    LnUrl { url: String },
    /// A Lightning Address (LUD-16) in the form `user@host`. The
    /// well-known URL is not constructed offline; call `resolve_input`
    /// to fetch and classify.
    LnUrlAddress { address: String },
}

/// The result of `resolve_input`: a fully-resolved input ready for
/// the caller's next action. LNURL bech32 strings and Lightning
/// Addresses are resolved over HTTP into typed pay or withdraw
/// request data; BOLT11 and node IDs pass through unchanged.
#[derive(Clone, uniffi::Enum)]
pub enum ResolvedInput {
    /// A BOLT11 Lightning invoice. No HTTP was performed.
    Bolt11 { invoice: ParsedInvoice },
    /// A Lightning node public key. No HTTP was performed.
    NodeId { node_id: String },
    /// An LNURL-pay endpoint with the service's parameters fetched.
    LnUrlPay { data: LnUrlPayRequestData },
    /// An LNURL-withdraw endpoint with the service's parameters fetched.
    LnUrlWithdraw { data: LnUrlWithdrawRequestData },
}

/// Synchronously classify the input. **No HTTP, no I/O.**
///
/// Recognises BOLT11 invoices, node IDs, LNURL bech32 strings, and
/// Lightning Addresses. Strips `lightning:` / `LIGHTNING:` prefixes
/// automatically.
///
/// LNURL inputs are decoded to their underlying URL but **not
/// fetched** — the caller chooses whether to resolve further (via
/// `resolve_input`) or to surface the URL to the user as-is.
pub fn parse_input(input: String) -> Result<ParsedInput, Error> {
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
    if let Some(result) = try_parse_bolt11(stripped) {
        return result;
    }

    // Try Lightning Address (user@domain)
    if let Some(result) = try_parse_lightning_address(stripped) {
        return Ok(result);
    }

    // Try Node ID
    if let Some(result) = try_parse_node_id(stripped) {
        return Ok(result);
    }

    Err(Error::Other("Unrecognized input".to_string()))
}

/// Asynchronously classify and resolve the input.
///
/// Internally calls `parse_input`. For BOLT11 and node IDs returns
/// immediately without I/O. For LNURL bech32 strings and Lightning
/// Addresses, performs the HTTP GET and returns the typed pay or
/// withdraw request data.
pub async fn resolve_input(input: String) -> Result<ResolvedInput, Error> {
    use gl_client::lnurl::models::LnUrlHttpClearnetClient;
    use gl_client::lnurl::{LnUrlResponse, LNURL};

    // Capture the user's original input (post-trim) so that
    // `data.lnurl` on the resolved response carries the exact string
    // the caller handed us.
    let original = input.trim().to_string();

    // The two LNURL-shaped branches converge to a single HTTP fetch
    // — the only branch-specific bit is how the URL is derived.
    let url = match parse_input(input)? {
        ParsedInput::Bolt11 { invoice } => return Ok(ResolvedInput::Bolt11 { invoice }),
        ParsedInput::NodeId { node_id } => return Ok(ResolvedInput::NodeId { node_id }),
        ParsedInput::LnUrl { url } => url,
        ParsedInput::LnUrlAddress { address } => {
            gl_client::lnurl::pay::parse_lightning_address(&address)
                .map_err(|e| Error::Other(e.to_string()))?
        }
    };

    let client = LNURL::new(LnUrlHttpClearnetClient::new());
    let response = client
        .resolve(&url)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok(match response {
        LnUrlResponse::Pay(d) => {
            let mut data: LnUrlPayRequestData = d.into();
            data.lnurl = original;
            ResolvedInput::LnUrlPay { data }
        }
        LnUrlResponse::Withdraw(d) => {
            let mut data: LnUrlWithdrawRequestData = d.into();
            data.lnurl = original;
            ResolvedInput::LnUrlWithdraw { data }
        }
    })
}

/// Try parsing as an LNURL bech32 string (LUD-01).
/// Returns None if the input doesn't look like an LNURL.
fn try_parse_lnurl(input: &str) -> Option<Result<ParsedInput, Error>> {
    if !input.to_uppercase().starts_with("LNURL1") {
        return None;
    }
    match gl_client::lnurl::utils::parse_lnurl(input) {
        Ok(url) => Some(Ok(ParsedInput::LnUrl { url })),
        Err(e) => Some(Err(Error::Other(format!("Invalid LNURL: {}", e)))),
    }
}

/// Try parsing as a Lightning Address (LUD-16): `user@domain.tld`.
fn try_parse_lightning_address(input: &str) -> Option<ParsedInput> {
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
    Some(ParsedInput::LnUrlAddress {
        address: input.to_string(),
    })
}

/// Try parsing as a BOLT11 invoice. Returns None if the input doesn't
/// look like an invoice, or Some(Result) if it does (even if malformed).
fn try_parse_bolt11(input: &str) -> Option<Result<ParsedInput, Error>> {
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

    Some(Ok(ParsedInput::Bolt11 {
        invoice: ParsedInvoice {
            bolt11: input.to_string(),
            payee_pubkey: Some(payee_pubkey),
            payment_hash,
            description,
            amount_msat,
            expiry,
            timestamp,
        },
    }))
}

/// Try parsing as a node ID (66-char hex → 33-byte compressed pubkey).
fn try_parse_node_id(input: &str) -> Option<ParsedInput> {
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
    Some(ParsedInput::NodeId {
        node_id: input.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed_variant_name(t: &ParsedInput) -> &'static str {
        match t {
            ParsedInput::Bolt11 { .. } => "Bolt11",
            ParsedInput::NodeId { .. } => "NodeId",
            ParsedInput::LnUrl { .. } => "LnUrl",
            ParsedInput::LnUrlAddress { .. } => "LnUrlAddress",
        }
    }

    fn resolved_variant_name(t: &ResolvedInput) -> &'static str {
        match t {
            ResolvedInput::Bolt11 { .. } => "Bolt11",
            ResolvedInput::NodeId { .. } => "NodeId",
            ResolvedInput::LnUrlPay { .. } => "LnUrlPay",
            ResolvedInput::LnUrlWithdraw { .. } => "LnUrlWithdraw",
        }
    }

    // ── parse_input (sync) ──────────────────────────────────────

    #[test]
    fn test_parse_input_bolt11() {
        let invoice = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";
        match parse_input(invoice.to_string()).unwrap() {
            ParsedInput::Bolt11 { invoice: parsed } => assert_eq!(parsed.amount_msat, Some(10)),
            other => panic!("Expected Bolt11, got {}", parsed_variant_name(&other)),
        }
    }

    #[test]
    fn test_parse_input_bolt11_with_lightning_prefix() {
        let invoice = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";
        let result = parse_input(format!("lightning:{}", invoice)).unwrap();
        assert!(matches!(result, ParsedInput::Bolt11 { .. }));
    }

    #[test]
    fn test_parse_input_node_id() {
        let node_id = "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619";
        match parse_input(node_id.to_string()).unwrap() {
            ParsedInput::NodeId { node_id: id } => assert_eq!(id, node_id),
            other => panic!("Expected NodeId, got {}", parsed_variant_name(&other)),
        }
    }

    #[test]
    fn test_parse_input_lnurl_decodes_url() {
        // Bech32-encoded "https://service.com/lnurl"
        let lnurl = "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2";
        match parse_input(lnurl.to_string()).unwrap() {
            ParsedInput::LnUrl { url } => assert!(url.starts_with("https://")),
            other => panic!("Expected LnUrl, got {}", parsed_variant_name(&other)),
        }
    }

    #[test]
    fn test_parse_input_lightning_address_returns_address() {
        match parse_input("user@example.com".to_string()).unwrap() {
            ParsedInput::LnUrlAddress { address } => assert_eq!(address, "user@example.com"),
            other => panic!("Expected LnUrlAddress, got {}", parsed_variant_name(&other)),
        }
    }

    #[test]
    fn test_parse_input_invalid_lnurl_errors() {
        assert!(parse_input("LNURL1INVALIDDATA".to_string()).is_err());
    }

    #[test]
    fn test_parse_input_address_no_dot_in_domain_errors() {
        assert!(parse_input("user@localhost".to_string()).is_err());
    }

    #[test]
    fn test_parse_input_empty_address_parts_errors() {
        assert!(parse_input("@example.com".to_string()).is_err());
        assert!(parse_input("user@".to_string()).is_err());
    }

    #[test]
    fn test_parse_input_unrecognized_errors() {
        assert!(parse_input("hello world".to_string()).is_err());
        assert!(parse_input("".to_string()).is_err());
        assert!(parse_input("   ".to_string()).is_err());
        assert!(parse_input("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string()).is_err());
    }

    #[test]
    fn test_parse_input_invalid_node_id_errors() {
        // 66 chars but starts with 0x04 (uncompressed pubkey prefix)
        assert!(parse_input(
            "04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619".to_string()
        )
        .is_err());
        // 66 non-hex chars
        assert!(parse_input(
            "not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string()
        )
        .is_err());
    }

    // ── resolve_input pass-through paths (no HTTP needed) ───────

    #[test]
    fn test_resolve_input_bolt11_passes_through() {
        let invoice = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcsh2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqclj9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9dha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58aguqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphmsywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0vp62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh38s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5j5r6drg6k6zcqj0fcwg";
        match crate::util::exec(resolve_input(invoice.to_string())).unwrap() {
            ResolvedInput::Bolt11 { invoice: parsed } => {
                assert_eq!(parsed.amount_msat, Some(10))
            }
            other => panic!("Expected Bolt11, got {}", resolved_variant_name(&other)),
        }
    }

    #[test]
    fn test_resolve_input_node_id_passes_through() {
        let node_id = "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619";
        match crate::util::exec(resolve_input(node_id.to_string())).unwrap() {
            ResolvedInput::NodeId { node_id: id } => assert_eq!(id, node_id),
            other => panic!("Expected NodeId, got {}", resolved_variant_name(&other)),
        }
    }

    #[test]
    fn test_resolve_input_invalid_lnurl_errors_before_http() {
        assert!(crate::util::exec(resolve_input("LNURL1INVALIDDATA".to_string())).is_err());
    }
}
