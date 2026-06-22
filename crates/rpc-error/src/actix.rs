//! Actix response adapter and middleware.

use std::future::{Ready, ready};

use ::actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use ::actix_web::http::StatusCode;
use ::actix_web::http::header::ContentType;
use ::actix_web::{Error as ActixError, HttpMessage, HttpResponse, Responder, ResponseError};
use futures_util::future::LocalBoxFuture;
use tracing::{error, warn};

use crate::{
    ErrorCode, RequestId, RequestIdGenerator, RpcError, RpcErrorLog, Severity,
    UuidRequestIdGenerator,
};

/// Header read by Actix middleware when an upstream request ID exists.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

tokio::task_local! {
    static CURRENT_REQUEST_ID: String;
}

impl ResponseError for RpcError {
    fn status_code(&self) -> StatusCode {
        status_code(self.status_code_u16())
    }

    fn error_response(&self) -> HttpResponse {
        let error = with_scoped_request_id(self.clone());
        let status = status_code(error.status_code_u16());
        let log_record = error.log_record();
        let mut response = HttpResponse::build(status)
            .insert_header(ContentType::json())
            .json(error.output());
        response.extensions_mut().insert(log_record);
        response
    }
}

/// Actix handler for unmatched routes.
pub async fn not_found_error_handler() -> impl Responder {
    RpcError::not_found("not-found", "Resource was not found")
        .unwrap_or_else(RpcError::internal)
        .error_response()
}

/// Actix middleware that logs standardized RPC errors.
#[derive(Clone, Debug, Default)]
pub struct RpcErrorLogger;

impl<S, B> Transform<S, ServiceRequest> for RpcErrorLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Error = ActixError;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Response = ServiceResponse<B>;
    type Transform = RpcErrorLoggerService<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RpcErrorLoggerService { service }))
    }
}

/// Service created by `RpcErrorLogger`.
pub struct RpcErrorLoggerService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RpcErrorLoggerService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Response = ServiceResponse<B>;

    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let path = request.path().to_owned();
        let method = request.method().to_string();
        let request_id = request_id_from_request(&request);
        request
            .extensions_mut()
            .insert(RequestId::new(request_id.clone()));
        let future = self.service.call(request);
        Box::pin(async move {
            let response = CURRENT_REQUEST_ID.scope(request_id.clone(), future).await?;
            if let Some(record) = response.response().extensions().get::<RpcErrorLog>() {
                log_record(&method, &path, record);
            } else if let Some(error) = response.response().error() {
                if let Some(error) = error.as_error::<RpcError>() {
                    let mut record = error.log_record();
                    if record.request_id.is_none() {
                        record.request_id = Some(request_id);
                    }
                    log_record(&method, &path, &record);
                } else {
                    error!(method, path, ?error, "Unhandled server error");
                }
            }
            Ok(response)
        })
    }
}

fn log_record(method: &str, path: &str, record: &RpcErrorLog) {
    match record.severity {
        Severity::Warn => warn!(
            method,
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
        ),
        Severity::Error => error!(
            method,
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
        ),
    }
}

fn status_code(status: u16) -> StatusCode {
    match StatusCode::from_u16(status) {
        Ok(status) => status,
        Err(_source) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<ErrorCode> for StatusCode {
    fn from(value: ErrorCode) -> Self {
        status_code(value.status_code_u16())
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

fn request_id_from_request(request: &ServiceRequest) -> String {
    let extension_request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|request_id| request_id.as_str().to_owned());
    if let Some(request_id) = extension_request_id {
        return request_id;
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
