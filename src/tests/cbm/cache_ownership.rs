// Project-owned graph-cache regressions.
//
// These tests pin the migration contract established in
// docs/architecture/CBM_PROJECT_CACHE_OWNERSHIP_MIGRATION_PLAN_2026-09-28.md:
// every memory/disk entry belongs to one canonical CBM project, and cache
// invalidation targets that project rather than unrelated active bridge state.

use crate::cbm::bridge::test_helpers::new_mock;
use crate::cbm::bridge::{GraphBridge, SymbolImportance, cbm_project_slug};
use crate::cbm::cache_store::GraphCacheStore;
use crate::cbm::config::CbmConfig;
use std::collections::HashMap;
use std::path::Path;

const FAR_FUTURE_EPOCH_MS: i64 = 4_102_444_800_000;
const SYMBOL_KEY: &str = "symbol_importance";

fn canonical(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn symbol_payload(name: &str, score: f64, file: &str) -> serde_json::Value {
    let mut symbols = HashMap::new();
    symbols.insert(
        name.to_string(),
        SymbolImportance {
            symbol: name.to_string(),
            score,
            file: file.to_string(),
        },
    );
    serde_json::to_value(symbols).expect("symbol payload")
}

fn seed_symbol_partition(
    store: &GraphCacheStore,
    root: &Path,
    project: &str,
    payload: &serde_json::Value,
) {
    store.put(
        &root.to_string_lossy(),
        &format!("{project}:{SYMBOL_KEY}"),
        &payload.to_string(),
        FAR_FUTURE_EPOCH_MS,
    );
}

fn read_symbol_partition(store: &GraphCacheStore, root: &Path, project: &str) -> Option<String> {
    store.get(&root.to_string_lossy(), &format!("{project}:{SYMBOL_KEY}"))
}

#[test]
fn project_switch_preserves_the_original_projects_memory_entry() {
    let mut symbols = HashMap::new();
    symbols.insert(
        "RepoAOnly".to_string(),
        SymbolImportance {
            symbol: "RepoAOnly".to_string(),
            score: 0.91,
            file: "src/a.rs".to_string(),
        },
    );
    let mut bridge = new_mock(symbols);

    let baseline = bridge
        .get_symbol_importance_mut()
        .expect("project A's seeded memory entry");
    assert!(baseline.contains_key("RepoAOnly"));

    bridge.set_project("project-b");
    bridge.set_project("test-project");

    let restored = bridge
        .get_symbol_importance_mut()
        .expect("switching away and back must retain project A's memory entry");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored["RepoAOnly"].file, "src/a.rs");
}

#[test]
fn explicit_project_disk_hydration_uses_that_projects_registered_root() {
    let cache_dir = tempfile::tempdir().expect("cache directory");
    let root_a = tempfile::tempdir().expect("project A root");
    let root_b = tempfile::tempdir().expect("project B root");
    let root_a_canonical = canonical(root_a.path());
    let root_b_canonical = canonical(root_b.path());
    let project_b = cbm_project_slug(&root_b_canonical);
    let db_path = cache_dir.path().join("graph-cache.db");

    let store = GraphCacheStore::open(&db_path).expect("cache store");
    seed_symbol_partition(
        &store,
        &root_b_canonical,
        &project_b,
        &symbol_payload("RepoBOnly", 0.73, "src/b.rs"),
    );

    let mut bridge = GraphBridge::try_create_with_roots(
        &CbmConfig {
            enabled: false,
            ..Default::default()
        },
        &root_a_canonical,
        std::slice::from_ref(&root_b_canonical),
    );
    bridge.attach_disk_cache(store);
    bridge.set_project(&project_b);

    let hydrated = bridge
        .get_symbol_importance_mut()
        .expect("project B must hydrate from project B's registered root");
    assert_eq!(hydrated.len(), 1);
    assert_eq!(hydrated["RepoBOnly"].file, "src/b.rs");
}

#[test]
fn active_project_invalidation_deletes_only_that_projects_disk_partition() {
    let cache_dir = tempfile::tempdir().expect("cache directory");
    let root_a = tempfile::tempdir().expect("project A root");
    let root_b = tempfile::tempdir().expect("project B root");
    let root_a_canonical = canonical(root_a.path());
    let root_b_canonical = canonical(root_b.path());
    let project_a = cbm_project_slug(&root_a_canonical);
    let project_b = cbm_project_slug(&root_b_canonical);
    let db_path = cache_dir.path().join("graph-cache.db");

    let store = GraphCacheStore::open(&db_path).expect("cache store");
    seed_symbol_partition(
        &store,
        &root_a_canonical,
        &project_a,
        &symbol_payload("RepoAOnly", 0.91, "src/a.rs"),
    );
    seed_symbol_partition(
        &store,
        &root_b_canonical,
        &project_b,
        &symbol_payload("RepoBOnly", 0.73, "src/b.rs"),
    );

    let mut bridge = GraphBridge::try_create_with_roots(
        &CbmConfig {
            enabled: false,
            ..Default::default()
        },
        &root_a_canonical,
        std::slice::from_ref(&root_b_canonical),
    );
    bridge.attach_disk_cache(store);
    bridge.set_project(&project_b);
    bridge.invalidate_cache();

    let inspector = GraphCacheStore::open(&db_path).expect("cache inspector");
    assert!(
        read_symbol_partition(&inspector, &root_a_canonical, &project_a).is_some(),
        "invalidating project B must preserve project A's disk partition"
    );
    assert!(
        read_symbol_partition(&inspector, &root_b_canonical, &project_b).is_none(),
        "invalidating project B must remove project B's disk partition"
    );
}
