// src/config.rs — Project-level configuration for Clean-CTX
// Reads .clean-ctx.json from the project root for custom settings

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::compression::Fidelity;
use crate::tokenizer::TokenizerKind;

// ── Smart defaults for intent-based fidelity selection ──────────────

/// Smart defaults for intent-based fidelity selection.
///
/// Maps high-level intents (`"refactor"`, `"overview"`, `"debug"`,
/// `"edit"`, `"implement"`) to compression fidelity levels. Used by
/// the heuristics engine when an explicit `fidelity` arg is not provided
/// but an `intent` is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDefaults {
    /// Fidelity for refactoring tasks — requires full structural detail.
    #[serde(default = "default_sd_refactor")]
    pub refactor: Fidelity,
    /// Fidelity for overview/summary tasks — maximum compression.
    #[serde(default = "default_sd_overview")]
    pub overview: Fidelity,
    /// Fidelity for debugging tasks — balanced detail vs compression.
    #[serde(default = "default_sd_debug")]
    pub debug: Fidelity,
    /// Fidelity for editing tasks — maximum compression, delta-friendly.
    #[serde(default = "default_sd_edit")]
    pub edit: Fidelity,
    /// Fidelity for implementation tasks — moderate detail.
    #[serde(default = "default_sd_implement")]
    pub implement: Fidelity,
}

impl Default for SmartDefaults {
    fn default() -> Self {
        Self {
            refactor: default_sd_refactor(),
            overview: default_sd_overview(),
            debug: default_sd_debug(),
            edit: default_sd_edit(),
            implement: default_sd_implement(),
        }
    }
}

fn default_sd_refactor() -> Fidelity { Fidelity::High }
fn default_sd_overview() -> Fidelity { Fidelity::Low }
fn default_sd_debug() -> Fidelity { Fidelity::Medium }
fn default_sd_edit() -> Fidelity { Fidelity::Low }
fn default_sd_implement() -> Fidelity { Fidelity::Medium }

// ── Heuristics configuration ───────────────────────────────────────

/// Heuristics configuration for automatic decisions.
///
/// Controls when `provide_code_context` switches between compression
/// strategies and fidelity levels automatically.
///
/// V2 (auto-inferred intent): files are now classified by content
/// signals (test, config, model/types, service/complex, implementation)
/// and fidelity is chosen based on classification + complexity score.
/// The core principle: more complex files → higher fidelity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsConfig {
    /// Files above this line count are treated as "large" → contributes
    /// to complexity scoring (no longer a direct Low trigger).
    #[serde(default = "default_large_file_threshold")]
    pub large_file_threshold: usize,
    /// File extensions (glob patterns) that always get high fidelity.
    /// Example: `["*.service.ts", "*.component.ts", "*.guard.ts"]`
    #[serde(default)]
    pub force_high_fidelity: Vec<String>,
    /// Whether to automatically detect and use the Angular Meta-Layer.
    #[serde(default = "default_true")]
    pub use_angular_meta: bool,

    // ── V2: Auto-classify thresholds ──────────────────────────────

    /// Min imports to classify as "service/complex" (High fidelity).
    #[serde(default = "default_complex_import_threshold")]
    pub complex_import_threshold: usize,
    /// Min functions to classify as "service/complex" (High fidelity).
    #[serde(default = "default_complex_fn_threshold")]
    pub complex_fn_threshold: usize,
    /// Min lines for complexity fallback to Medium fidelity.
    #[serde(default = "default_medium_lines")]
    pub medium_lines: usize,
    /// Min lines for complexity fallback to High fidelity.
    #[serde(default = "default_high_lines")]
    pub high_lines: usize,
    /// Whether to auto-classify files by content signals.
    /// When false, falls back to the old V1 behavior.
    #[serde(default = "default_true")]
    pub auto_classify: bool,
    /// Whether to check DB for prior fidelity on file re-visits.
    #[serde(default = "default_true")]
    pub session_aware_fidelity: bool,
}

