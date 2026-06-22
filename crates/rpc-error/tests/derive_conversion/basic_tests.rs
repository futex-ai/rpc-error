use serde_json::json;

use rpc_error::{Error, ErrorCode, ErrorOutput, ReasonCatalog, Severity};

use crate::support::{
    ApiError, FailingSerializeData, FirstCollisionError, NonCloneApiError, NonCloneData,
    NonCloneNestedError, NonCloneSource, SecondCollisionError, SerializationFailureError, UserData,
};

#[test]
fn converts_unit_variant_to_rpc_error() {
    let error: rpc_error::RpcError = ApiError::ValidationFailed.into();

    assert_eq!(error.code(), ErrorCode::BadRequest);
    assert_eq!(error.reason(), "validation-failed");
    assert_eq!(error.message(), "[derive-tests] validation failed");
    assert!(error.data().is_none());

    let recovered = ApiError::try_from(error).expect("recover typed error");
    assert_eq!(recovered, ApiError::ValidationFailed);
}

#[test]
fn unit_variant_rejects_unexpected_data() {
    let error = rpc_error::RpcError::new(
        ErrorCode::BadRequest,
        "validation-failed",
        "Validation failed",
    )
    .expect("construct rpc error")
    .with_data_or_internal(json!({ "field": "email" }));

    assert!(matches!(
        ApiError::try_from(error),
        Err(Error::UnexpectedData { reason, data })
            if reason == "validation-failed" && data == json!({ "field": "email" })
    ));
}

#[test]
fn converts_single_unnamed_field_roundtrip() {
    let data = UserData {
        id: 42,
        name: "Ada".to_owned(),
    };
    let error: rpc_error::RpcError = ApiError::UserNotFound(data.clone()).into();

    assert_eq!(error.code(), ErrorCode::NotFound);
    assert_eq!(error.reason(), "user-not-found");
    assert_eq!(
        error.data().cloned(),
        Some(json!({ "id": 42, "name": "Ada" }))
    );

    let recovered = ApiError::try_from(error).expect("recover typed error");
    assert_eq!(recovered, ApiError::UserNotFound(data));
}

#[test]
fn converts_tuple_and_named_fields_roundtrip() {
    let tuple: rpc_error::RpcError = ApiError::TupleInvalid("field".to_owned(), 7).into();
    assert_eq!(tuple.data().cloned(), Some(json!(["field", 7])));
    assert_eq!(
        ApiError::try_from(tuple).expect("recover tuple"),
        ApiError::TupleInvalid("field".to_owned(), 7)
    );

    let named_source = ApiError::DuplicateUser {
        id: 9,
        email: "ada@example.com".to_owned(),
    };
    let named: rpc_error::RpcError = named_source.clone().into();
    assert_eq!(
        named.data().cloned(),
        Some(json!({ "id": 9, "email": "ada@example.com" }))
    );
    assert_eq!(
        ApiError::try_from(named).expect("recover named"),
        named_source
    );
}

#[test]
fn severity_and_omitted_data_are_honored() {
    let error: rpc_error::RpcError = ApiError::WarnOnly("secret".to_owned()).into();

    assert_eq!(error.severity(), Severity::Warn);
    assert!(error.data().is_none());
    assert!(matches!(
        ApiError::try_from(error),
        Err(Error::MissingData { .. })
    ));
}

#[test]
fn structured_data_is_copied_to_logs() {
    let data = UserData {
        id: 7,
        name: "Visible".to_owned(),
    };
    let error: rpc_error::RpcError = ApiError::LoggableData(data).into();

    assert_eq!(
        error.data().cloned(),
        Some(json!({ "id": 7, "name": "Visible" }))
    );
    assert_eq!(
        error.log_record().data,
        Some(json!({ "id": 7, "name": "Visible" }))
    );
}

#[test]
fn internal_mapping_defaults_to_redacted_output() {
    let error: rpc_error::RpcError = ApiError::MappedInternal {
        secret: "db-password".to_owned(),
    }
    .into();

    assert_eq!(error.code(), ErrorCode::Internal);
    assert_eq!(error.reason(), "mapped-internal");
    assert_eq!(error.message(), "Internal server error");
    assert!(error.data().is_none());
    assert!(error.output().error.data.is_none());
    assert!(error.report().contains("db-password"));
}

#[test]
fn internal_mapping_can_explicitly_include_data() {
    let error: rpc_error::RpcError = ApiError::MappedInternalPublicData {
        operation_id: "op_123".to_owned(),
    }
    .into();

    assert_eq!(error.code(), ErrorCode::Internal);
    assert_eq!(error.message(), "Internal server error");
    assert_eq!(
        error.data().cloned(),
        Some(json!({ "operation_id": "op_123" }))
    );
}

