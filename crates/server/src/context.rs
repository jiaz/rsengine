use axum::response::Response;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use common::RequestContext;

/// Axum extractor that materialises a [`RequestContext`] for downstream handlers.
pub struct RequestContextExtractor(pub RequestContext);

impl RequestContextExtractor {
    pub fn into_inner(self) -> RequestContext {
        self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestContextExtractor
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context =
            RequestContext::from_http_parts(&parts.method, parts.uri.path(), &parts.headers);
        Ok(Self(context))
    }
}
