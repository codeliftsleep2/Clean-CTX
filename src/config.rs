// src/config.rs — Project-level configuration for Clean-CTX
// Reads .clean-ctx.json from the project root for custom settings

use crate::compression::Fidelity;
use crate::tokenizer::TokenizerKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[path = "config_impl.rs"]
mod config_impl;
pub(crate) use config_impl::glob_match;

#[path = "config_defaults.rs"]
mod config_defaults;
pub use config_defaults::{HeuristicsConfig, SmartDefaults};

#[path = "config_meta_layers.rs"]
mod config_meta_layers;
pub use config_meta_layers::{
    MetaLayerConfig, NgRxConfig, ReactiveFormsConfig, RoutingConfig, RxJsConfig, SignalsConfig,
    TestingConfig,
};

// ── Resource limits ───────────────────────────────────────────────

/// Resource limits and memory guardrails.
///
/// Controls maximum file sizes, workspace file counts, and memory
/// usage to prevent OOM crashes on large codebases. When limits are
/// exceeded, graceful error messages are returned instead of crashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum file size in bytes. Files larger than this are skipped
    /// with a warning. Default: 10 MB.
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: usize,

    /// Maximum number of files in a workspace. Workspaces with more
    /// files are rejected with an error. Default: 10,000.
    #[serde(default = "default_max_workspace_files")]
    pub max_workspace_files: usize,

    /// Maximum memory usage in bytes for compression operations.
    /// When exceeded, compression is aborted gracefully. Default: 512 MB.
    #[serde(default = "default_max_memory_bytes")]
    pub max_memory_bytes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_file_size_bytes: default_max_file_size(),
            max_workspace_files: default_max_workspace_files(),
            max_memory_bytes: default_max_memory_bytes(),
        }
    }
}

fn default_max_file_size() -> usize {
    10 * 1024 * 1024
} // 10 MB
fn default_max_workspace_files() -> usize {
    10_000
}
fn default_max_memory_bytes() -> usize {
    512 * 1024 * 1024
} // 512 MB

// ── Cache configuration ──────────────────────────────────────────

/// Prompt cache configuration for Anthropic API breakpoint optimization.
///
/// Controls cache breakpoint injection into JSON-RPC `_meta.cache_hints`
/// fields. When enabled, the MCP server annotates stable content responses
/// (system prompt vocabulary, tool definitions, persisted baselines) with
/// `cache_control` hints so the LLM never re-pays the 1.25× write
/// multiplier on content that hasn't changed.
///
/// Defaults are chosen for out-of-the-box savings: stable regions get
/// a 1-hour TTL, the rolling tail (dynamic content) gets the Anthropic
/// 5-minute default fallback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Master switch for prompt cache optimization annotations.
    /// When `false`, no `_meta.cache_hints` are injected into any response.
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// TTL for the system prompt / opcode vocabulary prompt resource.
    /// The vocabulary is stable across every session — it only changes
    /// on a binary version bump. Default: "1h".
    #[serde(default = "default_stable_ttl")]
    pub system_prompt_ttl: String,

    /// TTL for the MCP tool definitions block (~24k tokens).
    /// Tool definitions are stable across every session — they only
    /// change when tools are added/removed. Default: "1h".
    #[serde(default = "default_stable_ttl")]
    pub tools_ttl: String,

    /// TTL for persisted workspace baselines (unchanged files).
    /// Stable until file content changes. Default: "1h".
    #[serde(default = "default_stable_ttl")]
    pub baseline_ttl: String,

    /// TTL for the rolling tail (dynamic content that changes each turn).
    /// Matches Anthropic's 5-minute default fallback so we don't pay the
    /// 2.0× write multiplier on content that changes every turn.
    #[serde(default = "default_tail_ttl")]
    pub tail_ttl: String,

    /// Semantic version of the opcode vocabulary.
    /// Bumped only when opcodes/markers change in the codebase.
    #[serde(default = "default_vocab_version")]
    pub vocab_version: String,

    /// Semantic version of the tool definitions.
    /// Bumped only when tools are added or removed.
    #[serde(default = "default_tool_version")]
    pub tool_defs_version: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            system_prompt_ttl: default_stable_ttl(),
            tools_ttl: default_stable_ttl(),
            baseline_ttl: default_stable_ttl(),
            tail_ttl: default_tail_ttl(),
            vocab_version: default_vocab_version(),
            tool_defs_version: default_tool_version(),
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}
fn default_stable_ttl() -> String {
    "1h".to_string()
}
fn default_tail_ttl() -> String {
    "5m".to_string()
}
fn default_vocab_version() -> String {
    "v1".to_string()
}
fn default_tool_version() -> String {
    "v1".to_string()
}

