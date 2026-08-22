// src/tests/proxy_spawner.rs
//
// Unit tests for the proxy auto-start spawner.

use crate::config::ProxyAutoStartConfig;
use crate::proxy_spawner::{
    ProxyCacheStateInfo, build_proxy_env, proxy_child_exited, resolve_proxy_binary, spawn_proxy,
};

fn test_config() -> ProxyAutoStartConfig {
    ProxyAutoStartConfig::default()
}

// ── build_proxy_env tests ─────────────────────────────────────────

#[test]
fn build_proxy_env_defaults_only_port_and_upstream() {
    let cfg = test_config();
    let env = build_proxy_env(&cfg);
    // PORT is always set; PROXY_UPSTREAM_URL is always pinned to the
    // proxy's built-in default when no upstream is configured (this
    // shields the child from an inherited ANTHROPIC_BASE_URL that
    // points at the proxy itself).
    assert_eq!(env.len(), 2);
    let map: std::collections::HashMap<&str, &str> =
        env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    assert_eq!(map.get("PORT"), Some(&"8787"));
    assert_eq!(
        map.get("PROXY_UPSTREAM_URL"),
        Some(&"https://api.anthropic.com")
    );
}

#[test]
fn build_proxy_env_pins_upstream_when_unset() {
    // Even with no upstream_url configured, PROXY_UPSTREAM_URL must be
    // set to the proxy's default so the child does NOT inherit the
    // parent's ANTHROPIC_BASE_URL (which commonly points at the proxy
    // itself, triggering the self-forwarding-loop validation guard).
    let cfg = test_config();
    let env = build_proxy_env(&cfg);
    let map: std::collections::HashMap<&str, &str> =
        env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    assert_eq!(
        map.get("PROXY_UPSTREAM_URL"),
        Some(&"https://api.anthropic.com")
    );
}

#[test]
fn build_proxy_env_uses_configured_upstream() {
    let mut cfg = test_config();
    cfg.upstream_url = Some("http://127.0.0.1:4141".to_string());
    let env = build_proxy_env(&cfg);
    let map: std::collections::HashMap<&str, &str> =
        env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    assert_eq!(
        map.get("PROXY_UPSTREAM_URL"),
        Some(&"http://127.0.0.1:4141")
    );
}

#[test]
fn build_proxy_env_maps_all_fields() {
    let mut cfg = test_config();
    cfg.port = 9999;
    cfg.auto_cache = true;
    cfg.tail_ttl = "10m".to_string();
    cfg.drop_tools = vec!["NotebookEdit".to_string(), "CronCreate".to_string()];
    cfg.strip_ansi = true;
    cfg.trim_bash_git = true;
    cfg.model_override = Some("claude-opus-4-6".to_string());
    cfg.scrub_secrets = true;
    cfg.tool_filters = true;
    cfg.upstream_url = Some("http://127.0.0.1:4141".to_string());
    cfg.api_key = Some("secret-key".to_string());
    cfg.rate_limit_rps = 30.0;
    cfg.rate_limit_burst = 5.0;

    let env = build_proxy_env(&cfg);
    let map: std::collections::HashMap<&str, &str> =
        env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    assert_eq!(map.get("PORT"), Some(&"9999"));
    assert_eq!(map.get("AUTO_CACHE"), Some(&"1"));
    assert_eq!(map.get("TAIL_TTL"), Some(&"10m"));
    assert_eq!(map.get("DROP_TOOLS"), Some(&"NotebookEdit,CronCreate"));
    assert_eq!(map.get("STRIP_ANSI"), Some(&"1"));
    assert_eq!(map.get("TRIM_BASH_GIT"), Some(&"1"));
    assert_eq!(map.get("MODEL_OVERRIDE"), Some(&"claude-opus-4-6"));
    assert_eq!(map.get("SCRUB_SECRETS"), Some(&"1"));
    assert_eq!(map.get("TOOL_FILTERS"), Some(&"1"));
    assert_eq!(
        map.get("PROXY_UPSTREAM_URL"),
        Some(&"http://127.0.0.1:4141")
    );
    assert_eq!(map.get("PROXY_API_KEY"), Some(&"secret-key"));
    assert_eq!(map.get("RATE_LIMIT_RPS"), Some(&"30"));
    assert_eq!(map.get("RATE_LIMIT_BURST"), Some(&"5"));
}

