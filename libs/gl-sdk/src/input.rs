// Input parsing for BOLT11 invoices and Lightning node IDs.
// Works offline — no node connection needed.

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

/// The result of parsing user input.
#[derive(Clone, uniffi::Enum)]
pub enum InputType {
    /// A BOLT11 Lightning invoice.
    Bolt11 { invoice: ParsedInvoice },
    /// A Lightning node public key (66 hex characters, 33 bytes compressed).
    NodeId { node_id: String },
}

/// Parse a string and identify whether it's a BOLT11 invoice or a node ID.
///
/// Strips `lightning:` / `LIGHTNING:` prefixes automatically.
/// Returns an error if the input is not recognized or is malformed.
pub fn parse_input(input: String) -> Result<InputType, Error> {
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

    // Try BOLT11
    if let Some(input_type) = try_parse_bolt11(stripped) {
        return input_type;
    }

    // Try Node ID
    if let Some(input_type) = try_parse_node_id(stripped) {
        return Ok(input_type);
    }

    Err(Error::Other("Unrecognized input".to_string()))
}

/// Try parsing as a BOLT11 invoice. Returns None if the input doesn't
/// look like an invoice, or Some(Result) if it does (even if malformed).
fn try_parse_bolt11(input: &str) -> Option<Result<InputType, Error>> {
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

    Some(Ok(InputType::Bolt11 {
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
fn try_parse_node_id(input: &str) -> Option<InputType> {
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
    Some(InputType::NodeId {
        node_id: input.to_string(),
    })
}
