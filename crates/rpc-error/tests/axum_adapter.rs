#![cfg(feature = "axum")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use serde_json::Value;
use tower::ServiceExt;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::prelude::*;

use rpc_error::{RpcError, Severity};

#[tokio::test]
async fn into_response_sets_status_body_and_log_extension() {
    let response = RpcError::not_found("book-not-found", "Book was not found")
        .expect("create not found error")
        .into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(
        response
            .extensions()
            .get::<rpc_error::RpcErrorLog>()
            .is_some()
    );
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "not-found");
    assert_eq!(body["error"]["reason"], "book-not-found");
    assert!(body["error"]["request_id"].as_str().is_some());
}

#[tokio::test]
async fn middleware_preserves_error_response() {
    let app = Router::new()
        .route("/fail", get(fail))
        .layer(middleware::from_fn(rpc_error::axum::log_errors));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fail")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call router");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["reason"], "validation-failed");
}

#[tokio::test]
async fn middleware_uses_request_id_header() {
    let app = Router::new()
        .route("/fail", get(fail))
        .layer(middleware::from_fn(rpc_error::axum::log_errors));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/fail")
                .header(rpc_error::axum::REQUEST_ID_HEADER, "req-from-header")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call router");

    let body = response_json(response).await;
    assert_eq!(body["error"]["request_id"], "req-from-header");
}

#[tokio::test]
async fn log_response_emits_warn_severity_field() {
    let captured = CapturedEvents::default();
    let subscriber = tracing_subscriber::registry().with(captured.clone());
    tracing::subscriber::with_default(subscriber, || {
        let response = RpcError::bad_request("validation-failed", "Validation failed")
            .expect("create bad request error")
            .with_severity(Severity::Warn)
            .into_response();
        rpc_error::axum::log_response(&Method::GET, "/fail", &response);
    });

    let events = captured.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::WARN);
    assert_eq!(
        events[0].fields.get("severity").map(String::as_str),
        Some("warn")
    );
}

#[tokio::test]
async fn axum_smoke_server_returns_standard_error() {
    let app = Router::new()
        .route("/fail", get(fail))
        .layer(middleware::from_fn(rpc_error::axum::log_errors));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind smoke server");
    let address = listener.local_addr().expect("server address");
    let server = tokio::spawn(async move { axum::serve(listener, app).await });

    let response = reqwest::get(format!("http://{address}/fail"))
        .await
        .expect("call smoke server");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body = response
        .json::<Value>()
        .await
        .expect("decode smoke response");
    assert_eq!(body["error"]["code"], "bad-request");
    assert_eq!(body["error"]["reason"], "validation-failed");
    server.abort();
}

async fn fail() -> Result<&'static str, RpcError> {
    Err(
        RpcError::bad_request("validation-failed", "Validation failed")
            .expect("create bad request error"),
    )
}

#[derive(Clone, Default)]
struct CapturedEvents {
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl CapturedEvents {
    fn events(&self) -> Vec<CapturedEvent> {
        self.events.lock().expect("captured events lock").clone()
    }
}

impl<S> Layer<S> for CapturedEvents
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        self.events
            .lock()
            .expect("captured events lock")
            .push(CapturedEvent {
                level: *event.metadata().level(),
                fields: visitor.fields,
            });
    }
}

#[derive(Clone, Debug)]
struct CapturedEvent {
    level: Level,
    fields: BTreeMap<String, String>,
}

#[derive(Default)]
struct FieldVisitor {
    fields: BTreeMap<String, String>,
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name().to_owned(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_owned(), value.to_owned());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_owned(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_owned(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_owned(), value.to_string());
    }
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("decode response JSON")
}
