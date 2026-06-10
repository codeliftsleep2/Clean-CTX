// src/mcp/heuristics.rs
//
// Heuristics engine for `provide_code_context`.
//
// Decides the compression strategy (full vs delta) and fidelity level
// based on config, file characteristics, and explicit intent.
//
// Persistence-ready: when SQLite arrives, `decide()` will also check
// `ContextStore::has_context()` as part of the strategy decision.

use crate::compression::text_delta::TextDeltaComputer;
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
}

impl ContextDecision {
    /// Human-readable summary of the decision.
    pub fn summary(&self) -> String {
        let strategy_str = match self.strategy {
            ContextStrategy::FullCompress => "full_compress",
            ContextStrategy::DeltaTransport => "delta",
        };
        let angular_str = if self.is_angular { "angular" } else { "none" };
        format!(
            "fidelity={:?}, strategy={}, angular={}, lines={}",
            self.fidelity, strategy_str, angular_str, self.source_line_count
        )
    }
}

/// Resolve the effective fidelity for a `provide_code_context` call.
///
/// Priority order (highest first):
///   1. Explicit `fidelity` arg → use it directly.
///   2. Explicit `intent` arg → map via `config.smart_defaults`.
///   3. Extension matches `config.heuristics.force_high_fidelity` → "high".
///   4. Large file (> config.heuristics.large_file_threshold) → "low".
///   5. Config's `default_fidelity` → fallback.
fn resolve_fidelity(
    explicit_fidelity: Option<&str>,
    explicit_intent: Option<&str>,
    file_name: Option<&str>,
    source_line_count: usize,
    config: &CleanCtxConfig,
) -> Fidelity {
    // Priority 1: explicit fidelity arg
    if let Some(s) = explicit_fidelity {
        if let Ok(f) = Fidelity::parse(s) {
            return f;
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
        let f = Fidelity::parse_or_default(mapped);
        return f;
    }

    // Priority 3: file name matches force_high_fidelity patterns
    // Uses the full file name (e.g. "user.service.ts") rather than
    // just the extension ("ts") so patterns like "*.service.ts" work.
    if let Some(fname) = file_name {
        for pattern in &config.heuristics.force_high_fidelity {
            if glob_match_simple(pattern, fname) {
                return Fidelity::High;
            }
        }
    }

    // Priority 4: large file → low fidelity
    if source_line_count > config.heuristics.large_file_threshold {
        return Fidelity::Low;
    }

    // Priority 5: config default
    Fidelity::parse_or_default(&config.default_fidelity)
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

/// Simple glob matcher for force_high_fidelity patterns.
/// Supports `*` (any chars) and `?` (one char). Used only for
/// extension matching where patterns look like `"*.service.ts"`.
fn glob_match_simple(pattern: &str, text: &str) -> bool {
    let pbytes = pattern.as_bytes();
    let tbytes = text.as_bytes();
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi = None;
    let mut star_ti = 0;

    while ti < tbytes.len() {
        if pi < pbytes.len() && pbytes[pi] == b'*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if pi < pbytes.len() && (pbytes[pi] == tbytes[ti] || pbytes[pi] == b'?') {
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
    while pi < pbytes.len() && pbytes[pi] == b'*' {
        pi += 1;
    }
    pi == pbytes.len()
}

/// Count lines in source text efficiently.
fn count_lines(source: &str) -> usize {
    source.lines().count()
}

/// The main decision function for `provide_code_context`.
///
/// Takes all available inputs and returns a [`ContextDecision`] that
/// tells the caller exactly what to do (full compress or delta, which
/// fidelity, whether Angular is detected).
pub fn decide(
    file_path: &str,
    explicit_fidelity: Option<&str>,
    explicit_intent: Option<&str>,
    config: &CleanCtxConfig,
    text_delta_state: &TextDeltaComputer,
    ir_context: &ContextState,
    source: &str,
) -> ContextDecision {
    let path = Path::new(file_path);

    // Count lines from the source
    let line_count = count_lines(source);

    // Get file name for force_high_fidelity matching
    let file_name = path.file_name().and_then(|n| n.to_str());

    // Resolve fidelity
    let fidelity = resolve_fidelity(
        explicit_fidelity,
        explicit_intent,
        file_name,
        line_count,
        config,
    );

    // Determine strategy
    let path_alias = file_path.to_string(); // We'll use the alias from dict in the handler
    let has_delta_baseline = text_delta_state.has_baseline(&path_alias);
    let has_ir_baseline = ir_context.has_file(&path_alias);

    let strategy = if config.auto_delta && (has_delta_baseline || has_ir_baseline) {
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

    ContextDecision {
        fidelity,
        strategy,
        is_angular,
        source_line_count: line_count,
    }
}

#[cfg(test)]
#[path = "../tests/mcp/heuristics.rs"]
mod tests;