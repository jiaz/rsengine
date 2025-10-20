use serde::{Deserialize, Serialize};

/// Controls how a route should be rendered by the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RenderMode {
    /// Render synchronously and buffer the full response before sending it to the client.
    #[default]
    Blocking,
    /// Stream the response as chunks become available from the runtime.
    Streaming,
}

/// Declarative configuration for a renderable route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Stable identifier that is typically shared with the bundle registry.
    pub id: String,
    /// Human-readable route pattern (e.g. `/products/:id`).
    pub pattern: String,
    /// Rendering strategy for this route.
    #[serde(default)]
    pub render_mode: RenderMode,
    /// Optional time-to-live for cached render results, expressed in seconds.
    #[serde(default)]
    pub cache_ttl_seconds: Option<u64>,
}

impl RouteConfig {
    /// Creates a new route configuration using the provided identifier and pattern.
    pub fn new(id: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            pattern: pattern.into(),
            render_mode: RenderMode::default(),
            cache_ttl_seconds: None,
        }
    }
}
