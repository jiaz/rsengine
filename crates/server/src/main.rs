use std::{net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use runtime::{RenderRuntime, RuntimeConfig};
use server::{app::AppState, build_router, handlers, telemetry};

#[derive(Parser, Debug)]
#[command(author, version, about = "Rust SSR streaming server")]
struct Cli {
    /// Path to the JavaScript bundle that exports a `stream` handler.
    #[arg(long, value_name = "BUNDLE_PATH")]
    bundle: PathBuf,

    /// Friendly name used to tag logs and metrics for this runtime.
    #[arg(long, default_value = "rsengine")]
    runtime_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    telemetry::init_tracing().context("failed to initialise tracing")?;
    let _ = telemetry::init_metrics().context("failed to initialise metrics")?;

    let runtime_config = RuntimeConfig::new(cli.bundle).with_name(cli.runtime_name);
    let runtime =
        RenderRuntime::try_new(runtime_config).context("failed to initialise render runtime")?;
    handlers::register_process_metrics();

    let state = AppState::new(runtime);
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
