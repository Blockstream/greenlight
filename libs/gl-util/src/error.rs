//! # Greenlight Error Module
//!
//! This module provides a comprehensive error handling system for
//! greenlight. It features a generic error type that can be customized with
//! module- or crate-specific error codes, while maintaining compatibility
//! with gRPC status codes and providing rich error context.
use bytes::Bytes;
use core::error::Error as StdError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tonic;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsingError(String);

impl core::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not parse error: {}", self.0)
    }
}

impl StdError for ParsingError {}

// Convenient macro for creating ParsingError
macro_rules! parsing_error {
    ($($arg:tt)*) => {
        ParsingError(format!($($arg)*))
    };
}

/// Trait for defining module-specific error codes.
///
/// This trait should be implemented by enums that represent different error
/// categories in greenlight. The error codes should be unique integers that
/// can be serialized and transmitted over the network.
pub trait ErrorCode: core::fmt::Debug + core::fmt::Display + Clone + Send + Sync + 'static {
    /// Returns the numeric error code for this error type.
    ///
    /// This code should be unique within your application and stable
    /// across versions for backward compatibility.
    fn code(&self) -> i32;

    /// Attempts to construct an error code from its numeric representation.
    ///
    /// Returns `None` if the code is not recognized.
    fn from_code(code: i32) -> Option<Self>
    where
        Self: Sized;
}

/// JSON structure for transmitting error details over gRPC.
///
/// This structure is serialized into the `details` field of a
/// `tonic::Status` to provide structured error information that can be
/// parsed by clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrpcErrorDetails {
    pub code: i32,
    /// Optional hint to help users resolve the issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Extracted error information from a gRPC status.
///
/// This structure contains all the error information that was transmitted
/// over gRPC, including both the standard gRPC fields and our custom
/// structured error details.
#[derive(Debug, Clone)]
pub struct GrpcErrorInfo {
    pub code: i32,
    pub message: String,
    pub hint: Option<String>,
    pub grpc_code: tonic::Code,
}

/// Attempts to parse structured error information from a `tonic::Status`.
///
/// This implementation expects the status details to contain a JSON-encoded
/// `GrpcErrorDetails` structure. If parsing fails, a `serde_json::Error` is
/// returned.
impl TryFrom<tonic::Status> for GrpcErrorInfo {
    type Error = serde_json::Error;

    fn try_from(value: tonic::Status) -> std::result::Result<Self, Self::Error> {
        let parsed = serde_json::from_slice::<GrpcErrorDetails>(&value.details())?;
        Ok(GrpcErrorInfo {
            code: parsed.code,
            message: value.message().to_owned(),
            hint: parsed.hint,
            grpc_code: value.code(),
        })
    }
}

/// Extension trait for mapping error codes to gRPC status codes.
///
/// This trait should be implemented alongside `ErrorCode` to define
/// how your greenlight-specific errors map to standard gRPC status codes.
pub trait ErrorStatusConversionExt: ErrorCode {
    /// Maps this error to an appropriate gRPC status code.
    ///
    /// The returned status code should follow gRPC conventions:
    /// See: https://grpc.io/docs/guides/status-codes/
    fn status_code(&self) -> tonic::Code;
}

/// Generic error type that combines error codes with rich error context.
#[derive(Debug, Clone)]
pub struct GreenlightError<C: ErrorCode> {
    /// Error code for categorization and programmatic handling
    pub code: C,
    /// User-facing error message
    pub message: String,
    /// Optional hint to help users resolve the issue
    pub hint: Option<String>,
    /// Context for debugging
    pub context: Option<String>,
    /// Source error chain for debugging
    pub source: Option<Arc<dyn StdError + Send + Sync>>,
}

impl<C: ErrorCode> GreenlightError<C> {
    /// Creates a new error with the given code and message.
    pub fn new(code: C, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
            context: None,
            source: None,
        }
    }

    /// Adds a hint to help users resolve the issue.
    ///
    /// Hints should provide actionable guidance for end users.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Adds internal context for debugging.
    ///
    /// Context is meant for developers and should include information
    /// about what the system was doing when the error occurred.
    /// TODO: currently unarmed, but can be used to log errors in a
    /// standardized way in the future.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Adds a source error for error chaining.
    ///
    /// This is useful for preserving the original error that caused this error.
    ///
    pub fn with_source(mut self, source: impl StdError + Send + Sync + 'static) -> Self {
        self.source = Some(Arc::new(source));
        self
    }

    /// Adds a source error from an existing Arc.
    ///
    /// This is useful when forwarding errors that are already wrapped in an Arc,
    /// avoiding unnecessary allocations.
    pub fn with_source_arc(mut self, source: Arc<dyn StdError + Send + Sync>) -> Self {
        self.source = Some(source);
        self
    }

    /// Returns the numeric error code.
    ///
    /// This is a convenience method that calls `code()` on the error code.
    pub fn code(&self) -> i32 {
        self.code.code()
    }

    /// Converts this error to use a different error code type.
    ///
    /// This is useful when errors need to be converted between different
    /// modules or layers that use different error code enums.
    pub fn map_code<T: ErrorCode>(self, new_code: T) -> GreenlightError<T> {
        GreenlightError {
            code: new_code,
            message: self.message,
            hint: self.hint,
            context: self.context,
            source: self.source,
        }
    }
}