impl Default for HeuristicsConfig {
    fn default() -> Self {
        Self {
            large_file_threshold: default_large_file_threshold(),
            force_high_fidelity: Vec::new(),
            use_angular_meta: default_true(),
            complex_import_threshold: default_complex_import_threshold(),
            complex_fn_threshold: default_complex_fn_threshold(),
            medium_lines: default_medium_lines(),
            high_lines: default_high_lines(),
            auto_classify: default_true(),
            session_aware_fidelity: default_true(),
        }
    }
}

fn default_large_file_threshold() -> usize { 300 }
fn default_complex_import_threshold() -> usize { 15 }
fn default_complex_fn_threshold() -> usize { 10 }
fn default_medium_lines() -> usize { 300 }
fn default_high_lines() -> usize { 500 }

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

fn default_max_file_size() -> usize { 10 * 1024 * 1024 } // 10 MB
fn default_max_workspace_files() -> usize { 10_000 }
fn default_max_memory_bytes() -> usize { 512 * 1024 * 1024 } // 512 MB

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

fn default_cache_enabled() -> bool { true }
fn default_stable_ttl() -> String { "1h".to_string() }
fn default_tail_ttl() -> String { "5m".to_string() }
fn default_vocab_version() -> String { "v1".to_string() }
fn default_tool_version() -> String { "v1".to_string() }

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
            // RAM-first: persistence is opt-in, not on by default.
            // Every test calling McpState::new() with default config
            // opened the same .clean-ctx/persistence.db and contended
            // on SQLite file locks, causing 60s+ hangs in parallel runs.
            enabled: false,
            auto_save: default_true(),
            max_history_days: default_max_history_days(),
            db_path: default_db_path(),
        }
    }
}

fn default_max_history_days() -> u32 { 30 }
fn default_db_path() -> String { ".clean-ctx/persistence.db".to_string() }

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

    /// Per-framework Meta-Layer configuration (Phase 1: Angular only).
    ///
    /// Each entry configures one framework meta-layer. The key is
    /// the framework name (e.g. `"angular"`); the value is the
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

fn default_export_interval() -> u64 { 60 }

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

fn default_proxy_port() -> u16 { 8787 }
fn default_proxy_tail_ttl() -> String { "5m".to_string() }
fn default_proxy_rate_limit_rps() -> f64 { 60.0 }
fn default_proxy_rate_limit_burst() -> f64 { 10.0 }
fn default_proxy_start_grace_ms() -> u64 { 300 }

/// Per-framework Meta-Layer configuration.
///
/// Phase 1 ships only the `angular` variant. Future phases will add
/// `react`, `vue`, `svelte` variants following the same pattern. Each
/// framework gets its own struct (or, when the schema converges
/// further, a shared `MetaLayerConfig` with framework-specific
/// extension fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLayerConfig {
    /// Master switch for this framework's meta-layer. Defaults to
    /// `true` (the meta-layer runs whenever the framework is
    /// detected). Set to `false` in `.clean-ctx.json` to opt out
    /// of the meta-layer entirely (zero overhead).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for MetaLayerConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
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

/// Cached path to the `.clean-ctx.json` config file, looked up once per
/// process lifetime. Edits to the config require a server restart to take
/// effect (the config is treated as immutable for the session).
static CONFIG_PATH: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

impl ResourceLimits {
    /// Validate a file size against the configured limit.
    /// Returns `Ok(())` if the file is within limits, or an error message
    /// if it exceeds the maximum allowed size.
    pub fn check_file_size(&self, size: u64) -> Result<(), String> {
        if size > self.max_file_size_bytes as u64 {
            Err(format!(
                "File size {} bytes exceeds maximum allowed size of {} bytes ({} MB). \
                 Consider using compress_workspace or the streaming variant for large files.",
                size,
                self.max_file_size_bytes,
                self.max_file_size_bytes / (1024 * 1024)
            ))
        } else {
            Ok(())
        }
    }

