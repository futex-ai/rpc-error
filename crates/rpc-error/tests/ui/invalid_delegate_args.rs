use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[delegate(reason = "nested")]
    #[error("[ui] invalid delegate args {0}")]
    InvalidDelegateArgs(NestedError),
}

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum NestedError {
    #[not_found("nested")]
    #[error("[ui] nested")]
    Nested,
}

fn main() {}