/// Displays the error message.
impl<C: ErrorCode> core::fmt::Display for GreenlightError<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Implements the standard error trait for error chaining.
impl<C: ErrorCode> StdError for GreenlightError<C> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn StdError + 'static))
    }
}

/// Converts a `GreenlightError` into a `tonic::Status` for gRPC transmission.
///
/// The error details are JSON-encoded and included in the status details
/// field. If JSON serialization fails, a fallback JSON string is used.
impl<C: ErrorCode + ErrorStatusConversionExt> From<GreenlightError<C>> for tonic::Status {
    fn from(value: GreenlightError<C>) -> Self {
        let code = value.code.status_code();
        let details: Bytes = serde_json::to_vec(&GrpcErrorDetails {
            code: value.code(),
            hint: value.hint.clone(),
        })
        .unwrap_or_else(|_| {
            // Fallback to simple JSON if serialization fails
            // This ensures we always send valid JSON even if something goes wrong
            format!(
                "{{\"code\":{},\"message\":\"{}\"}}",
                value.code(),
                value.message,
            )
            .into_bytes()
        })
        .into();
        tonic::Status::with_details(code, value.message, details)
    }
}

/// Attempts to convert a `tonic::Status` back into a `GreenlightError`.
///
/// This requires that:
/// 1. The status contains valid JSON details in the expected format
/// 2. The error code in the details can be mapped to a valid `C` variant
///
/// Returns an `anyhow::Error` if parsing fails or the error code is unknown.
impl<C: ErrorCode> TryFrom<tonic::Status> for GreenlightError<C> {
    type Error = ParsingError;

    fn try_from(value: tonic::Status) -> std::result::Result<Self, Self::Error> {
        let grpc_err: GrpcErrorInfo = value
            .try_into()
            .map_err(|e| parsing_error!("failed to convert Status into GrpcErrorInfo {}", e))?;
        let code = C::from_code(grpc_err.code)
            .ok_or_else(|| parsing_error!("unknown error code: {}", grpc_err.code))?;
        Ok(Self {
            code,
            message: grpc_err.message,
            hint: grpc_err.hint,
            context: None,
            source: None,
        })
    }
}

/// Type alias for Core Lightning RPC error codes.
///
/// CLN uses specific numeric codes to indicate different types of failures
/// in payment operations. This type preserves the original error code
/// for debugging and logging purposes.
pub type ClnRpcError = i32;

/// Implementation of `ErrorCode` for CLN RPC errors.
///
/// This implementation treats all i32 values as valid error codes,
/// allowing us to preserve any error code returned by CLN without loss.
impl ErrorCode for ClnRpcError {
    fn code(&self) -> i32 {
        *self
    }

    fn from_code(code: i32) -> Option<Self>
    where
        Self: Sized,
    {
        Some(code)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_status_conversion() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        enum TestErrorCodes {
            FailedPrecondition = 101,
            NotFound = 202,
        }

        impl ErrorCode for TestErrorCodes {
            fn code(&self) -> i32 {
                *self as i32
            }

            fn from_code(_code: i32) -> Option<Self>
            where
                Self: Sized,
            {
                unimplemented!()
            }
        }

        impl core::fmt::Display for TestErrorCodes {
            fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                unimplemented!()
            }
        }

        impl ErrorStatusConversionExt for TestErrorCodes {
            fn status_code(&self) -> tonic::Code {
                match self {
                    TestErrorCodes::FailedPrecondition => tonic::Code::FailedPrecondition,
                    TestErrorCodes::NotFound => tonic::Code::NotFound,
                }
            }
        }

        type TestError = GreenlightError<TestErrorCodes>;

        let t_err = TestError::new(TestErrorCodes::FailedPrecondition, "a failed precondition")
            .with_hint("How to resolve it");
        let status: tonic::Status = t_err.clone().into();
        assert_eq!(status.message(), t_err.message);

        let mut details: serde_json::Value = serde_json::from_slice(status.details()).unwrap();
        assert_eq!(
            details["code"].take(),
            TestErrorCodes::FailedPrecondition.code()
        );
        assert_eq!(details["hint"].take(), "How to resolve it");
    }
}
