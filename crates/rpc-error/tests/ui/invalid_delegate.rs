use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[delegate]
    #[error("[ui] invalid delegate")]
    InvalidDelegate,
}

fn main() {}
