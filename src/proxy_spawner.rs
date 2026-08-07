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
        eprintln!("[clean-ctx] Warning: PROXY_BINARY_PATH set but not found: {}", p.display());
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
        env.push(("RATE_LIMIT_RPS".to_string(), config.rate_limit_rps.to_string()));
    }
    if config.rate_limit_burst != 10.0 {
        env.push(("RATE_LIMIT_BURST".to_string(), config.rate_limit_burst.to_string()));
    }

    env
}

/// Spawn the proxy child process.
///
/// Returns `Ok(Some(child))` on success, `Ok(None)` if auto-start is
/// disabled (unless `force` is set) or the binary could not be found
/// (non-fatal — the MCP server continues without the proxy), and `Err`
/// only on a genuine spawn failure.
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
            // Fast-fail detection: a child that exits within the first
            // 300 ms almost certainly failed to bind the port (already in
            // use) or failed config validation (e.g. self-forwarding
            // loop). Return `Ok(None)` so the caller treats it as "proxy
            // unavailable" instead of storing a dead child handle.
            std::thread::sleep(Duration::from_millis(300));
            if let Ok(Some(status)) = child.try_wait() {
                let _ = child.wait();
                eprintln!(
                    "[clean-ctx] WARNING: clean-ctx-proxy exited shortly after start ({}). \
                     Port {} may already be in use, or proxy configuration validation failed. \
                     Continuing without the proxy.",
                    status,
                    config.port
                );
                return Ok(None);
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

#[cfg(test)]
#[path = "tests/proxy_spawner.rs"]
mod tests;