// src/intelligence/pagerank.rs
//
// PageRank computation combining Clean-CTX IR graph scores with CBM symbol importance.
//
// The IR graph provides structural importance (call frequency, dependency depth)
// while CBM provides semantic importance (cross-file references, type resolution).
// The final score is a weighted combination: 60% IR, 40% CBM.

use std::collections::HashMap;

/// A single symbol's PageRank score with its source breakdown.
#[derive(Debug, Clone)]
pub struct PageRankScore {
    /// The symbol name (fully qualified where available).
    pub symbol: String,
    /// The file this symbol belongs to.
    pub file: String,
    /// Combined score (0.0 - 1.0).
    pub combined_score: f64,
    /// IR-derived score component (0.0 - 1.0).
    pub ir_score: f64,
    /// CBM-derived score component (0.0 - 1.0).
    pub cbm_score: f64,
}

/// Compute PageRank scores combining IR graph analysis with CBM symbol importance.
///
/// `ir_scores`: symbol → importance from the Clean-CTX IR call graph (0.0 - 1.0).
/// `cbm_scores`: symbol → importance from CBM's knowledge graph (0.0 - 1.0).
/// `ir_weight`: weight for IR scores (default 0.6). CBM weight = 1.0 - ir_weight.
///
/// Returns a map of symbol → combined PageRankScore.
pub fn compute_pagerank(
    ir_scores: HashMap<String, f64>,
    cbm_scores: HashMap<String, crate::cbm::SymbolImportance>,
    ir_weight: Option<f64>,
) -> HashMap<String, PageRankScore> {
    let ir_w = ir_weight.unwrap_or(0.6);
    let cbm_w = 1.0 - ir_w;
    let mut result = HashMap::new();

    // Normalize IR scores to 0.0 - 1.0 range
    let ir_max = ir_scores.values().cloned().fold(0.0_f64, f64::max);
    let normalized_ir: HashMap<&str, f64> = ir_scores.iter()
        .map(|(k, v)| (k.as_str(), if ir_max > 0.0 { v / ir_max } else { 0.0 }))
        .collect();

    // Normalize CBM scores to 0.0 - 1.0 range
    let cbm_max = cbm_scores.values().map(|s| s.score).fold(0.0_f64, f64::max);
    let normalized_cbm: HashMap<&str, f64> = cbm_scores.iter()
        .map(|(k, v)| (k.as_str(), if cbm_max > 0.0 { v.score / cbm_max } else { 0.0 }))
        .collect();

    // Build combined scores
    // Start with all CBM symbols
    for (sym, importance) in &cbm_scores {
        let ir = normalized_ir.get(sym.as_str()).copied().unwrap_or(0.0);
        let cbm = normalized_cbm.get(sym.as_str()).copied().unwrap_or(0.0);
        let combined = ir * ir_w + cbm * cbm_w;
        result.insert(sym.clone(), PageRankScore {
            symbol: sym.clone(),
            file: importance.file.clone(),
            combined_score: combined,
            ir_score: ir,
            cbm_score: cbm,
        });
    }

    // Add IR-only symbols (not in CBM)
    for sym in ir_scores.keys() {
        if !result.contains_key(sym) {
            let ir = normalized_ir.get(sym.as_str()).copied().unwrap_or(0.0);
            let combined = ir * ir_w; // cbm = 0.0
            result.insert(sym.clone(), PageRankScore {
                symbol: sym.clone(),
                file: String::new(),
                combined_score: combined,
                ir_score: ir,
                cbm_score: 0.0,
            });
        }
    }

    result
}

