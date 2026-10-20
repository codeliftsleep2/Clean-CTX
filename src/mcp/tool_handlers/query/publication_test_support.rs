//! One-shot late acquisition for final-publication regression tests.
use crate::mcp::McpState;
type Acquisition = Box<dyn FnOnce(&McpState)>;
thread_local! {
    static LATE: std::cell::RefCell<Option<Acquisition>> = const { std::cell::RefCell::new(None) };
}
pub(super) fn arm(acquisition: impl FnOnce(&McpState) + 'static) {
    LATE.with(|slot| *slot.borrow_mut() = Some(Box::new(acquisition)));
}
pub(super) fn run(state: &McpState) {
    let acquisition = LATE.with(|slot| slot.borrow_mut().take());
    if let Some(acquisition) = acquisition {
        acquisition(state);
    }
}
