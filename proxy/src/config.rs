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
        }
    }
}

impl ProxyConfig {
    /// Parse configuration from environment variables.
    pub fn from_env() -> Self {
        let port = env_var("PORT")
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8787);

        let upstream_url = env_var("ANTHROPIC_BASE_URL")
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
    }
    }

    /// Check if a tool should be dropped.
    #[allow(dead_code)]
    pub fn should_drop_tool(&self, name: &str) -> bool {
        self.drop_tools_set.contains(name)
    }
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
}