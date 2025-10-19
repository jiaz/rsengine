use axum::response::{IntoResponse, Response};
use axum::Json;
use common::{AppError, ErrorCode};
use serde::Serialize;

/// Wrapper type that enables converting `AppError` into HTTP responses.
#[derive(Debug)]
pub struct HttpError(pub AppError);

pub type HandlerResult<T> = Result<T, HttpError>;

impl From<AppError> for HttpError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

/// Standard HTTP error response envelope.
#[derive(Serialize)]
struct ErrorResponse {
    code: ErrorCode,
    message: String,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = self.0.status_code();
        let body = ErrorResponse {
            code: self.0.code(),
            message: self.0.message().to_string(),
        };

        (status, Json(body)).into_response()
    }
}
