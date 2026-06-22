//! Error severity levels used by logging adapters.

use serde::{Deserialize, Serialize};

/// Logging severity for standardized request errors.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Expected request failure that should not page operators.
    Warn,
    /// Unexpected or operational failure that should be logged as an error.
    #[default]
    Error,
}

impl Severity {
    /// Returns the stable lowercase string for the severity.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
