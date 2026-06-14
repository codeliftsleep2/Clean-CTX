// proxy/src/config.rs
//
// Environment-variable-driven configuration for the Clean-CTX Anthropic proxy.
// Mirrors Pino's env-var interface for familiarity.
//
// All vars are optional with sensible defaults.

use std::collections::HashSet;

/// Runtime configuration for the Anthropic prompt-cache proxy.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Port to bind (default: 8787). Always 127.0.0.1.
    pub port: u16,

    /// Upstream Anthropic API base URL.
    pub upstream_url: String,

    /// Enable auto-injection of cache_control breakpoints.
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

    /// Override model name in every request (e.g. "claude-opus-4-6").
    pub model_override: Option<String>,

    /// Enable request/response body logging.
    pub log_bodies: bool,

    /// Directory for log files.
    pub log_dir: String,
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