# rsengine

rsengine is a Rust-based server-side rendering service that pairs an Axum HTTP edge with a pluggable JavaScript runtime. The current milestone delivers the initial HTTP surface, observability scaffolding, and a stub runtime for future V8 integration.

## Workspace Layout

- `crates/common`: Shared domain types (`RouteConfig`, `AppError`, request context helpers).
- `crates/runtime`: Placeholder renderer facade that will later wrap the V8 isolate manager.
- `crates/server`: Axum HTTP server exposing health, readiness, metrics, and render endpoints.

## Getting Started

### Prerequisites

- Rust 1.75+ (a `rust-toolchain.toml` is provided to pin the stable channel with `rustfmt` and `clippy`).

### Install Dependencies

The project uses Cargo workspaces. Fetch crates and build artifacts:

```bash
cargo fetch
```

### Run the Server

```bash
cargo run -p server
```

The server listens on `0.0.0.0:3000` by default. Adjust the port via the `PORT` environment variable.

### Available Endpoints

- `GET /health` – Basic liveness probe including process uptime.
- `GET /ready` – Readiness probe reporting loaded routes.
- `GET /metrics` – Prometheus exposition endpoint.
- `GET /render/:route_id` – Placeholder render path that currently returns static HTML.

### Run Tests

```bash
cargo test
```

Integration tests exercise the handler logic directly to validate health, readiness, and render behaviour. Unit tests cover request context parsing and runtime stubs.

## Observability

The server wires `tracing` with configurable `RUST_LOG`, emits structured spans via `tower-http`'s `TraceLayer`, and registers Prometheus counters/histograms for HTTP request volume and latency.

## Next Steps

Future milestones will populate the runtime crate with a V8 isolate pool, extend the data orchestration layer, and hook the render bridge into real bundle management.
