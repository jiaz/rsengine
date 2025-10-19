use std::{net::SocketAddr, time::Instant};

use anyhow::{Context, Result};
use runtime::{RenderRuntime, RuntimeConfig};
use server::{app::AppState, build_router, handlers, telemetry};

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_tracing().context("failed to initialise tracing")?;
    let metrics_handle = telemetry::init_metrics().context("failed to initialise metrics")?;

    let runtime = RenderRuntime::new(RuntimeConfig::default());
    let routes = handlers::default_routes();
    let launched_at = Instant::now();
    handlers::register_process_metrics();

    let state = AppState::new(runtime, routes, metrics_handle, launched_at);
    let router = build_router(state);

    let addr = bind_address();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;

    tracing::info!("server listening on http://{addr}");

    let service = router.into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener, service)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server terminated unexpectedly")?;

    Ok(())
}

fn bind_address() -> SocketAddr {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    SocketAddr::from(([0, 0, 0, 0], port))
}

#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("received Ctrl+C, initiating shutdown");
        }
        _ = sigterm.recv() => {
            tracing::info!("received SIGTERM, initiating shutdown");
        }
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for Ctrl+C");
    tracing::info!("received Ctrl+C, initiating shutdown");
}
