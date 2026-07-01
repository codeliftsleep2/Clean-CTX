// src/intelligence/budget.rs
//
// R-29 Phase 3: Token budget tracking for knapsack packing.
//
// Tracks how many tokens have been used vs the budget limit, and
// how many symbols were dropped (and their token cost) so the
// `§BUDGET` manifest header can report exact numbers.

/// Token budget tracking for knapsack packing.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Maximum tokens allowed in the output.
    pub limit: usize,
    /// Tokens consumed by packed symbols so far.
    pub used: usize,
    /// Number of symbols dropped because they didn't fit the budget.
    pub dropped: usize,
    /// Token cost of all dropped symbols combined.
    pub dropped_tokens: usize,
}

impl TokenBudget {
    /// Create a new budget with the given token limit.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            used: 0,
            dropped: 0,
            dropped_tokens: 0,
        }
    }

    /// Check whether `token_cost` additional tokens would fit within
    /// the remaining budget.
    pub fn would_fit(&self, token_cost: usize) -> bool {
        self.used.saturating_add(token_cost) <= self.limit
    }

    /// Reserve `token_cost` tokens in the budget. Callers should
    /// check `would_fit` first.
    pub fn reserve(&mut self, token_cost: usize) {
        self.used = self.used.saturating_add(token_cost);
    }

    /// Record a symbol that was dropped (didn't fit).
    pub fn record_dropped(&mut self, token_cost: usize) {
        self.dropped = self.dropped.saturating_add(1);
        self.dropped_tokens = self.dropped_tokens.saturating_add(token_cost);
    }

    /// Number of tokens remaining in the budget.
    pub fn remaining(&self) -> usize {
        self.limit.saturating_sub(self.used)
    }

    /// Format the `§BUDGET` header line.
    pub fn format_header(&self) -> String {
        format!(
            "§BUDGET {} used={} dropped={} dropped_tokens={}",
            self.limit, self.used, self.dropped, self.dropped_tokens
        )
    }
}

#[cfg(test)]
#[path = "../tests/intelligence/budget.rs"]
mod tests;