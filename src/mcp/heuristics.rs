// src/mcp/heuristics.rs
//
// Heuristics engine V2 for `provide_code_context`.
//
// Decides the compression strategy (full vs delta) and fidelity level
// based on config, file characteristics, content-based classification,
// evidence from the persistence DB, and explicit intent.
//
// V2 (auto-inferred intent): files are classified by cheap content
// signals (test, config, model/types, service/complex, implementation)
// and fidelity is chosen based on classification + complexity score.
// Core principle: more complex files → higher fidelity.
//
// Priority order:
//   1. Explicit `fidelity` arg → use it directly.
//   2. Explicit `intent` arg → map via `config.smart_defaults`.
//   3. DB baseline fidelity (if session_aware_fidelity is on AND file
//      hasn't changed since last compress).
//   4. Content-based classification (test→Low, config→Low, model→Medium,
//      service→High, implementation→Medium).
//   5. Complexity-based fallback (imports + functions + lines → fidelity).
//   6. Config's `default_fidelity` → last resort.

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::replay::ContextState;
use std::path::Path;

/// What compression strategy should `provide_code_context` use?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStrategy {
    /// First time seeing this file — do full compression.
    FullCompress,
    /// File seen before in this session — use delta transport.
    DeltaTransport,
}

/// Content-based file classification (V2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileClass {
    /// Test files: #[test], #[cfg(test)], @Test, describe(, it(, /test/ in path
    Test,
    /// Config files: config/settings in path, .json/.toml/.yaml, small const-only files
    Config,
    /// Model/type files: high ratio of struct/enum/type to function definitions
    Model,
    /// Service/complex files: many imports + many functions
    Service,
    /// Implementation files: .component., .controller., .handler., .service. in path
    Implementation,
    /// Unclassified — falls through to complexity scoring
    General,
}

/// The resolved decision for a single `provide_code_context` call.
#[derive(Debug, Clone)]
pub struct ContextDecision {
    /// The resolved fidelity level.
    pub fidelity: Fidelity,
    /// The compression strategy to use.
    pub strategy: ContextStrategy,
    /// Whether Angular Meta-Layer markers should be emitted.
    pub is_angular: bool,
    /// Source line count (used for large-file heuristics).
    pub source_line_count: usize,
    /// V2: the file classification that was used (or General if none).
    pub file_class: FileClass,
    /// Whether the CBM Intelligence Layer influenced this decision.
    pub cbm_informed: bool,
}

impl ContextDecision {
    /// Human-readable summary of the decision.
    pub fn summary(&self) -> String {
        let strategy_str = match self.strategy {
            ContextStrategy::FullCompress => "full_compress",
            ContextStrategy::DeltaTransport => "delta",
        };
        let angular_str = if self.is_angular { "angular" } else { "none" };
        let class_str = match self.file_class {
            FileClass::Test => "test",
            FileClass::Config => "config",
            FileClass::Model => "model",
            FileClass::Service => "service",
            FileClass::Implementation => "implementation",
            FileClass::General => "general",
        };
        let cbm_str = if self.cbm_informed {
            "cbm_informed"
        } else {
            "no_cbm"
        };
        format!(
            "fidelity={:?}, strategy={}, class={}, angular={}, lines={}, cbm={}",
            self.fidelity, strategy_str, class_str, angular_str, self.source_line_count, cbm_str
        )
    }
}

// ── Signal Detection Functions (cheap string scans, no tree-sitter) ──

/// Count import-like lines in source text.
/// Rust: "use " | TS/JS: "import " | C#: "using "
fn count_imports(source: &str) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("use ")
                || trimmed.starts_with("extern crate ")
                || trimmed.starts_with("import ")
                || trimmed.starts_with("from ")
                || trimmed.starts_with("using ")
        })
        .count()
}

