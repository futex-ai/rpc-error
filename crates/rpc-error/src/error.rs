//! Runtime error contract for the `rpc-error` crate itself.

use serde_json::Value;

use crate::ErrorCode;

/// Crate-local result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised while constructing or decoding standardized RPC errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A reason code failed kebab-case validation.
    #[error("[rpc-error/reason] invalid reason code `{value}`")]
    InvalidReasonCode {
        /// Invalid reason-code value.
        value: String,
    },
    /// The supplied standard code string is unknown.
    #[error("[rpc-error/code] unknown error code `{value}`")]
    UnknownErrorCode {
        /// Unknown code string.
        value: String,
    },
    /// Structured error data could not be serialized.
    #[error("[rpc-error/data] failed to serialize error data")]
    DataSerialization {
        /// Serialization failure source.
        #[source]
        source: serde_json::Error,
    },
    /// Structured error data could not be deserialized.
    #[error("[rpc-error/data] failed to deserialize error data for reason `{reason}`")]
    DataDeserialization {
        /// Reason code being decoded.
        reason: String,
        /// Data that failed deserialization.
        data: Value,
        /// Deserialization failure source.
        #[source]
        source: serde_json::Error,
    },
    /// A typed error variant required structured data but none was present.
    #[error("[rpc-error/data] missing error data for reason `{reason}`")]
    MissingData {
        /// Reason code being decoded.
        reason: String,
    },
    /// A typed error variant did not accept structured data but data was present.
    #[error("[rpc-error/data] unexpected error data for reason `{reason}`")]
    UnexpectedData {
        /// Reason code being decoded.
        reason: String,
        /// Data that was not part of the typed variant contract.
        data: Value,
    },
    /// A typed error variant matched the reason but not the standard code.
    #[error(
        "[rpc-error/conversion] code mismatch for reason `{reason}`: expected `{expected}`, got `{actual}`"
    )]
    CodeMismatch {
        /// Reason code being decoded.
        reason: String,
        /// Expected standard code for the reason.
        expected: ErrorCode,
        /// Actual standard code in the envelope.
        actual: ErrorCode,
    },
    /// An envelope reason code did not match the target typed error enum.
    #[error("[rpc-error/conversion] unknown reason `{reason}`")]
    UnknownReason {
        /// Unknown reason code.
        reason: String,
    },
    /// A top-level reason catalog contains a duplicate reason.
    #[error("[rpc-error/catalog] duplicate reason code `{reason}`")]
    DuplicateReasonCode {
        /// Duplicate reason code.
        reason: String,
    },
}
