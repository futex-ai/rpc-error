//! Standard error code definitions.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::Error;

/// Standard RPC error code mapped to one HTTP status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The request is malformed or failed general validation.
    BadRequest,
    /// A specific argument is invalid.
    InvalidArgument,
    /// The system state does not satisfy a required precondition.
    FailedPrecondition,
    /// A supplied value is outside the supported range.
    OutOfRange,
    /// Authentication is required or invalid.
    Unauthenticated,
    /// The authenticated caller does not have permission.
    PermissionDenied,
    /// The requested resource does not exist.
    NotFound,
    /// The operation was aborted because of a conflict.
    Aborted,
    /// The resource already exists.
    AlreadyExists,
    /// The caller exhausted a rate, quota, or resource limit.
    ResourceExhausted,
    /// The operation was cancelled.
    Cancelled,
    /// The request payload is too large.
    PayloadTooLarge,
    /// The dependency or service is unavailable.
    Unavailable,
    /// The server failed internally.
    Internal,
    /// The operation deadline was exceeded.
    DeadlineExceeded,
}

impl ErrorCode {
    /// Returns the stable kebab-case string for the code.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "bad-request",
            Self::InvalidArgument => "invalid-argument",
            Self::FailedPrecondition => "failed-precondition",
            Self::OutOfRange => "out-of-range",
            Self::Unauthenticated => "unauthenticated",
            Self::PermissionDenied => "permission-denied",
            Self::NotFound => "not-found",
            Self::Aborted => "aborted",
            Self::AlreadyExists => "already-exists",
            Self::ResourceExhausted => "resource-exhausted",
            Self::Cancelled => "cancelled",
            Self::PayloadTooLarge => "payload-too-large",
            Self::Unavailable => "unavailable",
            Self::Internal => "internal",
            Self::DeadlineExceeded => "deadline-exceeded",
        }
    }

    /// Returns the HTTP status code number for the standard code.
    pub const fn status_code_u16(self) -> u16 {
        match self {
            Self::BadRequest
            | Self::InvalidArgument
            | Self::FailedPrecondition
            | Self::OutOfRange => 400,
            Self::Unauthenticated => 401,
            Self::PermissionDenied => 403,
            Self::NotFound => 404,
            Self::Aborted | Self::AlreadyExists => 409,
            Self::ResourceExhausted => 429,
            Self::Cancelled => 406,
            Self::PayloadTooLarge => 413,
            Self::Unavailable => 503,
            Self::Internal => 500,
            Self::DeadlineExceeded => 504,
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ErrorCode {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "bad-request" => Ok(Self::BadRequest),
            "invalid-argument" => Ok(Self::InvalidArgument),
            "failed-precondition" => Ok(Self::FailedPrecondition),
            "out-of-range" => Ok(Self::OutOfRange),
            "unauthenticated" => Ok(Self::Unauthenticated),
            "permission-denied" => Ok(Self::PermissionDenied),
            "not-found" => Ok(Self::NotFound),
            "aborted" => Ok(Self::Aborted),
            "already-exists" => Ok(Self::AlreadyExists),
            "resource-exhausted" => Ok(Self::ResourceExhausted),
            "cancelled" => Ok(Self::Cancelled),
            "payload-too-large" => Ok(Self::PayloadTooLarge),
            "unavailable" => Ok(Self::Unavailable),
            "internal" => Ok(Self::Internal),
            "deadline-exceeded" => Ok(Self::DeadlineExceeded),
            _ => Err(Error::UnknownErrorCode {
                value: value.to_owned(),
            }),
        }
    }
}
