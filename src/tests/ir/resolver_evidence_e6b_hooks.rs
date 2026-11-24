//! Deterministic validation race injection; never present in product builds.
use std::{cell::RefCell, path::Path};
type Hook = Box<dyn FnOnce(&Path)>;
thread_local! {static AFTER_METADATA:RefCell<Option<Hook>>=const{RefCell::new(None)};}
thread_local! {static AFTER_CAPTURE:RefCell<Option<Hook>>=const{RefCell::new(None)};}
pub(crate) fn set(hook: impl FnOnce(&Path) + 'static) {
    AFTER_METADATA.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
}
pub(crate) fn after_metadata(path: &Path) {
    let hook = AFTER_METADATA.with(|h| h.borrow_mut().take());
    if let Some(hook) = hook {
        hook(path);
    }
}
pub(crate) fn set_capture(hook: impl FnOnce(&Path) + 'static) {
    AFTER_CAPTURE.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
}
pub(crate) fn after_capture(path: &Path) {
    let hook = AFTER_CAPTURE.with(|h| h.borrow_mut().take());
    if let Some(hook) = hook {
        hook(path);
    }
}

thread_local! {static BEFORE_SERVING:RefCell<Option<Box<dyn FnOnce()>>>=const{RefCell::new(None)};}
pub(crate) fn set_serving(hook: impl FnOnce() + 'static) {
    BEFORE_SERVING.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
}
pub(crate) fn before_serving() {
    let hook = BEFORE_SERVING.with(|h| h.borrow_mut().take());
    if let Some(hook) = hook {
        hook();
    }
}
