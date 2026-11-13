//! Production-boundary discovery only; source Arc payloads are never copied here.
use super::{RootInventoryManifest, root_inventories, root_key};
use crate::mcp::acquisition::{InventoryEntry, InventoryManifest, SourceKind};
use serde_json::json;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[path = "proof_manifest_design_af_perf_002.rs"]
pub(crate) mod comparison;

#[derive(Clone, Default, serde::Serialize)]
pub(crate) struct Work {
    calls: usize,
    roots: usize,
    input_entries: usize,
    predicates: usize,
    rejected: usize,
    entry_clones: usize,
    cloned_string_bytes: usize,
    shared_source_bytes: usize,
    root_manifests: usize,
    output_entries: usize,
    output_capacity: usize,
    grouped_clones: usize,
    grouped_string_bytes: usize,
    grouped_shared_source_bytes: usize,
    groups: usize,
    grouped_entries: usize,
    grouped_capacity: usize,
    root_receipts: Vec<serde_json::Value>,
    skipped_calls: usize,
    proof_receipts: Vec<serde_json::Value>,
    unused_predicates: usize,
    unused_clones: usize,
    unused_string_bytes: usize,
    unused_manifests: usize,
    unused_entries: usize,
    unused_capacity: usize,
    combined_receipts: Vec<serde_json::Value>,
}
thread_local! {static COUNTS: RefCell<Work> = RefCell::new(Work::default());}
pub(crate) fn begin(roots: usize, entries: usize) -> bool {
    let skip = comparison::should_skip();
    if !comparison::observing() {
        return skip;
    }
    COUNTS.with(|w| {
        let mut w = w.borrow_mut();
        w.calls += 1;
        w.roots += roots;
        w.input_entries += entries;
        w.skipped_calls += usize::from(skip);
    });
    skip
}
pub(crate) fn predicate(entry: &InventoryEntry, selected: bool) {
    if !comparison::observing() {
        return;
    }
    COUNTS.with(|w| {
        let mut w = w.borrow_mut();
        w.predicates += 1;
        if comparison::explicitly_ordinary() {
            w.unused_predicates += 1;
            if selected {
                w.unused_clones += 1;
                w.unused_string_bytes += entry.owner.len() + entry.source_hash.len();
            }
        }
        if selected {
            w.entry_clones += 1;
            w.cloned_string_bytes += entry.owner.len() + entry.source_hash.len();
            w.shared_source_bytes += entry.source.len();
        } else {
            w.rejected += 1;
        }
    });
}
pub(crate) fn manifest(root: &str, manifest: &InventoryManifest) {
    if !comparison::observing() {
        return;
    }
    COUNTS.with(|w|{
    let mut w=w.borrow_mut();w.root_manifests+=1;w.output_entries+=manifest.entries.len();w.output_capacity+=manifest.entries.capacity();
    if comparison::explicitly_ordinary(){w.unused_manifests+=1;w.unused_entries+=manifest.entries.len();w.unused_capacity+=manifest.entries.capacity();}
    w.root_receipts.push(json!({"root":root,"digest":manifest.digest,"attempted":manifest.attempted,"complete":manifest.complete,
        "entries":manifest.entries.iter().map(|e|json!({"owner":e.owner,"hash":e.source_hash,"kind":format!("{:?}",e.source_kind)})).collect::<Vec<_>>()}));
});
}
pub(crate) fn combined(manifest: &InventoryManifest) {
    if !comparison::observing() {
        return;
    }
    COUNTS.with(|w|w.borrow_mut().combined_receipts.push(json!({"digest":manifest.digest,"attempted":manifest.attempted,
        "complete":manifest.complete,"capacity":manifest.entries.capacity(),"entries":manifest.entries.iter().map(|e|
        json!({"owner":e.owner,"kind":format!("{:?}",e.source_kind),"hash":e.source_hash,"source_len":e.source.len()})).collect::<Vec<_>>()})));
}
pub(crate) fn group_clone(entry: &InventoryEntry) {
    if !comparison::observing() {
        return;
    }
    COUNTS.with(|w| {
        let mut w = w.borrow_mut();
        w.grouped_clones += 1;
        w.grouped_string_bytes += entry.owner.len() + entry.source_hash.len();
        w.grouped_shared_source_bytes += entry.source.len();
    });
}
pub(crate) fn group_capacity(len: usize, capacity: usize) {
    if !comparison::observing() {
        return;
    }
    COUNTS.with(|w| {
        let mut w = w.borrow_mut();
        w.groups += 1;
        w.grouped_entries += len;
        w.grouped_capacity += capacity;
    });
}
fn reset() {
    COUNTS.with(|w| *w.borrow_mut() = Work::default());
}
fn snapshot() -> Work {
    COUNTS.with(|w| w.borrow().clone())
}

