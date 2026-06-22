use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[rpc_error(code = "bad-request", reason = "validation_failed")]
    #[error("[ui] invalid reason")]
    InvalidReason,
}

fn main() {}
