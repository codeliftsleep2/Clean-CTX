//! Test-only actual-pipeline switch. No release behavior or persistent state.
use super::{Work, reset, snapshot};
use serde_json::{Value, json};
use std::cell::Cell;
use std::path::{Path, PathBuf};

thread_local! {
    static PROJECT: Cell<bool> = const {Cell::new(false)};
    static DEMAND: Cell<Option<bool>> = const {Cell::new(None)};
    static OBSERVE: Cell<bool> = const {Cell::new(true)};
}
pub(crate) fn capture(required: bool) {
    DEMAND.with(|v| v.set(Some(required)));
}
pub(crate) fn observing() -> bool {
    OBSERVE.with(Cell::get)
}
pub(crate) fn should_skip() -> bool {
    PROJECT.with(Cell::get) && DEMAND.with(Cell::get) == Some(false)
}
pub(crate) fn explicitly_ordinary() -> bool {
    DEMAND.with(Cell::get) == Some(false)
}

#[path = "manifest_demand_integrated_af_perf_002.rs"]
mod integrated;
#[path = "proof_manifest_repair_af_perf_002.rs"]
mod repair;
pub(crate) fn proof_receipt(value: Value) {
    super::COUNTS.with(|v| v.borrow_mut().proof_receipts.push(value));
}
struct Mode(bool, Option<bool>, bool);
impl Mode {
    fn new(project: bool, observe: bool) -> Self {
        Self(
            PROJECT.with(|v| v.replace(project)),
            DEMAND.with(|v| v.replace(None)),
            OBSERVE.with(|v| v.replace(observe)),
        )
    }
}
impl Drop for Mode {
    fn drop(&mut self) {
        PROJECT.with(|v| v.set(self.0));
        DEMAND.with(|v| v.set(self.1));
        OBSERVE.with(|v| v.set(self.2));
    }
}

fn state(roots: &[PathBuf]) -> crate::mcp::McpState {
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.persistence.enabled = false;
    cfg.proxy.port = 0;
    cfg.additional_roots = roots
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(cfg);
    *state.graph_bridge_lock() = None;
    state
}
fn dispatch(state: &crate::mcp::McpState, root: &Path, name: &str, proof: bool) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({"arguments":{
        "type":"find_entities","name":name,"declarations_only":proof,"workspaceRoot":root.to_string_lossy()}}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}
fn identity(state: &crate::mcp::McpState, sources: &[(PathBuf, String)]) -> Value {
    let owners: Vec<_> = sources
        .iter()
        .map(|(path, source)| {
            assert_eq!(&std::fs::read_to_string(path).unwrap(), source);
            let owner = state.semantic_owner_path(&path.to_string_lossy());
            let alias = state.alias_for_path(&owner);
            let tuples = alias
                .as_ref()
                .and_then(|a| state.ir_context_read().get_ir(a).cloned());
            let hash = alias
                .as_ref()
                .and_then(|a| state.ir_context_read().get_source_hash(a).cloned());
            let cached = alias
                .as_ref()
                .and_then(|a| state.llm_text_cache_lock().get(a).cloned());
            json!({"owner":owner,"alias":alias,"tuples":tuples,"hash":hash,"cached":cached})
        })
        .collect();
    assert!(state.persistence_store_lock().is_none());
    json!({"owners":owners,"edges":state.workspace_index_read().edge_count(),"durable_store":null})
}
fn phases(
    state: &crate::mcp::McpState,
    roots: &[PathBuf],
    sources: &[(PathBuf, String)],
    project: bool,
) -> Vec<(Value, Work)> {
    let mut result = Vec::new();
    for (phase, name, proof) in [
        ("ordinary_cold", "AbsentA", false),
        ("ordinary_warm", "AbsentA", false),
        ("different_name", "AbsentB", false),
        ("proof_after_ordinary", "Wanted0_0", true),
        ("proof_warm", "Wanted0_0", true),
    ] {
        let _mode = Mode::new(project, true);
        reset();
        let response = dispatch(state, &roots[0], name, proof);
        let work = snapshot();
        assert_eq!(
            DEMAND.with(Cell::get),
            Some(proof),
            "Missing actual authority observation for {phase}"
        );
        assert!(response.get("error").is_none(), "{response}");
        if proof {
            assert_eq!(work.skipped_calls, 0);
        } else if project {
            assert_eq!(work.predicates, 0);
            assert_eq!(work.entry_clones, 0);
            assert_eq!(work.root_manifests, 0);
        }
        result.push((
            json!({"phase":phase,"response":response,"identity":identity(state,sources)}),
            work,
        ));
    }
    result
}
fn local(roots: &[PathBuf], proof: bool, project: bool) -> u128 {
    let state = state(roots); // Configuration/initialization outside the paired local scan.
    let _mode = Mode::new(project, false);
    capture(proof);
    let selected = super::super::configured_roots(&state, Some(&roots[0].to_string_lossy()));
    let started = std::time::Instant::now();
    let scan = super::super::scan(&state, &selected, "AbsentLocal");
    let provisional = if proof {
        super::super::super::acquisition::prepare_clean_acquisition(
            crate::mcp::acquisition::AcquisitionTarget::declarations("Wanted0_0"),
            &scan.inventories,
            &state.config.exclude_patterns,
        )
    } else {
        None
    };
    drop(std::hint::black_box((scan, provisional)));
    started.elapsed().as_nanos()
}

