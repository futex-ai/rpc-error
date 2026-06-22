#![cfg(feature = "actix")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use actix_web::body::to_bytes;
use actix_web::http::StatusCode;
use actix_web::{App, HttpResponse, HttpServer, test, web};
use serde_json::Value;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::prelude::*;

use rpc_error::{RpcError, Severity};

#[actix_web::test]
async fn response_error_sets_status_and_body() {
    let app = test::init_service(App::new().route("/fail", web::get().to(fail))).await;
    let response =
        test::call_service(&app, test::TestRequest::get().uri("/fail").to_request()).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "bad-request");
    assert_eq!(body["error"]["reason"], "validation-failed");
    assert!(body["error"]["request_id"].as_str().is_some());
}

#[actix_web::test]
async fn middleware_preserves_error_response() {
    let app = test::init_service(
        App::new()
            .wrap(rpc_error::actix::RpcErrorLogger)
            .route("/fail", web::get().to(fail)),
    )
    .await;
    let response =
        test::call_service(&app, test::TestRequest::get().uri("/fail").to_request()).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["reason"], "validation-failed");
}

#[actix_web::test]
async fn middleware_uses_request_id_header() {
    let app = test::init_service(
        App::new()
            .wrap(rpc_error::actix::RpcErrorLogger)
            .route("/fail", web::get().to(fail)),
    )
    .await;
    let request = test::TestRequest::get()
        .uri("/fail")
        .insert_header((rpc_error::actix::REQUEST_ID_HEADER, "req-from-header"))
        .to_request();
    let response = test::call_service(&app, request).await;

    let body = response_json(response).await;
    assert_eq!(body["error"]["request_id"], "req-from-header");
}

#[actix_web::test]
async fn middleware_logs_warn_severity_field() {
    let captured = CapturedEvents::default();
    let subscriber = tracing_subscriber::registry().with(captured.clone());
    let app = test::init_service(
        App::new()
            .wrap(rpc_error::actix::RpcErrorLogger)
            .route("/warn", web::get().to(warn_fail)),
    )
    .await;

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let response =
            test::call_service(&app, test::TestRequest::get().uri("/warn").to_request()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    let events = captured.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::WARN);
    assert_eq!(
        events[0].fields.get("severity").map(String::as_str),
        Some("warn")
    );
}

#[actix_web::test]
async fn not_found_helper_logs_from_response_extension() {
    let captured = CapturedEvents::default();
    let subscriber = tracing_subscriber::registry().with(captured.clone());
    let app = test::init_service(
        App::new()
            .wrap(rpc_error::actix::RpcErrorLogger)
            .default_service(web::to(rpc_error::actix::not_found_error_handler)),
    )
    .await;

    {
        let _guard = tracing::subscriber::set_default(subscriber);
        let response =
            test::call_service(&app, test::TestRequest::get().uri("/missing").to_request()).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    let events = captured.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, Level::ERROR);
    assert_eq!(
        events[0].fields.get("status").map(String::as_str),
        Some("404")
    );
}

#[actix_web::test]
async fn actix_smoke_server_returns_standard_error() {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(rpc_error::actix::RpcErrorLogger)
            .route("/fail", web::get().to(fail))
    })
    .bind(("127.0.0.1", 0))
    .expect("bind smoke server");
    let address = server.addrs()[0];
    let server = server.run();
    let handle = server.handle();
    actix_web::rt::spawn(server);

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
    handle.stop(true).await;
}

async fn fail() -> Result<HttpResponse, RpcError> {
    Err(
        RpcError::bad_request("validation-failed", "Validation failed")
            .expect("create bad request error"),
    )
}

async fn warn_fail() -> Result<HttpResponse, RpcError> {
    Err(
        RpcError::bad_request("validation-failed", "Validation failed")
            .expect("create bad request error")
            .with_severity(Severity::Warn),
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

async fn response_json<B>(response: actix_web::dev::ServiceResponse<B>) -> Value
where
    B: actix_web::body::MessageBody + 'static,
    B::Error: std::fmt::Debug,
{
    let bytes = to_bytes(response.into_body())
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("decode response JSON")
}