/// Count function definitions in source text.
/// Rust: "fn " at line start | TS/JS: "function " | C#: type patterns
fn count_functions(source: &str) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Rust
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn ")
            {
                return true;
            }
            // TypeScript/JavaScript
            if trimmed.starts_with("function ")
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("async function ")
            {
                return true;
            }
            // C# method patterns: return type followed by method name and (
            if (trimmed.starts_with("void ")
                || trimmed.starts_with("Task ")
                || trimmed.starts_with("int ")
                || trimmed.starts_with("string ")
                || trimmed.starts_with("bool ")
                || trimmed.starts_with("async "))
                && trimmed.contains('(')
                && trimmed.contains(')')
            {
                return true;
            }
            false
        })
        .count()
}

/// Count struct/enum/trait/interface/type definitions.
/// M-1 fix: excludes `impl ` blocks (method implementations, not type definitions).
fn count_structs_enums(source: &str) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Rust — exclude bare `impl ` (method impl blocks like `impl User {`)
            // Keep `impl<` (generic impl) and `impl Trait for Type` (trait impls)
            trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ")
        // Only count impl blocks that have generics or trait syntax
            || trimmed.starts_with("impl<")
            || (trimmed.starts_with("impl ") && trimmed.contains(" for "))
        // TypeScript
            || trimmed.starts_with("interface ") || trimmed.starts_with("export interface ")
            || trimmed.starts_with("type ") || trimmed.starts_with("export type ")
            || trimmed.starts_with("class ") || trimmed.starts_with("export class ")
        // C#
            || trimmed.starts_with("class ") || trimmed.starts_with("public class ")
            || trimmed.starts_with("struct ") || trimmed.starts_with("public struct ")
            || trimmed.starts_with("interface ") || trimmed.starts_with("public interface ")
            || trimmed.starts_with("enum ") || trimmed.starts_with("public enum ")
        })
        .count()
}

/// Count test markers in source text.
/// M-2 fix: removed `fn test_` (too broad — catches test helper functions).
/// Relies on explicit test markers and path detection instead.
fn count_test_markers(source: &str) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("#[test]")
                || trimmed.starts_with("#[cfg(test)]")
                || trimmed.starts_with("@Test")
                || trimmed.starts_with("describe(")
                || trimmed.starts_with("it(")
        })
        .count()
}

/// Check if the file path suggests a test file.
fn is_test_path(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    // Match /test/ or /tests/ or /__tests__/ as path segments (not substrings)
    lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains("/__tests__/")
        || lower.contains(".test.")
        || lower.contains(".spec.")
}

/// Check if the file path suggests a config file.
/// M-3 fix: uses path segment matching instead of substring match.
/// "config.rs" or "/config/" matches, but "configure.rs" does not.
fn is_config_path(file_path: &str) -> bool {
    let path = Path::new(file_path);
    // Check each path component for exact "config" or "settings"
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy().to_lowercase();
        if s == "config" || s == "settings" || s == "configs" {
            return true;
        }
    }
    // Check file extension for config-like formats
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if ext_lower == "json"
            || ext_lower == "toml"
            || ext_lower == "yaml"
            || ext_lower == "yml"
            || ext_lower == "env"
        {
            return true;
        }
    }
    false
}

/// Check if the file path suggests an implementation file.
fn is_implementation_path(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    lower.contains(".component.")
        || lower.contains(".controller.")
        || lower.contains(".handler.")
        || lower.contains(".middleware.")
        || lower.contains(".service.")
        || lower.contains(".repository.")
        || lower.contains(".guard.")
        || lower.contains(".interceptor.")
        || lower.contains(".resolver.")
        || lower.contains(".pipe.")
}

/// Check if the file path is an Angular `.component.html` template.
///
/// ANGULAR_HTML_COMPRESSION_PLAN Phase 3: these files are classified
/// as `FileClass::Implementation` with `Fidelity::Medium` default, and
/// "template editing" (intent = "edit") triggers High fidelity.
fn is_angular_template_path(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    lower.ends_with(".component.html")
}

// ── Content-Based Classification (V2) ──────────────────────────────

