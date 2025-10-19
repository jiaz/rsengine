use std::{collections::HashMap, sync::Arc, time::Instant};

use axum::{http::Request, response::Response, routing::get, Router};
use common::RouteConfig;
use metrics::{histogram, increment_counter};
use metrics_exporter_prometheus::PrometheusHandle;
use runtime::RenderRuntime;
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{info_span, Span};

use crate::handlers;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

struct AppStateInner {
    runtime: Arc<RenderRuntime>,
    routes: HashMap<String, RouteConfig>,
    metrics_handle: PrometheusHandle,
    launched_at: Instant,
}

#[allow(dead_code)]
fn _assert_app_state_send_sync() {
    fn assert<T: Send + Sync>() {}
    assert::<AppState>();
}

#[allow(dead_code)]
fn _assert_router_service(router: Router) {
    use axum::{body::Body, http::Request};
    fn assert<T: tower::Service<Request<Body>>>() {}
    let _ = router;
    assert::<Router>();
}

impl AppState {
    pub fn new(
        runtime: RenderRuntime,
        routes: Vec<RouteConfig>,
        metrics_handle: PrometheusHandle,
        launched_at: Instant,
    ) -> Self {
        let routes_map = routes
            .into_iter()
            .map(|route| (route.id.clone(), route))
            .collect::<HashMap<_, _>>();

        Self(Arc::new(AppStateInner {
            runtime: Arc::new(runtime),
            routes: routes_map,
            metrics_handle,
            launched_at,
        }))
    }

    pub fn runtime(&self) -> Arc<RenderRuntime> {
        Arc::clone(&self.0.runtime)
    }

    pub fn route(&self, id: &str) -> Option<RouteConfig> {
        self.0.routes.get(id).cloned()
    }

    pub fn route_count(&self) -> usize {
        self.0.routes.len()
    }

    pub fn metrics_handle(&self) -> PrometheusHandle {
        self.0.metrics_handle.clone()
    }

    pub fn launched_at(&self) -> Instant {
        self.0.launched_at
    }
}

pub fn build_router(state: AppState) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("unknown")
                .to_string();

            info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                request_id = %request_id,
                status = tracing::field::Empty,
                latency_ms = tracing::field::Empty,
            )
        })
        .on_request(|_request: &Request<_>, _span: &Span| {
            tracing::debug!("request started");
        })
        .on_response(|response: &Response, latency: std::time::Duration, span: &Span| {
            let status = response.status();
            span.record("status", tracing::field::display(status));
            span.record("latency_ms", latency.as_secs_f64() * 1000.0);

            let status_label = status.as_str().to_owned();

            increment_counter!("http_requests_total", "status" => status_label.clone());
            histogram!("http_request_duration_seconds", latency.as_secs_f64(), "status" => status_label);

            tracing::debug!("request completed");
        })
        .on_failure(|error: ServerErrorsFailureClass, latency: std::time::Duration, span: &Span| {
            let error_label = error.to_string();
            span.record("status", tracing::field::display(&error_label));
            span.record("latency_ms", latency.as_secs_f64() * 1000.0);

            increment_counter!("http_requests_total", "status" => "error");
            histogram!("http_request_duration_seconds", latency.as_secs_f64(), "status" => "error");

            tracing::warn!(status = %error_label, latency_ms = latency.as_secs_f64() * 1000.0, "request failed");
        });

    let service_stack = ServiceBuilder::new()
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::new(
            http::header::HeaderName::from_static("x-request-id"),
        ))
        .layer(SetRequestIdLayer::new(
            http::header::HeaderName::from_static("x-request-id"),
            MakeRequestUuid::default(),
        ))
        .into_inner();

    Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::readiness))
        .route("/metrics", get(handlers::metrics))
        .route("/render/:route_id", get(handlers::render_route))
        .fallback(handlers::not_found)
        .layer(service_stack)
        .layer(axum::Extension(state))
}
