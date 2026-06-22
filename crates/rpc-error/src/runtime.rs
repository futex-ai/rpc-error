//! Core runtime error type.

use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::catalog::is_valid_reason_code;
use crate::{
    Error, ErrorCode, ErrorOutput, RequestIdGenerator, Result, Severity, UuidRequestIdGenerator,
};

/// Server-side log metadata copied onto framework responses.
#[derive(Clone, Debug)]
pub struct RpcErrorLog {
    /// Standard error code.
    pub code: ErrorCode,
    /// Stable application reason code.
    pub reason: String,
    /// HTTP status code number.
    pub status: u16,
    /// Logging severity.
    pub severity: Severity,
    /// Optional request identifier.
    pub request_id: Option<String>,
    /// Optional structured data.
    pub data: Option<Value>,
    /// Source-chain report for server logs.
    pub report: String,
}

/// Framework-neutral standardized RPC error.
#[derive(Clone, Debug)]
pub struct RpcError {
    code: ErrorCode,
    reason: String,
    message: String,
    data: Option<Value>,
    severity: Severity,
    source: Option<Arc<dyn StdError + Send + Sync>>,
    request_id: Option<String>,
}

impl RpcError {
    /// Creates a new standardized error.
    pub fn new(
        code: ErrorCode,
        reason: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self> {
        let reason = reason.into();
        validate_reason(&reason)?;
        Ok(Self {
            code,
            reason,
            message: message.into(),
            data: None,
            severity: Severity::Error,
            source: None,
            request_id: None,
        })
    }

    /// Creates a bad-request error.
    pub fn bad_request(reason: impl Into<String>, message: impl Into<String>) -> Result<Self> {
        Self::new(ErrorCode::BadRequest, reason, message)
    }

    /// Creates an unauthenticated error.
    pub fn unauthenticated(reason: impl Into<String>, message: impl Into<String>) -> Result<Self> {
        Self::new(ErrorCode::Unauthenticated, reason, message)
    }

    /// Creates a permission-denied error.
    pub fn permission_denied(
        reason: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self> {
        Self::new(ErrorCode::PermissionDenied, reason, message)
    }

    /// Creates a not-found error.
    pub fn not_found(reason: impl Into<String>, message: impl Into<String>) -> Result<Self> {
        Self::new(ErrorCode::NotFound, reason, message)
    }

    /// Creates an already-exists conflict error.
    pub fn already_exists(reason: impl Into<String>, message: impl Into<String>) -> Result<Self> {
        Self::new(ErrorCode::AlreadyExists, reason, message)
    }

    /// Creates an unavailable error.
    pub fn unavailable(reason: impl Into<String>, message: impl Into<String>) -> Result<Self> {
        Self::new(ErrorCode::Unavailable, reason, message)
    }

    /// Creates a redacted internal error with the supplied source.
    pub fn internal<E>(source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            code: ErrorCode::Internal,
            reason: "internal-error".to_owned(),
            message: "Internal server error".to_owned(),
            data: None,
            severity: Severity::Error,
            source: Some(Arc::new(source)),
            request_id: None,
        }
    }

    /// Returns the standard error code.
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns the stable application reason code.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the client-safe message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns optional structured data.
    pub fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }

    /// Returns the logging severity.
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the HTTP status code number.
    pub const fn status_code_u16(&self) -> u16 {
        self.code.status_code_u16()
    }

    /// Returns the request identifier when one is attached.
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Adds structured data to the error.
    pub fn with_data<T>(mut self, data: T) -> Result<Self>
    where
        T: Serialize,
    {
        self.data = Some(match serde_json::to_value(data) {
            Ok(value) => value,
            Err(source) => return Err(Error::DataSerialization { source }),
        });
        Ok(self)
    }

    /// Adds structured data or returns a redacted internal error on failure.
    pub fn with_data_or_internal<T>(mut self, data: T) -> Self
    where
        T: Serialize,
    {
        self.data = match serde_json::to_value(data) {
            Ok(value) => Some(value),
            Err(source) => return Self::internal(source),
        };
        self
    }

    /// Adds a server-side source error.
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Arc::new(source));
        self
    }

    /// Adds a logging severity.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Adds a request identifier.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Ensures the error has a request identifier.
    pub fn ensure_request_id<G>(mut self, generator: &G) -> Self
    where
        G: RequestIdGenerator,
    {
        if self.request_id.is_none() {
            self.request_id = Some(generator.next_request_id());
        }
        self
    }

    /// Ensures the error has a UUID request identifier.
    pub fn ensure_uuid_request_id(self) -> Self {
        self.ensure_request_id(&UuidRequestIdGenerator)
    }

    /// Returns the serializable client-facing error envelope.
    pub fn output(&self) -> ErrorOutput {
        ErrorOutput::from(self)
    }

    /// Returns all server-side source errors.
    pub fn sources(&self) -> Vec<&(dyn StdError + 'static)> {
        let mut sources = Vec::new();
        let mut current: &dyn StdError = self;
        while let Some(source) = current.source() {
            sources.push(source);
            current = source;
        }
        sources
    }

    /// Returns a source-chain report for server logs.
    pub fn report(&self) -> String {
        let mut output = format!("{}: {}", self.reason, self.message);
        let mut current: &dyn StdError = self;
        while let Some(source) = current.source() {
            output.push_str("\n  Caused by: ");
            output.push_str(&source.to_string());
            current = source;
        }
        output
    }

    /// Returns log metadata for framework adapters.
    pub fn log_record(&self) -> RpcErrorLog {
        RpcErrorLog {
            code: self.code,
            reason: self.reason.clone(),
            status: self.status_code_u16(),
            severity: self.severity,
            request_id: self.request_id.clone(),
            data: self.data.clone(),
            report: self.report(),
        }
    }
}

impl Display for RpcError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} -> {}", self.code, self.reason, self.message)
    }
}

impl StdError for RpcError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn StdError + 'static))
    }
}

fn validate_reason(value: &str) -> Result<()> {
    if is_valid_reason_code(value) {
        return Ok(());
    }
    Err(Error::InvalidReasonCode {
        value: value.to_owned(),
    })
}