#[test]
fn af002_proof_manifest_actual_comparison() {
    let _serial = super::super::super::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _responses = crate::protocol::handler_response_serial();
    for (roots_count, files, padding, overlap, missing) in [
        (1, 1, 0, false, false),
        (1, 32, 0, false, false),
        (4, 8, 0, false, false),
        (8, 4, 0, false, false),
        (1, 32, 4096, false, false),
        (4, 8, 4096, false, false),
        (4, 8, 0, true, false),
        (8, 4, 0, true, false),
        (4, 8, 4096, true, false),
        (4, 0, 0, false, false),
        (4, 8, 0, false, true),
        (4, 8, 0, true, true),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let mut roots = Vec::new();
        let mut sources = Vec::new();
        for i in 0..roots_count {
            let root = if overlap {
                temp.path()
                    .join("shared")
                    .join((0..i).map(|_| "nested").collect::<PathBuf>())
            } else {
                temp.path().join(format!("r{i}"))
            };
            std::fs::create_dir_all(&root).unwrap();
            for j in 0..files {
                let path = root.join(format!("f{i}_{j}.rs"));
                let source = format!("// {}\nstruct Wanted{i}_{j};\n", "共有".repeat(padding));
                std::fs::write(&path, &source).unwrap();
                sources.push((path, source));
            }
            roots.push(root);
        }
        // Duplicate configured root must follow normal canonical deduplication.
        roots.push(roots[0].clone());
        if missing {
            roots.push(temp.path().join("missing"));
        }
        if missing && overlap {
            std::fs::write(roots[0].join("malformed.rs"), [0xff, 0xfe]).unwrap();
        }
        let original = phases(&state(&roots), &roots, &sources, false);
        let candidate = phases(&state(&roots), &roots, &sources, true);
        for (index, ((before, bw), (after, aw))) in original.iter().zip(&candidate).enumerate() {
            assert!(
                before == after,
                "Complete public/proof/currentness mismatch R={roots_count} overlap={overlap} missing={missing} phase={index}"
            );
            if index >= 3 {
                assert_eq!(
                    serde_json::to_value(bw).unwrap(),
                    serde_json::to_value(aw).unwrap(),
                    "Proof-backed work/manifests differ"
                );
                if index == 3 && files > 0 {
                    assert!(
                        bw.root_manifests > 0 && bw.grouped_clones > 0,
                        "No positive proof consumer reached"
                    );
                }
                if !missing && files > 0 {
                    assert_eq!(
                        before["response"]["result"]["structuredContent"]["coverage"]["source_complete"],
                        true
                    );
                    if index == 3 {
                        assert!(
                            !bw.proof_receipts.is_empty(),
                            "Missing actual proof/provenance receipt"
                        );
                    }
                }
                if missing {
                    assert_eq!(
                        before["response"]["result"]["structuredContent"]["coverage"]["source_complete"],
                        false
                    );
                }
            }
            if missing && index == 1 {
                assert!(
                    bw.calls > 0 && aw.skipped_calls > 0,
                    "Incomplete ordinary retry disappeared"
                );
            }
            eprintln!(
                "AF002_PROOF_MANIFEST_PUBLIC {}",
                json!({"roots":roots_count,"files_per_root":files,"padding":padding,
                "overlap":overlap,"missing":missing,"phase":index,"original_work":bw,"candidate_work":aw,"complete_identity":before})
            );
        }
        for proof in [false, true] {
            for _ in 0..3 {
                local(&roots, proof, false);
                local(&roots, proof, true);
            }
            let mut before = Vec::new();
            let mut after = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    before.push(local(&roots, proof, false));
                    after.push(local(&roots, proof, true));
                } else {
                    after.push(local(&roots, proof, true));
                    before.push(local(&roots, proof, false));
                }
            }
            before.sort_unstable();
            after.sort_unstable();
            eprintln!(
                "AF002_PROOF_MANIFEST_COST {}",
                json!({"roots":roots_count,"files_per_root":files,"padding":padding,
                "overlap":overlap,"missing":missing,"proof_required":proof,"original_median_ns":before[12],
                "candidate_median_ns":after[12],"original_samples_ns":before,"candidate_samples_ns":after})
            );
        }
    }
}

