use super::*;

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

fn default_sd_refactor() -> Fidelity {
    Fidelity::High
}
fn default_sd_overview() -> Fidelity {
    Fidelity::Low
}
fn default_sd_debug() -> Fidelity {
    Fidelity::Medium
}
fn default_sd_edit() -> Fidelity {
    Fidelity::Edit
}
fn default_sd_implement() -> Fidelity {
    Fidelity::Medium
}

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
    /// Auto-select Edit fidelity for implementation/service files
    /// when no explicit intent/fidelity is given. When true, files
    /// classified as Service or Implementation get `Fidelity::Edit`
    /// so method bodies are carried verbatim for safe edits.
    #[serde(default = "default_true")]
    pub auto_edit_mode: bool,
    /// File classes that auto-select Edit fidelity when `auto_edit_mode`
    /// is on and no explicit intent/fidelity is given. Class names match
    /// the `FileClass` variants as lowercase strings ("service",
    /// "implementation", etc.). Defaults to ["service", "implementation"].
    #[serde(default = "default_edit_auto_classifications")]
    pub edit_auto_classifications: Vec<String>,
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
            auto_edit_mode: default_true(),
            edit_auto_classifications: default_edit_auto_classifications(),
        }
    }
}

fn default_edit_auto_classifications() -> Vec<String> {
    vec!["service".to_string(), "implementation".to_string()]
}

fn default_large_file_threshold() -> usize {
    300
}
fn default_complex_import_threshold() -> usize {
    15
}
fn default_complex_fn_threshold() -> usize {
    10
}
fn default_medium_lines() -> usize {
    300
}
fn default_high_lines() -> usize {
    500
}
