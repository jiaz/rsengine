use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use common::{AppError, ErrorCode, RequestContext};
use serde_json::json;
use tokio::fs as tokio_fs;
use tracing::debug;

/// Configuration parameters for the render runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Friendly name used in logs to differentiate runtime pools.
    pub name: String,
    /// Path to the JavaScript bundle that exposes the `stream` handler.
    pub bundle_path: PathBuf,
}

impl RuntimeConfig {
    /// Creates a new runtime configuration for the provided bundle.
    pub fn new(bundle_path: impl Into<PathBuf>) -> Self {
        Self {
            name: "default".to_string(),
            bundle_path: bundle_path.into(),
        }
    }

    /// Overrides the human readable name for the runtime.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

/// Thin wrapper around an SSR runtime bundle that can stream responses.
#[derive(Debug, Clone)]
pub struct RenderRuntime {
    config: Arc<RuntimeConfig>,
}

impl RenderRuntime {
    /// Validates the bundle and constructs a new runtime.
    pub fn try_new(config: RuntimeConfig) -> Result<Self, AppError> {
        validate_bundle(&config.bundle_path)?;
        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Returns the canonical bundle path currently loaded.
    pub fn bundle_path(&self) -> &Path {
        &self.config.bundle_path
    }

    /// Produces HTML fragments that will be streamed to the client.
    pub async fn stream_response<W>(
        &self,
        context: &RequestContext,
        writer: &mut W,
    ) -> Result<(), AppError>
    where
        W: ResponseWriter,
    {
        debug!(
            request_id = %context.trace.request_id,
            bundle = %self.config.bundle_path.display(),
            runtime = %self.config.name,
            "render runtime invoked",
        );

        let script = tokio_fs::read_to_string(&self.config.bundle_path)
            .await
            .map_err(|err| {
                AppError::new(
                    ErrorCode::Internal,
                    format!(
                        "failed to read bundle '{}'",
                        self.config.bundle_path.display()
                    ),
                )
                .with_source(err)
            })?;

        if !script.contains("stream") {
            return Err(AppError::new(
                ErrorCode::BadRequest,
                "bundle does not define a `stream` handler",
            ));
        }

        let request_snapshot = serde_json::to_string_pretty(&json!({
            "method": context.method,
            "path": context.path,
            "headers": context.headers,
            "cookies": context.cookies,
            "trace": {
                "request_id": context.trace.request_id,
                "trace_id": context.trace.trace_id,
                "parent_trace_id": context.trace.parent_trace_id,
            }
        }))
        .map_err(|err| {
            AppError::new(ErrorCode::Internal, "failed to serialise request context")
                .with_source(err)
        })?;

        let chunks = [
            "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">".to_string(),
            format!("<title>{}</title></head><body>", self.config.name),
            format!(
                "<h1>Streaming SSR response</h1><p>Bundle: {}</p>",
                self.config.bundle_path.display()
            ),
            format!(
                "<section><h2>Request Context</h2><pre>{}</pre></section>",
                request_snapshot
            ),
            "<section><h2>Bundle Source</h2><pre>".to_string(),
            html_escape::encode_text(&script).to_string(),
            "</pre></section><script>// stream handler executed inside V8 in future milestones</script>"
                .to_string(),
            "</body></html>".to_string(),
        ];

        for chunk in chunks {
            writer.write(chunk).await?;
        }

        Ok(())
    }
}
/// Abstraction over a streaming sink that receives rendered HTML chunks.
#[async_trait]
pub trait ResponseWriter: Send {
    /// Writes the provided chunk to the underlying sink.
    async fn write(&mut self, chunk: String) -> Result<(), AppError>;
}

fn validate_bundle(path: &Path) -> Result<(), AppError> {
    let metadata = fs::metadata(path).map_err(|err| {
        AppError::new(
            ErrorCode::BadRequest,
            format!("bundle '{}' could not be read", path.display()),
        )
        .with_source(err)
    })?;

    if !metadata.is_file() {
        return Err(AppError::new(
            ErrorCode::BadRequest,
            format!("bundle '{}' is not a file", path.display()),
        ));
    }

    let contents = fs::read_to_string(path).map_err(|err| {
        AppError::new(
            ErrorCode::Internal,
            format!("failed to load bundle '{}'", path.display()),
        )
        .with_source(err)
    })?;

    if !contents.contains("stream") {
        return Err(AppError::new(
            ErrorCode::BadRequest,
            format!("bundle '{}' is missing a `stream` export", path.display()),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, Method};
    use tempfile::NamedTempFile;

    struct CollectingWriter {
        chunks: Vec<String>,
    }

    impl CollectingWriter {
        fn new() -> Self {
            Self { chunks: Vec::new() }
        }
    }

    #[async_trait]
    impl ResponseWriter for CollectingWriter {
        async fn write(&mut self, chunk: String) -> Result<(), AppError> {
            self.chunks.push(chunk);
            Ok(())
        }
    }

    #[tokio::test]
    async fn runtime_streams_chunks() {
        let mut bundle = NamedTempFile::new().expect("tmp file");
        std::io::Write::write_all(
            &mut bundle,
            b"export function stream(ctx) { ctx.write('<div>hello</div>'); }",
        )
        .expect("write bundle");

        let runtime = RenderRuntime::try_new(RuntimeConfig::new(bundle.path())).expect("runtime");

        let context = RequestContext::from_http_parts(&Method::GET, "/stream", &HeaderMap::new());

        let mut writer = CollectingWriter::new();
        runtime
            .stream_response(&context, &mut writer)
            .await
            .expect("chunks");

        assert!(writer
            .chunks
            .iter()
            .any(|chunk| chunk.contains("Streaming SSR response")));
    }

    #[test]
    fn validate_bundle_requires_stream_handler() {
        let mut bundle = NamedTempFile::new().expect("tmp file");
        std::io::Write::write_all(&mut bundle, b"console.log('no handler');")
            .expect("write bundle");

        let result = RenderRuntime::try_new(RuntimeConfig::new(bundle.path()));
        assert!(result.is_err());
    }
}
