use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static TRACING_INIT: OnceLock<()> = OnceLock::new();
static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
static METRICS_GUARD: Mutex<()> = Mutex::new(());

/// Configures global tracing subscribers using `tracing-subscriber`.
pub fn init_tracing() -> Result<()> {
    if TRACING_INIT.get().is_some() {
        return Ok(());
    }

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));

    if let Err(err) = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_level(true).compact())
        .try_init()
    {
        // Ignore attempts to re-initialise the global subscriber.
        if TRACING_INIT.get().is_none() {
            return Err(anyhow::Error::from(err));
        }
    }

    let _ = TRACING_INIT.set(());
    Ok(())
}

/// Installs the global Prometheus recorder and returns a handle for scraping metrics.
pub fn init_metrics() -> Result<PrometheusHandle> {
    if let Some(handle) = METRICS_HANDLE.get() {
        return Ok(handle.clone());
    }

    let _lock = METRICS_GUARD.lock().expect("metrics init mutex poisoned");
    if let Some(handle) = METRICS_HANDLE.get() {
        return Ok(handle.clone());
    }

    let recorder = PrometheusBuilder::new()
        .set_quantiles(&[0.5, 0.9, 0.99])
        .context("invalid quantile configuration")?
        .install_recorder()
        .context("failed to install Prometheus recorder")?;

    metrics::describe_histogram!(
        "http_request_duration_seconds",
        "Latency distribution for HTTP requests handled by the server"
    );
    metrics::describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests processed by the server"
    );
    metrics::describe_gauge!(
        "process_start_time_seconds",
        "Unix timestamp for the process start time"
    );

    let _ = METRICS_HANDLE.set(recorder.clone());
    Ok(recorder)
}
