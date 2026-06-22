use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[rpc_error(code = "bad-request", reason = "validation-failed")]
    #[error("[ui] first")]
    First,

    #[rpc_error(code = "not-found", reason = "validation-failed")]
    #[error("[ui] second")]
    Second,
}

fn main() {}
