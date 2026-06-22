use serde::{Deserialize, Serialize};

use rpc_error::RpcError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct UserData {
    pub(crate) id: u64,
    pub(crate) name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct NonCloneData {
    pub(crate) id: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct FailingSerializeData {
    pub(crate) id: u64,
}

impl Serialize for FailingSerializeData {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(<S::Error as serde::ser::Error>::custom(
            "failing data serialize",
        ))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("[derive-tests] non clone source")]
pub(crate) struct NonCloneSource;

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize, PartialEq)]
pub(crate) enum NestedError {
    #[rpc_error(code = "permission-denied", reason = "nested-denied")]
    #[error("[derive-tests] nested denied")]
    Denied,
}

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize, PartialEq)]
pub(crate) enum OtherNestedError {
    #[rpc_error(code = "not-found", reason = "other-nested-missing")]
    #[error("[derive-tests] other nested missing")]
    Missing,

    #[rpc_error(code = "bad-request", reason = "other-nested-data")]
    #[error("[derive-tests] other nested data")]
    Data(UserData),
}

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize, PartialEq)]
pub(crate) enum ApiError {
    #[rpc_error(code = "bad-request", reason = "validation-failed")]
    #[error("[derive-tests] validation failed")]
    ValidationFailed,

    #[not_found("user-not-found")]
    #[error("[derive-tests] user not found: {0:?}")]
    UserNotFound(UserData),

    #[bad_request("tuple-invalid")]
    #[error("[derive-tests] tuple invalid")]
    TupleInvalid(String, u32),

    #[already_exists("duplicate-user")]
    #[error("[derive-tests] duplicate user {id}")]
    DuplicateUser {
        /// User identifier.
        id: u64,
        /// User email.
        email: String,
    },

    #[bad_request("warn-only", severity = "warn", data = "omit")]
    #[error("[derive-tests] warn only")]
    WarnOnly(String),

    #[bad_request("loggable-data")]
    #[error("[derive-tests] loggable data")]
    LoggableData(UserData),

    #[internal("mapped-internal")]
    #[error("[derive-tests] mapped internal leaked {secret}")]
    MappedInternal { secret: String },

    #[internal("mapped-internal-public-data", data = "include")]
    #[error("[derive-tests] mapped internal public data")]
    MappedInternalPublicData { operation_id: String },

    #[bad_request("serde-shaped")]
    #[error("[derive-tests] serde shaped {id}")]
    SerdeShaped {
        #[serde(rename = "public_id")]
        id: u64,
        #[serde(skip)]
        secret: String,
        #[serde(default)]
        retryable: bool,
    },

    #[delegate]
    #[error("[derive-tests] nested {0}")]
    Nested(NestedError),

    #[delegate]
    #[error("[derive-tests] other nested {0}")]
    OtherNested(OtherNestedError),

    #[error("[derive-tests] hidden internal")]
    HiddenInternal,
}

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize, PartialEq)]
pub(crate) enum FirstCollisionError {
    #[bad_request("first-collision")]
    #[error("[derive-tests] first collision")]
    Shared { id: u64 },
}

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize, PartialEq)]
pub(crate) enum SecondCollisionError {
    #[bad_request("second-collision")]
    #[error("[derive-tests] second collision")]
    Shared { name: String },
}

#[derive(Debug, thiserror::Error, RpcError)]
pub(crate) enum NonCloneNestedError {
    #[not_found("non-clone-nested")]
    #[error("[derive-tests] non clone nested")]
    Missing,
}

#[derive(Debug, thiserror::Error, RpcError)]
pub(crate) enum NonCloneApiError {
    #[bad_request("non-clone-named")]
    #[error("[derive-tests] non clone named")]
    Named { data: NonCloneData },

    #[delegate]
    #[error("[derive-tests] non clone delegate {0}")]
    Nested(NonCloneNestedError),

    #[error("[derive-tests] non clone internal {0}")]
    Internal(NonCloneSource),
}

#[derive(Debug, thiserror::Error, RpcError)]
pub(crate) enum SerializationFailureError {
    #[bad_request("serialization-failed")]
    #[error("[derive-tests] serialization failed")]
    Data(FailingSerializeData),
}

#[derive(Debug, thiserror::Error, RpcError)]
pub(crate) enum DuplicateDelegateChildError {
    #[bad_request("delegated-duplicate")]
    #[error("[derive-tests] delegated duplicate child")]
    Child,
}

#[derive(Debug, thiserror::Error, RpcError)]
pub(crate) enum DuplicateDelegateParentError {
    #[not_found("delegated-duplicate")]
    #[error("[derive-tests] delegated duplicate parent")]
    Parent,

    #[delegate]
    #[error("[derive-tests] delegated duplicate child {0}")]
    Child(DuplicateDelegateChildError),
}
