// proxy/tests/integration_test.rs
//
// End-to-end integration test for the Clean-CTX Anthropic proxy.
// Single sequential test to avoid Windows port-reuse conflicts.

use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::watch;

/// A minimal mock upstream that returns valid Anthropic-shaped JSON.
/// Does NOT collect request bodies — just returns a fixed response.
struct MockUpstream {
    url: String,
    count: std::sync::Arc<AtomicUsize>,
    shutdown_tx: watch::Sender<bool>,
    _h: tokio::task::JoinHandle<()>,
}

impl MockUpstream {
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let count = std::sync::Arc::new(AtomicUsize::new(0));
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let c = count.clone();
        let h = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => break,
                    a = listener.accept() => {
                        if let Ok((stream, _)) = a {
                            let io = hyper_util::rt::TokioIo::new(stream);
                            let c = c.clone();
                            tokio::spawn(async move {
                                let svc = hyper::service::service_fn(move |_req: hyper::Request<hyper::body::Incoming>| {
                                    let c = c.clone();
                                    async move {
                                        c.fetch_add(1, Ordering::SeqCst);
                                        // Return a valid Anthropic response — no body collection.
                                        let resp_body = json!({
                                            "id": "msg_test", "type": "message", "role": "assistant",
                                            "content": [{"type": "text", "text": "ok"}],
                                            "model": "claude-sonnet-4-20250514", "stop_reason": "end_turn",
                                            "usage": {"input_tokens": 10, "output_tokens": 5,
                                                       "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}
                                        });
                                        let body = serde_json::to_vec(&resp_body).unwrap();
                                        Ok::<_, hyper::Error>(
                                            hyper::Response::builder()
                                                .status(200)
                                                .header("content-type", "application/json")
                                                .body(http_body_util::Full::new(bytes::Bytes::from(body)))
                                                .unwrap()
                                        )
                                    }
                                });
                                let _ = hyper::server::conn::http1::Builder::new()
                                    .serve_connection(io, svc).await;
                            });
                        }
                    }
                }
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Self {
            url: addr,
            count,
            shutdown_tx,
            _h: h,
        }
    }
    fn request_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl Drop for MockUpstream {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
    }
}

#[tokio::test]
async fn test_proxy_full() {
    let mock = MockUpstream::start().await;

    // Sanity: mock reachable directly
    let direct = reqwest::Client::new()
        .post(&mock.url)
        .json(&json!({"test": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(direct.status().as_u16(), 200);
    assert_eq!(mock.request_count(), 1);

    // Start proxy pointing at mock
    let plistener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let pport = plistener.local_addr().unwrap().port();
    let config = clean_ctx_proxy::config::ProxyConfig {
        port: pport,
        upstream_url: mock.url.clone(),
        auto_cache: true,
        tail_ttl: "5m".to_string(),
        drop_tools: vec!["NotebookEdit".to_string(), "CronCreate".to_string()],
        drop_tools_set: vec!["NotebookEdit".to_string(), "CronCreate".to_string()]
            .into_iter()
            .collect(),
        strip_ansi: true,
        trim_bash_git: false,
        model_override: None,
        log_bodies: false,
        log_dir: ".clean-ctx/proxy-logs".to_string(),
        scrub_secrets: false,
        tool_filters: false,
        platform: None,
        api_key: None,
        rate_limit_rps: 60.0,
        rate_limit_burst: 10.0,
        max_request_body_size: 10 * 1024 * 1024,
        sliding_window_enabled: false,
        sliding_window_max_age_turns: 20,
        sliding_window_force_preserve_floor: 15,
    };
    let (tx, rx) = watch::channel(false);
    let ph = tokio::spawn(async move {
        clean_ctx_proxy::server::run_server_with_listener(plistener, config, rx)
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let client = reqwest::Client::new();
    let proxy_url = format!("http://127.0.0.1:{pport}");

    // 1. GET passthrough (non-messages path) — should proxy through
    let resp = client
        .get(format!("{proxy_url}/v1/other"))
        .header("x-api-key", "sk-test")
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        200,
        "GET passthrough should return 200"
    );
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 2. POST /v1/messages with tools (trigger cache injection + tool drop)
    let payload = json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 8192,
        "tools": [
            {"name": "Bash", "description": "Run", "input_schema": {"type": "object"}},
            {"name": "Read", "description": "Read", "input_schema": {"type": "object"}},
            {"name": "NotebookEdit", "description": "Edit", "input_schema": {"type": "object"}}
        ],
        "system": [
            {"type": "text", "text": "You are Claude."},
            {"type": "text", "text": "Longer ".to_owned() + &"x".repeat(600)}
        ],
        "messages": [{"role": "user", "content": [{"type": "text", "text": "Hello"}]}]
    });
    let resp = client
        .post(format!("{proxy_url}/v1/messages"))
        .header("x-api-key", "sk-test")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        200,
        "POST /v1/messages should return 200"
    );

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    // Mock should have received 1 direct + 1 passthrough GET + 1 proxied POST = 3
    assert!(
        mock.request_count() >= 3,
        "mock got {} requests",
        mock.request_count()
    );

    // Cleanup
    tx.send(true).ok();
    ph.await.ok();
}
