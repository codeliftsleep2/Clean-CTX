//! Test-only counts of delegated complete-set searches; never admission authority.
thread_local! {
    static POSITIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static COUNTS: std::cell::Cell<[usize;4]> = const { std::cell::Cell::new([0;4]) };
}
pub(crate) fn construction(index: usize) {
    super::admission_work_metrics::delegated(index);
    COUNTS.with(|c| {
        let mut counts = c.get();
        counts[index] += 1;
        c.set(counts);
    });
}
pub(crate) fn reset() {
    COUNTS.with(|c| c.set([0; 4]));
    POSITIONS.with(|c| c.set(0));
}
pub(crate) fn counts() -> [usize; 4] {
    COUNTS.with(std::cell::Cell::get)
}

pub(crate) fn position() {
    POSITIONS.with(|c| c.set(c.get() + 1));
}
pub(crate) fn positions() -> usize {
    POSITIONS.with(std::cell::Cell::get)
}
