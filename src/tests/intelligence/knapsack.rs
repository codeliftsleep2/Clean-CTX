use super::super::super::intelligence::budget::TokenBudget;
use super::super::super::intelligence::knapsack::{RankedSymbol, pack_to_budget, sort_by_rank};

fn make_symbol(symbol: &str, rank: f64, token_cost: usize) -> RankedSymbol {
    RankedSymbol {
        symbol: symbol.to_string(),
        file_alias: "α1".to_string(),
        rank,
        token_cost,
    }
}

#[test]
fn test_pack_exact_fit() {
    let symbols = vec![
        make_symbol("UserService", 0.9, 500),
        make_symbol("Logger", 0.3, 300),
    ];
    let mut budget = TokenBudget::new(800);
    let packed = pack_to_budget(symbols, &mut budget);
    assert_eq!(packed.len(), 2);
    assert_eq!(budget.used, 800);
    assert_eq!(budget.dropped, 0);
}

#[test]
fn test_pack_overflow_drops_lowest() {
    let symbols = vec![
        make_symbol("UserService", 0.9, 500),
        make_symbol("PaymentGateway", 0.7, 400),
        make_symbol("Logger", 0.3, 200),
    ];
    let mut budget = TokenBudget::new(800);
    let packed = pack_to_budget(symbols, &mut budget);
    // UserService (500) + PaymentGateway (400) = 900 > 800
    // So PaymentGateway should be dropped (it's the second one checked)
    // Actually greedy: UserService (500) fits, PaymentGateway (400) doesn't fit (500+400=900>800)
    // Logger (200) fits (500+200=700<=800)
    assert_eq!(packed.len(), 2);
    assert_eq!(packed[0].symbol, "UserService");
    assert_eq!(packed[1].symbol, "Logger");
    assert_eq!(budget.used, 700);
    assert_eq!(budget.dropped, 1);
    assert_eq!(budget.dropped_tokens, 400);
}

#[test]
fn test_pack_single_oversized_symbol() {
    let symbols = vec![make_symbol("HugeFile", 0.9, 5000)];
    let mut budget = TokenBudget::new(1000);
    let packed = pack_to_budget(symbols, &mut budget);
    assert!(packed.is_empty());
    assert_eq!(budget.dropped, 1);
    assert_eq!(budget.dropped_tokens, 5000);
}

#[test]
fn test_pack_budget_larger_than_workspace() {
    let symbols = vec![make_symbol("A", 0.9, 100), make_symbol("B", 0.5, 200)];
    let mut budget = TokenBudget::new(10000);
    let packed = pack_to_budget(symbols, &mut budget);
    assert_eq!(packed.len(), 2);
    assert_eq!(budget.dropped, 0);
    assert_eq!(budget.used, 300);
}

#[test]
fn test_pack_zero_budget() {
    let symbols = vec![make_symbol("A", 0.9, 100)];
    let mut budget = TokenBudget::new(0);
    let packed = pack_to_budget(symbols, &mut budget);
    assert!(packed.is_empty());
    assert_eq!(budget.dropped, 1);
}

#[test]
fn test_pack_empty_symbols() {
    let symbols = vec![];
    let mut budget = TokenBudget::new(1000);
    let packed = pack_to_budget(symbols, &mut budget);
    assert!(packed.is_empty());
    assert_eq!(budget.used, 0);
    assert_eq!(budget.dropped, 0);
}

#[test]
fn test_sort_by_rank() {
    let symbols = vec![
        make_symbol("low", 0.1, 100),
        make_symbol("high", 0.9, 100),
        make_symbol("medium", 0.5, 100),
    ];
    let sorted = sort_by_rank(symbols);
    assert_eq!(sorted[0].symbol, "high");
    assert_eq!(sorted[1].symbol, "medium");
    assert_eq!(sorted[2].symbol, "low");
}

#[test]
fn test_pack_preserves_rank_order() {
    let symbols = vec![
        make_symbol("High", 0.9, 300),
        make_symbol("Medium", 0.5, 300),
        make_symbol("Low", 0.1, 300),
    ];
    let mut budget = TokenBudget::new(500);
    let packed = pack_to_budget(symbols, &mut budget);
    // High (300) fits, Medium (300) doesn't fit (600>500), Low (300) doesn't fit
    assert_eq!(packed.len(), 1);
    assert_eq!(packed[0].symbol, "High");
    assert_eq!(budget.dropped, 2);
}

#[test]
fn test_budget_header_after_pack() {
    let symbols = vec![
        make_symbol("A", 0.9, 400),
        make_symbol("B", 0.5, 400),
        make_symbol("C", 0.1, 400),
    ];
    let mut budget = TokenBudget::new(500);
    let _packed = pack_to_budget(symbols, &mut budget);
    let header = budget.format_header();
    assert!(header.contains("§BUDGET 500"));
    assert!(header.contains("used=400"));
    assert!(header.contains("dropped=2"));
    assert!(header.contains("dropped_tokens=800"));
}
