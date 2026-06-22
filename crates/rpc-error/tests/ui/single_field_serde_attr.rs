use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[bad_request("single-field-serde")]
    #[error("[ui] single field serde")]
    Single(#[serde(skip)] String),
}

fn main() {}