// ── Persistence configuration (placeholder) ────────────────────────

/// Persistence configuration for SQLite-backed cross-session storage.
///
/// Controls where and how the `ContextStore` (via `SqliteStore`) persists
/// compression baselines, deltas, and session history across IDE restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Master switch for persistence. When `false`, all operations
    /// are purely in-memory (current behaviour).
    #[serde(default)]
    pub enabled: bool,
    /// Automatically save context after each compression/delta operation.
    #[serde(default = "default_true")]
    pub auto_save: bool,
    /// Maximum days to retain history before pruning.
    #[serde(default = "default_max_history_days")]
    pub max_history_days: u32,
    /// Path to the SQLite database file (relative to project root).
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            // Persistence is ON by default — cross-session compression
            // history is a core feature. The A-14 CI detection auto-disables
            // persistence in CI environments (CI=true, TF_BUILD, etc.) to
            // prevent SQLite file lock contention in parallel test runs.
            enabled: true,
            auto_save: default_true(),
            max_history_days: default_max_history_days(),
            db_path: default_db_path(),
        }
    }
}

fn default_max_history_days() -> u32 {
    30
}
fn default_db_path() -> String {
    ".clean-ctx/persistence.db".to_string()
}

// ── Main config struct ─────────────────────────────────────────────

/// Project-level configuration loaded from `.clean-ctx.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanCtxConfig {
    /// Custom type aliases: short_name → original_type
    #[serde(default)]
    pub type_aliases: BTreeMap<String, String>,

    /// Fidelity override per file extension
    #[serde(default)]
    pub fidelity_overrides: BTreeMap<String, Fidelity>,

    /// File/directory patterns to exclude from compression.
    ///
    /// Supports simple glob syntax: `*` matches any sequence of non-separator
    /// characters, `?` matches exactly one non-separator character. Patterns
    /// are matched against each **path segment** (component), so `"dist"`
    /// matches any directory or file named `dist`, but `"distribute"` does
    /// NOT match `"dist"` (unlike the old substring check). Use `"dist*"`
    /// to match both.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Additional workspace roots to scan for cross-file symbol resolution.
    ///
    /// Each entry is an absolute path to a directory that should be
    /// treated as part of the workspace for type detection, symbol
    /// compression, and CBM graph queries. Paths are validated at
    /// check time; a path that doesn't exist is silently skipped rather
    /// than erroring the whole config.
    ///
    /// Example `.clean-ctx.json`:
    /// ```json
    /// { "additional_roots": ["C:\\Users\\me\\source\\repos\\Test"] }
    /// ```
    #[serde(default)]
    pub additional_roots: Vec<String>,

    /// Custom behavior markers: marker → description
    #[serde(default)]
    pub custom_markers: BTreeMap<String, String>,

    /// Default fidelity level if not specified
    #[serde(default = "default_fidelity")]
    pub default_fidelity: Fidelity,

    /// Whether to enable diff-aware compression
    #[serde(default = "default_true")]
    pub diff_compression: bool,

    /// Whether to enable workspace-wide type detection
    #[serde(default = "default_true")]
    pub workspace_type_detection: bool,

    /// Per-framework Meta-Layer configuration.
    ///
    /// Each entry configures one framework meta-layer. The key is
    /// the framework name (for example `"angular"` or `"dotnet"`); the value is the
    /// per-framework config struct. A missing entry means the
    /// framework meta-layer is on (default behaviour — see the
    /// framework-specific config for the opt-out flag).
    ///
    /// Example `.clean-ctx.json`:
    /// ```json
    /// { "meta_layers": { "angular": { "enabled": false } } }
    /// ```
    #[serde(default)]
    pub meta_layers: BTreeMap<String, MetaLayerConfig>,

    /// Smart defaults for intent-based fidelity selection.
    #[serde(default)]
    pub smart_defaults: SmartDefaults,

    /// Heuristics configuration for automatic decisions.
    #[serde(default)]
    pub heuristics: HeuristicsConfig,

    /// Persistence configuration (placeholder for future SQLite layer).
    #[serde(default)]
    pub persistence: PersistenceConfig,

    /// Auto-detect Angular files and enable Meta-Layer markers.
    #[serde(default = "default_true")]
    pub auto_angular: bool,

    /// Automatically use deltas for follow-up edits in `provide_code_context`.
    #[serde(default = "default_true")]
    pub auto_delta: bool,

    /// Default tokenizer backend for token counting.
    ///
    /// Supported values: `"o200k"` (default), `"cl100k"`, `"claude"`, `"llama3"`.
    /// This can be overridden per-tool-call via the `tokenizer` argument.
    #[serde(default)]
    pub tokenizer: TokenizerKind,

    /// Prompt cache configuration for Anthropic API breakpoint optimization.
    /// Controls injection of `_meta.cache_hints` into MCP responses for
    /// stable content regions (vocabulary, tools, baselines).
    #[serde(default)]
    pub cache: CacheConfig,

    /// Resource limits and memory guardrails.
    #[serde(default)]
    pub resource_limits: ResourceLimits,

    /// CBM (codebase-memory-mcp) integration configuration.
    /// Controls how Clean-CTX discovers, launches, and communicates
    /// with the CBM server for graph intelligence.
    #[serde(default)]
    pub cbm: crate::cbm::CbmConfig,

    /// Intelligence Layer configuration (CBM-informed fidelity,
    /// PageRank, blast radius). When enabled, the heuristics engine
    /// consults CBM symbol importance scores to adjust compression
    /// fidelity for high- or low-importance files.
    #[serde(default)]
    pub intelligence: IntelligenceConfig,

    /// Observability configuration for metrics export.
    #[serde(default)]
    pub observability: ObservabilityConfig,

    /// Proxy auto-start configuration. When `auto_start` is `true`,
    /// the MCP server spawns the `clean-ctx-proxy` binary as a child
    /// process on startup and terminates it on shutdown.
    #[serde(default)]
    pub proxy: ProxyAutoStartConfig,
}

