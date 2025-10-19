use std::collections::BTreeMap;

use cookie::Cookie;
use http::{header, HeaderMap, Method};
use serde::Serialize;
use uuid::Uuid;

/// Trace identifiers extracted from incoming requests to aid logging and telemetry correlation.
#[derive(Debug, Clone, Serialize)]
pub struct TraceContext {
    pub request_id: Uuid,
    pub trace_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_trace_id: Option<String>,
}

/// Normalised request metadata that is passed through the render pipeline.
#[derive(Debug, Clone, Serialize)]
pub struct RequestContext {
    pub trace: TraceContext,
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub cookies: BTreeMap<String, String>,
}

impl RequestContext {
    /// Builds a new request context from HTTP primitives.
    pub fn from_http_parts(method: &Method, path: impl Into<String>, headers: &HeaderMap) -> Self {
        let path = path.into();
        let request_id = extract_request_id(headers);
        let trace_id = extract_trace_id(headers).unwrap_or_else(Uuid::new_v4);
        let parent_trace_id = headers
            .get("traceparent")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_owned());

        let mut header_map = BTreeMap::new();
        for (name, value) in headers.iter() {
            if let Ok(value) = value.to_str() {
                header_map.insert(name.as_str().to_string(), value.to_string());
            }
        }

        let cookies = extract_cookies(headers);

        Self {
            trace: TraceContext {
                request_id,
                trace_id,
                parent_trace_id,
            },
            method: method.to_string(),
            path,
            headers: header_map,
            cookies,
        }
    }
}

fn extract_request_id(headers: &HeaderMap) -> Uuid {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(Uuid::new_v4)
}

fn extract_trace_id(headers: &HeaderMap) -> Option<Uuid> {
    headers
        .get("x-trace-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn extract_cookies(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|cookie_header| cookie_header.split(';'))
        .filter_map(|raw| {
            let trimmed = raw.trim();
            Cookie::parse(trimmed.to_owned()).ok()
        })
        .fold(BTreeMap::new(), |mut acc, cookie| {
            acc.insert(cookie.name().to_string(), cookie.value().to_string());
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_is_generated_when_missing() {
        let headers = HeaderMap::new();
        let ctx = RequestContext::from_http_parts(&Method::GET, "/foo", &headers);
        assert!(!ctx.trace.request_id.is_nil());
    }

    #[test]
    fn cookies_are_parsed() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "session=abc123; theme=dark".parse().expect("valid header"),
        );
        let ctx = RequestContext::from_http_parts(&Method::GET, "/cookies", &headers);

        assert_eq!(ctx.cookies.get("session"), Some(&"abc123".to_string()));
        assert_eq!(ctx.cookies.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn trace_id_prefers_header_value() {
        let mut headers = HeaderMap::new();
        let trace_id = Uuid::new_v4();
        headers.insert("x-trace-id", trace_id.to_string().parse().unwrap());
        let ctx = RequestContext::from_http_parts(&Method::GET, "/trace", &headers);

        assert_eq!(ctx.trace.trace_id, trace_id);
    }
}
