//! Fixed-size request-local counters for the explicitly enabled public adapter.
//! Diagnostic observations only; never authority, publication or cache inputs.
use std::cell::RefCell;
#[derive(Clone, Copy, Default, serde::Serialize)]
pub(crate) struct Counts {
    bytes_read: u64,
    validation_bytes: u64,
    read_events: u64,
    parses: u64,
    dependency_parses: u64,
    program_attempts: u64,
    program_results: u64,
    program_successes: u64,
    automatic_type_calls: u64,
    compiler_library_calls: u64,
    witness_checks: u64,
}
thread_local! {static CURRENT:RefCell<Option<Counts>>=const{RefCell::new(None)};}
pub(crate) struct Scope {
    previous: Option<Counts>,
}
pub(crate) fn begin() -> Scope {
    Scope {
        previous: CURRENT.with(|c| c.replace(Some(Counts::default()))),
    }
}
impl Scope {
    pub(crate) fn snapshot(&self) -> Counts {
        CURRENT.with(|c| c.borrow().unwrap_or_default())
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.with(|c| {
            c.replace(self.previous);
        });
    }
}
fn event(f: impl FnOnce(&mut Counts)) {
    CURRENT.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            f(c);
        }
    });
}
pub(crate) fn read(bytes: usize, validation: bool) {
    event(|c| {
        c.bytes_read = c.bytes_read.saturating_add(bytes as u64);
        c.read_events = c.read_events.saturating_add(1);
        if validation {
            c.validation_bytes = c.validation_bytes.saturating_add(bytes as u64);
        }
    });
}
pub(crate) fn parse(path: &std::path::Path) {
    event(|c| {
        c.parses = c.parses.saturating_add(1);
        c.dependency_parses = c.dependency_parses.saturating_add(u64::from(
            path.components().any(|p| p.as_os_str() == "node_modules"),
        ));
    });
}
pub(crate) fn program_attempt() {
    event(|c| c.program_attempts = c.program_attempts.saturating_add(1));
}
pub(crate) fn program_result(success: bool) {
    event(|c| {
        c.program_results = c.program_results.saturating_add(1);
        c.program_successes = c.program_successes.saturating_add(u64::from(success));
    });
}
pub(crate) fn automatic() {
    event(|c| c.automatic_type_calls = c.automatic_type_calls.saturating_add(1));
}
pub(crate) fn library() {
    event(|c| c.compiler_library_calls = c.compiler_library_calls.saturating_add(1));
}
pub(crate) fn witness() {
    event(|c| c.witness_checks = c.witness_checks.saturating_add(1));
}

#[cfg(test)]
#[path = "../../tests/ir/e6c_legacy_probe.rs"]
pub(crate) mod legacy_probe;
