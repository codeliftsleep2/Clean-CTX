use super::super::super::intelligence::budget::TokenBudget;

#[test]
fn test_budget_new() {
    let b = TokenBudget::new(8192);
    assert_eq!(b.limit, 8192);
    assert_eq!(b.used, 0);
    assert_eq!(b.dropped, 0);
    assert_eq!(b.dropped_tokens, 0);
}

#[test]
fn test_budget_would_fit() {
    let b = TokenBudget::new(100);
    assert!(b.would_fit(50));
    assert!(b.would_fit(100));
    assert!(!b.would_fit(101));
}

#[test]
fn test_budget_reserve() {
    let mut b = TokenBudget::new(100);
    b.reserve(40);
    assert_eq!(b.used, 40);
    assert!(b.would_fit(60));
    assert!(!b.would_fit(61));
}

#[test]
fn test_budget_record_dropped() {
    let mut b = TokenBudget::new(100);
    b.record_dropped(30);
    assert_eq!(b.dropped, 1);
    assert_eq!(b.dropped_tokens, 30);
    b.record_dropped(20);
    assert_eq!(b.dropped, 2);
    assert_eq!(b.dropped_tokens, 50);
}

#[test]
fn test_budget_remaining() {
    let mut b = TokenBudget::new(100);
    assert_eq!(b.remaining(), 100);
    b.reserve(30);
    assert_eq!(b.remaining(), 70);
    b.reserve(70);
    assert_eq!(b.remaining(), 0);
}

#[test]
fn test_budget_format_header() {
    let mut b = TokenBudget::new(8192);
    b.reserve(5000);
    b.record_dropped(1200);
    b.record_dropped(800);
    let header = b.format_header();
    assert!(header.contains("§BUDGET 8192"));
    assert!(header.contains("used=5000"));
    assert!(header.contains("dropped=2"));
    assert!(header.contains("dropped_tokens=2000"));
}

#[test]
fn test_budget_saturating_arithmetic() {
    let mut b = TokenBudget::new(100);
    b.reserve(200); // would exceed limit, but saturating_add caps at usize::MAX
    // saturating_add caps at usize::MAX, not the budget limit.
    // The budget limit is enforced by would_fit() before reserve().
    assert_eq!(b.used, 200);
    // would_fit correctly rejects this
    assert!(!b.would_fit(1));
}

#[test]
fn test_budget_zero_limit() {
    let mut b = TokenBudget::new(0);
    assert!(!b.would_fit(1));
    assert!(b.would_fit(0));
    b.reserve(0);
    assert_eq!(b.used, 0);
    let header = b.format_header();
    assert!(header.contains("§BUDGET 0"));
}