/// Classify a file based on its content and path.
///
/// Returns the classification and the fidelity it maps to.
fn classify_file(file_path: &str, source: &str, config: &CleanCtxConfig) -> (FileClass, Fidelity) {
    let line_count = source.lines().count();
    let import_count = count_imports(source);
    let fn_count = count_functions(source);
    let struct_count = count_structs_enums(source);
    let test_markers = count_test_markers(source);

    // Tier 1: Test files
    if test_markers > 0 || is_test_path(file_path) {
        return (FileClass::Test, Fidelity::Low);
    }

    // ANGULAR_HTML_COMPRESSION_PLAN Phase 3: `.component.html` files
    // are implementation files with Medium fidelity default. This must
    // be checked BEFORE the config classification (small HTML files
    // would otherwise be misclassified as Config).
    if is_angular_template_path(file_path) {
        return (FileClass::Implementation, Fidelity::Medium);
    }

    // Tier 2: Config files
    if is_config_path(file_path) {
        return (FileClass::Config, Fidelity::Low);
    }
    // Small files with mostly const/static declarations → config-like
    if line_count < 50 && fn_count == 0 && struct_count <= 1 {
        return (FileClass::Config, Fidelity::Low);
    }

    // Tier 3: Model/type files (high struct:fn ratio)
    if struct_count > 0 && fn_count == 0 {
        return (FileClass::Model, Fidelity::Medium);
    }
    if struct_count > fn_count * 3 && struct_count >= 3 {
        return (FileClass::Model, Fidelity::Medium);
    }

    // Tier 4: Service/complex files (many imports + many functions)
    if import_count >= config.heuristics.complex_import_threshold
        && fn_count >= config.heuristics.complex_fn_threshold
    {
        return (FileClass::Service, Fidelity::High);
    }

    // Tier 5: Implementation files (path-based)
    if is_implementation_path(file_path) {
        return (FileClass::Implementation, Fidelity::Medium);
    }
    // Moderate-size files with a mix of imports/functions/types → implementation
    if line_count > 200 && (import_count > 0 || fn_count > 0 || struct_count > 0) {
        return (FileClass::Implementation, Fidelity::Medium);
    }

    // Tier 6: General — falls through to complexity scoring
    (FileClass::General, Fidelity::Low)
}

// ── Complexity-Based Fallback (V2 — REVERSED from V1) ──────────────

/// Determine fidelity from complexity metrics when no classifier matched.
///
/// V2 principle: more complex files → higher fidelity.
fn fidelity_from_complexity(source: &str, line_count: usize, config: &CleanCtxConfig) -> Fidelity {
    let import_count = count_imports(source);
    let fn_count = count_functions(source);

    // High complexity: many imports + many functions + large file
    if import_count > 20 && fn_count > 15 && line_count > config.heuristics.high_lines {
        return Fidelity::High;
    }

    // Medium complexity: non-trivial implementation
    if import_count > 10 || fn_count > 10 || line_count > config.heuristics.medium_lines {
        return Fidelity::Medium;
    }

    // Low complexity: manageable size
    if line_count > 100 {
        return Fidelity::Low;
    }

    // Very small files: config default
    config.default_fidelity
}

// ── Fidelity Resolution (V2) ───────────────────────────────────────

