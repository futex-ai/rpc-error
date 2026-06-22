use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[rpc_error(code = "bad-request", reason = "secret-rejected", data = "omti")]
    #[error("[ui] secret rejected")]
    SecretRejected { secret: String },
}

fn main() {}