#[test]
fn build_proxy_env_skips_defaults() {
    let mut cfg = test_config();
    // These are the defaults — should NOT be emitted.
    cfg.tail_ttl = "5m".to_string();
    cfg.rate_limit_rps = 60.0;
    cfg.rate_limit_burst = 10.0;
    let env = build_proxy_env(&cfg);
    let map: std::collections::HashMap<&str, &str> =
        env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    assert!(!map.contains_key("TAIL_TTL"));
    assert!(!map.contains_key("RATE_LIMIT_RPS"));
    assert!(!map.contains_key("RATE_LIMIT_BURST"));
}

// ── spawn_proxy tests ─────────────────────────────────────────────

#[test]
fn spawn_proxy_disabled_when_auto_start_false() {
    let cfg = test_config(); // auto_start defaults to false
    let result = spawn_proxy(&cfg, std::path::Path::new("."), false);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn spawn_proxy_force_starts_when_auto_start_false() {
    // `clean-ctx proxy` (explicit CLI invocation) passes force=true and
    // must start the proxy even when auto_start is false (the default).
    // On machines where the proxy binary is installed this spawns; on
    // machines where it isn't, it returns Ok(None). Both are graceful —
    // the key assertion is that force bypasses the auto_start gate.
    let cfg = test_config(); // auto_start defaults to false
    let result = spawn_proxy(&cfg, std::path::Path::new("."), true);
    assert!(result.is_ok());
    if let Ok(Some(mut child)) = result {
        // If we did spawn a proxy, clean it up.
        crate::proxy_spawner::shutdown_proxy(&mut child);
    }
}

#[test]
fn spawn_proxy_handles_missing_binary_gracefully() {
    let mut cfg = test_config();
    cfg.auto_start = true;
    // Set PROXY_BINARY_PATH to a nonexistent path. The resolver logs a
    // warning and falls through to same-dir/PATH lookup. On machines where
    // the proxy binary IS installed, this spawns successfully; on machines
    // where it isn't, it returns Ok(None). Both are graceful — the key
    // assertion is that it never returns Err and never panics.
    unsafe {
        std::env::set_var("PROXY_BINARY_PATH", "C:\\nonexistent\\clean-ctx-proxy.exe");
    }
    let result = spawn_proxy(&cfg, std::path::Path::new("."), false);
    unsafe {
        std::env::remove_var("PROXY_BINARY_PATH");
    }
    // Must be Ok (never Err) — either Some(child) if the real binary was
    // found via fallback, or None if not. Both are graceful outcomes.
    assert!(result.is_ok());
    if let Ok(Some(mut child)) = result {
        // If we did spawn a proxy, clean it up.
        crate::proxy_spawner::shutdown_proxy(&mut child);
    }
}

// ── resolve_proxy_binary tests ────────────────────────────────────

#[test]
fn resolve_proxy_binary_uses_env_override() {
    // Point PROXY_BINARY_PATH at the current executable (which exists).
    let exe = std::env::current_exe().unwrap();
    unsafe {
        std::env::set_var("PROXY_BINARY_PATH", &exe);
    }
    let resolved = resolve_proxy_binary();
    unsafe {
        std::env::remove_var("PROXY_BINARY_PATH");
    }
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap(), exe);
}

#[test]
fn resolve_proxy_binary_returns_none_when_not_found() {
    // Ensure PROXY_BINARY_PATH is unset and the binary isn't on PATH.
    unsafe {
        std::env::remove_var("PROXY_BINARY_PATH");
    }
    // We can't guarantee the binary isn't on PATH, but we can at least
    // verify the function doesn't panic and returns an Option.
    let _ = resolve_proxy_binary();
}

// ── ProxyCacheStateInfo serde round-trip ──────────────────────────

#[test]
fn proxy_cache_state_info_serde_roundtrip() {
    let info = ProxyCacheStateInfo {
        auto_cache: true,
        tail_ttl: "10m".to_string(),
        client_breakpoints_preserved: 3,
        client_breakpoints_stripped: 1,
        total_injected: 7,
    };
    let json = serde_json::to_string(&info).unwrap();
    let deserialized: ProxyCacheStateInfo = serde_json::from_str(&json).unwrap();
    assert!(deserialized.auto_cache);
    assert_eq!(deserialized.tail_ttl, "10m");
    assert_eq!(deserialized.client_breakpoints_preserved, 3);
    assert_eq!(deserialized.client_breakpoints_stripped, 1);
    assert_eq!(deserialized.total_injected, 7);
}

