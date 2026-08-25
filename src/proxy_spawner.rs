// src/proxy_spawner.rs
//
// Auto-start spawner for the Clean-CTX proxy.
//
// When `proxy.auto_start` is enabled in `.clean-ctx.json`, the MCP
// server spawns the `clean-ctx-proxy` binary as a child process on
// startup and terminates it on shutdown. All settings from
// `ProxyAutoStartConfig` are mapped to the proxy's environment
// variables (see `proxy/src/config.rs`), so the proxy itself needs
// zero changes to honor them.

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::config::ProxyAutoStartConfig;

/// Binary name for the proxy, with platform-specific extension.
fn proxy_binary_name() -> &'static str {
    if cfg!(windows) {
        "clean-ctx-proxy.exe"
    } else {
        "clean-ctx-proxy"
    }
}

/// Resolve the path to the `clean-ctx-proxy` binary.
///
/// Resolution order:
///   1. `PROXY_BINARY_PATH` env var (explicit override)
///   2. Same directory as the running MCP binary
///   3. `cargo run -p clean-ctx-proxy` fallback (dev mode) — returns `None`
///      here; the caller decides whether to issue the cargo fallback.
///
/// Returns `None` if the binary cannot be found.
pub fn resolve_proxy_binary() -> Option<PathBuf> {
    // 1. Explicit override via PROXY_BINARY_PATH.
    if let Ok(path) = std::env::var("PROXY_BINARY_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
        eprintln!(
            "[clean-ctx] Warning: PROXY_BINARY_PATH set but not found: {}",
            p.display()
        );
    }

    // 2. Same directory as the running MCP binary.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(proxy_binary_name());
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 3. Fall back to a PATH lookup (covers `cargo install` and
    //    `cargo run -p clean-ctx-proxy` builds that place the binary
    //    on PATH).
    if let Some(path) = which(proxy_binary_name()) {
        return Some(path);
    }

    None
}

/// Poor-man's `which`: search the PATH for the given executable name.
fn which(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Build the environment for the proxy child process from config.
///
/// Returns a `Vec<(String, String)>` of env var name → value. Only
/// vars that differ from the proxy's defaults are set (the proxy's
/// `from_env` applies defaults for anything unset), keeping the
/// environment minimal. `None` values (e.g. no model override) are
/// skipped entirely.
pub fn build_proxy_env(config: &ProxyAutoStartConfig) -> Vec<(String, String)> {
    let mut env = Vec::new();

    // Always set the port so the proxy binds the configured port.
    env.push(("PORT".to_string(), config.port.to_string()));

    if config.auto_cache {
        env.push(("AUTO_CACHE".to_string(), "1".to_string()));
    }
    if config.tail_ttl != "5m" {
        env.push(("TAIL_TTL".to_string(), config.tail_ttl.clone()));
    }
    if !config.drop_tools.is_empty() {
        env.push(("DROP_TOOLS".to_string(), config.drop_tools.join(",")));
    }
    if config.strip_ansi {
        env.push(("STRIP_ANSI".to_string(), "1".to_string()));
    }
    if config.trim_bash_git {
        env.push(("TRIM_BASH_GIT".to_string(), "1".to_string()));
    }
    if let Some(ref model) = config.model_override {
        env.push(("MODEL_OVERRIDE".to_string(), model.clone()));
    }
    if config.scrub_secrets {
        env.push(("SCRUB_SECRETS".to_string(), "1".to_string()));
    }
    if config.tool_filters {
        env.push(("TOOL_FILTERS".to_string(), "1".to_string()));
    }
    // ALWAYS pin the upstream URL. The proxy's `from_env` falls back to
    // `ANTHROPIC_BASE_URL` when `PROXY_UPSTREAM_URL` is unset — and the
    // MCP client commonly sets `ANTHROPIC_BASE_URL` to point at the proxy
    // itself (http://127.0.0.1:8787). Inheriting that would trigger the
    // proxy's self-forwarding-loop validation guard and crash it at
    // startup. Explicitly setting the upstream (config value, or the
    // proxy's built-in default when unset) shields the child from the
    // parent's environment.
    let upstream = config
        .upstream_url
        .clone()
        .unwrap_or_else(|| "https://api.anthropic.com".to_string());
    env.push(("PROXY_UPSTREAM_URL".to_string(), upstream));
    if let Some(ref key) = config.api_key {
        env.push(("PROXY_API_KEY".to_string(), key.clone()));
    }
    if config.rate_limit_rps != 60.0 {
        env.push((
            "RATE_LIMIT_RPS".to_string(),
            config.rate_limit_rps.to_string(),
        ));
    }
    if config.rate_limit_burst != 10.0 {
        env.push((
            "RATE_LIMIT_BURST".to_string(),
            config.rate_limit_burst.to_string(),
        ));
    }

    env
}

/// Check whether something is already listening on the given port.
///
/// Attempts a TCP connect to `127.0.0.1:{port}` with a short timeout.
/// Returns `true` if the connect succeeds (a proxy — or any other
/// process — is already bound to the port).
pub fn port_in_use(port: u16) -> bool {
    // The format string is always a valid socket address, so the parse
    // cannot fail; unwrap is safe here.
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

/// Spawn the proxy child process.
///
/// Returns `Ok(Some(child))` on success, `Ok(None)` if auto-start is
/// disabled (unless `force` is set), the binary could not be found, or
/// a proxy is **already running** on the configured port (adopted, not
/// duplicated — non-fatal, the MCP server continues), and `Err` only
/// on a genuine spawn failure.
///
/// `force` bypasses the `auto_start` gate. The MCP server passes
/// `false` (it already gates on `config.proxy.auto_start`); the
/// standalone `clean-ctx proxy` CLI command passes `true` so an
/// explicit invocation starts the proxy regardless of the config flag.
pub fn spawn_proxy(
    config: &ProxyAutoStartConfig,
    cwd: &Path,
    force: bool,
) -> Result<Option<Child>, String> {
    if !config.auto_start && !force {
        return Ok(None);
    }

    // Port-ownership detection: if something is already listening on the
    // configured port, adopt it instead of spawning a duplicate. This
    // prevents a second Clean-CTX instance (or a standalone proxy) from
    // being clobbered, and ensures shutdown does NOT kill a proxy owned
    // by another process (we return Ok(None) → no child to manage).
    if port_in_use(config.port) {
        eprintln!(
            "[clean-ctx] Proxy already running on port {} — adopting existing instance.",
            config.port
        );
        return Ok(None);
    }

    let binary = match resolve_proxy_binary() {
        Some(b) => b,
        None => {
            eprintln!(
                "[clean-ctx] Warning: proxy.auto_start is enabled but the clean-ctx-proxy \
                 binary was not found. Set PROXY_BINARY_PATH or install clean-ctx-proxy. \
                 Continuing without the proxy."
            );
            return Ok(None);
        }
    };

    let mut cmd = Command::new(&binary);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::inherit());

    // Apply the config → env mapping.
    for (k, v) in build_proxy_env(config) {
        cmd.env(&k, &v);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            // Fast-fail detection: a child that exits within the grace
            // period (configurable via `start_grace_ms`, default 300ms)
            // almost certainly failed to bind the port (already in use)
            // or failed config validation (e.g. self-forwarding loop).
            // Return `Ok(None)` so the caller treats it as "proxy
            // unavailable" instead of storing a dead child handle.
            let grace = Duration::from_millis(config.start_grace_ms);
            std::thread::sleep(grace);
            if let Ok(Some(status)) = child.try_wait() {
                let _ = child.wait();
                eprintln!(
                    "[clean-ctx] WARNING: clean-ctx-proxy exited shortly after start ({}). \
                     Port {} may already be in use, or proxy configuration validation failed. \
                     Continuing without the proxy.",
                    status, config.port
                );
                return Ok(None);
            }

            // Liveness probe: even if the child hasn't exited, verify the
            // port is actually listening before declaring success. This
            // catches the case where the proxy is still starting up (slow
            // disk/AV) but hasn't bound yet — we give it a short extra
            // window rather than declaring it dead.
            if !port_in_use(config.port) {
                // Give it one more grace period to bind.
                std::thread::sleep(grace);
                if let Ok(Some(status)) = child.try_wait() {
                    let _ = child.wait();
                    eprintln!(
                        "[clean-ctx] WARNING: clean-ctx-proxy exited during startup ({}). \
                         Continuing without the proxy.",
                        status
                    );
                    return Ok(None);
                }
            }

            eprintln!(
                "[clean-ctx] Proxy started on http://127.0.0.1:{} (pid {})",
                config.port,
                child.id()
            );
            Ok(Some(child))
        }
        Err(e) => Err(format!(
            "Failed to spawn clean-ctx-proxy ({}): {e}",
            binary.display()
        )),
    }
}

/// Terminate a spawned proxy child.
///
/// On Windows this uses `taskkill /PID <pid> /T /F` (tree kill to
/// catch any grandchildren); on Unix it sends `SIGTERM` and falls
/// back to `SIGKILL` after a 3-second grace period.
pub fn shutdown_proxy(child: &mut Child) {
    #[cfg(windows)]
    {
        // Windows: taskkill /T /F kills the process tree.
        let pid = child.id().to_string();
        let _ = Command::new("taskkill")
            .arg("/PID")
            .arg(&pid)
            .arg("/T")
            .arg("/F")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        // Reap the child.
        let _ = child.wait();
    }

    #[cfg(not(windows))]
    {
        // Unix: send SIGTERM, wait up to 3s, then SIGKILL.
        let _ = Command::new("kill")
            .arg(child.id().to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(_status) = child.try_wait().unwrap_or(None) {
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(child.id().to_string())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let _ = child.wait();
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

/// Check whether a spawned proxy child has exited (crashed).
///
/// Returns `true` if the child has terminated, `false` if it is still
/// running. Used by the MCP server to detect a proxy crash and log it.
pub fn proxy_child_exited(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(Some(_status)) => true,
        Ok(None) => false,
        Err(_) => false,
    }
}

/// Cache configuration reported by the proxy's `GET /cache/state` endpoint.
///
/// Phase 5 (cache-hint transport): bridges the MCP-side `_meta.cache_hints`
/// system and the proxy-side HTTP `cache_control` injection. The MCP server
/// stores this in `McpState.proxy_cache` and uses it to align its own
/// breakers with the proxy's actual injection behavior.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProxyCacheStateInfo {
    /// Whether the proxy is injecting breakpoints into /v1/messages bodies.
    #[serde(default)]
    pub auto_cache: bool,
    /// TTL used for the proxy's rolling-tail breakpoint.
    #[serde(default)]
    pub tail_ttl: String,
    /// Requests where client-sent breakpoints were preserved (injection skipped).
    #[serde(default)]
    pub client_breakpoints_preserved: u64,
    /// Requests where client-sent breakpoints were stripped.
    #[serde(default)]
    pub client_breakpoints_stripped: u64,
    /// Requests where breakpoints were injected.
    #[serde(default)]
    pub total_injected: u64,
}

/// Query the proxy's cache configuration via `GET /cache/state`.
///
/// Bridges the MCP-side `_meta.cache_hints` system and the proxy-side HTTP
/// `cache_control` injection. Returns the proxy's `auto_cache`, `tail_ttl`,
/// and breakpoint-preservation stats so the MCP server can align its own
/// breakers with the proxy's behavior.
///
/// A short 2-second global timeout is applied because this runs at MCP server
/// startup — a proxy that accepts TCP but hangs on HTTP must not block server
/// startup indefinitely.
///
/// Returns `None` if the proxy is not reachable (not running, or the
/// endpoint is unavailable) — non-fatal, the MCP server continues.
pub fn query_proxy_cache_state(port: u16) -> Option<ProxyCacheStateInfo> {
    let url = format!("http://127.0.0.1:{port}/cache/state");
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(2)))
        .build()
        .new_agent();
    let resp = agent.get(&url).call().ok()?;
    resp.into_body().read_json::<ProxyCacheStateInfo>().ok()
}

#[cfg(test)]
#[path = "tests/proxy_spawner.rs"]
mod tests;
