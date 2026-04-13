// LNURL types for UniFFI language bindings.
//
// These are thin wrappers around gl-client's protocol types, adding
// UniFFI annotations so they can be exported to Python, Kotlin, Swift,
// and Ruby. Protocol logic lives in gl-client; this module only does
// type conversion.

use gl_client::lnurl::models as wire;

// ── Resolved endpoint data ──────────────────────────────────────────

/// Result of resolving an LNURL or lightning address via HTTP.
#[derive(Clone, uniffi::Enum)]
pub enum ResolvedLnUrl {
    /// The endpoint is an LNURL-pay service (LUD-06).
    Pay { data: LnUrlPayRequestData },
    /// The endpoint is an LNURL-withdraw service (LUD-03).
    Withdraw { data: LnUrlWithdrawRequestData },
}

/// Data from an LNURL-pay endpoint (LUD-06).
///
/// Contains the service's accepted amount range and metadata.
/// Returned inside `ResolvedLnUrl::Pay` after resolving an LNURL.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlPayRequestData {
    /// The callback URL to request an invoice from.
    pub callback: String,
    /// Minimum amount the service accepts, in millisatoshis.
    pub min_sendable: u64,
    /// Maximum amount the service accepts, in millisatoshis.
    pub max_sendable: u64,
    /// Raw metadata JSON string (array of `["mime", "content"]` pairs).
    pub metadata: String,
    /// Maximum comment length the service accepts. 0 means no comments.
    pub comment_allowed: u64,
    /// Human-readable description extracted from metadata.
    pub description: String,
    /// The original LNURL or lightning address that was resolved.
    pub lnurl: String,
}

/// Data from an LNURL-withdraw endpoint (LUD-03).
///
/// Contains the service's accepted withdrawal range and session key.
/// Returned inside `ResolvedLnUrl::Withdraw` after resolving an LNURL.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlWithdrawRequestData {
    /// The callback URL to submit the invoice to.
    pub callback: String,
    /// Ephemeral secret linking this wallet session to the service.
    pub k1: String,
    /// Default description for the invoice.
    pub default_description: String,
    /// Minimum withdrawable amount in millisatoshis.
    pub min_withdrawable: u64,
    /// Maximum withdrawable amount in millisatoshis.
    pub max_withdrawable: u64,
    /// The original LNURL that was resolved.
    pub lnurl: String,
}

// ── User request types ──────────────────────────────────────────────

/// Request to execute an LNURL-pay flow.
///
/// Combines the resolved service data with the user's chosen amount.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlPayRequest {
    /// The resolved pay request data from `resolve_lnurl()`.
    pub data: LnUrlPayRequestData,
    /// Amount to pay in millisatoshis.
    pub amount_msat: u64,
    /// Optional comment to send with the payment.
    pub comment: Option<String>,
}

/// Request to execute an LNURL-withdraw flow.
///
/// Combines the resolved service data with the user's chosen amount.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlWithdrawRequest {
    /// The resolved withdraw request data from `resolve_lnurl()`.
    pub data: LnUrlWithdrawRequestData,
    /// Amount to withdraw in millisatoshis.
    pub amount_msat: u64,
    /// Optional description for the invoice (overrides default).
    pub description: Option<String>,
}

// ── Result types ────────────────────────────────────────────────────

/// Result of an LNURL-pay operation.
#[derive(Clone, uniffi::Enum)]
pub enum LnUrlPayResult {
    /// Payment succeeded.
    EndpointSuccess { data: LnUrlPaySuccessData },
    /// The LNURL service returned an error.
    EndpointError { data: LnUrlErrorData },
}

/// Successful LNURL-pay result data.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlPaySuccessData {
    /// The payment preimage (proof of payment), hex-encoded.
    pub payment_preimage: String,
    /// Optional success action from the service (LUD-09).
    pub success_action: Option<SuccessActionProcessed>,
}

/// Result of an LNURL-withdraw operation.
#[derive(Clone, uniffi::Enum)]
pub enum LnUrlWithdrawResult {
    /// The service accepted our invoice and will pay it.
    Ok { data: LnUrlWithdrawSuccessData },
    /// The LNURL service returned an error.
    ErrorStatus { data: LnUrlErrorData },
}

/// Successful LNURL-withdraw result data.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlWithdrawSuccessData {
    /// The BOLT11 invoice that was submitted for withdrawal.
    pub invoice: String,
}

/// Error returned by an LNURL service endpoint.
#[derive(Clone, uniffi::Record)]
pub struct LnUrlErrorData {
    pub reason: String,
}

// ── Success action types (LUD-09 / LUD-10) ─────────────────────────

/// A processed success action from an LNURL-pay callback.
///
/// For Message and Url this is passed through as-is. For Aes the
/// ciphertext has been decrypted using the payment preimage.
#[derive(Clone, uniffi::Enum)]
pub enum SuccessActionProcessed {
    /// Display a message to the user.
    Message { message: String },
    /// Display a URL to the user.
    Url { description: String, url: String },
    /// Decrypted AES payload (LUD-10).
    Aes { description: String, plaintext: String },
}

// ── From conversions (gl-client → gl-sdk) ───────────────────────────

