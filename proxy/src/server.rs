// proxy/src/server.rs
//
// HTTP server for the Clean-CTX Anthropic proxy.
//
// Binds to 127.0.0.1:{PORT} and forwards requests to upstream.
// /v1/messages requests are intercepted for cache injection + transforms.
// All other paths pass through unchanged.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicU64};
use tokio::sync::{watch, RwLock};
use hyper::body::{Incoming, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Method, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use bytes::Bytes as BytesType;
use tokio::net::TcpListener;
use tracing::{info, warn, error, debug};

use crate::cache::{self, CacheStats, inject_breakpoints};
use crate::config::ProxyConfig;
use crate::error::ProxyError;
use crate::logger::{self, LogStats};
use crate::transform::{self, TransformStats};

/// Maximum request body size (10 MB).
pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Upstream request timeout (connect + read).
pub const UPSTREAM_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Maximum concurrent connections.
pub const MAX_CONNECTIONS: usize = 256;

/// Shared proxy server state.
pub struct ProxyState {
    pub config: ProxyConfig,
    pub cache_stats: CacheStats,
    pub transform_stats: TransformStats,
    #[allow(dead_code)]
    pub log_stats: LogStats,
    pub request_counter: AtomicU64,
    /// Shared reqwest client for connection reuse.
    pub http_client: reqwest::Client,
}

impl ProxyState {
    pub fn new(config: ProxyConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(UPSTREAM_TIMEOUT)
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            cache_stats: CacheStats::default(),
            transform_stats: TransformStats::default(),
            log_stats: LogStats::default(),
            request_counter: AtomicU64::new(0),
            http_client,
        }
    }

    pub fn next_req_id(&self) -> String {
        let n = self.request_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("{:06}", n)
    }
}

type SharedState = Arc<RwLock<ProxyState>>;

/// Start the proxy server. Runs until `shutdown_rx` receives a signal.
pub async fn run_server(
    config: ProxyConfig,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<(), ProxyError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;
    run_server_with_listener(listener, config, shutdown_rx).await
}

/// Start the proxy server with a pre-bound listener (for testing).
pub async fn run_server_with_listener(
    listener: TcpListener,
    config: ProxyConfig,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), ProxyError> {
    info!("[proxy] Clean-CTX proxy listening on http://{}", listener.local_addr().unwrap());
    info!("[proxy] Upstream: {}", config.upstream_url);

    let state = Arc::new(RwLock::new(ProxyState::new(config)));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONNECTIONS));

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (stream, peer_addr) = match accept_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("[proxy] Accept failed: {e}");
                        continue;
                    }
                };

                let state = state.clone();
                let semaphore = semaphore.clone();
                let io = TokioIo::new(stream);

                tokio::spawn(async move {
                    // Acquire a permit (connection limit)
                    let _permit = match semaphore.acquire().await {
                        Ok(p) => p,
                        Err(_) => {
                            warn!("[proxy] Connection limit reached, rejecting {peer_addr}");
                            return;
                        }
                    };

                    let svc = service_fn(move |req| {
                        handle_request(req, state.clone())
                    });

                    if let Err(e) = http1::Builder::new()
                        .serve_connection(io, svc)
                        .await
                    {
                        // Use typed error inspection instead of string matching
                        if !is_connection_closed(&e) {
                            warn!("[proxy] Connection error from {peer_addr}: {e}");
                        }
                    }
                });
            }
            _ = shutdown_rx.changed() => {
                info!("[proxy] Shutdown signal received, stopping listener");
                break;
            }
        }
    }

    Ok(())
}

/// Check if a hyper error is a normal connection close.
fn is_connection_closed(e: &hyper::Error) -> bool {
    e.is_incomplete_message() || e.is_closed()
}

