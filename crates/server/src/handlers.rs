use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};

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
use futures_util::stream;
use tracing::debug;

use crate::{
    app::AppState,
    context::RequestContextExtractor,
    errors::{HandlerResult, HttpError},
};

pub async fn stream(
    Extension(state): Extension<AppState>,
    RequestContextExtractor(context): RequestContextExtractor,
) -> HandlerResult<impl IntoResponse> {
    debug!(request_id = %context.trace.request_id, "stream request received");

    let runtime = state.runtime();
    let chunks = runtime
        .stream_response(&context)
        .await
        .map_err(HttpError::from)?;

    let stream = stream::iter(
        chunks
            .into_iter()
            .map(|chunk| Ok::<Bytes, Infallible>(Bytes::from(chunk))),
    );
    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        )],
        body,
    ))
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