/// Resolve the effective fidelity for a `provide_code_context` call.
///
/// Returns both the fidelity and the file classification (C-2 fix).
///
/// Priority order (highest first):
///   1. Explicit `fidelity` arg → use it directly.
///   2. Explicit `intent` arg → map via `config.smart_defaults`.
///   3. DB baseline fidelity (if session_aware_fidelity is on AND a
///      previous fidelity is available).
///   4. Content-based classification (test→Low, config→Low, model→Medium,
///      service→High, implementation→Medium).
///   5. Complexity-based fallback (imports + functions + lines → fidelity).
///   6. Config's `default_fidelity` → last resort.
#[allow(clippy::too_many_arguments)]
fn resolve_fidelity(
    explicit_fidelity: Option<&str>,
    explicit_intent: Option<&str>,
    file_path: &str,
    file_name: Option<&str>,
    source: &str,
    source_line_count: usize,
    config: &CleanCtxConfig,
    // C-1 fix: previous fidelity from the persistence DB
    stored_fidelity: Option<Fidelity>,
) -> Result<(Fidelity, FileClass), String> {
    // Priority 1: explicit fidelity arg
    // Gap 2 fix: if an explicit fidelity is provided but fails to parse,
    // return an error instead of silently falling back to the default.
    if let Some(s) = explicit_fidelity {
        match Fidelity::parse(s) {
            Ok(f) => return Ok((f, FileClass::General)),
            Err(e) => return Err(e.to_string()),
        }
    }

    // Priority 2: explicit intent → map via smart_defaults
    if let Some(intent) = explicit_intent {
        let mapped = match intent {
            "refactor" => &config.smart_defaults.refactor,
            "overview" => &config.smart_defaults.overview,
            "debug" => &config.smart_defaults.debug,
            "edit" => &config.smart_defaults.edit,
            "implement" => &config.smart_defaults.implement,
            _ => &config.default_fidelity,
        };
        // ANGULAR_HTML_COMPRESSION_PLAN Phase 3: "template editing"
        // (intent = "edit" on a `.component.html` file) triggers High
        // fidelity so the LLM sees the full semantic template.
        if intent == "edit" && is_angular_template_path(file_path) {
            return Ok((Fidelity::High, FileClass::Implementation));
        }
        return Ok((*mapped, FileClass::General));
    }

    // Priority 3: file name matches force_high_fidelity patterns
    // P1-7: Uses consolidated crate::config::glob_match instead of local duplicate.
    if let Some(fname) = file_name {
        for pattern in &config.heuristics.force_high_fidelity {
            if crate::config::glob_match(pattern, fname) {
                return Ok((Fidelity::High, FileClass::Service));
            }
        }
    }

    // C-1 fix: Priority 3.5 — DB baseline fidelity (session-aware)
    if config.heuristics.session_aware_fidelity {
        if let Some(db_fidelity) = stored_fidelity {
            // Re-use the fidelity from the DB baseline.
            // Determine the class by running the classifier (cheap, done
            // here to populate the decision summary).
            if config.heuristics.auto_classify {
                let (class, _) = classify_file(file_path, source, config);
                return Ok((db_fidelity, class));
            }
            return Ok((db_fidelity, FileClass::General));
        }
    }

    // Priority 4: content-based classification (V2)
    if config.heuristics.auto_classify {
        let (class, fidelity) = classify_file(file_path, source, config);
        if class != FileClass::General {
            return Ok((fidelity, class));
        }
    }

    // Priority 5: complexity-based fallback (V2 — reversed from V1)
    if config.heuristics.auto_classify {
        let fidelity = fidelity_from_complexity(source, source_line_count, config);
        return Ok((fidelity, FileClass::General));
    }

    // V1 fallback (when auto_classify is disabled)
    if source_line_count > config.heuristics.large_file_threshold {
        return Ok((Fidelity::Low, FileClass::General));
    }

    // Priority 6: config default
    Ok((config.default_fidelity, FileClass::General))
}

