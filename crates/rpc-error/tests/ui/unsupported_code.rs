use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[rpc_error(code = "teapot", reason = "short-and-stout")]
    #[error("[ui] unsupported code")]
    UnsupportedCode,
}

fn main() {}
