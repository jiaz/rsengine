use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::Path,
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    response::{Html, IntoResponse, Response},
    Extension, Json,
};
use common::{AppError, ErrorCode, RouteConfig};
use serde::Serialize;
use tracing::debug;

use crate::{
    app::AppState,
    context::RequestContextExtractor,
    errors::{HandlerResult, HttpError},
};

#[derive(Serialize)]
struct HealthPayload {
    status: &'static str,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct ReadyPayload {
    status: &'static str,
    routes_loaded: usize,
}

pub async fn health(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let uptime = state.launched_at().elapsed();
    Json(HealthPayload {
        status: "ok",
        uptime_seconds: uptime.as_secs(),
    })
}

pub async fn readiness(Extension(state): Extension<AppState>) -> impl IntoResponse {
    Json(ReadyPayload {
        status: "ready",
        routes_loaded: state.route_count(),
    })
}

pub async fn metrics(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let metrics = state.metrics_handle().render();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4"),
        )],
        metrics,
    )
}

pub async fn render_route(
    Path(route_id): Path<String>,
    Extension(state): Extension<AppState>,
    RequestContextExtractor(context): RequestContextExtractor,
) -> HandlerResult<impl IntoResponse> {
    let route = state
        .route(&route_id)
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, format!("unknown route '{route_id}'")))
        .map_err(HttpError::from)?;

    debug!(
        route_id = %route_id,
        request_id = %context.trace.request_id,
        "render request received",
    );

    let runtime = state.runtime();
    let rendered = runtime
        .render(&route, &context)
        .await
        .map_err(HttpError::from)?;

    Ok(Html(rendered))
}

pub async fn not_found() -> Response {
    HttpError::from(AppError::new(ErrorCode::NotFound, "resource not found")).into_response()
}

pub fn register_process_metrics() {
    if let Ok(epoch) = SystemTime::now().duration_since(UNIX_EPOCH) {
        metrics::gauge!(
            "process_start_time_seconds",
            epoch.as_secs_f64(),
            "service" => "rsengine_server",
        );
    }
}

pub fn default_routes() -> Vec<RouteConfig> {
    vec![
        RouteConfig::new("home", "/"),
        RouteConfig::new("product", "/product/:id"),
    ]
}
