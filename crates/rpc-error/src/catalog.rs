//! Reason-code catalog support.

use std::collections::BTreeSet;

use crate::{ErrorCode, Result};

/// One reason-code definition for a typed error contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReasonDefinition {
    /// Standard error code paired with the reason.
    pub code: ErrorCode,
    /// Stable application reason code.
    pub reason: &'static str,
}

/// Provides reason-code definitions for a typed error enum.
pub trait ReasonCatalog {
    /// Returns all directly mapped reason definitions for this error contract.
    fn reason_definitions() -> &'static [ReasonDefinition];
}

/// Provides all reason-code catalogs reachable from a typed error enum.
pub trait ReasonCatalogs {
    /// Returns direct and delegated reason-code catalogs for this error contract.
    fn reason_catalogs() -> Vec<&'static [ReasonDefinition]>;
}

/// Validates that all provided reason definitions are unique.
pub fn validate_unique_reasons(catalogs: &[&'static [ReasonDefinition]]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for catalog in catalogs {
        for definition in *catalog {
            if !seen.insert(definition.reason) {
                return Err(crate::Error::DuplicateReasonCode {
                    reason: definition.reason.to_owned(),
                });
            }
        }
    }
    Ok(())
}

/// Returns whether a reason code is lowercase kebab-case ASCII.
pub(crate) fn is_valid_reason_code(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    let mut previous_dash = false;
    for byte in value.bytes() {
        let valid = byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
        if !valid {
            return false;
        }
        if byte == b'-' {
            if previous_dash {
                return false;
            }
            previous_dash = true;
            continue;
        }
        previous_dash = false;
    }
    !previous_dash
}
