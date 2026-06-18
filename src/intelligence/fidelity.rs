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

use std::collections::HashMap;
use crate::compressor::Fidelity;

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
        // Check if this symbol's file path matches our target
        if !file_path.contains(&info.file) && !info.file.contains(file_path) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbm::SymbolImportance;

    fn make_importance(symbol: &str, score: f64, file: &str) -> HashMap<String, SymbolImportance> {
        let mut map = HashMap::new();
        map.insert(symbol.to_string(), SymbolImportance {
            symbol: symbol.to_string(),
            score,
            file: file.to_string(),
        });
        map
    }

    #[test]
    fn test_empty_map_returns_fallback() {
        let result = cbm_informed_fidelity("src/user.rs", &HashMap::new(), FidelityRecommendation::NoRecommendation);
        assert_eq!(result, FidelityRecommendation::NoRecommendation);
    }

    #[test]
    fn test_high_importance_forces_high() {
        let importances = make_importance("UserService", 0.9, "user.rs");
        let result = cbm_informed_fidelity("src/user.rs", &importances, FidelityRecommendation::NoRecommendation);
        assert_eq!(result, FidelityRecommendation::ForceHigh);
    }

    #[test]
    fn test_low_importance_forces_low() {
        let importances = make_importance("UserService", 0.2, "user.rs");
        let result = cbm_informed_fidelity("src/user.rs", &importances, FidelityRecommendation::NoRecommendation);
        assert_eq!(result, FidelityRecommendation::ForceLow);
    }

    #[test]
    fn test_medium_importance_no_recommendation() {
        let importances = make_importance("UserService", 0.6, "user.rs");
        let result = cbm_informed_fidelity("src/user.rs", &importances, FidelityRecommendation::NoRecommendation);
        assert_eq!(result, FidelityRecommendation::NoRecommendation);
    }

    #[test]
    fn test_non_matching_file_uses_fallback() {
        let importances = make_importance("UserService", 0.9, "other.rs");
        let result = cbm_informed_fidelity("src/user.rs", &importances, FidelityRecommendation::NoRecommendation);
        // No direct match, but max_score > 0 — should be conservative
        assert_eq!(result, FidelityRecommendation::NoRecommendation);
    }

    #[test]
    fn test_fallback_passthrough() {
        let result = cbm_informed_fidelity("src/user.rs", &HashMap::new(), FidelityRecommendation::ForceHigh);
        assert_eq!(result, FidelityRecommendation::ForceHigh);
    }

    #[test]
    fn test_apply_force_high() {
        assert_eq!(apply_recommendation(&FidelityRecommendation::ForceHigh), Some(Fidelity::High));
    }

    #[test]
    fn test_apply_force_low() {
        assert_eq!(apply_recommendation(&FidelityRecommendation::ForceLow), Some(Fidelity::Low));
    }

    #[test]
    fn test_apply_no_recommendation() {
        assert_eq!(apply_recommendation(&FidelityRecommendation::NoRecommendation), None);
    }
}