#[test]
fn af002_proof_manifest_unknown_demand_retains() {
    let _mode = Mode::new(true, true);
    assert!(!should_skip(), "Unknown caller must retain construction");
    capture(true);
    assert!(!should_skip());
    capture(false);
    assert!(should_skip());
}

#[test]
fn af002_proof_manifest_enabled_persistence() {
    let _serial = super::super::super::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _responses = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("wanted.rs");
    std::fs::write(&file, "struct Wanted;\n").unwrap();
    let mut results = Vec::new();
    for project in [false, true] {
        for name in ["durable.db", "durable.db-wal", "durable.db-shm"] {
            let path = root.path().join(name);
            if path.exists() {
                std::fs::remove_file(path).unwrap();
            }
        }
        let mut cfg = crate::tests::test_config();
        cfg.cbm.enabled = false;
        cfg.proxy.port = 0;
        cfg.persistence.enabled = true;
        cfg.persistence.auto_save = true;
        cfg.persistence.db_path = root
            .path()
            .join("durable.db")
            .to_string_lossy()
            .into_owned();
        cfg.additional_roots = vec![root.path().to_string_lossy().into_owned()];
        let state = crate::mcp::McpState::new(cfg);
        *state.graph_bridge_lock() = None;
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            "compress_code_context",
            &json!({"arguments":{
            "filePath":file.to_string_lossy(),"workspaceRoot":root.path().to_string_lossy(),"fidelity":"edit"}}),
            &state,
        );
        let compressed = crate::protocol::captured_responses().pop().unwrap();
        assert!(compressed.get("error").is_none(), "{compressed}");
        let durable = || {
            let guard = state.persistence_store_lock();
            let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
            let context = sqlite
                .load_durable_context(&file.to_string_lossy(), None)
                .unwrap()
                .expect("Actual persisted context");
            format!("{context:?}")
        };
        let before = durable();
        let _mode = Mode::new(project, true);
        reset();
        let response = dispatch(&state, root.path(), "Wanted", true);
        let work = snapshot();
        let after = durable();
        assert_eq!(DEMAND.with(Cell::get), Some(true));
        assert_eq!(work.skipped_calls, 0);
        assert!(work.root_manifests > 0 && !work.proof_receipts.is_empty());
        assert_eq!(
            response["result"]["structuredContent"]["coverage"]["source_complete"],
            true
        );
        results.push(json!({"compressed":compressed,"response":response,"before_durable":before,"after_durable":after,"work":work}));
    }
    assert!(
        results[0] == results[1],
        "Enabled-persistence complete public/proof/durable identities differ"
    );
    eprintln!("AF002_PROOF_MANIFEST_DURABLE {}", results[0]);
}
