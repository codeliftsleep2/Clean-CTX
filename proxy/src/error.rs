// proxy/src/error.rs
//
// Error types for the Clean-CTX Anthropic proxy.

use thiserror::Error;

/// Top-level proxy error type.
#[derive(Debug, Error)]
pub enum ProxyError {
    /// I/O error (socket bind, file read/write, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP error from hyper.
    #[error("HTTP error: {0}")]
    Hyper(#[from] hyper::Error),

    /// Upstream request failed.
    #[error("Upstream request failed: {0}")]
    Upstream(#[from] reqwest::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Body serialization error.
    #[error("Body error: {0}")]
    Body(String),

    /// Invalid HTTP header value.
    #[error("Invalid header: {0}")]
    Header(#[from] hyper::header::InvalidHeaderValue),

    /// Invalid URI.
    #[error("Invalid URI: {0}")]
    Uri(#[from] hyper::http::uri::InvalidUri),

    /// Invalid status code.
    #[error("Invalid status: {0}")]
    Status(#[from] hyper::http::status::InvalidStatusCode),

    /// Invalid proxy configuration.
    #[error("Configuration error: {0}")]
    Config(String),
}
