// proxy/src/config.rs
//
// Environment-variable-driven configuration for the Clean-CTX Anthropic proxy.
// Mirrors Pino's env-var interface for familiarity.
//
// All vars are optional with sensible defaults.

use std::collections::{HashMap, HashSet};

/// Per-program filter override configuration.
///
/// This is part of the public configuration API for tool output filtering.
/// It is not currently used in the proxy code path but is available for
/// future use and for API completeness.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ToolFilterConfig {
    /// Global enable/disable for all filters.
    pub enabled: bool,

    /// Per-program max_lines overrides.
    pub max_lines_overrides: HashMap<String, usize>,

    /// Per-program enable/disable overrides.
    pub disabled_programs: HashSet<String>,
}

#[allow(dead_code)]
impl ToolFilterConfig {
    /// Parse from a JSON value (for .clean-ctx.json config file).
    #[allow(dead_code)]
    pub fn from_json(value: &serde_json::Value) -> Self {
        let enabled = value["enabled"].as_bool().unwrap_or(true);

        let mut max_lines_overrides = std::collections::HashMap::new();
        if let Some(overrides) = value["max_lines_overrides"].as_object() {
            for (program, lines) in overrides {
                if let Some(n) = lines.as_u64() {
                    max_lines_overrides.insert(program.clone(), n as usize);
                }
            }
        }

        let mut disabled_programs = std::collections::HashSet::new();
        if let Some(disabled) = value["disabled_programs"].as_array() {
            for program in disabled {
                if let Some(name) = program.as_str() {
                    disabled_programs.insert(name.to_string());
                }
            }
        }

        Self {
            enabled,
            max_lines_overrides,
            disabled_programs,
        }
    }

    /// Check if a program is disabled.
    #[allow(dead_code)]
    pub fn is_disabled(&self, program: &str) -> bool {
        self.disabled_programs.contains(program)
    }

    /// Get max_lines override for a program.
    #[allow(dead_code)]
    pub fn max_lines_for(&self, program: &str) -> Option<usize> {
        self.max_lines_overrides.get(program).copied()
    }
}

/// Runtime configuration for the Clean-CTX proxy.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Port to bind (default: 8787). Always 127.0.0.1.
    pub port: u16,

    /// Upstream API base URL (works with any platform).
    pub upstream_url: String,

    /// Enable auto-injection of cache_control breakpoints (Anthropic only).
    pub auto_cache: bool,

    /// TTL for the rolling-tail breakpoint (default: "5m").
    pub tail_ttl: String,

    /// Comma-separated tool names to remove from body.tools.
    pub drop_tools: Vec<String>,

    /// Set of tool names to drop (for fast lookup).
    pub drop_tools_set: HashSet<String>,

    /// Strip ANSI escape codes from text + tool_result blocks.
    pub strip_ansi: bool,

    /// Truncate Bash tool description at "Committing changes" section.
    pub trim_bash_git: bool,

    /// Override model name in every request.
    pub model_override: Option<String>,

    /// Enable request/response body logging.
    pub log_bodies: bool,

    /// Directory for log files.
    pub log_dir: String,

    /// Enable secret scrubbing in tool results.
    pub scrub_secrets: bool,

    /// Enable tool output filtering.
    pub tool_filters: bool,

    /// Platform override ("anthropic", "openai", "generic", or auto-detect).
    pub platform: Option<String>,

    /// Optional API key for X-Api-Key header authentication.
    /// When set, all requests must include a matching X-Api-Key header.
    pub api_key: Option<String>,

    /// Per-client rate limit: requests per second (default: 60).
    pub rate_limit_rps: f64,

    /// Per-client rate limit: burst window size (default: 10).
    pub rate_limit_burst: f64,

    /// Maximum request body size in bytes (default: 10 MB).
    pub max_request_body_size: usize,

    /// Enable sliding context window (age-based tool-result truncation).
    pub sliding_window_enabled: bool,

    /// Maximum age in turns before a tool result is aged (stubbed).
    pub sliding_window_max_age_turns: usize,

    /// Number of most recent turns to always preserve (force-preserve floor).
    pub sliding_window_force_preserve_floor: usize,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port: 8787,
            upstream_url: "https://api.anthropic.com".to_string(),
            auto_cache: false,
            tail_ttl: "5m".to_string(),
            drop_tools: Vec::new(),
            drop_tools_set: HashSet::new(),
            strip_ansi: false,
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
            max_request_body_size: 10 * 1024 * 1024, // 10 MB
            sliding_window_enabled: false,
            sliding_window_max_age_turns: 20,
            sliding_window_force_preserve_floor: 15,
        }
    }
}

