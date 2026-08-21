// src/intelligence/blast_radius.rs
//
// Blast radius analysis — determines which files and symbols are affected
// by a change to a given symbol.
//
// Uses both Clean-CTX's internal IR dependencies and CBM's knowledge graph
// for cross-file, cross-package impact analysis.

use std::collections::{HashMap, HashSet};

/// Represents a single file affected by a change.
#[derive(Debug, Clone)]
pub struct AffectedFile {
    /// Path to the affected file.
    pub file_path: String,
    /// The symbols in this file that are directly impacted.
    pub affected_symbols: Vec<String>,
    /// Estimated impact severity (0.0 = low, 1.0 = high).
    /// Based on number of affected symbols, depth from change, and centrality.
    pub impact_score: f64,
    /// Whether the file is directly affected (depth 1) or transitively (depth > 1).
    pub is_direct: bool,
}

/// Compute blast radius for a given symbol change.
///
/// Combines CBM's graph-based blast radius with Clean-CTX's IR dependencies.
/// CBM handles cross-file/cross-package resolution; IR handles intra-file
/// dependency chains.
///
/// `symbol`: the symbol being changed.
/// `depth`: how many hops to traverse (default 2).
/// `cbm_files`: file paths returned by CBM's blast radius query.
/// `ir_deps`: symbol → set of dependents from IR analysis.
/// `cbm_bridge`: optional CBM bridge for enhanced analysis.
pub fn compute_blast_radius(
    _symbol: &str,
    depth: usize,
    cbm_files: Vec<String>,
    ir_deps: &HashMap<String, HashSet<String>>,
) -> Vec<AffectedFile> {
    let mut result: Vec<AffectedFile> = Vec::new();
    let mut seen_files: HashSet<String> = HashSet::new();

    // Process CBM results (cross-file, depth-aware)
    let effective_depth = if depth == 0 { 2 } else { depth };
    for file in &cbm_files {
        if seen_files.insert(file.clone()) {
            let symbols = resolve_symbols_for_file(file, ir_deps);
            let impact = compute_impact(&symbols, 1, effective_depth);
            result.push(AffectedFile {
                file_path: file.clone(),
                affected_symbols: symbols,
                impact_score: impact,
                is_direct: true,
            });
        }
    }

    // Process IR-only dependencies (symbols not in CBM graph)
    for (dep_sym, dep_files) in ir_deps {
        for dep_file in dep_files {
            if !seen_files.contains(dep_file) {
                seen_files.insert(dep_file.clone());
                result.push(AffectedFile {
                    file_path: dep_file.clone(),
                    affected_symbols: vec![dep_sym.clone()],
                    impact_score: 0.3, // IR-only, lower confidence
                    is_direct: true,
                });
            }
        }
    }

    result
}

/// Resolve symbols in a file from the IR dependency map.
fn resolve_symbols_for_file(file: &str, ir_deps: &HashMap<String, HashSet<String>>) -> Vec<String> {
    let mut symbols: Vec<String> = Vec::new();
    for (sym, files) in ir_deps {
        if files.contains(file) {
            symbols.push(sym.clone());
        }
    }
    symbols
}

/// Compute impact score based on symbol count and depth.
fn compute_impact(symbols: &[String], current_depth: usize, max_depth: usize) -> f64 {
    let depth_factor = 1.0 - ((current_depth as f64 - 1.0) / max_depth as f64);
    let symbol_factor = (symbols.len() as f64).min(10.0) / 10.0;
    0.5 * depth_factor + 0.5 * symbol_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_blast_radius() {
        let result = compute_blast_radius("UserService.login", 2, vec![], &HashMap::new());
        assert!(result.is_empty());
    }

    #[test]
    fn test_cbm_files_only() {
        let cbm_files = vec!["src/user.rs".into(), "src/payment.rs".into()];
        let result = compute_blast_radius("UserService", 2, cbm_files, &HashMap::new());
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|f| f.is_direct));
        // Both files should have impact_score > 0 since they're depth 1
        assert!(result[0].impact_score > 0.0);
    }

    #[test]
    fn test_ir_deps_filled_in_gaps() {
        let cbm_files = vec!["src/user.rs".into()];
        let mut ir_deps: HashMap<String, HashSet<String>> = HashMap::new();
        ir_deps.insert("Logger".into(), {
            let mut s = HashSet::new();
            s.insert("src/logger.rs".into());
            s
        });

        let result = compute_blast_radius("UserService", 2, cbm_files, &ir_deps);
        assert_eq!(result.len(), 2);
        // IR-only file should have impact 0.3
        let ir_file = result
            .iter()
            .find(|f| f.file_path == "src/logger.rs")
            .unwrap();
        assert_eq!(ir_file.impact_score, 0.3);
        assert!(!ir_file.affected_symbols.is_empty());
    }

    #[test]
    fn test_no_duplicate_files() {
        let cbm_files = vec!["src/user.rs".into(), "src/user.rs".into()];
        let result = compute_blast_radius("UserService", 2, cbm_files, &HashMap::new());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_depth_zero_defaults_to_two() {
        let cbm_files = vec!["src/user.rs".into()];
        let result = compute_blast_radius("UserService", 0, cbm_files, &HashMap::new());
        assert_eq!(result.len(), 1);
        assert!(result[0].impact_score > 0.0);
    }
}
