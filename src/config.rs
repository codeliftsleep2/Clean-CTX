// src/config.rs — Project-level configuration for Clean-CTX
// Reads .clean-ctx.json from the project root for custom settings

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
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
    pub refactor: String,
    /// Fidelity for overview/summary tasks — maximum compression.
    #[serde(default = "default_sd_overview")]
    pub overview: String,
    /// Fidelity for debugging tasks — balanced detail vs compression.
    #[serde(default = "default_sd_debug")]
    pub debug: String,
    /// Fidelity for editing tasks — maximum compression, delta-friendly.
    #[serde(default = "default_sd_edit")]
    pub edit: String,
    /// Fidelity for implementation tasks — moderate detail.
    #[serde(default = "default_sd_implement")]
    pub implement: String,
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

fn default_sd_refactor() -> String { "high".to_string() }
fn default_sd_overview() -> String { "low".to_string() }
fn default_sd_debug() -> String { "medium".to_string() }
fn default_sd_edit() -> String { "low".to_string() }
fn default_sd_implement() -> String { "medium".to_string() }

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

/// Persistence configuration (placeholder for future SQLite layer).
///
/// Currently parsed but not acted upon. The fields define where and
/// how the SQLite-backed `ContextStore` will persist compression
/// baselines, deltas, and session history across IDE restarts.
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
            enabled: true,
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
    pub fidelity_overrides: BTreeMap<String, String>,

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
    pub default_fidelity: String,

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

    /// CBM (codebase-memory-mcp) integration configuration.
    /// Controls how Clean-CTX discovers, launches, and communicates
    /// with the CBM server for graph intelligence.
    #[serde(default)]
    pub cbm: crate::cbm::CbmConfig,
}

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

fn default_fidelity() -> String {
    "low".to_string()
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
            cbm: crate::cbm::CbmConfig::default(),
        }
    }
}

/// Cached path to the `.clean-ctx.json` config file, looked up once per
/// process lifetime. Edits to the config require a server restart to take
/// effect (the config is treated as immutable for the session).
static CONFIG_PATH: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

impl CleanCtxConfig {
    /// Load configuration from the project directory, walking up to find `.clean-ctx.json`
    pub fn load(start_dir: &Path) -> Self {
        if let Some(config_path) = Self::find_config(start_dir) {
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
        }
    }

    /// Walk up from start_dir looking for `.clean-ctx.json`.
    /// Result is cached in a process-global `OnceLock` so subsequent calls
    /// do not touch the filesystem.
    pub(crate) fn find_config(start_dir: &Path) -> Option<PathBuf> {
        CONFIG_PATH
            .get_or_init(|| Self::find_config_uncached(start_dir))
            .clone()
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
    pub fn get_fidelity_for_extension(&self, ext: &str) -> Option<&str> {
        self.fidelity_overrides.get(ext).map(|s| s.as_str())
    }

    /// Generate a default config file content
    pub fn default_config_content() -> String {
        let default = Self::default();
        serde_json::to_string_pretty(&default).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Minimal glob matcher supporting `*` (matches any non-separator characters)
/// and `?` (matches exactly one non-separator character). All other characters
/// are matched literally. This is intentionally simple — full `globset` support
/// can be added later if needed.
fn glob_match(pattern: &str, text: &str) -> bool {
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
