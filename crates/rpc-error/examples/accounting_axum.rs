//! Accounting-style Axum migration example.

use axum::Router;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use rpc_error::{ErrorCode, RpcError, validate_unique_reasons};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum AccountingApiError {
    #[rpc_error(
        code = "bad-request",
        reason = "validation-failed",
        message = "Validation failed"
    )]
    #[error("[accounting-api] validation failed")]
    ValidationFailed,

    #[rpc_error(
        code = "unauthenticated",
        reason = "unauthorized",
        message = "Authentication required"
    )]
    #[error("[accounting-api] authentication required")]
    Unauthorized,

    #[rpc_error(
        code = "internal",
        reason = "internal-error",
        message = "Internal server error"
    )]
    #[error("[accounting-api] internal error")]
    Internal,
}

fn main() -> rpc_error::Result<()> {
    validate_unique_reasons(&AccountingApiError::reason_catalogs())?;
    let error: rpc_error::RpcError = AccountingApiError::ValidationFailed.into();
    let response = error.into_response();
    assert_eq!(
        response.status().as_u16(),
        ErrorCode::BadRequest.status_code_u16()
    );
    let app: Router<()> = Router::new()
        .route("/example", get(route_error))
        .layer(middleware::from_fn(rpc_error::axum::log_errors));
    let _ = app;
    Ok(())
}

async fn route_error() -> Result<&'static str, rpc_error::RpcError> {
    Err(AccountingApiError::ValidationFailed.into())
}
