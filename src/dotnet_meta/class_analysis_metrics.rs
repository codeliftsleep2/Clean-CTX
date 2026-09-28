//! Test-only counter for .NET per-class analysis regressions.

thread_local! {
    static ANALYSIS_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record_analysis() {
    ANALYSIS_COUNT.with(|count| count.set(count.get() + 1));
}

pub(crate) fn reset_analysis_count() {
    ANALYSIS_COUNT.with(|count| count.set(0));
}

pub(crate) fn analysis_count() -> usize {
    ANALYSIS_COUNT.with(std::cell::Cell::get)
}
