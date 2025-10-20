use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    response::IntoResponse,
    Extension,
};
use bytes::Bytes;
use common::{AppError, ErrorCode};
use html_escape::encode_text;
use runtime::ResponseWriter;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tracing::{debug, error, warn};

use crate::{app::AppState, context::RequestContextExtractor, errors::HandlerResult};

pub async fn stream(
    Extension(state): Extension<AppState>,
    RequestContextExtractor(context): RequestContextExtractor,
) -> HandlerResult<impl IntoResponse> {
    debug!(request_id = %context.trace.request_id, "stream request received");

    let runtime = state.runtime();
    let (sender, receiver) = mpsc::channel::<Bytes>(16);
    let headers = [(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    )];

    let request_id = context.trace.request_id;
    tokio::spawn(async move {
        let mut writer = ChannelStreamWriter::new(sender);
        if let Err(err) = runtime.stream_response(&context, &mut writer).await {
            error!(request_id = %request_id, error = %err, "render runtime failed");
            if let Err(send_err) = writer.write_error(&err).await {
                warn!(
                    request_id = %request_id,
                    error = %send_err,
                    "failed to stream error chunk"
                );
            }
        }
    });

    let stream = ReceiverStream::new(receiver).map(|chunk| Ok::<Bytes, Infallible>(chunk));
    let body = Body::from_stream(stream);

    Ok((StatusCode::OK, headers, body))
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

struct ChannelStreamWriter {
    sender: mpsc::Sender<Bytes>,
}

impl ChannelStreamWriter {
    fn new(sender: mpsc::Sender<Bytes>) -> Self {
        Self { sender }
    }

    async fn send(&mut self, chunk: String) -> Result<(), AppError> {
        self.sender.send(Bytes::from(chunk)).await.map_err(|_| {
            AppError::new(
                ErrorCode::Internal,
                "response stream closed before chunk could be delivered",
            )
        })
    }

    async fn write_error(&mut self, error: &AppError) -> Result<(), AppError> {
        let chunk = format!(
            "<section><h2>Render Error</h2><pre>{}</pre></section>",
            encode_text(error.message())
        );
        self.send(chunk).await
    }
}

#[async_trait]
impl ResponseWriter for ChannelStreamWriter {
    async fn write(&mut self, chunk: String) -> Result<(), AppError> {
        self.send(chunk).await
    }
}