impl ProxyConfig {
    /// Parse configuration from environment variables.
    pub fn from_env() -> Self {
        let port = env_var("PORT")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8787);

        // Upstream resolution precedence:
        //   1. PROXY_UPSTREAM_URL  — dedicated upstream target (recommended)
        //   2. COPILOT_BRIDGE_URL  — convenience alias for the Copilot bridge
        //   3. ANTHROPIC_BASE_URL  — legacy fallback (may be set by the client)
        //   4. https://api.anthropic.com — default
        //
        // Using a dedicated var decouples the proxy's upstream from the
        // client-facing ANTHROPIC_BASE_URL. Without this, pointing Claude Code
        // at the proxy (ANTHROPIC_BASE_URL=http://127.0.0.1:8787) would make
        // the proxy forward requests back to itself (self-forwarding loop).
        let upstream_url = env_var("PROXY_UPSTREAM_URL")
            .or_else(|| env_var("COPILOT_BRIDGE_URL"))
            .or_else(|| env_var("ANTHROPIC_BASE_URL"))
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        let auto_cache = env_var("AUTO_CACHE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let tail_ttl = env_var("TAIL_TTL")
            .unwrap_or_else(|| "5m".to_string());

        let drop_tools: Vec<String> = env_var("DROP_TOOLS")
            .map(|v| v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
            .unwrap_or_default();

        let drop_tools_set: HashSet<String> = drop_tools.iter().cloned().collect();

        let strip_ansi = env_var("STRIP_ANSI")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let trim_bash_git = env_var("TRIM_BASH_GIT")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let model_override = env_var("MODEL_OVERRIDE");

        let log_bodies = env_var("LOG_BODIES")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let scrub_secrets = env_var("SCRUB_SECRETS")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let tool_filters = env_var("TOOL_FILTERS")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let log_dir = env_var("LOG_DIR")
            .unwrap_or_else(|| ".clean-ctx/proxy-logs".to_string());

        Self {
            port,
            upstream_url,
            auto_cache,
            tail_ttl,
            drop_tools,
            drop_tools_set,
            strip_ansi,
            trim_bash_git,
            model_override,
            log_bodies,
            log_dir,
            scrub_secrets,
            tool_filters,
            platform: env_var("PLATFORM"),
            api_key: env_var("PROXY_API_KEY"),
            rate_limit_rps: env_var("RATE_LIMIT_RPS")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(60.0),
            rate_limit_burst: env_var("RATE_LIMIT_BURST")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(10.0),
            max_request_body_size: env_var("MAX_REQUEST_BODY_SIZE")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(10 * 1024 * 1024),
            sliding_window_enabled: env_var("SLIDING_WINDOW")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            sliding_window_max_age_turns: env_var("SLIDING_WINDOW_MAX_AGE")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(20),
            sliding_window_force_preserve_floor: env_var("SLIDING_WINDOW_FLOOR")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(15),
        }
    }

    /// Check if a tool should be dropped.
    #[allow(dead_code)]
    pub fn should_drop_tool(&self, name: &str) -> bool {
        self.drop_tools_set.contains(name)
    }

    /// Validate the configuration for runtime correctness.
    ///
    /// Currently checks for self-forwarding loops: the upstream URL must not
    /// point at this proxy's own listener address. Without this guard, a
    /// misconfigured `ANTHROPIC_BASE_URL` (e.g. the client pointing at the
    /// proxy while the proxy inherits the same env var) would cause the proxy
    /// to forward requests to itself indefinitely.
    pub fn validate(&self) -> Result<(), crate::error::ProxyError> {
        if let Some(port) = upstream_port(&self.upstream_url) {
            if port == self.port {
                return Err(crate::error::ProxyError::Config(format!(
                    "Upstream URL {} resolves to this proxy's own listener port {} — \
                     self-forwarding loop detected. Set PROXY_UPSTREAM_URL to the \
                     real upstream target (e.g. http://127.0.0.1:4141 for the \
                     Copilot bridge, or https://api.anthropic.com).",
                    self.upstream_url, self.port
                )));
            }
        }
        Ok(())
    }
}

/// Extract the port from an upstream URL string, if present.
///
/// Handles `http://127.0.0.1:4141`, `http://localhost:4141`, and
/// `https://example.com:8443`. Returns `None` for default-port URLs
/// (e.g. `https://api.anthropic.com` without an explicit port).
fn upstream_port(url: &str) -> Option<u16> {
    // Strip scheme (http:// or https://).
    let after_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    // Take up to the first slash — the authority (host[:port]) portion.
    let authority = after_scheme.split('/').next()?;
    // Handle IPv6 literals like [::1]:4141.
    let port_part = if let Some(rest) = authority.strip_prefix('[') {
        rest.split_once("]:")?.1
    } else {
        authority.rsplit_once(':')?.1
    };
    port_part.parse::<u16>().ok()
}

/// Read an environment variable, trimming whitespace.
fn env_var(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) => {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = ProxyConfig::default();
        assert_eq!(cfg.port, 8787);
        assert!(!cfg.auto_cache);
        assert_eq!(cfg.tail_ttl, "5m");
        assert!(!cfg.strip_ansi);
        assert!(!cfg.tool_filters);
    }