#[test]
fn named_variant_data_honors_serde_field_attributes() {
    let error: rpc_error::RpcError = ApiError::SerdeShaped {
        id: 12,
        secret: "token".to_owned(),
        retryable: false,
    }
    .into();

    assert_eq!(
        error.data().cloned(),
        Some(json!({ "public_id": 12, "retryable": false }))
    );

    let recovered = ApiError::try_from(error).expect("recover serde shaped error");
    assert_eq!(
        recovered,
        ApiError::SerdeShaped {
            id: 12,
            secret: String::new(),
            retryable: false,
        }
    );
}

#[test]
fn named_variant_data_honors_serde_default_when_decoding() {
    let error = rpc_error::RpcError::bad_request("serde-shaped", "Serde shaped")
        .expect("construct serde shaped error")
        .with_data(json!({ "public_id": 99 }))
        .expect("attach serde shaped data");

    assert_eq!(
        ApiError::try_from(error).expect("recover serde shaped error"),
        ApiError::SerdeShaped {
            id: 99,
            secret: String::new(),
            retryable: false,
        }
    );
}

#[test]
fn generated_data_helpers_do_not_collide_across_enums() {
    let first: rpc_error::RpcError = FirstCollisionError::Shared { id: 1 }.into();
    let second: rpc_error::RpcError = SecondCollisionError::Shared {
        name: "two".to_owned(),
    }
    .into();

    assert_eq!(first.data().cloned(), Some(json!({ "id": 1 })));
    assert_eq!(second.data().cloned(), Some(json!({ "name": "two" })));
}

#[test]
fn non_clone_errors_convert_to_rpc_error() {
    let named: rpc_error::RpcError = NonCloneApiError::Named {
        data: NonCloneData { id: 55 },
    }
    .into();
    assert_eq!(named.data().cloned(), Some(json!({ "data": { "id": 55 } })));

    let delegated: rpc_error::RpcError =
        NonCloneApiError::Nested(NonCloneNestedError::Missing).into();
    assert_eq!(delegated.reason(), "non-clone-nested");

    let internal: rpc_error::RpcError = NonCloneApiError::Internal(NonCloneSource).into();
    assert_eq!(internal.code(), ErrorCode::Internal);
    assert!(internal.report().contains("non clone source"));
}

#[test]
fn data_serialization_failure_preserves_serialization_source() {
    let error: rpc_error::RpcError =
        SerializationFailureError::Data(FailingSerializeData { id: 1 }).into();

    assert_eq!(error.code(), ErrorCode::Internal);
    assert_eq!(error.reason(), "internal-error");
    assert!(error.data().is_none());
    assert!(error.report().contains("failing data serialize"));
    assert!(
        !error
            .report()
            .contains("[derive-tests] serialization failed")
    );
}

#[test]
fn unmapped_variants_become_internal_errors() {
    let error: rpc_error::RpcError = ApiError::HiddenInternal.into();

    assert_eq!(error.code(), ErrorCode::Internal);
    assert_eq!(error.reason(), "internal-error");
}

#[test]
fn error_output_converts_back_to_typed_error() {
    let output = ErrorOutput {
        error: rpc_error::ErrorDetail {
            code: ErrorCode::NotFound,
            reason: "user-not-found".to_owned(),
            message: "User was not found".to_owned(),
            request_id: Some("req_123".to_owned()),
            data: Some(json!({ "id": 99, "name": "Grace" })),
        },
    };

    assert_eq!(
        ApiError::try_from(output).expect("recover from output"),
        ApiError::UserNotFound(UserData {
            id: 99,
            name: "Grace".to_owned(),
        })
    );
}

#[test]
fn error_output_rejects_reason_code_mismatch() {
    let output = ErrorOutput {
        error: rpc_error::ErrorDetail {
            code: ErrorCode::Internal,
            reason: "user-not-found".to_owned(),
            message: "User was not found".to_owned(),
            request_id: None,
            data: Some(json!({ "id": 99, "name": "Grace" })),
        },
    };

    assert!(matches!(
        ApiError::try_from(output),
        Err(Error::CodeMismatch {
            reason,
            expected: ErrorCode::NotFound,
            actual: ErrorCode::Internal,
        }) if reason == "user-not-found"
    ));
}

#[test]
fn reason_catalog_is_generated() {
    let reasons = ApiError::reason_definitions();

    assert!(
        reasons
            .iter()
            .any(|item| item.reason == "validation-failed")
    );
    assert!(reasons.iter().any(|item| item.reason == "duplicate-user"));
}