/// Intelligence Layer configuration.
///
/// Controls whether the CBM-informed fidelity pipeline runs inside
/// the heuristics engine. When enabled and the CBM graph bridge is
/// available, per-file symbol importance scores from CBM can
/// override the standard fidelity decision:
///
///   - High importance (>0.8) → force High fidelity
///   - Low importance (<0.4) → force Low fidelity
///   - Medium (0.4-0.8) → defer to standard heuristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    /// Master switch. When `true` (default), CBM-informed fidelity
    /// recommendations are consulted in the heuristics engine.
    /// When `false`, the intelligence layer is entirely skipped
    /// (zero overhead).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Enable blast radius analysis (depth-1 affected files).
    /// When enabled, the compression output includes depth-1 affected
    /// files from CBM with context-aware fidelity selection.
    #[serde(default)]
    pub blast_radius_enabled: bool,
    /// Maximum number of blast radius files to include per request.
    /// Prevents token explosion from highly-connected symbols.
    /// Default: 10 files.
    #[serde(default = "default_max_blast_radius")]
    pub max_blast_radius_files: usize,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            blast_radius_enabled: false,
            max_blast_radius_files: 10,
        }
    }
}

fn default_max_blast_radius() -> usize {
    10
}

/// Observability configuration for metrics export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable periodic metrics export to stdout. Default: false.
    #[serde(default)]
    pub export_metrics: bool,
    /// Interval in seconds between metrics snapshots. Default: 60.
    #[serde(default = "default_export_interval")]
    pub export_interval_secs: u64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            export_metrics: false,
            export_interval_secs: default_export_interval(),
        }
    }
}

fn default_export_interval() -> u64 {
    60
}

// ── Proxy auto-start configuration ────────────────────────────────

