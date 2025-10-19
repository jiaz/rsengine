use std::time::Instant;

use axum::{
    extract::Path,
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    Extension,
};
use common::RequestContext;
use http_body_util::BodyExt;
use runtime::{RenderRuntime, RuntimeConfig};
use server::{app::AppState, context::RequestContextExtractor, handlers, telemetry};

fn test_state() -> AppState {
    telemetry::init_tracing().ok();
    let metrics = telemetry::init_metrics().expect("metrics initialisation");
    let runtime = RenderRuntime::new(RuntimeConfig::default());
    let routes = handlers::default_routes();
    AppState::new(runtime, routes, metrics, Instant::now())
}

#[tokio::test]
async fn health_endpoint_reports_ok() {
    let state = test_state();

    let response = handlers::health(Extension(state.clone()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}

#[tokio::test]
async fn readiness_reports_route_count() {
    let state = test_state();

    let response = handlers::readiness(Extension(state.clone()))
        .await
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        payload.get("status").and_then(|v| v.as_str()),
        Some("ready")
    );
    assert_eq!(
        payload.get("routes_loaded").and_then(|v| v.as_u64()),
        Some(2)
    );
}

#[tokio::test]
async fn render_route_returns_placeholder_markup() {
    let state = test_state();
    let path = Path("home".to_string());
    let context = RequestContextExtractor(RequestContext::from_http_parts(
        &Method::GET,
        "/render/home",
        &HeaderMap::new(),
    ));

    let response = handlers::render_route(path, Extension(state), context)
        .await
        .unwrap()
        .into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("SSR placeholder"));
    assert!(body.contains("Route: /"));
}
