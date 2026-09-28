//! Test-only counter for the Spring combined evaluation boundary.

thread_local! {
    static EVALUATION_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DETECTION_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record_evaluation() {
    EVALUATION_COUNT.with(|count| count.set(count.get() + 1));
}

pub(crate) fn reset_evaluation_count() {
    EVALUATION_COUNT.with(|count| count.set(0));
}

pub(crate) fn evaluation_count() -> usize {
    EVALUATION_COUNT.with(std::cell::Cell::get)
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
