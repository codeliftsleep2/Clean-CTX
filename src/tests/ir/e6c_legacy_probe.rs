//! Root-scoped cross-worker instrumentation for the public v1 comparison only.
//! Unlike E1's thread-local counters, this observes hydration's Rayon workers.
use std::{path::Path, sync::Mutex};
#[derive(Clone, Copy, Default, Debug, serde::Serialize)]
pub(crate) struct Counts {
    pub automatic: u64,
    pub program: u64,
    pub dependency_parses: u64,
}
static ACTIVE: Mutex<Option<(String, Counts)>> = Mutex::new(None);
pub(crate) struct Guard;
pub(crate) fn start(root: &Path) -> Guard {
    let mut slot = ACTIVE.lock().unwrap();
    assert!(slot.is_none());
    *slot = Some((shape(root), Counts::default()));
    Guard
}
fn shape(path: &Path) -> String {
    path.to_string_lossy().replace(r"\\?\", "").to_lowercase()
}
fn event(file: &Path, apply: impl FnOnce(&mut Counts)) {
    let mut slot = ACTIVE.lock().unwrap();
    if let Some((root, counts)) = slot.as_mut() {
        let file = shape(file);
        if file == *root || file.starts_with(&format!("{root}{}", std::path::MAIN_SEPARATOR)) {
            apply(counts);
        }
    }
}
pub(crate) fn automatic(file: &Path) {
    event(file, |c| c.automatic += 1);
}
pub(crate) fn program(file: &Path) {
    event(file, |c| c.program += 1);
}
pub(crate) fn parse(file: &Path) {
    if file.components().any(|p| p.as_os_str() == "node_modules") {
        event(file, |c| c.dependency_parses += 1);
    }
}
impl Guard {
    pub(crate) fn snapshot(&self) -> Counts {
        ACTIVE.lock().unwrap().as_ref().unwrap().1
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        *ACTIVE.lock().unwrap() = None;
    }
}
