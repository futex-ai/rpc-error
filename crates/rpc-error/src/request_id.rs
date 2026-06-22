//! Request identifier generation and request-scoped transport.

use std::fmt::{Display, Formatter};

/// Request identifier value shared through framework request extensions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RequestId(String);

impl RequestId {
    /// Creates a request identifier from a trusted application value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the request identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Generates request identifiers for adapters that need one.
pub trait RequestIdGenerator: Send + Sync {
    /// Returns the next request identifier.
    fn next_request_id(&self) -> String;
}

/// UUID v4 request identifier generator.
#[derive(Clone, Debug, Default)]
pub struct UuidRequestIdGenerator;

impl RequestIdGenerator for UuidRequestIdGenerator {
    fn next_request_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
