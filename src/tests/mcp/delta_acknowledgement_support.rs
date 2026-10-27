//! Test-only fault at the committed acknowledgement/rendering boundary.
use std::cell::RefCell;

thread_local! {
    static REMOVE_SOURCE: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) struct RemovalGuard;

impl Drop for RemovalGuard {
    fn drop(&mut self) {
        let _ = REMOVE_SOURCE.with(|slot| slot.borrow_mut().take());
    }
}

pub(crate) fn remove_source_after_commit(file: &str) -> RemovalGuard {
    REMOVE_SOURCE.with(|slot| {
        assert!(slot.borrow().is_none(), "removal already armed");
        *slot.borrow_mut() = Some(file.to_owned());
    });
    RemovalGuard
}

pub(crate) fn after_commit(state: &crate::mcp::McpState, file: &str) {
    let remove = REMOVE_SOURCE.with(|slot| {
        let mut armed = slot.borrow_mut();
        if armed.as_deref() == Some(file) {
            let _ = armed.take();
            true
        } else {
            false
        }
    });
    if remove {
        std::fs::remove_file(file).expect("remove source after delta commit");
        state.invalidate_source_cache(file);
    }
}
