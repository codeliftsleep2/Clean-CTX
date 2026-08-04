// proxy/src/server.rs
//
// HTTP server for the Clean-CTX Anthropic proxy.
//
// Binds to 127.0.0.1:{PORT} and forwards requests to upstream.
// /v1/messages requests are intercepted for transforms + cache injection.
// All other paths pass through unchanged.
//
// This module handles HTTP concerns ONLY. Transform orchestration is
// delegated to the Pipeline abstraction (pipeline.rs). Filter loading
// is delegated to filter_loader.rs.

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
use crate::filter_loader::load_builtin_filters;
use crate::filter_registry::FilterRegistry;
use crate::logger::{self, LogStats};
use crate::pipeline::{Pipeline, PipelineConfig};
use crate::platform::{self, PlatformAdapter};
use crate::rate_limiter::RateLimiter;
use crate::transform::TransformStats;

/// Maximum request body size (10 MB).
/// Used in tests to verify body size limits.
#[allow(dead_code)]
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
    /// Shared filter registry (loaded once at startup, Arc for zero-copy sharing).
    pub filter_registry: Arc<FilterRegistry>,
    /// Per-client-IP rate limiter (only used when api_key is set).
    pub rate_limiter: RateLimiter,
}

impl ProxyState {
    pub fn new(config: ProxyConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(UPSTREAM_TIMEOUT)
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        // Delegate filter loading to filter_loader module (SRP compliance)
        let filter_registry = Arc::new(load_builtin_filters());

        // Create rate limiter with configured limits
        let rate_limiter = RateLimiter::new(config.rate_limit_rps, config.rate_limit_burst);

        Self {
            config,
            cache_stats: CacheStats::default(),
            transform_stats: TransformStats::default(),
            log_stats: LogStats::default(),
            request_counter: AtomicU64::new(0),
            http_client,
            filter_registry,
            rate_limiter,
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
                    let _permit = match semaphore.acquire().await {
                        Ok(p) => p,
                        Err(_) => {
                            warn!("[proxy] Connection limit reached, rejecting {peer_addr}");
                            return;
                        }
                    };

                    let svc = service_fn(move |req| {
                        handle_request(req, state.clone(), peer_addr)
                    });

                    if let Err(e) = http1::Builder::new()
                        .serve_connection(io, svc)
                        .await
                    {
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

fn is_connection_closed(e: &hyper::Error) -> bool {
    e.is_incomplete_message() || e.is_closed()
}

/// Build a 401 Unauthorized response.
fn unauthorized_response(msg: &str) -> Response<Full<BytesType>> {
    let body = serde_json::json!({
        "error": msg
    });
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(Full::new(BytesType::from(
            serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string())
        )))
        .unwrap()
}

/// Build a 429 Too Many Requests response.
fn rate_limited_response() -> Response<Full<BytesType>> {
    let body = serde_json::json!({
        "error": "Rate limit exceeded. Try again later."
    });
    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("content-type", "application/json")
        .header("retry-after", "1")
        .body(Full::new(BytesType::from(
            serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string())
        )))
        .unwrap()
}

/// Build a 502 Bad Gateway response.
fn bad_gateway_response(msg: &str) -> Response<Full<BytesType>> {
    let body = serde_json::json!({
        "error": msg
    });
    Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .header("content-type", "application/json")
        .body(Full::new(BytesType::from(
            serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string())
        )))
        .unwrap()
}

/// Handle an incoming HTTP request.
async fn handle_request(
    req: Request<Incoming>,
    state: SharedState,
    peer_addr: SocketAddr,
) -> Result<Response<Full<BytesType>>, std::convert::Infallible> {
    let req_id = {
        let guard = state.read().await;
        guard.next_req_id()
    };

    let (parts, body) = req.into_parts();
    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    debug!("[{req_id}] {method} {path}");

    // ── Authentication check (if api_key is configured) ──────────
    {
        let guard = state.read().await;
        if let Some(ref expected_key) = guard.config.api_key {
            let provided_key = parts.headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if provided_key != expected_key {
                warn!("[{req_id}] Unauthorized request (bad or missing API key)");
                return Ok(unauthorized_response("Missing or invalid API key. Provide via X-Api-Key header."));
            }

            // Rate limit check (only when auth is active).
            // SECURITY: Use the actual TCP peer address — X-Forwarded-For is
            // caller-controlled and can be spoofed per-request to bypass the
            // rate limiter. This is a local proxy with no trusted upstream.
            let client_ip = peer_addr.ip().to_string();
            if !guard.rate_limiter.check(&client_ip).await {
                warn!("[{req_id}] Rate limit exceeded for {client_ip}");
                return Ok(rate_limited_response());
            }
        }
    }

    // ── GET /stats: return proxy stats as JSON ────────────────
    if path == "/stats" && method == Method::GET {
        return Ok(handle_stats_endpoint(state).await);
    }

    // Detect platform from path to determine intercept endpoint
    // Use exact path matching to avoid intercepting non-API endpoints
    let should_intercept = method == Method::POST && (
        path == "/v1/messages"            // Anthropic
        || path == "/v1/chat/completions" // OpenAI
        || path == "/chat"                // Generic (root-level only)
    );

    if should_intercept {
        match handle_messages_request(parts, body, &req_id, state).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                error!("[{req_id}] Error handling /v1/messages: {e}");
                Ok(bad_gateway_response("Proxy error"))
            }
        }
    } else {
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
    let max_body_size = {
        let guard = state.read().await;
        guard.config.max_request_body_size
    };
    let body_bytes = read_body(body, max_body_size).await?;

    let mut body_value: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| ProxyError::Body(format!("Failed to parse request JSON: {e}")))?;

    // H-2 fix: Extract everything needed for transforms under a read lock,
    // run the CPU-intensive pipeline without holding any lock, then acquire
    // a write lock only to commit the stats delta at the end.
    let (pipeline_config, adapter, auto_cache, tail_ttl, log_bodies, log_dir) = {
        let guard = state.read().await;
        let pipeline_config = PipelineConfig {
            drop_tools_set: guard.config.drop_tools_set.clone(),
            strip_ansi: guard.config.strip_ansi,
            trim_bash_git: guard.config.trim_bash_git,
            model_override: guard.config.model_override.clone(),
            scrub_secrets: guard.config.scrub_secrets,
            tool_filters: guard.config.tool_filters,
            filter_registry: Some(guard.filter_registry.clone()),
            sliding_window_enabled: guard.config.sliding_window_enabled,
            sliding_window_max_age_turns: guard.config.sliding_window_max_age_turns,
            sliding_window_force_preserve_floor: guard.config.sliding_window_force_preserve_floor,
        };
        let adapter: Box<dyn PlatformAdapter> = if let Some(ref platform_name) = guard.config.platform {
            platform::get_platform(platform_name)
        } else {
            platform::detect_platform(&body_value)
        };
        (
            pipeline_config,
            adapter,
            guard.config.auto_cache,
            guard.config.tail_ttl.clone(),
            guard.config.log_bodies,
            guard.config.log_dir.clone(),
        )
    };

    // Run the CPU-intensive pipeline with no lock held.
    let pipeline = Pipeline::build(&pipeline_config);
    let mut local_transform_stats = crate::transform::TransformStats::default();
    pipeline.run(&mut body_value, &mut local_transform_stats, &pipeline_config, adapter.as_ref());

    // Inject cache breakpoints (cheap — no lock needed for the transform itself).
    let mut local_cache_stats = crate::cache::CacheStats::default();
    if auto_cache && adapter.platform_name() == "anthropic" {
        inject_breakpoints(&mut body_value, &tail_ttl, &mut local_cache_stats);
    }

    // Commit stats under a write lock (brief, no heavy work inside).
    {
        let mut guard = state.write().await;
        guard.transform_stats.merge(&local_transform_stats);
        guard.transform_stats.sliding_window.reset_request();
        guard.cache_stats.merge(&local_cache_stats);
    }

    // Log request body if configured (fire-and-forget, never under any lock).
    if log_bodies {
        let log_dir = PathBuf::from(&log_dir);
        let body_value_clone = body_value.clone();
        let req_id_owned = req_id.to_string();
        tokio::spawn(async move {
            let mut log_stats = LogStats::default();
            if let Err(e) = logger::log_request(&log_dir, &req_id_owned, &body_value_clone, &mut log_stats).await {
                warn!("[{req_id_owned}] Failed to log request: {e}");
            }
        });
    }

    let modified_body = serde_json::to_vec(&body_value)?;
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
    let max_body_size = {
        let guard = state.read().await;
        guard.config.max_request_body_size
    };
    let body_bytes = match read_body(body, max_body_size).await {
        Ok(b) => b,
        Err(e) => {
            error!("[{req_id}] Failed to read body: {e}");
            return Ok(bad_gateway_response("Proxy error"));
        }
    };

    match forward_to_upstream(parts, &body_bytes, req_id, state).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("[{req_id}] Upstream error: {e}");
            Ok(bad_gateway_response("Upstream error"))
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
    let (upstream_url, auto_cache, http_client, platform_name) = {
        let guard = state.read().await;
        (
            guard.config.upstream_url.clone(),
            guard.config.auto_cache,
            guard.http_client.clone(),
            guard.config.platform.clone(),
        )
    };

    let path_and_query = parts.uri.path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(parts.uri.path());

    if !path_and_query.starts_with('/') {
        return Err(ProxyError::Body(format!("Invalid path: {path_and_query}")));
    }

    let upstream_uri = format!(
        "{}{}",
        upstream_url.trim_end_matches('/'),
        path_and_query
    );

    let mut req_builder = http_client.request(
        parts.method.clone(),
        &upstream_uri,
    );

    for (name, value) in parts.headers.iter() {
        let name_str = name.as_str();
        if !name_str.eq_ignore_ascii_case("host")
            && !name_str.eq_ignore_ascii_case("connection")
            && !name_str.eq_ignore_ascii_case("keep-alive")
            && !name_str.eq_ignore_ascii_case("transfer-encoding")
            && !name_str.eq_ignore_ascii_case("te")
            && !name_str.eq_ignore_ascii_case("trailer")
            && !name_str.eq_ignore_ascii_case("upgrade")
        {
            req_builder = req_builder.header(name_str, value.as_bytes());
        }
    }

    // Inject platform-specific headers
    if auto_cache && platform_name.as_deref() != Some("openai") {
        req_builder = req_builder.header(
            "anthropic-beta",
            cache::anthropic_beta_header(),
        );
    }

    req_builder = req_builder.header("X-Request-ID", req_id);
    req_builder = req_builder.body(body_bytes.to_vec());

    let upstream_response = req_builder.send().await?;
    let status = upstream_response.status();
    let resp_headers = upstream_response.headers().clone();
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

    let mut response = Response::builder()
        .status(status);

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

    Ok(response
        .body(Full::new(resp_bytes))
        .unwrap())
}

