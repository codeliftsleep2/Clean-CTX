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

use crate::compressor::Fidelity;
use std::collections::{HashMap, HashSet};

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