/// Get the top-N symbols by combined PageRank score.
pub fn top_symbols(scores: &HashMap<String, PageRankScore>, n: usize) -> Vec<&PageRankScore> {
    let mut sorted: Vec<&PageRankScore> = scores.values().collect();
    sorted.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(n);
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbm::SymbolImportance;

    #[test]
    fn test_compute_pagerank_empty() {
        let result = compute_pagerank(HashMap::new(), HashMap::new(), None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_pagerank_ir_only() {
        let mut ir = HashMap::new();
        ir.insert("UserService.login".into(), 10.0);
        ir.insert("PaymentGateway.charge".into(), 5.0);
        let result = compute_pagerank(ir, HashMap::new(), None);
        assert_eq!(result.len(), 2);
        // UserService.login is more important
        assert!(result["UserService.login"].combined_score > result["PaymentGateway.charge"].combined_score);
        // IR-only symbols have 0.0 cbm_score
        assert_eq!(result["UserService.login"].cbm_score, 0.0);
    }

    #[test]
    fn test_compute_pagerank_cbm_only() {
        let mut cbm = HashMap::new();
        cbm.insert("AuthService".into(), SymbolImportance {
            symbol: "AuthService".into(), score: 0.9, file: "auth.rs".into(),
        });
        cbm.insert("Logger".into(), SymbolImportance {
            symbol: "Logger".into(), score: 0.3, file: "log.rs".into(),
        });
        let result = compute_pagerank(HashMap::new(), cbm, None);
        assert_eq!(result.len(), 2);
        assert!(result["AuthService"].combined_score > result["Logger"].combined_score);
        assert_eq!(result["AuthService"].ir_score, 0.0);
    }

    #[test]
    fn test_compute_pagerank_combined() {
        // IR scores for UserService (fully qualified) and PaymentGateway
        let mut ir = HashMap::new();
        ir.insert("UserService".into(), 5.0);  // Same name as CBM entry
        ir.insert("PaymentGateway.charge".into(), 10.0);

        // CBM gives high importance to AuthService and moderate to UserService
        let mut cbm = HashMap::new();
        cbm.insert("UserService".into(), SymbolImportance {  // Same name matched
            symbol: "UserService".into(), score: 0.6, file: "user.rs".into(),
        });
        cbm.insert("AuthService".into(), SymbolImportance {
            symbol: "AuthService".into(), score: 0.9, file: "auth.rs".into(),
        });

        let result = compute_pagerank(ir, cbm, Some(0.6));
        // 3 entries: UserService (IR + CBM), PaymentGateway.charge (IR-only), AuthService (CBM-only)
        assert_eq!(result.len(), 3);

        // UserService should have both IR and CBM scores (same key in both maps)
        let us = &result["UserService"];
        assert!(us.ir_score > 0.0, "IR score for UserService should be > 0");
        assert!(us.cbm_score > 0.0, "CBM score for UserService should be > 0");

        // PaymentGateway is IR-only (cbm_score = 0)
        let pg = &result["PaymentGateway.charge"];
        assert_eq!(pg.cbm_score, 0.0);
    }

    #[test]
    fn test_top_symbols_orders_by_score() {
        let mut scores = HashMap::new();
        scores.insert("low".into(), PageRankScore {
            symbol: "low".into(), file: "".into(),
            combined_score: 0.1, ir_score: 0.0, cbm_score: 0.0,
        });
        scores.insert("high".into(), PageRankScore {
            symbol: "high".into(), file: "".into(),
            combined_score: 0.9, ir_score: 0.0, cbm_score: 0.0,
        });
        scores.insert("medium".into(), PageRankScore {
            symbol: "medium".into(), file: "".into(),
            combined_score: 0.5, ir_score: 0.0, cbm_score: 0.0,
        });

        let top = top_symbols(&scores, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].symbol, "high");
        assert_eq!(top[1].symbol, "medium");
    }

    #[test]
    fn test_top_symbols_limit_less_than_total() {
        let mut scores = HashMap::new();
        scores.insert("a".into(), PageRankScore {
            symbol: "a".into(), file: "".into(),
            combined_score: 0.1, ir_score: 0.0, cbm_score: 0.0,
        });
        scores.insert("b".into(), PageRankScore {
            symbol: "b".into(), file: "".into(),
            combined_score: 0.9, ir_score: 0.0, cbm_score: 0.0,
        });

        let top = top_symbols(&scores, 10);
        assert_eq!(top.len(), 2);
    }
}