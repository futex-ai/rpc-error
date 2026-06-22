use rpc_error::RpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[bad_request("first-contract")]
    #[not_found("second-contract")]
    #[error("[ui] duplicate variant attributes")]
    DuplicateAttributes,
}

fn main() {}
