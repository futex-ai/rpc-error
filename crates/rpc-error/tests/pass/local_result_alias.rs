use serde::{Deserialize, Serialize};

use rpc_error::{ErrorCode, RpcError};

mod shadowed_scope {
    use super::*;

    type Result<T> = std::result::Result<T, String>;

    #[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize, PartialEq)]
    pub enum ShadowedError {
        #[bad_request("shadowed-result")]
        #[error("[pass] shadowed result")]
        Shadowed,
    }

    pub fn convert(error: rpc_error::RpcError) -> std::result::Result<ShadowedError, rpc_error::Error> {
        ShadowedError::try_from(error)
    }

    pub fn local_result() -> Result<()> {
        Ok(())
    }
}

fn main() {
    let error = rpc_error::RpcError::new(
        ErrorCode::BadRequest,
        "shadowed-result",
        "Shadowed result",
    )
    .expect("build rpc error");

    let recovered = shadowed_scope::convert(error).expect("recover typed error");
    assert_eq!(recovered, shadowed_scope::ShadowedError::Shadowed);
    shadowed_scope::local_result().expect("local result alias still works");
}
