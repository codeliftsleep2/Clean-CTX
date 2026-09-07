// src/intelligence/fidelity.rs
//
// CBM-informed fidelity recommendations for the heuristics engine.
//
// When CBM is available, its symbol importance scores influence the
// compression fidelity decision:
//
//   - High importance (>0.8) → force High fidelity
//   - Medium importance (0.4-0.8) → use intent-based selection
//   - Low importance (<0.4) → force Low fidelity
//
// When CBM is unavailable, the recommendation is `NoRecommendation` and
// the existing heuristics pipeline runs unmodified.
//
// Phase 2 (Filter-First Architecture): `build_cbm_skip_set` identifies
// low-importance symbols that should be EXCLUDED from compression
// entirely. This replaces the post-compression enrichment pattern.

use crate::compression::Fidelity;
use crate::cbm::SymbolImportance;
use std::collections::{HashMap, HashSet};

/// Request-scoped CBM intelligence for a single context-compilation request.
///
/// This struct carries CBM-derived advisory intelligence that is specific to
/// one compilation request. It is NOT session-scoped state and must NOT be
/// stored in `WorkspaceIndex`, `CompiledIR`, or any persistent structure.
///
/// Ownership lifecycle:
/// - Created during `compute_strategy()` after CBM consultation
/// - Carried by `ContextDecision` into `compile_file_ir_focused()`
/// - Consumed by the compiler's skip-set filtering machinery
/// - Dropped when the request completes
///
/// The compiler itself never sees this type — it only receives the derived
/// `skip_set: Option<HashSet<String>>` parameter, keeping the compiler
/// CBM-agnostic.
#[derive(Debug, Clone)]
pub struct CbmIntelligence {
    /// CBM symbol importance scores (symbol → importance).
    /// Session-cached by GraphBridge; cloned here for request-local use.
    pub importance: HashMap<String, SymbolImportance>,
    /// Symbols to exclude from compression (score < 0.4).
    /// Derived from `importance` via `build_cbm_skip_set`.
    pub skip_set: HashSet<String>,
    /// Request-scoped data-flow context (Phase D1): the bounded set of
    /// workspace symbols/files that participate in a data flow relevant to
    /// this compilation request. `None` when data-flow consultation was not
    /// performed or produced no candidates.
    ///
    /// This is **advisory context expansion intelligence** — it never enters
    /// `WorkspaceIndex`, `CompiledIR`, or semantic identity. The compiler
    /// consumes only the derived `skip_set` parameter; this field is carried
    /// request-scoped purely for context-selection/expansion decisions above
    /// the compiler.
    pub data_flow: Option<crate::cbm::bridge::DataFlowContext>,
    /// Request-scoped cross-service context (Phase D2): the bounded set of
    /// workspace symbols/files reachable through a cross-service-relevant
    /// path from this request's seed symbols
    /// (`trace_path(mode="cross_service")`). `None` when cross-service
    /// consultation was not performed or produced no candidates.
    ///
    /// Same advisory, request-scoped, ephemeral semantics as `data_flow` —
    /// never `WorkspaceIndex`, `CompiledIR`, or semantic identity. The claim
    /// is reachability only, never service identity or architectural
    /// service discovery.
    pub cross_service: Option<crate::cbm::bridge::DataFlowContext>,
}

/// A fidelity recommendation from the intelligence layer.
#[derive(Debug, Clone, PartialEq)]
pub enum FidelityRecommendation {
    /// Force high fidelity regardless of other signals.
    ForceHigh,
    /// Force low fidelity regardless of other signals.
    ForceLow,
    /// No strong signal — defer to the standard heuristics pipeline.
    NoRecommendation,
}

/// Determine fidelity recommendation based on CBM symbol importance for a file.
///
/// `file_path`: the file being compressed.
/// `symbol_importance`: map of symbol name → SymbolImportance from CBM bridge.
/// `fallback`: what to return if no CBM data is available.
///
/// Returns `ForceHigh` if any symbol in the file has high importance (>0.8),
/// `ForceLow` if all symbols have low importance (<0.4), or `NoRecommendation`.
pub fn cbm_informed_fidelity(
    file_path: &str,
    symbol_importance: &HashMap<String, crate::cbm::SymbolImportance>,
    fallback: FidelityRecommendation,
) -> FidelityRecommendation {
    if symbol_importance.is_empty() {
        return fallback;
    }

    let mut max_score = 0.0_f64;
    let mut any_match = false;

    for info in symbol_importance.values() {
        // P1-11: Use proper path matching instead of contains().
        // Before fix: file_path.contains(&info.file) — false positives on substring matches.
        // "user.rs" would match "src/user_service.rs" and "api.rs" would match "src/api_handler.rs".
        if !path_matches(file_path, &info.file) {
            continue;
        }
        any_match = true;
        if info.score > max_score {
            max_score = info.score;
        }
    }

    if !any_match {
        // No symbols matched this file — fallback
        return if max_score > 0.0 {
            // We have data but no direct matches — be conservative
            FidelityRecommendation::NoRecommendation
        } else {
            fallback
        };
    }

    if max_score > 0.8 {
        FidelityRecommendation::ForceHigh
    } else if max_score < 0.4 {
        FidelityRecommendation::ForceLow
    } else {
        FidelityRecommendation::NoRecommendation
    }
}

