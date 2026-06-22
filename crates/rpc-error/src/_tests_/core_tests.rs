use std::str::FromStr;

use serde_json::json;

use crate::{
    Error, ErrorCode, ReasonDefinition, RpcError, UuidRequestIdGenerator, validate_unique_reasons,
};

#[derive(Debug, thiserror::Error)]
#[error("[rpc-error/tests] secret detail")]
struct SecretSource;

struct FailsSerialization;

impl serde::Serialize for FailsSerialization {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("serialization failed"))
    }
}

#[test]
fn standard_codes_serialize_as_kebab_case() {
    let value =
        serde_json::to_value(ErrorCode::FailedPrecondition).expect("serialize standard error code");

    assert_eq!(value, json!("failed-precondition"));
    assert_eq!(
        ErrorCode::from_str("failed-precondition").expect("parse standard error code"),
        ErrorCode::FailedPrecondition
    );
}

#[test]
fn standard_codes_map_to_http_statuses() {
    let mappings = [
        (ErrorCode::BadRequest, 400),
        (ErrorCode::InvalidArgument, 400),
        (ErrorCode::FailedPrecondition, 400),
        (ErrorCode::OutOfRange, 400),
        (ErrorCode::Unauthenticated, 401),
        (ErrorCode::PermissionDenied, 403),
        (ErrorCode::NotFound, 404),
        (ErrorCode::Aborted, 409),
        (ErrorCode::AlreadyExists, 409),
        (ErrorCode::ResourceExhausted, 429),
        (ErrorCode::Cancelled, 406),
        (ErrorCode::PayloadTooLarge, 413),
        (ErrorCode::Unavailable, 503),
        (ErrorCode::Internal, 500),
        (ErrorCode::DeadlineExceeded, 504),
    ];

    for (code, status) in mappings {
        assert_eq!(code.status_code_u16(), status);
    }
}

#[test]
fn envelope_snapshot_includes_standard_fields() {
    let error = RpcError::not_found("book-not-found", "Book was not found")
        .expect("create not found error")
        .with_data(json!({ "book_id": "book_123" }))
        .expect("attach structured data")
        .with_request_id("req_123");

    insta::assert_json_snapshot!("standard_error_envelope", error.output());
}

#[test]
fn internal_errors_are_redacted_but_keep_report() {
    let error = RpcError::internal(SecretSource);

    assert_eq!(error.code(), ErrorCode::Internal);
    assert_eq!(error.reason(), "internal-error");
    assert_eq!(error.message(), "Internal server error");
    assert!(error.output().error.data.is_none());
    assert!(error.report().contains("secret detail"));
}

#[test]
fn invalid_reason_codes_are_rejected() {
    let result = RpcError::bad_request("validation_failed", "Validation failed");

    assert!(matches!(result, Err(Error::InvalidReasonCode { .. })));
}

#[test]
fn data_serialization_failure_is_typed() {
    let result = RpcError::bad_request("bad-data", "Bad data")
        .expect("create error")
        .with_data(FailsSerialization);

    assert!(matches!(result, Err(Error::DataSerialization { .. })));
}

#[test]
fn data_serialization_failure_can_be_redacted_to_internal() {
    let error = RpcError::bad_request("bad-data", "Bad data")
        .expect("create error")
        .with_data_or_internal(FailsSerialization);

    assert_eq!(error.code(), ErrorCode::Internal);
    assert_eq!(error.message(), "Internal server error");
}

#[test]
fn request_id_generation_fills_missing_id() {
    let error = RpcError::bad_request("validation-failed", "Validation failed")
        .expect("create error")
        .ensure_request_id(&UuidRequestIdGenerator);

    assert!(error.request_id().is_some());
}

#[test]
fn reason_catalog_detects_duplicates() {
    const FIRST: &[ReasonDefinition] = &[ReasonDefinition {
        code: ErrorCode::BadRequest,
        reason: "validation-failed",
    }];
    const SECOND: &[ReasonDefinition] = &[ReasonDefinition {
        code: ErrorCode::NotFound,
        reason: "validation-failed",
    }];

    let result = validate_unique_reasons(&[FIRST, SECOND]);

    assert!(matches!(result, Err(Error::DuplicateReasonCode { .. })));
}