    #[test]
    fn test_drop_tools_parsing() {
        let mut cfg = ProxyConfig::default();
        cfg.drop_tools = vec!["Tool1".to_string(), "Tool2".to_string()];
        cfg.drop_tools_set = cfg.drop_tools.iter().cloned().collect();
        assert!(cfg.should_drop_tool("Tool1"));
        assert!(cfg.should_drop_tool("Tool2"));
        assert!(!cfg.should_drop_tool("Tool3"));
    }

    #[test]
    fn test_upstream_port_extraction() {
        assert_eq!(upstream_port("http://127.0.0.1:4141"), Some(4141));
        assert_eq!(upstream_port("http://localhost:8787"), Some(8787));
        assert_eq!(upstream_port("https://example.com:8443"), Some(8443));
        assert_eq!(upstream_port("http://[::1]:4141"), Some(4141));
        // Default-port URLs have no explicit port.
        assert_eq!(upstream_port("https://api.anthropic.com"), None);
        // Non-HTTP schemes are not parsed.
        assert_eq!(upstream_port("ftp://example.com:21"), None);
    }

    #[test]
    fn test_validate_rejects_self_forwarding() {
        let cfg = ProxyConfig {
            port: 8787,
            upstream_url: "http://127.0.0.1:8787".to_string(),
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("self-forwarding loop"));
    }

    #[test]
    fn test_validate_accepts_external_upstream() {
        let cfg = ProxyConfig {
            port: 8787,
            upstream_url: "http://127.0.0.1:4141".to_string(),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_accepts_default_anthropic() {
        let cfg = ProxyConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_proxy_upstream_url_precedence() {
        // PROXY_UPSTREAM_URL takes precedence over ANTHROPIC_BASE_URL.
        unsafe {
            std::env::set_var("PROXY_UPSTREAM_URL", "http://127.0.0.1:4141");
            std::env::set_var("ANTHROPIC_BASE_URL", "http://127.0.0.1:8787");
        }
        let cfg = ProxyConfig::from_env();
        assert_eq!(cfg.upstream_url, "http://127.0.0.1:4141");
        unsafe {
            std::env::remove_var("PROXY_UPSTREAM_URL");
            std::env::remove_var("ANTHROPIC_BASE_URL");
        }
    }

    #[test]
    fn test_copilot_bridge_url_alias() {
        // COPILOT_BRIDGE_URL is used when PROXY_UPSTREAM_URL is absent.
        unsafe {
            std::env::set_var("COPILOT_BRIDGE_URL", "http://127.0.0.1:4141");
            std::env::remove_var("PROXY_UPSTREAM_URL");
            std::env::remove_var("ANTHROPIC_BASE_URL");
        }
        let cfg = ProxyConfig::from_env();
        assert_eq!(cfg.upstream_url, "http://127.0.0.1:4141");
        unsafe {
            std::env::remove_var("COPILOT_BRIDGE_URL");
        }
    }
}