/// Build a skip set of low-importance symbols for a file.
///
/// Returns symbol names with score < 0.4 that match the given file path.
/// The compression pipeline uses this set to drop low-importance symbols
/// entirely, so CBM reduces token output instead of adding enrichment.
///
/// Returns an empty set if CBM is unavailable or no low-importance symbols
/// are found for this file.
pub fn build_cbm_skip_set(
    file_path: &str,
    symbol_importance: &HashMap<String, crate::cbm::SymbolImportance>,
) -> HashSet<String> {
    let mut skip = HashSet::new();
    for info in symbol_importance.values() {
        if info.score < 0.4 {
            // P1-11: Use proper path matching instead of contains().
            if path_matches(file_path, &info.file) {
                skip.insert(info.symbol.clone());
            }
        }
    }
    skip
}

/// Apply the fidelity recommendation to get a concrete Fidelity.
/// Returns `Some(fidelity)` if the recommendation overrides, `None` if
/// the existing pipeline should decide.
pub fn apply_recommendation(rec: &FidelityRecommendation) -> Option<Fidelity> {
    match rec {
        FidelityRecommendation::ForceHigh => Some(Fidelity::High),
        FidelityRecommendation::ForceLow => Some(Fidelity::Low),
        FidelityRecommendation::NoRecommendation => None,
    }
}

/// Select the highest-importance symbols for a file to use as data-flow
/// trace seeds (Phase D1).
///
/// Returns at most `max_seeds` symbols (sorted by score, descending) whose
/// importance entry path-matches the requested file. An empty result means
/// CBM knows no symbols for this file — the caller skips data-flow consultation.
pub fn data_flow_seed_symbols(
    file_path: &str,
    symbol_importance: &HashMap<String, crate::cbm::SymbolImportance>,
    max_seeds: usize,
) -> Vec<String> {
    let mut seeds: Vec<_> = symbol_importance
        .values()
        .filter(|info| path_matches(file_path, &info.file))
        .collect();
    seeds.sort_by(|a, b| b.score.total_cmp(&a.score));
    seeds
        .into_iter()
        .take(max_seeds)
        .map(|info| info.symbol.clone())
        .collect()
}

/// Bound a merged data-flow context to the configured maxima (Phase D1).
///
/// Expansion must be bounded — a single seed can produce a large trace.
pub fn bound_data_flow(
    ctx: crate::cbm::bridge::DataFlowContext,
    max_symbols: usize,
    max_files: usize,
) -> crate::cbm::bridge::DataFlowContext {
    let mut symbols: Vec<_> = ctx.symbols.into_iter().collect();
    symbols.sort();
    symbols.truncate(max_symbols);
    let mut files: Vec<_> = ctx.files.into_iter().collect();
    files.sort();
    files.truncate(max_files);
    crate::cbm::bridge::DataFlowContext {
        symbols: symbols.into_iter().collect(),
        files: files.into_iter().collect(),
    }
}

/// Apply the Phase D1 skip-set interaction decision:
///
/// **Data-flow-identified in-file symbols are retained** (removed from the
/// skip-set). Rationale (evidence from the existing context-selection
/// architecture): the skip-set's purpose is to exclude *low-importance*
/// symbols (`build_cbm_skip_set`, score < 0.4). A symbol that CBM's
/// data-flow trace explicitly identifies as participating in the request's
/// data flow is request-relevant despite its static centrality — excluding
/// it would defeat the filter's stated purpose (which is token reduction on
/// *irrelevant* symbols, not on request-relevant ones). The retention is
/// bounded: only symbols present in `data_flow.symbols` whose importance
/// entry path-matches the current file are removed; all other skip-set
/// semantics are unchanged. This is an advisory, request-scoped decision —
/// it never alters `WorkspaceIndex`, `CompiledIR`, or semantic facts.
/// Returns the *modified* skip-set (caller owns it}.
pub fn retain_data_flow_relevant_symbols(
    file_path: &str,
    skip_set: &HashSet<String>,
    symbol_importance: &HashMap<String, crate::cbm::SymbolImportance>,
    data_flow: &crate::cbm::bridge::DataFlowContext,
) -> HashSet<String> {
    retain_relevant_symbols(file_path, skip_set, symbol_importance, data_flow)
}