/// Check if a file is likely an Angular file by scanning its source.
fn detect_angular(file_path: &str, source: &str) -> bool {
    // Fast path — check extension first
    let path = Path::new(file_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "ts" && ext != "js" {
        return false;
    }

    // Scan for Angular decorators — these are cheap string checks
    // that avoid a full tree-sitter parse.
    let angular_patterns = [
        "@Component",
        "@Directive",
        "@Pipe",
        "@Injectable",
        "@NgModule",
        "@Input",
        "@Output",
    ];
    for pattern in &angular_patterns {
        if source.contains(pattern) {
            return true;
        }
    }
    false
}

/// P1-7: Consolidated — uses `crate::config::glob_match` instead of local copy.
/// Previously duplicated as `glob_match_simple` in this file, with identical logic.
/// Both exclude patterns (config.rs) and force_high_fidelity patterns (heuristics.rs)
/// now use the same canonical implementation.
/// Count lines in source text efficiently.
fn count_lines(source: &str) -> usize {
    source.lines().count()
}

/// The main decision function for `provide_code_context`.
///
/// Takes all available inputs and returns a [`ContextDecision`] that
/// tells the caller exactly what to do (full compress or delta, which
/// fidelity, whether Angular is detected).
///
/// C-1 fix: `stored_fidelity` allows the caller to pass a previously
/// persisted fidelity from the DB, enabling session-aware re-use.
#[allow(clippy::too_many_arguments)]
pub fn decide(
    file_path: &str,
    explicit_fidelity: Option<&str>,
    explicit_intent: Option<&str>,
    config: &CleanCtxConfig,
    ir_context: &ContextState,
    source: &str,
    // The dict alias (e.g. "α1") for this file — used to look up
    // delta baselines that were stored under the alias.
    path_alias: Option<&str>,
    // C-1: Previously persisted fidelity from the DB, if available.
    stored_fidelity: Option<Fidelity>,
) -> Result<ContextDecision, String> {
    let path = Path::new(file_path);

    // Count lines from the source
    let line_count = count_lines(source);

    // Get file name for force_high_fidelity matching
    let file_name = path.file_name().and_then(|n| n.to_str());

    // C-2 fix: resolve_fidelity now returns Result<(Fidelity, FileClass), String>
    // Gap 2 fix: propagate the error to the caller so it can return -32602.
    // The previous sentinel approach (returning Low/General and relying on the
    // caller to re-parse explicit_fidelity) was fragile and duplicated parse
    // logic. Now the error is propagated directly via `?`.
    let (mut fidelity, file_class) = resolve_fidelity(
        explicit_fidelity,
        explicit_intent,
        file_path,
        file_name,
        source,
        line_count,
        config,
        stored_fidelity,
    )?;

    // Auto-edit mode: when enabled and no explicit intent/fidelity was
    // provided, Service and Implementation files get Fidelity::Edit so
    // method bodies are carried verbatim for safe edits.
    if config.heuristics.auto_edit_mode && explicit_fidelity.is_none() && explicit_intent.is_none()
    {
        // M-4 (Gap 2.1 fix): the class → Edit mapping is now configurable
        // via `edit_auto_classifications` instead of being hardcoded.
        let class_key = match file_class {
            FileClass::Test => Some("test"),
            FileClass::Config => Some("config"),
            FileClass::Model => Some("model"),
            FileClass::Service => Some("service"),
            FileClass::Implementation => Some("implementation"),
            FileClass::General => None,
        };
        if let Some(key) = class_key {
            if config
                .heuristics
                .edit_auto_classifications
                .iter()
                .any(|c| c == key)
            {
                fidelity = Fidelity::Edit;
            }
        }
    }

    // Determine strategy: check for baselines using the dict alias
    // (where they're actually stored), falling back to raw path.
    let check_key = path_alias.unwrap_or(file_path);
    let has_ir_baseline = ir_context.has_file(check_key);

    // Delta transport only makes sense when the prior baseline was
    // compiled at the SAME fidelity. When the caller explicitly changes
    // `fidelity` (or `intent`, which maps to a fidelity), the prior
    // baseline's wire format is incompatible with `apply_delta` — the
    // delta would reference ops that never existed at the new fidelity,
    // producing a bare summary line with no structured payload. Force a
    // full compress in that case so the response is always consumable.
    let explicit_fidelity_or_intent = explicit_fidelity.is_some() || explicit_intent.is_some();
    let strategy = if config.auto_delta && !explicit_fidelity_or_intent && has_ir_baseline {
        ContextStrategy::DeltaTransport
    } else {
        ContextStrategy::FullCompress
    };

    // Angular detection
    let is_angular = if config.auto_angular {
        detect_angular(file_path, source)
    } else {
        false
    };

    Ok(ContextDecision {
        fidelity,
        strategy,
        is_angular,
        source_line_count: line_count,
        file_class,
        cbm_informed: false,
    })
}

#[cfg(test)]
#[path = "../tests/mcp/heuristics.rs"]
mod tests;
