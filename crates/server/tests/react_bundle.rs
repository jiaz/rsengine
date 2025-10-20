use std::path::PathBuf;

use async_trait::async_trait;
use common::{AppError, RequestContext};
use http::{HeaderMap, Method};
use runtime::{RenderRuntime, ResponseWriter, RuntimeConfig};

#[derive(Default)]
struct CollectingWriter {
    chunks: Vec<String>,
}

#[async_trait]
impl ResponseWriter for CollectingWriter {
    async fn write(&mut self, chunk: String) -> Result<(), AppError> {
        self.chunks.push(chunk);
        Ok(())
    }
}

#[tokio::test]
async fn react_bundle_streams_html() {
    let bundle_path = std::env::var("RSENGINE_TEST_BUNDLE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("examples/react-ssr-stream/dist/app.bundle.js"));

    if !bundle_path.exists() {
        eprintln!(
            "skipping react_bundle_streams_html: bundle '{}' missing",
            bundle_path.display()
        );
        return;
    }

    let runtime = RenderRuntime::try_new(RuntimeConfig::new(&bundle_path)).expect("runtime");
    let context = RequestContext::from_http_parts(&Method::GET, "/stream", &HeaderMap::new());

    let mut writer = CollectingWriter::default();
    runtime
        .stream_response(&context, &mut writer)
        .await
        .expect("rendered chunks");

    assert!(writer
        .chunks
        .iter()
        .any(|chunk| chunk.contains("Streaming SSR response")));
    assert!(writer
        .chunks
        .iter()
        .any(|chunk| chunk.contains("Bundle Source")));
}
