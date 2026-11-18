//! Test-only operation-local work counters; no production cache or authority.
thread_local! {
    static COUNTS: std::cell::Cell<(usize,usize)> = const { std::cell::Cell::new((0,0)) };
}
#[cfg(feature = "angular")]
pub(crate) fn projection() {
    COUNTS.with(|c| {
        let (p, m) = c.get();
        c.set((p + 1, m));
    });
}
#[cfg(feature = "angular")]
pub(crate) fn mapping() {
    COUNTS.with(|c| {
        let (p, m) = c.get();
        c.set((p, m + 1));
    });
}
#[cfg(feature = "angular")]
pub(crate) fn reset() {
    COUNTS.with(|c| c.set((0, 0)));
}
#[cfg(feature = "angular")]
pub(crate) fn counts() -> (usize, usize) {
    COUNTS.with(std::cell::Cell::get)
}
