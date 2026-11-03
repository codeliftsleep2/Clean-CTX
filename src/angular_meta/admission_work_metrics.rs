//! Test-only complete-projection counts for one synchronous admission operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Consumer {
    Activate,
    Match,
    Deactivate,
    Load,
    ActivateChild,
    Eager,
    Lazy,
    Children,
    Providers,
}
thread_local! {
    static COUNTS: std::cell::Cell<[usize; 9]> = const { std::cell::Cell::new([0; 9]) };
}
pub(crate) fn projection(consumer: Consumer) {
    operation_projection(consumer as usize);
    COUNTS.with(|c| {
        let mut counts = c.get();
        counts[consumer as usize] += 1;
        c.set(counts);
    });
}
pub(crate) fn reset() {
    COUNTS.with(|c| c.set([0; 9]));
}
pub(crate) fn counts() -> [usize; 9] {
    COUNTS.with(std::cell::Cell::get)
}

/// Observation only: no state here participates in production admission.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Operations {
    pub calls: usize,
    pub maximum: [usize; 14],
    pub witnesses: usize,
}
thread_local! {
    static OPERATIONS: std::cell::RefCell<(Operations, Vec<[usize; 14]>)> =
        const { std::cell::RefCell::new((Operations { calls: 0, maximum: [0;14], witnesses: 0 }, Vec::new())) };
}
pub(crate) struct Admission;
pub(crate) fn begin() -> Admission {
    OPERATIONS.with(|s| {
        let mut s = s.borrow_mut();
        s.0.calls += 1;
        s.1.push([0; 14]);
    });
    Admission
}
impl Drop for Admission {
    fn drop(&mut self) {
        OPERATIONS.with(|s| {
            let mut s = s.borrow_mut();
            let counts = s.1.pop().expect("admission observation frame");
            for (i, count) in counts.into_iter().enumerate() {
                s.0.maximum[i] = s.0.maximum[i].max(count);
            }
        });
    }
}
fn operation_projection(index: usize) {
    OPERATIONS.with(|s| {
        if let Some(frame) = s.borrow_mut().1.last_mut() {
            frame[index] += 1;
        }
    });
}
pub(crate) fn resolver() {
    operation_projection(9);
}
pub(crate) fn witness() {
    OPERATIONS.with(|s| s.borrow_mut().0.witnesses += 1);
}
pub(crate) fn reset_operations() {
    OPERATIONS.with(|s| {
        let mut s = s.borrow_mut();
        assert!(s.1.is_empty());
        s.0 = Operations::default();
    });
}
pub(crate) fn operations() -> Operations {
    OPERATIONS.with(|s| s.borrow().0)
}

pub(crate) fn delegated(index: usize) {
    operation_projection(10 + index);
}
