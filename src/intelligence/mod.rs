// src/intelligence/mod.rs
//
// Intelligence Layer (Phase 2 — CBM Integration).
//
// Combines Clean-CTX's internal IR analysis with CBM's knowledge graph
// to provide smarter compression decisions:
//
//   - PageRank:     combines IR call-graph scores with CBM symbol importance
//   - Blast Radius: enhanced cross-file impact analysis
//   - Fidelity:     CBM-informed fidelity scoring for heuristics engine
//
// This module is entirely self-contained. It consumes data from both
// `crate::ir` and `crate::cbm` but owns all logic.

pub mod pagerank;
pub mod blast_radius;
pub mod fidelity;
pub mod budget;
pub mod knapsack;

pub use pagerank::compute_pagerank;
pub use blast_radius::compute_blast_radius;
pub use fidelity::{cbm_informed_fidelity, FidelityRecommendation};
pub use knapsack::{pack_to_budget, sort_by_rank, RankedSymbol};
