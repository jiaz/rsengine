use std::fmt;

use anyhow::Error as AnyError;
use http::StatusCode;
use serde::Serialize;

/// High-level classification for application errors.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    BadRequest,
    NotFound,
    UpstreamFailure,
    Internal,
}

impl ErrorCode {
    /// Converts the error code into an HTTP status code used when surfacing errors over HTTP.
    pub fn status_code(self) -> StatusCode {
        match self {
            ErrorCode::BadRequest => StatusCode::BAD_REQUEST,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::UpstreamFailure => StatusCode::BAD_GATEWAY,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Canonical application error type shared between crates.
#[derive(Debug)]
pub struct AppError {
    code: ErrorCode,
    message: String,
    source: Option<AnyError>,
}

impl AppError {
    /// Creates a new error with the provided code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Attaches a source error for additional context.
    pub fn with_source(mut self, source: impl Into<AnyError>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Returns the public error code.
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns the user-safe error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the HTTP status code associated with this error.
    pub fn status_code(&self) -> StatusCode {
        self.code.status_code()
    }

    /// Provides a serialisable payload that can be sent back to HTTP clients.
    pub fn as_payload(&self) -> ErrorPayload<'_> {
        ErrorPayload {
            code: self.code,
            message: &self.message,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|err| err.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Standard error payload returned to HTTP clients.
#[derive(Debug, Serialize)]
pub struct ErrorPayload<'a> {
    pub code: ErrorCode,
    pub message: &'a str,
}