/// Apply the Phase D2 skip-set interaction decision:
///
/// **Cross-service-identified in-file symbols are retained** (removed from
/// the skip-set), with exactly the same semantics and rationale as
/// [`retain_data_flow_relevant_symbols`]: a symbol CBM's cross-service trace
/// identifies as reachable through a cross-service-relevant path from the
/// request's seeds is request-relevant despite low static centrality.
/// Retention is restricted to symbols belonging to the **requested current
/// workspace file** (via `path_matches` over the importance entries), so a
/// cross-repository symbol name that happens to collide cannot retain an
/// unrelated current-workspace symbol outside the requested file. Union
/// semantics with the D1 pass: both passes only REMOVE from the skip-set,
/// so a symbol retained by either relevance source stays retained, and no
/// symbol is ever added. Advisory, request-scoped, bounded — never
/// `WorkspaceIndex`, `CompiledIR`, or semantic identity.
pub fn retain_cross_service_relevant_symbols(
    file_path: &str,
    skip_set: &HashSet<String>,
    symbol_importance: &HashMap<String, crate::cbm::SymbolImportance>,
    cross_service: &crate::cbm::bridge::DataFlowContext,
) -> HashSet<String> {
    retain_relevant_symbols(file_path, skip_set, symbol_importance, cross_service)
}

/// Shared retention core for D1 (data-flow) and D2 (cross-service).
///
/// The smallest mechanically necessary adaptation (Phase D2): the D1
/// retention logic was already exactly the semantics D2 requires — the
/// trace-context parameter is the same `DataFlowContext` type — so the body
/// was extracted verbatim into this private helper and both public
/// retention functions delegate to it. No behavior change for D1 callers.
fn retain_relevant_symbols(
    file_path: &str,
    skip_set: &HashSet<String>,
    symbol_importance: &HashMap<String, crate::cbm::SymbolImportance>,
    ctx: &crate::cbm::bridge::DataFlowContext,
) -> HashSet<String> {
    // Bare-name lookup of importance entries for THIS file.
    let in_file: HashSet<&str> = symbol_importance
        .values()
        .filter(|info| path_matches(file_path, &info.file))
        .map(|info| info.symbol.as_str())
        .collect();
    skip_set
        .iter()
        .filter(|&symbol| {
            let trace_match = ctx.symbols.iter().any(|t_sym| t_sym == symbol)
                | ctx.symbols.iter().any(|t_sym| t_sym.ends_with(&format!(".{symbol}")));
            !(trace_match && in_file.contains(symbol.as_str()))
        })
        .cloned()
        .collect()
}

