//! Framework-neutral RPC and HTTP error contracts.

#![warn(unreachable_pub)]

extern crate self as rpc_error;

#[cfg(feature = "actix")]
pub mod actix;
#[cfg(feature = "axum")]
pub mod axum;
mod catalog;
mod code;
mod error;
mod output;
mod request_id;
mod runtime;
mod severity;

pub use catalog::{ReasonCatalog, ReasonCatalogs, ReasonDefinition, validate_unique_reasons};
pub use code::ErrorCode;
pub use error::{Error, Result};
pub use output::{ErrorDetail, ErrorOutput};
pub use request_id::{RequestId, RequestIdGenerator, UuidRequestIdGenerator};
pub use runtime::{RpcError, RpcErrorLog};
pub use severity::Severity;

#[cfg(feature = "derive")]
pub use rpc_error_derive::RpcError;

/// Macro support re-exports.
#[doc(hidden)]
pub mod __private {
    /// JSON support used by generated derive code.
    pub use serde_json;
}

#[cfg(test)]
#[path = "_tests_/core_tests.rs"]
mod core_tests;
