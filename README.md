# rsengine

rsengine is a Rust-based server-side rendering service that pairs an Axum HTTP edge with a pluggable JavaScript runtime. The current milestone focuses on streaming SSR: the server exposes a single `/stream` endpoint and can be pointed at a user-provided JavaScript bundle that defines a `stream` handler.

## Workspace Layout

- `crates/common`: Shared domain types (`RouteConfig`, `AppError`, request context helpers).
- `crates/runtime`: Placeholder renderer facade that validates/loading bundles and emits HTML chunks.
- `crates/server`: Axum HTTP server exposing the streaming endpoint and wiring telemetry.

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
cargo run -p server -- --bundle ./examples/hello.bundle.js
```

The `--bundle` flag points to a JavaScript module that exports a `stream` function. The function is not executed yet (the current runtime is a stub), but the bundle is validated and its contents are streamed back as part of the response. The server listens on `0.0.0.0:3000` by default; adjust the port via the `PORT` environment variable.

For a richer demo that produces a bundle capable of React 18 streaming SSR, see [`examples/react-ssr-stream`](examples/react-ssr-stream/README.md).

### Streaming Endpoint

- `GET /stream` – Streams an HTML response that includes request metadata and the referenced bundle contents.

### Run Tests

```bash
cargo test
```

Integration tests exercise the streaming handler directly. Unit tests cover request context parsing and runtime bundle validation.

## Observability

The server wires `tracing` with configurable `RUST_LOG`, emits structured spans via `tower-http`'s `TraceLayer`, and registers Prometheus counters/histograms for HTTP request volume and latency.

## Next Steps

Future milestones will populate the runtime crate with a V8 isolate pool, extend the data orchestration layer, and hook the render bridge into real bundle management.