/// P1-11: Proper path matching — checks if two file paths point to the same file.
/// Uses path segment matching instead of string contains() to avoid false positives.
/// For example, "user.rs" should NOT match "src/user_service.rs".
fn path_matches(file_path: &str, target_file: &str) -> bool {
    let path = std::path::Path::new(file_path);
    let target = std::path::Path::new(target_file);
    // Match if paths are equal, or one ends with the other (subpath match)
    path == target || path.ends_with(target) || target.ends_with(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbm::SymbolImportance;

    fn make_importance(symbol: &str, score: f64, file: &str) -> HashMap<String, SymbolImportance> {
        let mut map = HashMap::new();
        map.insert(
            symbol.to_string(),
            SymbolImportance {
                symbol: symbol.to_string(),
                score,
                file: file.to_string(),
            },
        );
        map
    }

    // ── cbm_informed_fidelity tests ──────────────────────────────────

    #[test]
    fn test_empty_map_returns_fallback() {
        let result = cbm_informed_fidelity(
            "src/user.rs",
            &HashMap::new(),
            FidelityRecommendation::NoRecommendation,
        );
        assert_eq!(result, FidelityRecommendation::NoRecommendation);
    }

    #[test]
    fn test_high_importance_forces_high() {
        let importances = make_importance("UserService", 0.9, "user.rs");
        let result = cbm_informed_fidelity(
            "src/user.rs",
            &importances,
            FidelityRecommendation::NoRecommendation,
        );
        assert_eq!(result, FidelityRecommendation::ForceHigh);
    }

    #[test]
    fn test_low_importance_forces_low() {
        let importances = make_importance("UserService", 0.2, "user.rs");
        let result = cbm_informed_fidelity(
            "src/user.rs",
            &importances,
            FidelityRecommendation::NoRecommendation,
        );
        assert_eq!(result, FidelityRecommendation::ForceLow);
    }

    #[test]
    fn test_medium_importance_no_recommendation() {
        let importances = make_importance("UserService", 0.6, "user.rs");
        let result = cbm_informed_fidelity(
            "src/user.rs",
            &importances,
            FidelityRecommendation::NoRecommendation,
        );
        assert_eq!(result, FidelityRecommendation::NoRecommendation);
    }

    #[test]
    fn test_non_matching_file_uses_fallback() {
        let importances = make_importance("UserService", 0.9, "other.rs");
        let result = cbm_informed_fidelity(
            "src/user.rs",
            &importances,
            FidelityRecommendation::NoRecommendation,
        );
        // No direct match, but max_score > 0 — should be conservative
        assert_eq!(result, FidelityRecommendation::NoRecommendation);
    }

    #[test]
    fn test_fallback_passthrough() {
        let result = cbm_informed_fidelity(
            "src/user.rs",
            &HashMap::new(),
            FidelityRecommendation::ForceHigh,
        );
        assert_eq!(result, FidelityRecommendation::ForceHigh);
    }

    #[test]
    fn test_apply_force_high() {
        assert_eq!(
            apply_recommendation(&FidelityRecommendation::ForceHigh),
            Some(Fidelity::High)
        );
    }

    #[test]
    fn test_apply_force_low() {
        assert_eq!(
            apply_recommendation(&FidelityRecommendation::ForceLow),
            Some(Fidelity::Low)
        );
    }

    #[test]
    fn test_apply_no_recommendation() {
        assert_eq!(
            apply_recommendation(&FidelityRecommendation::NoRecommendation),
            None
        );
    }

    // ── build_cbm_skip_set tests ────────────────────────────────────

    #[test]
    fn test_build_skip_set_low() {
        let importances = make_importance("UtilityHelper", 0.2, "utils.rs");
        let skip = build_cbm_skip_set("src/utils.rs", &importances);
        assert!(
            skip.contains("UtilityHelper"),
            "Low-importance symbol should be in skip set"
        );
        assert_eq!(skip.len(), 1);
    }

    #[test]
    fn test_build_skip_set_medium() {
        let importances = make_importance("NormalService", 0.6, "service.rs");
        let skip = build_cbm_skip_set("src/service.rs", &importances);
        assert!(
            !skip.contains("NormalService"),
            "Medium-importance symbol should NOT be in skip set"
        );
    }

    #[test]
    fn test_build_skip_set_high() {
        let importances = make_importance("CriticalAPI", 0.95, "api.rs");
        let skip = build_cbm_skip_set("src/api.rs", &importances);
        assert!(
            !skip.contains("CriticalAPI"),
            "High-importance symbol should NOT be in skip set"
        );
    }

    #[test]
    fn test_build_skip_set_empty() {
        let skip = build_cbm_skip_set("src/file.rs", &HashMap::new());
        assert!(
            skip.is_empty(),
            "Empty importance map should produce empty skip set"
        );
    }

    #[test]
    fn test_build_skip_set_unrelated_file() {
        let importances = make_importance("LowSymbol", 0.1, "other.rs");
        let skip = build_cbm_skip_set("src/user.rs", &importances);
        assert!(
            !skip.contains("LowSymbol"),
            "Symbol in unrelated file should NOT be in skip set"
        );
    }

    #[test]
    fn test_build_skip_set_multiple() {
        let mut map = HashMap::new();
        map.insert(
            "SymA".to_string(),
            SymbolImportance {
                symbol: "SymA".to_string(),
                score: 0.15,
                file: "file.rs".to_string(),
            },
        );
        map.insert(
            "SymB".to_string(),
            SymbolImportance {
                symbol: "SymB".to_string(),
                score: 0.9,
                file: "file.rs".to_string(),
            },
        );
        map.insert(
            "SymC".to_string(),
            SymbolImportance {
                symbol: "SymC".to_string(),
                score: 0.3,
                file: "file.rs".to_string(),
            },
        );
        let skip = build_cbm_skip_set("file.rs", &map);
        assert!(skip.contains("SymA"), "SymA (0.15) should be skipped");
        assert!(!skip.contains("SymB"), "SymB (0.9) should NOT be skipped");
        assert!(skip.contains("SymC"), "SymC (0.3) should be skipped");
        assert_eq!(skip.len(), 2);
    }
}