fn original(roots: &[PathBuf], combined: &InventoryManifest) -> Vec<RootInventoryManifest> {
    roots
        .iter()
        .map(|root| {
            let root = root_key(root);
            let entries = combined
                .entries
                .iter()
                .filter(|entry| Path::new(&entry.owner).starts_with(Path::new(&root)))
                .cloned()
                .collect();
            RootInventoryManifest {
                root,
                manifest: InventoryManifest::new(entries, combined.attempted, combined.complete),
            }
        })
        .collect()
}
fn compare(roots: &[PathBuf], manifest: &InventoryManifest) {
    let old = original(roots, manifest);
    reset();
    let actual = root_inventories(roots, manifest);
    let w = snapshot();
    assert_eq!(old.len(), actual.len());
    for (old, new) in old.iter().zip(&actual) {
        assert_eq!(old.root, new.root);
        assert_eq!(old.manifest, new.manifest);
        for entry in &new.manifest.entries {
            assert!(manifest.entries.iter().any(
                |input| input.owner == entry.owner && Arc::ptr_eq(&input.source, &entry.source)
            ));
        }
    }
    assert_eq!(w.predicates, roots.len() * manifest.entries.len());
    assert_eq!(w.entry_clones + w.rejected, w.predicates);
}
#[test]
fn af002_root_inventory_contracts() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path();
    let nested = base.join("nested");
    let roots = vec![
        base.to_path_buf(),
        nested.clone(),
        base.join("missing"),
        base.to_path_buf(),
        base.join("./nested"),
    ];
    let entry = |owner: String, hash: &str| InventoryEntry {
        owner,
        source_kind: SourceKind::Rust,
        source_hash: hash.into(),
        source: Arc::new("共有".repeat(4096)),
    };
    let entries = vec![
        entry(root_key(&base.join("a.rs")), "a"),
        entry(root_key(&nested.join("b.rs")), "b"),
        entry(root_key(&base.join("nested-other/c.rs")), "boundary"),
        entry("".into(), "malformed"),
        entry("relative/../odd.rs".into(), "relative"),
        entry(root_key(&nested.join("b.rs")), "duplicate"),
    ];
    for (attempted, complete) in [(true, true), (true, false), (false, true)] {
        // Raw duplicates/malformed spellings are deliberate helper inputs, not admitted production inventory.
        let manifest = InventoryManifest {
            entries: entries.clone(),
            digest: "untrusted-fixture".into(),
            attempted,
            complete,
        };
        compare(&roots, &manifest);
        compare(&[], &manifest);
        compare(
            &roots,
            &InventoryManifest::new(Vec::new(), attempted, complete),
        );
    }
}

fn state(roots: &[PathBuf]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    config.proxy.port = 0;
    config.additional_roots = roots
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}
fn dispatch(state: &crate::mcp::McpState, root: &Path, name: &str) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({"arguments":{
        "type":"find_entities","name":name,"workspaceRoot":root.to_string_lossy()}}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}
#[test]
fn af002_root_inventory_public_dimensions() {
    let _serial = super::super::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _responses = crate::protocol::handler_response_serial();
    for (root_count, files, padding, overlap, missing) in [
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
        let base = temp.path();
        let mut roots = Vec::new();
        let mut sources = Vec::new();
        for i in 0..root_count {
            let root = if overlap {
                base.join("shared")
                    .join((0..i).map(|_| "nested").collect::<PathBuf>())
            } else {
                base.join(format!("r{i}"))
            };
            std::fs::create_dir_all(&root).unwrap();
            for j in 0..files {
                let path = root.join(format!("f{i}_{j}.rs"));
                let source = format!("// {}\nstruct Inventory{i}_{j};\n", "共有".repeat(padding));
                std::fs::write(&path, &source).unwrap();
                sources.push((path, source));
            }
            roots.push(root);
        }
        if missing {
            roots.push(base.join("missing"));
        }
        let malformed = overlap && missing;
        if malformed {
            std::fs::write(roots[0].join("malformed.rs"), [0xff, 0xfe]).unwrap();
        }
        let state = state(&roots);
        for (phase, name) in [
            ("cold", "AbsentInventoryA"),
            ("warm", "AbsentInventoryA"),
            ("different_name", "AbsentInventoryB"),
        ] {
            reset();
            let start = std::time::Instant::now();
            let response = dispatch(&state, &roots[0], name);
            let elapsed = start.elapsed().as_nanos();
            let w = snapshot();
            assert!(response.get("error").is_none(), "{response}");
            // Ordinary discovery no longer constructs unused per-root proof manifests.
            assert_eq!(w.calls, 0, "ordinary discovery constructed proof manifests");
            assert_eq!(w.predicates, 0);
            assert_eq!(w.entry_clones, 0);
            assert_eq!(w.root_manifests, 0);
            assert_eq!(w.predicates, w.entry_clones + w.rejected);
            for (path, source) in &sources {
                assert_eq!(&std::fs::read_to_string(path).unwrap(), source);
            }
            let source_receipts: Vec<_> = sources
                .iter()
                .map(|(path, source)| {
                    let owner = state.semantic_owner_path(&path.to_string_lossy());
                    let alias = state.alias_for_path(&owner);
                    let hash = alias
                        .as_ref()
                        .and_then(|a| state.ir_context_read().get_source_hash(a).cloned());
                    let expected = state.cache_read().compute_hash(source.as_bytes());
                    if let Some(hash) = &hash {
                        assert_eq!(hash, &expected);
                    }
                    json!({"owner":owner,"alias":alias,"live_hash":hash,"expected_hash":expected})
                })
                .collect();
            if malformed {
                assert_eq!(
                    std::fs::read(roots[0].join("malformed.rs")).unwrap(),
                    [0xff, 0xfe]
                );
            }
            let retained_response = serde_json::to_value(&response).unwrap();
            let drop_start = std::time::Instant::now();
            drop(response);
            let response_drop_ns = drop_start.elapsed().as_nanos();
            eprintln!(
                "AF002_ROOT_INVENTORY {}",
                json!({"roots":root_count,"files_per_root":files,"padding_repetitions":padding,
                "overlap":overlap,"missing":missing,"phase":phase,"work":w,"actual_operation_ns":elapsed,
                "malformed_source":malformed,"sources":source_receipts,
                "response_drop_ns":response_drop_ns,"response":retained_response})
            );
        }
    }
}