impl From<wire::PayRequestResponse> for LnUrlPayRequestData {
    fn from(r: wire::PayRequestResponse) -> Self {
        Self {
            description: r.description().unwrap_or_default(),
            callback: r.callback,
            min_sendable: r.min_sendable,
            max_sendable: r.max_sendable,
            metadata: r.metadata,
            comment_allowed: r.comment_allowed.unwrap_or(0),
            lnurl: String::new(), // caller sets this after conversion
        }
    }
}

impl From<wire::WithdrawRequestResponse> for LnUrlWithdrawRequestData {
    fn from(r: wire::WithdrawRequestResponse) -> Self {
        Self {
            callback: r.callback,
            k1: r.k1,
            default_description: r.default_description,
            min_withdrawable: r.min_withdrawable,
            max_withdrawable: r.max_withdrawable,
            lnurl: String::new(), // caller sets this after conversion
        }
    }
}

impl From<wire::ProcessedSuccessAction> for SuccessActionProcessed {
    fn from(a: wire::ProcessedSuccessAction) -> Self {
        match a {
            wire::ProcessedSuccessAction::Message { message } => {
                SuccessActionProcessed::Message { message }
            }
            wire::ProcessedSuccessAction::Url { description, url } => {
                SuccessActionProcessed::Url { description, url }
            }
            wire::ProcessedSuccessAction::Aes {
                description,
                plaintext,
            } => SuccessActionProcessed::Aes {
                description,
                plaintext,
            },
        }
    }
}

impl From<gl_client::lnurl::LnUrlResponse> for ResolvedLnUrl {
    fn from(r: gl_client::lnurl::LnUrlResponse) -> Self {
        match r {
            gl_client::lnurl::LnUrlResponse::Pay(data) => ResolvedLnUrl::Pay {
                data: data.into(),
            },
            gl_client::lnurl::LnUrlResponse::Withdraw(data) => ResolvedLnUrl::Withdraw {
                data: data.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pay_request_data_from_conversion() {
        let wire_resp = wire::PayRequestResponse {
            callback: "https://example.com/cb".to_string(),
            max_sendable: 100000,
            min_sendable: 1000,
            tag: "payRequest".to_string(),
            metadata: r#"[["text/plain", "Buy coffee"]]"#.to_string(),
            comment_allowed: Some(140),
        };

        let data: LnUrlPayRequestData = wire_resp.into();
        assert_eq!(data.callback, "https://example.com/cb");
        assert_eq!(data.min_sendable, 1000);
        assert_eq!(data.max_sendable, 100000);
        assert_eq!(data.comment_allowed, 140);
        assert_eq!(data.description, "Buy coffee");
        assert!(data.lnurl.is_empty()); // caller sets this
    }

    #[test]
    fn test_pay_request_data_no_comment_allowed() {
        let wire_resp = wire::PayRequestResponse {
            callback: "https://example.com/cb".to_string(),
            max_sendable: 100000,
            min_sendable: 1000,
            tag: "payRequest".to_string(),
            metadata: r#"[["text/plain", "test"]]"#.to_string(),
            comment_allowed: None,
        };

        let data: LnUrlPayRequestData = wire_resp.into();
        assert_eq!(data.comment_allowed, 0);
    }

    #[test]
    fn test_withdraw_request_data_from_conversion() {
        let wire_resp = wire::WithdrawRequestResponse {
            tag: "withdrawRequest".to_string(),
            callback: "https://example.com/withdraw".to_string(),
            k1: "secret123".to_string(),
            default_description: "Withdraw from service".to_string(),
            min_withdrawable: 1000,
            max_withdrawable: 50000,
        };

        let data: LnUrlWithdrawRequestData = wire_resp.into();
        assert_eq!(data.callback, "https://example.com/withdraw");
        assert_eq!(data.k1, "secret123");
        assert_eq!(data.default_description, "Withdraw from service");
        assert_eq!(data.min_withdrawable, 1000);
        assert_eq!(data.max_withdrawable, 50000);
    }

    #[test]
    fn test_processed_success_action_from_message() {
        let processed = wire::ProcessedSuccessAction::Message {
            message: "Thanks!".to_string(),
        };
        let sdk: SuccessActionProcessed = processed.into();
        match sdk {
            SuccessActionProcessed::Message { message } => assert_eq!(message, "Thanks!"),
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_processed_success_action_from_url() {
        let processed = wire::ProcessedSuccessAction::Url {
            description: "View order".to_string(),
            url: "https://example.com/order".to_string(),
        };
        let sdk: SuccessActionProcessed = processed.into();
        match sdk {
            SuccessActionProcessed::Url { description, url } => {
                assert_eq!(description, "View order");
                assert_eq!(url, "https://example.com/order");
            }
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_processed_success_action_from_aes() {
        let processed = wire::ProcessedSuccessAction::Aes {
            description: "Your code".to_string(),
            plaintext: "ABC-123".to_string(),
        };
        let sdk: SuccessActionProcessed = processed.into();
        match sdk {
            SuccessActionProcessed::Aes {
                description,
                plaintext,
            } => {
                assert_eq!(description, "Your code");
                assert_eq!(plaintext, "ABC-123");
            }
            _ => panic!("Expected Aes variant"),
        }
    }
}