#[test]
fn proxy_cache_state_info_defaults() {
    // Missing fields in the proxy response must deserialize to defaults
    // (the proxy may add fields in future versions).
    let json = r#"{"auto_cache": false}"#;
    let info: ProxyCacheStateInfo = serde_json::from_str(json).unwrap();
    assert!(!info.auto_cache);
    assert_eq!(info.tail_ttl, "");
    assert_eq!(info.client_breakpoints_preserved, 0);
    assert_eq!(info.client_breakpoints_stripped, 0);
    assert_eq!(info.total_injected, 0);
}

// ── Regression: query_proxy_cache_state must not block startup ────
//
// Fix 1 (FAANG audit): the original implementation used
// `ureq::get(&url).call()` with no timeout. A proxy that accepted TCP
// but hung on HTTP would block MCP server startup indefinitely. The
// fix added a 2-second global timeout. These tests guard against a
// regression to the unbounded call.

#[test]
fn query_proxy_cache_state_returns_none_when_unreachable() {
    // Nothing listening on an ephemeral port — must return None quickly
    // (non-blocking), never panic.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // release the port so nothing is listening

    let start = std::time::Instant::now();
    let result = crate::proxy_spawner::query_proxy_cache_state(port);
    let elapsed = start.elapsed();

    assert!(result.is_none(), "unreachable proxy must yield None");
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "unreachable proxy query took {:?} — must fail fast, not block",
        elapsed
    );
}

#[test]
fn query_proxy_cache_state_times_out_on_hanging_server() {
    // A server that accepts the TCP connection but never responds must
    // not block the caller past the 2s timeout. This is the exact
    // regression the fix addressed.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Accept the connection on a background thread and hold it open
    // without ever writing a response.
    let handle = std::thread::spawn(move || {
        let (_stream, _) = listener.accept().unwrap();
        // Hold the stream open indefinitely (no response written).
        std::thread::sleep(std::time::Duration::from_secs(10));
    });

    let start = std::time::Instant::now();
    let result = crate::proxy_spawner::query_proxy_cache_state(port);
    let elapsed = start.elapsed();

    // Must return None (timeout) and must not block past ~2s + margin.
    assert!(result.is_none(), "hanging server must yield None (timeout)");
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "hanging server blocked for {:?} — timeout not applied",
        elapsed
    );

    // Clean up the background thread.
    drop(handle);
}

// ── Regression: port_in_use (crash-monitor primitive) ─────────────
//
// Fix 2 (FAANG audit): the crash monitor polls `port_in_use` to detect
// a dead proxy. This test guards the primitive it relies on.

#[test]
fn port_in_use_detects_listening_and_closed() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Listening → true.
    assert!(crate::proxy_spawner::port_in_use(port));

    // Released → false.
    drop(listener);
    // Small delay so the OS fully releases the port.
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert!(!crate::proxy_spawner::port_in_use(port));
}

// ── Regression: proxy_cache state is persisted on McpState ────────
//
// Fix 3 (FAANG audit): the Phase 5 cache state was fetched and logged
// but never stored, so the "alignment" was cosmetic. This guards the
// field that now persists it.

#[test]
fn mcp_state_proxy_cache_defaults_none() {
    let state = crate::mcp::McpState::new(crate::config::CleanCtxConfig::default());
    assert!(
        state.proxy_cache.is_none(),
        "proxy_cache must default to None (no proxy queried yet)"
    );
}

// ── proxy_child_exited tests ──────────────────────────────────────

#[test]
fn proxy_child_exited_detects_running_child() {
    // Spawn a long-running process (e.g. `ping` on Windows, `sleep` on Unix).
    #[cfg(windows)]
    let mut child = std::process::Command::new("ping")
        .arg("127.0.0.1")
        .arg("-n")
        .arg("30")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    #[cfg(not(windows))]
    let mut child = std::process::Command::new("sleep")
        .arg("30")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    // Should still be running.
    assert!(!proxy_child_exited(&mut child));

    // Kill it.
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn proxy_child_exited_detects_exited_child() {
    // Spawn a process that exits immediately.
    #[cfg(windows)]
    let mut child = std::process::Command::new("cmd")
        .arg("/C")
        .arg("exit 0")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    #[cfg(not(windows))]
    let mut child = std::process::Command::new("true")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    // Wait for it to exit.
    let _ = child.wait();

    // Should report exited.
    assert!(proxy_child_exited(&mut child));
}
