// src/tests/analytics.rs
//
// Tests for the BPE engine cache (FAANG audit F-01). The whole point of
// the `OnceLock` is that repeated `bpe()` calls return the *same*
// underlying engine — i.e. it loads exactly once.

use super::*;

#[test]
fn bpe_returns_same_pointer_repeatedly() {
    // Two calls should return the same `&CoreBPE` reference (i.e. the
    // process-global was populated exactly once). This is the headline
    // invariant: if the engine ever reloads under us, the entire
    // `OnceLock` cache is doing nothing.
    let a = bpe() as *const tiktoken_rs::CoreBPE;
    let b = bpe() as *const tiktoken_rs::CoreBPE;
    let c = bpe_or_init().expect("bpe should be loadable in tests") as *const tiktoken_rs::CoreBPE;

    assert_eq!(a, b, "bpe() must return the same engine across calls");
    assert_eq!(a, c, "bpe() and bpe_or_init() must return the same engine");
}

#[test]
fn bpe_or_init_is_idempotent() {
    // The init path is safe to call multiple times; the second call
    // short-circuits because the OnceLock is already populated.
    let first = bpe_or_init().expect("first init must succeed") as *const tiktoken_rs::CoreBPE;
    let second = bpe_or_init().expect("second init must succeed") as *const tiktoken_rs::CoreBPE;
    let third = bpe_or_init().expect("third init must succeed") as *const tiktoken_rs::CoreBPE;

    assert_eq!(first, second);
    assert_eq!(second, third);
}

#[test]
fn calculate_savings_smoke_test() {
    // A trivial input/output pair: counts should be non-zero and
    // `savings_percentage` should be in [0, 100].
    let raw = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let compressed = "$0 $1 $2 $3 $4 $5 $6 $7 $8 $0 $1 $2";

    let meta = calculate_savings(raw, compressed);

    assert!(meta.raw_tokens > 0, "raw token count must be > 0");
    assert!(meta.compressed_tokens > 0, "compressed token count must be > 0");
    assert!(
        (0.0..=100.0).contains(&meta.savings_percentage),
        "savings_percentage out of range: {}",
        meta.savings_percentage
    );
}

#[test]
fn calculate_savings_empty_input() {
    // Empty input should not divide-by-zero and should report 0% savings.
    let meta = calculate_savings("", "anything");

    assert_eq!(meta.raw_tokens, 0);
    assert_eq!(meta.savings_percentage, 0.0);
}
