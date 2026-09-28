//! Test-only extraction counter for single-evaluation regressions.

thread_local! {
    static EXTRACTION_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record_extraction() {
    EXTRACTION_COUNT.with(|count| count.set(count.get() + 1));
}

pub(crate) fn reset_extraction_count() {
    EXTRACTION_COUNT.with(|count| count.set(0));
}

pub(crate) fn extraction_count() -> usize {
    EXTRACTION_COUNT.with(std::cell::Cell::get)
}