/// Handle an incoming HTTP request.
async fn handle_request(
    req: Request<Incoming>,
    state: SharedState,
) -> Result<Response<Full<BytesType>>, std::convert::Infallible> {
    let req_id = {
        let guard = state.read().await;
        guard.next_req_id()
    };

    let (parts, body) = req.into_parts();
    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    debug!("[{req_id}] {method} {path}");

    // Only intercept POST /v1/messages
    if path == "/v1/messages" && method == Method::POST {
        match handle_messages_request(parts, body, &req_id, state).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                error!("[{req_id}] Error handling /v1/messages: {e}");
                Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(Full::new(BytesType::from(
                        "{\"error\": \"Proxy error\"}".to_string()
                    )))
                    .unwrap())
            }
        }
    } else {
        // Passthrough: forward to upstream unchanged
        forward_request(parts, body, &req_id, state).await
    }
}

/// Handle a /v1/messages request: parse body, apply transforms, inject cache, forward.
async fn handle_messages_request(
    parts: hyper::http::request::Parts,
    body: Incoming,
    req_id: &str,
    state: SharedState,
) -> Result<Response<Full<BytesType>>, ProxyError> {
    // Read the full body with size limit (read_body enforces MAX_BODY_SIZE)
    let body_bytes = read_body(body).await?;

    let mut body_value: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| ProxyError::Body(format!("Failed to parse request JSON: {e}")))?;

    // Single lock acquisition: extract config + apply transforms
    {
        let mut guard = state.write().await;

        // Extract config values
        let auto_cache = guard.config.auto_cache;
        let strip_ansi = guard.config.strip_ansi;
        let trim_bash_git = guard.config.trim_bash_git;
        let tail_ttl = guard.config.tail_ttl.clone();
        let model_override = guard.config.model_override.clone();
        let drop_tools_set = guard.config.drop_tools_set.clone();

        // Apply transforms
        transform::drop_tools(&mut body_value, &drop_tools_set, &mut guard.transform_stats);

        if strip_ansi {
            transform::strip_ansi(&mut body_value, &mut guard.transform_stats);
        }

        if trim_bash_git {
            transform::trim_bash_git(&mut body_value, &mut guard.transform_stats);
        }

        if let Some(ref model) = model_override {
            transform::override_model(&mut body_value, model, &mut guard.transform_stats);
        }

        if auto_cache {
            inject_breakpoints(&mut body_value, &tail_ttl, &mut guard.cache_stats);
        }

        // Log request body if configured
        if guard.config.log_bodies {
            let log_dir = PathBuf::from(&guard.config.log_dir);
            let body_value_clone = body_value.clone();
            let req_id_owned = req_id.to_string();
            tokio::spawn(async move {
                let mut log_stats = LogStats::default();
                if let Err(e) = logger::log_request(&log_dir, &req_id_owned, &body_value_clone, &mut log_stats).await {
                    warn!("[{req_id_owned}] Failed to log request: {e}");
                }
            });
        }
    }

    // Re-serialize the transformed body
    let modified_body = serde_json::to_vec(&body_value)?;

    // Forward to upstream
    let response = forward_to_upstream(parts, &modified_body, req_id, state).await?;
    Ok(response)
}

/// Forward a request to the upstream Anthropic API.
async fn forward_request(
    parts: hyper::http::request::Parts,
    body: Incoming,
    req_id: &str,
    state: SharedState,
) -> Result<Response<Full<BytesType>>, std::convert::Infallible> {
    let body_bytes = match read_body(body).await {
        Ok(b) => b,
        Err(e) => {
            error!("[{req_id}] Failed to read body: {e}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Full::new(BytesType::from(
                    "{\"error\": \"Proxy error\"}".to_string()
                )))
                .unwrap());
        }
    };

    match forward_to_upstream(parts, &body_bytes, req_id, state).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("[{req_id}] Upstream error: {e}");
            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Full::new(BytesType::from(
                    "{\"error\": \"Upstream error\"}".to_string()
                )))
                .unwrap())
        }
    }
}

