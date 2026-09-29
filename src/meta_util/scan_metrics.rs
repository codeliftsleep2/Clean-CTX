//! Test-only work counters for meta-layer scanner migration regressions.

thread_local! {
    static LEGACY_MEMBERSHIP_CALL_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record_legacy_membership_call() {
    LEGACY_MEMBERSHIP_CALL_COUNT.with(|count| count.set(count.get() + 1));
}

pub(crate) fn reset_legacy_membership_call_count() {
    LEGACY_MEMBERSHIP_CALL_COUNT.with(|count| count.set(0));
}

pub(crate) fn legacy_membership_call_count() -> usize {
    LEGACY_MEMBERSHIP_CALL_COUNT.with(std::cell::Cell::get)
}
