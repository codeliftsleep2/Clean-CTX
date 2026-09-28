//! Test-only counter for Angular detection-pass regressions.

thread_local! {
    static DETECTION_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record_detection() {
    DETECTION_COUNT.with(|count| count.set(count.get() + 1));
}

pub(crate) fn reset_detection_count() {
    DETECTION_COUNT.with(|count| count.set(0));
}

pub(crate) fn detection_count() -> usize {
    DETECTION_COUNT.with(std::cell::Cell::get)
}