/// Forward a request to the upstream with the given body bytes.
async fn forward_to_upstream(
    parts: hyper::http::request::Parts,
    body_bytes: &[u8],
    req_id: &str,
    state: SharedState,
) -> Result<Response<Full<BytesType>>, ProxyError> {
    // Read config and client in a single read lock
    let (upstream_url, auto_cache, http_client) = {
        let guard = state.read().await;
        (guard.config.upstream_url.clone(), guard.config.auto_cache, guard.http_client.clone())
    };

    // Validate path to prevent URI injection
    let path_and_query = parts.uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(parts.uri.path());

    if !path_and_query.starts_with('/') {
        return Err(ProxyError::Body(format!("Invalid path: {path_and_query}")));
    }

    // Build the upstream URI
    let upstream_uri = format!(
        "{}{}",
        upstream_url.trim_end_matches('/'),
        path_and_query
    );

    // Build request using shared client
    let mut req_builder = http_client.request(
        parts.method.clone(),
        &upstream_uri,
    );

    // Forward headers (skip host, connection, and other hop-by-hop headers)
    for (name, value) in parts.headers.iter() {
        let name_lower = name.as_str().to_lowercase();
        if name_lower != "host"
            && name_lower != "connection"
            && name_lower != "keep-alive"
            && name_lower != "transfer-encoding"
            && name_lower != "te"
            && name_lower != "trailer"
            && name_lower != "upgrade"
        {
            req_builder = req_builder.header(name.as_str(), value.as_bytes());
        }
    }

    // Add anthropic-beta header for extended cache TTL
    if auto_cache {
        req_builder = req_builder.header(
            "anthropic-beta",
            cache::anthropic_beta_header(),
        );
    }

    // Add X-Request-ID for distributed tracing
    req_builder = req_builder.header("X-Request-ID", req_id);

    // Set the body
    req_builder = req_builder.body(body_bytes.to_vec());

    // Send the request
    let upstream_response = req_builder.send().await?;
    let status = upstream_response.status();
    let resp_headers = upstream_response.headers().clone();

    // Read the response body
    let resp_bytes = upstream_response.bytes().await?;

    debug!("[{req_id}] Upstream returned {status} ({} bytes)", resp_bytes.len());

    // Log response body if configured
    {
        let guard = state.read().await;
        if guard.config.log_bodies {
            let log_dir = PathBuf::from(&guard.config.log_dir);
            let resp_bytes_clone = resp_bytes.clone();
            let req_id_owned = req_id.to_string();
            tokio::spawn(async move {
                let mut log_stats = LogStats::default();
                if let Err(e) = logger::log_response(&log_dir, &req_id_owned, &resp_bytes_clone, &mut log_stats).await {
                    warn!("[{req_id_owned}] Failed to log response: {e}");
                }
            });
        }
    }

    // Build the response — only forward safe headers
    let mut response = Response::builder()
        .status(status);

    // Whitelist of safe response headers to forward
    let safe_headers = [
        "content-type", "content-length", "x-request-id",
        "anthropic-ratelimit-requests-limit", "anthropic-ratelimit-requests-remaining",
        "anthropic-ratelimit-tokens-limit", "anthropic-ratelimit-tokens-remaining",
        "anthropic-ratelimit-tokens-reset", "anthropic-ratelimit-requests-reset",
    ];

    for (name, value) in resp_headers.iter() {
        if safe_headers.contains(&name.as_str()) {
            response = response.header(name, value);
        }
    }

    // resp_bytes is already Bytes — avoid needless copy
    Ok(response
        .body(Full::new(resp_bytes))
        .unwrap())
}

/// Read the full body from an incoming hyper request with size limit.
async fn read_body(body: Incoming) -> Result<Bytes, ProxyError> {
    use http_body_util::BodyExt;
    let collected = body.collect().await?;
    let bytes = collected.to_bytes();
    if bytes.len() > MAX_BODY_SIZE {
        return Err(ProxyError::Body(format!(
            "Request body too large: {} bytes (max {})",
            bytes.len(),
            MAX_BODY_SIZE
        )));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_req_id_generation() {
        let config = ProxyConfig::default();
        let state = ProxyState::new(config);
        let id1 = state.next_req_id();
        let id2 = state.next_req_id();
        assert_ne!(id1, id2);
        assert_eq!(id1, "000000");
        assert_eq!(id2, "000001");
    }
}