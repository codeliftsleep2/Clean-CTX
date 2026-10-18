//! Test-only structural counters for semantic compilation work.
//!
//! These counters observe production execution. They never participate in
//! scheduling, semantic authority, caching, or completeness decisions.

static FULL_SEMANTIC_PARSES: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

pub(crate) fn reset() {
    FULL_SEMANTIC_PARSES.store(0, std::sync::atomic::Ordering::SeqCst);
}

pub(crate) fn record_full_semantic_parse() {
    FULL_SEMANTIC_PARSES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

pub(crate) fn full_semantic_parses() -> usize {
    FULL_SEMANTIC_PARSES.load(std::sync::atomic::Ordering::SeqCst)
}
