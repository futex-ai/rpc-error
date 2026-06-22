//! Serializable client-facing error envelopes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ErrorCode, RpcError};

/// Standard top-level error response body.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorOutput {
    /// Error detail payload.
    pub error: ErrorDetail,
}

/// Standard serialized error details.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ErrorDetail {
    /// Standard error code.
    pub code: ErrorCode,
    /// Stable application reason code.
    pub reason: String,
    /// Client-safe error message.
    pub message: String,
    /// Request identifier for correlating support and logs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Optional structured client-facing context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl From<&RpcError> for ErrorOutput {
    fn from(value: &RpcError) -> Self {
        Self {
            error: ErrorDetail {
                code: value.code(),
                reason: value.reason().to_owned(),
                message: value.message().to_owned(),
                request_id: value.request_id().map(ToOwned::to_owned),
                data: value.data().cloned(),
            },
        }
    }
}

impl From<RpcError> for ErrorOutput {
    fn from(value: RpcError) -> Self {
        Self::from(&value)
    }
}
