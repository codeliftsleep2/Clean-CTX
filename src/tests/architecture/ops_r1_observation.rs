//! Isolated-child OPS-R1 diagnostics, not production authority or a fitness budget.
use std::sync::atomic::{AtomicU64, Ordering};
static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(crate) fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}
pub(crate) static COUNTS: [AtomicU64; 15] = [const { AtomicU64::new(0) }; 15];
const NAMES: [&str; 15] = [
    "inventory_attempts",
    "inventory_entries",
    "nominations",
    "nominated_owners",
    "compile_attempts",
    "compile_completed",
    "project_acquisitions",
    "project_units",
    "host_read_misses",
    "query_compilations",
    "witness_checks",
    "publication_attempts",
    "publication_completed",
    "project_loop_iterations",
    "returned_batch_high_water",
];
pub(crate) fn batch_results(count: usize) {
    if active() {
        COUNTS[14].fetch_max(count as u64, Ordering::Relaxed);
    }
}
pub(crate) fn record(slot: usize, count: usize) {
    if active() {
        COUNTS[slot].fetch_add(count as u64, Ordering::Relaxed);
    }
}
pub(crate) fn snapshot() -> serde_json::Value {
    let mut values: serde_json::Map<String, serde_json::Value> = NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                (*name).to_string(),
                COUNTS[i].load(Ordering::Relaxed).into(),
            )
        })
        .collect();
    for (name, counter) in EXTRA_NAMES.iter().zip(&EXTRA) {
        values.insert((*name).into(), counter.load(Ordering::Relaxed).into());
    }
    serde_json::Value::Object(values)
}
// Payload counts are lower bounds, not allocator size or RSS attribution.
const EXTRA_NAMES: [&str; 13] = [
    "distinct_operation_bases",
    "shared_acquisitions",
    "compile_live",
    "compile_peak",
    "acquire_live",
    "acquire_peak",
    "prepared_jobs",
    "prepared_source_bytes",
    "sealed_proofs",
    "sealed_source_payload_bytes",
    "inventory_owner_bytes",
    "results_completed",
    "acquisitions_completed",
];
static EXTRA: [AtomicU64; 13] = [const { AtomicU64::new(0) }; 13];
pub(crate) fn extra(slot: usize, count: usize) {
    if active() {
        EXTRA[slot].fetch_add(count as u64, Ordering::Relaxed);
    }
}
pub(crate) fn enter(slot: usize) -> Live {
    let enabled = active();
    if enabled {
        let count = EXTRA[slot].fetch_add(1, Ordering::Relaxed) + 1;
        EXTRA[slot + 1].fetch_max(count, Ordering::Relaxed);
    }
    Live { slot, enabled }
}
pub(crate) struct Live {
    slot: usize,
    enabled: bool,
}
impl Drop for Live {
    fn drop(&mut self) {
        if self.enabled {
            EXTRA[self.slot].fetch_sub(1, Ordering::Relaxed);
        }
    }
}
pub(crate) fn sample(directory: &std::path::Path) -> Sample {
    use std::sync::{Arc, atomic::AtomicBool};
    let stop = Arc::new(AtomicBool::new(false));
    if std::env::var("OPS_R1_CAPTURE").ok().as_deref() != Some("1") {
        return Sample { stop, worker: None };
    }
    for count in &COUNTS {
        count.store(0, Ordering::Relaxed);
    }
    for count in &EXTRA {
        count.store(0, Ordering::Relaxed);
    }
    ACTIVE.store(true, Ordering::Relaxed);
    let worker_stop = Arc::clone(&stop);
    let directory = directory.to_path_buf();
    let worker = std::thread::spawn(move || {
        use std::io::Write;
        let mut file = std::fs::File::create(directory.join("stages.jsonl")).unwrap();
        while !worker_stop.load(Ordering::Relaxed) {
            writeln!(file, "{}", snapshot()).unwrap();
            file.flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        writeln!(file, "{}", snapshot()).unwrap();
    });
    Sample {
        stop,
        worker: Some(worker),
    }
}
pub(crate) struct Sample {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Sample {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
        ACTIVE.store(false, Ordering::Relaxed);
    }
}
