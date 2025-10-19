use common::{AppError, ErrorCode, RequestContext, RouteConfig};
use tracing::debug;

/// Configuration parameters for the render runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Friendly name used in logs to differentiate runtime pools.
    pub name: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
        }
    }
}

/// Placeholder runtime implementation that mimics the external rendering interface.
#[derive(Debug, Default, Clone)]
pub struct RenderRuntime {
    config: RuntimeConfig,
}

impl RenderRuntime {
    /// Creates a new runtime with the provided configuration.
    pub fn new(config: RuntimeConfig) -> Self {
        Self { config }
    }

    /// Performs a synthetic render and returns a basic HTML envelope.
    pub async fn render(
        &self,
        route: &RouteConfig,
        context: &RequestContext,
    ) -> Result<String, AppError> {
        debug!(
            request_id = %context.trace.request_id,
            route = %route.pattern,
            runtime = %self.config.name,
            "render runtime invoked",
        );

        if route.pattern.is_empty() {
            return Err(AppError::new(
                ErrorCode::BadRequest,
                "route pattern must be provided",
            ));
        }

        Ok(format!(
            "<html><body><h1>SSR placeholder</h1><p>Route: {}</p><p>Request ID: {}</p></body></html>",
            route.pattern, context.trace.request_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::RenderMode;
    use http::{HeaderMap, Method};

    #[tokio::test]
    async fn runtime_returns_placeholder_markup() {
        let runtime = RenderRuntime::default();
        let mut route = RouteConfig::new("home", "/");
        route.render_mode = RenderMode::Blocking;

        let context = RequestContext::from_http_parts(&Method::GET, "/", &HeaderMap::new());

        let result = runtime.render(&route, &context).await.unwrap();
        assert!(result.contains("Route: /"));
    }
}