/// Read the full body from an incoming hyper request with size limit.
///
/// C-1 fix: accumulate frames one-at-a-time and enforce the size cap
/// *during* streaming rather than after buffering the entire body.
/// This prevents an oversized request from causing OOM before the check runs.
async fn read_body(body: Incoming, max_size: usize) -> Result<Bytes, ProxyError> {
    use http_body_util::BodyExt;
    use bytes::BufMut;

    let mut buf = bytes::BytesMut::new();
    // Drive the body frame-by-frame so we can enforce the cap incrementally.
    let mut body = std::pin::pin!(body);
    while let Some(frame) = body.frame().await {
        let frame = frame?;
        if let Ok(data) = frame.into_data() {
            if buf.len() + data.len() > max_size {
                return Err(ProxyError::Body(format!(
                    "Request body too large (max {} bytes)",
                    max_size
                )));
            }
            buf.put(data);
        }
    }
    Ok(buf.freeze())
}

/// Handle `GET /stats` — return proxy filter + cache + sliding window stats as JSON.
async fn handle_stats_endpoint(state: SharedState) -> Response<Full<BytesType>> {
    let guard = state.read().await;
    let filter_stats = crate::filter_stats::FilterStats::new();
    let json = serde_json::json!({
        "filter_stats": filter_stats,
        "cache_stats": guard.cache_stats,
        "rate_limiter": guard.rate_limiter.stats_summary().await,
        "api_key_configured": guard.config.api_key.is_some(),
        "sliding_window": guard.transform_stats.sliding_window,
    });
    let body = serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Full::new(BytesType::from(body)))
        .unwrap()
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

    #[test]
    fn test_unauthorized_response_format() {
        let resp = unauthorized_response("Missing or invalid API key");
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert_eq!(content_type, "application/json");
    }

    #[test]
    fn test_rate_limited_response_format() {
        let resp = rate_limited_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(resp.headers().get("retry-after").is_some());
    }

}
