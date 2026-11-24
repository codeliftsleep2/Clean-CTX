//! E1 measurement-only counters. Never compiled into a production executable.
use std::{cell::RefCell, collections::BTreeMap, path::Path};

#[derive(Default, Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Counts {
    pub bytes_read: usize,
    pub validation_bytes: usize,
    pub parses: usize,
    pub dependency_parses: usize,
    pub parse_bytes: usize,
    pub source_reads: BTreeMap<String, usize>,
    pub acquisition_results: usize,
    pub acquisition_successes: usize,
    pub automatic_type_calls: usize,
    pub library_calls: usize,
}
thread_local! {
    static COUNTS: RefCell<Option<Counts>> = const { RefCell::new(None) };
}
pub(crate) fn reset() {
    COUNTS.with(|c| *c.borrow_mut() = Some(Counts::default()));
    crate::work_observations_af_perf_002::reset();
}
pub(super) fn parse(path: &Path, bytes: usize) {
    COUNTS.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            c.parses += 1;
            c.parse_bytes += bytes;
            c.dependency_parses +=
                usize::from(path.components().any(|p| p.as_os_str() == "node_modules"));
        }
    });
}
pub(super) fn read(path: &Path, bytes: usize, validation: bool) {
    COUNTS.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            c.bytes_read += bytes;
            c.validation_bytes += if validation { bytes } else { 0 };
            *c.source_reads
                .entry(path.to_string_lossy().into_owned())
                .or_default() += 1;
        }
    });
}
pub(crate) fn snapshot() -> Counts {
    COUNTS.with(|c| c.borrow().clone().unwrap_or_default())
}
pub(super) fn acquisition(success: bool) {
    COUNTS.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            c.acquisition_results += 1;
            c.acquisition_successes += usize::from(success);
        }
    });
}
pub(super) fn traversal(library: bool) {
    COUNTS.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            if library {
                c.library_calls += 1;
            } else {
                c.automatic_type_calls += 1;
            }
        }
    });
}
pub(crate) fn finish() -> serde_json::Value {
    let c = crate::work_observations_af_perf_002::snapshot();
    serde_json::json!({
        "io": snapshot(), "witness_checks": c.witnesses,
        "unique_witnesses": c.identities.len(), "witness_reads": c.reads,
        "hashes": c.hashes, "directory_enumerations": c.enumerations,
        "metadata_operations": c.metadata, "canonicalizations": c.canonicalizations,
    })
}
