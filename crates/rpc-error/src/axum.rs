//! Axum response adapter and middleware.

use ::axum::Json;
use ::axum::extract::Request;
use ::axum::http::{Method, StatusCode};
use ::axum::middleware::Next;
use ::axum::response::{IntoResponse, Response};
use tracing::{error, warn};

use crate::{
    RequestId, RequestIdGenerator, RpcError, RpcErrorLog, Severity, UuidRequestIdGenerator,
};

/// Header read by Axum middleware when an upstream request ID exists.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

tokio::task_local! {
    static CURRENT_REQUEST_ID: String;
}

impl IntoResponse for RpcError {
    fn into_response(self) -> Response {
        let error = with_scoped_request_id(self);
        let status = status_code(error.status_code_u16());
        let log_record = error.log_record();
        let mut response = (status, Json(error.output())).into_response();
        response.extensions_mut().insert(log_record);
        response
    }
}

/// Axum middleware function that scopes request IDs and logs RPC errors.
pub async fn log_errors(mut request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let request_id = request_id_from_request(&request);
    request
        .extensions_mut()
        .insert(RequestId::new(request_id.clone()));
    let response = CURRENT_REQUEST_ID
        .scope(request_id, next.run(request))
        .await;
    log_response(&method, &path, &response);
    response
}

/// Logs a response when it carries standardized RPC error metadata.
pub fn log_response(method: &Method, path: &str, response: &Response) {
    let Some(record) = response.extensions().get::<RpcErrorLog>() else {
        return;
    };
    match record.severity {
        Severity::Warn => warn_rpc_error(method, path, record),
        Severity::Error => error_rpc_error(method, path, record),
    }
}

fn warn_rpc_error(method: &Method, path: &str, record: &RpcErrorLog) {
    warn!(
        method = %method,
        path,
        status = record.status,
        code = %record.code,
        reason = record.reason,
        severity = record.severity.as_str(),
        request_id = ?record.request_id,
        data = ?record.data,
        report = record.report,
        is_request_error = true,
        "RPC Error"
    );
}

fn error_rpc_error(method: &Method, path: &str, record: &RpcErrorLog) {
    error!(
        method = %method,
        path,
        status = record.status,
        code = %record.code,
        reason = record.reason,
        severity = record.severity.as_str(),
        request_id = ?record.request_id,
        data = ?record.data,
        report = record.report,
        is_request_error = true,
        "RPC Error"
    );
}

fn status_code(status: u16) -> StatusCode {
    match StatusCode::from_u16(status) {
        Ok(status) => status,
        Err(_source) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn with_scoped_request_id(error: RpcError) -> RpcError {
    if error.request_id().is_some() {
        return error;
    }
    match CURRENT_REQUEST_ID.try_with(ToOwned::to_owned) {
        Ok(request_id) => error.with_request_id(request_id),
        Err(_source) => error.ensure_uuid_request_id(),
    }
}

fn request_id_from_request(request: &Request) -> String {
    if let Some(request_id) = request.extensions().get::<RequestId>() {
        return request_id.as_str().to_owned();
    }
    if let Some(request_id) = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
    {
        return request_id.to_owned();
    }
    UuidRequestIdGenerator.next_request_id()
}
