use rpc_error::{Error, ErrorCode, RpcError};

use crate::support::{ApiError, NestedError, OtherNestedError};

#[test]
fn delegate_variants_roundtrip_by_unknown_reason() {
    let error: rpc_error::RpcError = ApiError::Nested(NestedError::Denied).into();

    assert_eq!(error.reason(), "nested-denied");
    assert_eq!(
        ApiError::try_from(error).expect("recover delegated"),
        ApiError::Nested(NestedError::Denied)
    );
}

#[test]
fn multiple_delegate_variants_are_tried_in_order() {
    let error: rpc_error::RpcError = ApiError::OtherNested(OtherNestedError::Missing).into();

    assert_eq!(error.reason(), "other-nested-missing");
    assert_eq!(
        ApiError::try_from(error).expect("recover second delegate"),
        ApiError::OtherNested(OtherNestedError::Missing)
    );
}

#[test]
fn delegate_fallback_preserves_decode_errors() {
    let error = RpcError::bad_request("other-nested-data", "Other nested data")
        .expect("create delegated bad request");

    assert!(matches!(
        ApiError::try_from(error),
        Err(Error::MissingData { reason }) if reason == "other-nested-data"
    ));
}

#[test]
fn delegate_fallback_preserves_code_mismatch_errors() {
    let error = RpcError::new(
        ErrorCode::Internal,
        "other-nested-missing",
        "Other nested missing",
    )
    .expect("create mismatched delegated error");

    assert!(matches!(
        ApiError::try_from(error),
        Err(Error::CodeMismatch {
            reason,
            expected: ErrorCode::NotFound,
            actual: ErrorCode::Internal,
        }) if reason == "other-nested-missing"
    ));
}
