// src/cbm/config.rs
//
// CBM (codebase-memory-mcp) integration configuration.
// Entirely self-contained — CleanCtxConfig references this via a single `cbm` field.

use serde::{Deserialize, Serialize};

/// CBM integration configuration, loaded from `.clean-ctx.json` `cbm` key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbmConfig {
    /// Path to the `codebase-memory-mcp` binary. When `None`, auto-detect on PATH.
    #[serde(default)]
    pub binary_path: Option<String>,

    /// Automatically launch CBM as subprocess on first graph query. Default: `true`.
    #[serde(default = "default_auto_launch")]
    pub auto_launch: bool,

    /// TTL (seconds) for cached graph data. Default: 300 (5 min).
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,

    /// Master switch. When `false`, all CBM features disabled. Default: `true`.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Timeout (ms) for CBM queries. Default: 30000 (30s).
    #[serde(default = "default_query_timeout_ms")]
    pub query_timeout_ms: u64,

    /// Minimum compatible CBM version (semver). Warn at startup if incompatible.
    #[serde(default)]
    pub cbm_min_version: Option<String>,

    /// Log directory for CBM proxy logs. Relative to project root.
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
}

impl Default for CbmConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
            auto_launch: default_auto_launch(),
            cache_ttl: default_cache_ttl(),
            enabled: default_enabled(),
            query_timeout_ms: default_query_timeout_ms(),
            cbm_min_version: None,
            log_dir: default_log_dir(),
        }
    }
}

fn default_auto_launch() -> bool { true }
fn default_cache_ttl() -> u64 { 300 }
fn default_enabled() -> bool { true }
fn default_query_timeout_ms() -> u64 { 30000 }
fn default_log_dir() -> String { ".clean-ctx/cbm-logs".to_string() }

/// Status of the CBM integration, surfaced via `get_cbm_status` and `context_stats`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum CbmStatus {
    Available,
    Degraded(String),
    #[default]
    Unavailable,
}

impl CbmStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, CbmStatus::Available)
    }

    pub fn summary(&self) -> &str {
        match self {
            CbmStatus::Available => "available",
            CbmStatus::Degraded(_) => "degraded",
            CbmStatus::Unavailable => "unavailable",
        }
    }
}