    /// Validate a workspace file count against the configured limit.
    /// Returns `Ok(())` if the workspace is within limits, or an error
    /// message if it exceeds the maximum allowed file count.
    pub fn check_workspace_file_count(&self, count: usize) -> Result<(), String> {
        if count > self.max_workspace_files {
            Err(format!(
                "Workspace contains {} files, which exceeds the maximum allowed count of {} files. \
                 Please reduce the workspace size or adjust the `resource_limits.max_workspace_files` \
                 setting in `.clean-ctx.json`.",
                count, self.max_workspace_files
            ))
        } else {
            Ok(())
        }
    }

    /// Validate memory usage against the configured limit.
    /// Returns `Ok(())` if the estimated memory usage is within limits,
    /// or an error message if it exceeds the maximum allowed memory.
    pub fn check_memory_usage(&self, estimated_bytes: usize) -> Result<(), String> {
        if estimated_bytes > self.max_memory_bytes {
            Err(format!(
                "Estimated memory usage {} bytes exceeds maximum allowed memory of {} bytes ({} MB). \
                 Consider processing files in smaller batches or adjusting the \
                 `resource_limits.max_memory_bytes` setting in `.clean-ctx.json`.",
                estimated_bytes,
                self.max_memory_bytes,
                self.max_memory_bytes / (1024 * 1024)
            ))
        } else {
            Ok(())
        }
    }
}