/// Auto-start configuration for the Clean-CTX proxy.
///
/// When `auto_start` is `true`, the MCP server spawns the `clean-ctx-proxy`
/// binary as a child process on startup, maps each field to the proxy's
/// environment variables (see `proxy/src/config.rs`), and terminates the
/// child on shutdown. Defaults mirror the proxy's env-var defaults so an
/// empty JSON block behaves identically to an unconfigured proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAutoStartConfig {
    /// Master switch. When `true`, the MCP server spawns the proxy on
    /// startup and terminates it on shutdown. Default: `false`.
    #[serde(default)]
    pub auto_start: bool,
    /// Port to bind (always 127.0.0.1). Maps to `PORT`. Default: 8787.
    #[serde(default = "default_proxy_port")]
    pub port: u16,
    /// Enable cache_control breakpoint injection (Anthropic only).
    /// Maps to `AUTO_CACHE`. Default: `false`.
    #[serde(default)]
    pub auto_cache: bool,
    /// TTL for the rolling-tail breakpoint. Maps to `TAIL_TTL`.
    /// Default: "5m".
    #[serde(default = "default_proxy_tail_ttl")]
    pub tail_ttl: String,
    /// Comma-separated tool names to remove from request bodies.
    /// Maps to `DROP_TOOLS`. Default: empty.
    #[serde(default)]
    pub drop_tools: Vec<String>,
    /// Strip ANSI escape codes from tool results. Maps to `STRIP_ANSI`.
    /// Default: `false`.
    #[serde(default)]
    pub strip_ansi: bool,
    /// Truncate Bash tool output at "Committing changes". Maps to
    /// `TRIM_BASH_GIT`. Default: `false`.
    #[serde(default)]
    pub trim_bash_git: bool,
    /// Override the model name in every request. Maps to `MODEL_OVERRIDE`.
    /// Default: none.
    #[serde(default)]
    pub model_override: Option<String>,
    /// Enable secret scrubbing in tool results. Maps to `SCRUB_SECRETS`.
    /// Default: `false`.
    #[serde(default)]
    pub scrub_secrets: bool,
    /// Enable TOML-based tool output filtering. Maps to `TOOL_FILTERS`.
    /// Default: `false`.
    #[serde(default)]
    pub tool_filters: bool,
    /// Dedicated upstream URL. Maps to `PROXY_UPSTREAM_URL`.
    /// Default: none (the proxy falls back to `https://api.anthropic.com`).
    #[serde(default)]
    pub upstream_url: Option<String>,
    /// Optional API key for `X-Api-Key` header authentication.
    /// Maps to `PROXY_API_KEY`. Default: none.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Per-client requests per second. Maps to `RATE_LIMIT_RPS`.
    /// Default: 60.
    #[serde(default = "default_proxy_rate_limit_rps")]
    pub rate_limit_rps: f64,
    /// Per-client burst window size. Maps to `RATE_LIMIT_BURST`.
    /// Default: 10.
    #[serde(default = "default_proxy_rate_limit_burst")]
    pub rate_limit_burst: f64,
    /// Startup grace period in milliseconds before the spawner declares
    /// a freshly-spawned proxy dead. Slow disks or antivirus scanners can
    /// delay binary startup past the default 300ms; raise this if you see
    /// spurious "exited shortly after start" warnings. Default: 300.
    #[serde(default = "default_proxy_start_grace_ms")]
    pub start_grace_ms: u64,
}

impl Default for ProxyAutoStartConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            port: default_proxy_port(),
            auto_cache: false,
            tail_ttl: default_proxy_tail_ttl(),
            drop_tools: Vec::new(),
            strip_ansi: false,
            trim_bash_git: false,
            model_override: None,
            scrub_secrets: false,
            tool_filters: false,
            upstream_url: None,
            api_key: None,
            rate_limit_rps: default_proxy_rate_limit_rps(),
            rate_limit_burst: default_proxy_rate_limit_burst(),
            start_grace_ms: default_proxy_start_grace_ms(),
        }
    }
}

fn default_proxy_port() -> u16 {
    8787
}
fn default_proxy_tail_ttl() -> String {
    "5m".to_string()
}
fn default_proxy_rate_limit_rps() -> f64 {
    60.0
}
fn default_proxy_rate_limit_burst() -> f64 {
    10.0
}
fn default_proxy_start_grace_ms() -> u64 {
    300
}

fn default_fidelity() -> Fidelity {
    Fidelity::Low
}

fn default_true() -> bool {
    true
}

impl Default for CleanCtxConfig {
    fn default() -> Self {
        Self {
            type_aliases: BTreeMap::new(),
            fidelity_overrides: BTreeMap::new(),
            exclude_patterns: Vec::new(),
            additional_roots: Vec::new(),
            custom_markers: BTreeMap::new(),
            default_fidelity: default_fidelity(),
            diff_compression: default_true(),
            workspace_type_detection: default_true(),
            meta_layers: BTreeMap::new(),
            smart_defaults: SmartDefaults::default(),
            heuristics: HeuristicsConfig::default(),
            persistence: PersistenceConfig::default(),
            auto_angular: default_true(),
            auto_delta: default_true(),
            tokenizer: TokenizerKind::default(),
            cache: CacheConfig::default(),
            resource_limits: ResourceLimits::default(),
            cbm: crate::cbm::CbmConfig::default(),
            intelligence: IntelligenceConfig::default(),
            observability: ObservabilityConfig::default(),
            proxy: ProxyAutoStartConfig::default(),
        }
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/proptest/glob_matcher.rs"]
mod proptest_tests;
