use std::io::Write;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use runtime::{RenderRuntime, RuntimeConfig};
use server::{app::AppState, build_router, telemetry};
use tempfile::NamedTempFile;
use tower::ServiceExt;

fn test_state(bundle_path: &std::path::Path) -> AppState {
    telemetry::init_tracing().ok();
    telemetry::init_metrics().ok();
    let runtime = RenderRuntime::try_new(RuntimeConfig::new(bundle_path)).expect("runtime");
    AppState::new(runtime)
}

fn write_bundle() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("bundle temp file");
    writeln!(
        file,
        "export function stream(context) {{ context.write('<div>Hello</div>'); }}"
    )
    .expect("write bundle");
    file
}

#[tokio::test]
async fn stream_endpoint_returns_chunked_html() {
    let bundle = write_bundle();
    let app = build_router(test_state(bundle.path()));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/stream")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(html.contains("Streaming SSR response"));
    assert!(html.contains("Hello"));
}