impl CleanCtxConfig {
    /// Load configuration from the project directory, walking up to find `.clean-ctx.json`
    pub fn load(start_dir: &Path) -> Self {
        let mut config = if let Some(config_path) = Self::find_config(start_dir) {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => {
                            eprintln!("[clean-ctx] Loaded config from: {}", config_path.display());
                            config
                        }
                        Err(e) => {
                            eprintln!("[clean-ctx] Warning: Failed to parse {}: {}", config_path.display(), e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[clean-ctx] Warning: Failed to read {}: {}", config_path.display(), e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        };

        // A-14: Auto-disable persistence in CI environments to prevent
        // stale persistence.db from leaking between CI builds.
        // Checks common CI environment variables: CI, TF_BUILD, GITHUB_ACTIONS, GITLAB_CI
        if config.persistence.enabled && Self::is_ci_environment() {
            eprintln!("[clean-ctx] CI environment detected — disabling persistence to prevent stale database issues");
            config.persistence.enabled = false;
        }

        config
    }

    /// Walk up from start_dir looking for `.clean-ctx.json`.
    /// Result is cached in a process-global `OnceLock` so subsequent calls
    /// do not touch the filesystem.
    pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
        CONFIG_PATH
            .get_or_init(|| Self::find_config_uncached(start_dir))
            .clone()
    }

    /// Check if running in a CI/CD environment by detecting common CI env vars.
    ///
    /// A-14: Used to auto-disable persistence in CI to prevent stale
    /// `persistence.db` from leaking between builds and causing SQLite
    /// file lock contention in parallel test runs.
    ///
    /// P1-8: Previously checked `CI == "true"` only. Now checks for any
    /// non-empty CI value (`"true"`, `"1"`, `"yes"`, etc.) since different
    /// CI systems set CI to different values. Also checks Bitbucket Pipelines
    /// and Buildkite which were missing from the original list.
    pub fn is_ci_environment() -> bool {
        // CI is the most universal CI variable — set by GitHub Actions,
        // GitLab CI, CircleCI, Travis CI, Bitbucket Pipelines, Buildkite, etc.
        // Some set it to "true", others to "1" or "yes".
        std::env::var("CI").is_ok_and(|v| !v.is_empty() && v != "false")
            || std::env::var("TF_BUILD").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GITLAB_CI").is_ok()
            || std::env::var("JENKINS_URL").is_ok()
            || std::env::var("CIRCLECI").is_ok()
            || std::env::var("TRAVIS").is_ok()
            // Additional CI systems
            || std::env::var("BITBUCKET_BUILD_NUMBER").is_ok()
            || std::env::var("BUILDKITE").is_ok()
    }

    /// Uncached directory walk — called exactly once via [`find_config`].
    fn find_config_uncached(start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir.to_path_buf();
        loop {
            let config_path = current.join(".clean-ctx.json");
            if config_path.exists() {
                return Some(config_path);
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Check if a file path should be excluded.
    ///
    /// F-12 (FAANG audit): previously used `path.contains(pattern)` which
    /// matched substrings — `"dist"` would exclude `"src/distribute/utils.ts"`.
    ///
    /// The new strategy has two tiers:
    /// 1. **Exact-segment match** (for plain patterns like `"dist"`): glob
    ///    matching against each path component. `"dist"` matches a directory
    ///    literally named `dist` but NOT `distribute`.
    /// 2. **Substring-glob match** (for patterns containing `.` like
    ///    `".test."` or `"*.spec.ts"`): glob matching against the full file
    ///    name, allowing the pattern to appear anywhere within it.
    pub fn is_excluded(&self, path: &str) -> bool {
        self.matching_exclude_patterns(path).is_some()
    }

    /// F-FINAL-04: Return the *list* of exclude patterns that matched
    /// the given path (empty if the path is not excluded). The
    /// `is_excluded` shim above is preserved for backward compatibility.
    /// This richer variant is what the workspace manifest emits so the
    /// user can see *why* a file was excluded.
    pub fn matching_exclude_patterns(&self, path: &str) -> Option<Vec<String>> {
        let path_obj = Path::new(path);
        let file_name = path_obj
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut matched: Vec<String> = Vec::new();
        for pattern in &self.exclude_patterns {
            // Tier 1: exact-segment glob match.
            let mut tier1_matched = false;
            for component in path_obj.components() {
                let segment = component.as_os_str().to_string_lossy();
                if glob_match(pattern, &segment) {
                    tier1_matched = true;
                    break;
                }
            }

            // Tier 2: filename-oriented pattern.
            let mut tier2_matched = false;
            if pattern.contains('.') {
                if (pattern.contains('*') || pattern.contains('?'))
                    && glob_match(pattern, &file_name)
                {
                    tier2_matched = true;
                }
                if !pattern.contains('*') && !pattern.contains('?')
                    && file_name.contains(pattern.as_str())
                {
                    tier2_matched = true;
                }
            }

            if tier1_matched || tier2_matched {
                matched.push(pattern.clone());
            }
        }
        if matched.is_empty() {
            None
        } else {
            Some(matched)
        }
    }

    /// Get fidelity override for a file extension
    pub fn get_fidelity_for_extension(&self, ext: &str) -> Option<Fidelity> {
        self.fidelity_overrides.get(ext).copied()
    }

    /// Generate a default config file content
    pub fn default_config_content() -> String {
        let default = Self::default();
        serde_json::to_string_pretty(&default).unwrap_or_else(|_| "{}".to_string())
    }
}

/// P1-7: Made `pub(crate)` so heuristics.rs and other modules use this
/// single implementation instead of duplicating the logic.
///
/// Minimal glob matcher supporting `*` (matches any non-separator characters)
/// and `?` (matches exactly one non-separator character). All other characters
/// are matched literally. This is intentionally simple — full `globset` support
/// can be added later if needed.
pub(crate) fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_impl(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_impl(pattern: &[u8], text: &[u8]) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi = None; // pattern position after last '*'
    let mut star_ti = 0;    // text position when '*' was matched

    while ti < text.len() {
        if pi < pattern.len() && pattern[pi] == b'*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if pi < pattern.len() && (pattern[pi] == text[ti] || pattern[pi] == b'?') {
            pi += 1;
            ti += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }

    pi == pattern.len()
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/proptest/glob_matcher.rs"]
mod proptest_tests;
