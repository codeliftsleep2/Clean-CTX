// src/intelligence/knapsack.rs
//
// R-29 Phase 3: Greedy knapsack packing for token budget enforcement.
//
// After all symbols have been compressed and ranked by PageRank score,
// the knapsack pass selects the highest-ranked symbols that fit within
// the caller's token budget. No symbol is partially included — all-or-nothing
// packing preserves semantic integrity.
//
// The algorithm is a simple greedy approach: sort by rank descending,
// then include each symbol if it fits within the remaining budget.
// This is optimal for the "fractional knapsack" variant (which this is,
// since symbols are indivisible but we can choose any subset) when items
// are sorted by value density — and since all symbols have equal "value
// per token" (they're all compressed at their appropriate fidelity), the
// greedy approach by rank is equivalent to sorting by value density.

use crate::intelligence::budget::TokenBudget;

/// A single ranked symbol with its compressed token cost.
#[derive(Debug, Clone)]
pub struct RankedSymbol {
    /// Symbol name (fully qualified where available).
    pub symbol: String,
    /// File alias (αN) this symbol belongs to.
    pub file_alias: String,
    /// Combined PageRank score (0.0 - 1.0).
    pub rank: f64,
    /// Token cost of this symbol's compressed representation.
    pub token_cost: usize,
}

/// Greedy knapsack: pack the highest-ranked symbols into the budget.
///
/// `symbols`: already rank-sorted descending (highest rank first).
/// `budget`: mutable budget tracker.
///
/// Returns the subset of symbols that fit within the budget, in rank order.
/// Symbols that don't fit are recorded in `budget` via `record_dropped`.
pub fn pack_to_budget(
    symbols: Vec<RankedSymbol>,
    budget: &mut TokenBudget,
) -> Vec<RankedSymbol> {
    if symbols.is_empty() {
        return symbols;
    }

    let mut packed: Vec<RankedSymbol> = Vec::new();

    for sym in symbols {
        if budget.would_fit(sym.token_cost) {
            budget.reserve(sym.token_cost);
            packed.push(sym);
        } else {
            budget.record_dropped(sym.token_cost);
        }
    }

    packed
}

/// Sort symbols by rank descending (highest first).
/// This is a convenience function for callers that haven't pre-sorted.
pub fn sort_by_rank(mut symbols: Vec<RankedSymbol>) -> Vec<RankedSymbol> {
    symbols.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(std::cmp::Ordering::Equal));
    symbols
}

#[cfg(test)]
#[path = "../tests/intelligence/knapsack.rs"]
mod tests;