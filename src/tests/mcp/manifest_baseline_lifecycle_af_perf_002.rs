//! Frozen positive lifecycle cases, including provider and invalidation boundaries.
use super::super::{Mode, identity, state};
use super::hydration;
use super::{Records, assert_inbound, call, inbound, reset, snapshot};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn csharp(root: &Path) -> Vec<(PathBuf, String)> {
    std::fs::create_dir_all(root).unwrap();
    let sources = vec![
        (
            root.join("ContextA.cs"),
            "public sealed class ContextA {}\n".to_owned(),
        ),
        (
            root.join("Consumer.cs"),
            "public sealed class Consumer { public Consumer(ContextA value) {} }\n".to_owned(),
        ),
    ];
    for (path, text) in &sources {
        std::fs::write(path, text).unwrap();
    }
    sources
}
pub(super) fn inbound_and_invalidation(records: &mut Records, base: &Path) {
    let root = base.join("inbound");
    let mut sources = csharp(&root);
    let added = root.join("Added.cs");
    if added.exists() {
        std::fs::remove_file(&added).unwrap();
    }
    let state = state(std::slice::from_ref(&root));
    let mut cold_generation = None;
    for phase in ["cold", "warm"] {
        let _mode = Mode::new(false, true);
        reset();
        let response = inbound(&state, &root);
        let work = snapshot();
        assert_eq!(super::super::DEMAND.with(std::cell::Cell::get), Some(true));
        assert_inbound(&response);
        if phase == "cold" {
            assert!(work.root_manifests > 0 && !work.proof_receipts.is_empty());
        }
        if phase == "cold" {
            cold_generation = Some(
                response["result"]["structuredContent"]["coverage"]["source_generation"].clone(),
            );
        }
        records.add(
            json!({"inbound":phase,"response":response,"identity":identity(&state,&sources)}),
            &work,
            true,
        );
    }
    let text = "public sealed class Added { public Added(ContextA value) {} }\n".to_owned();
    std::fs::write(&added, &text).unwrap();
    sources.push((added, text));
    reset();
    let refreshed = call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot":root.to_string_lossy(),"scope":"requested_root"}),
    );
    records.add(
        json!({"inbound":"refresh","response":refreshed}),
        &snapshot(),
        false,
    );
    let _mode = Mode::new(false, true);
    reset();
    let response = inbound(&state, &root);
    let work = snapshot();
    assert_inbound(&response);
    assert!(
        work.root_manifests > 0 && work.grouped_clones > 0,
        "Invalidation did not reacquire"
    );
    let generation = &response["result"]["structuredContent"]["coverage"]["source_generation"];
    assert!(!generation.is_null());
    assert_ne!(
        Some(generation),
        cold_generation.as_ref(),
        "Stale proof generation after source change"
    );
    for (path, source) in &sources {
        let owner = state.semantic_owner_path(&path.to_string_lossy());
        let expected = state.cache_read().compute_hash(source.as_bytes());
        let entries: Vec<_> = work
            .root_receipts
            .iter()
            .flat_map(|receipt| receipt["entries"].as_array().unwrap())
            .filter(|entry| entry["owner"].as_str() == Some(owner.as_str()))
            .collect();
        assert!(
            !entries.is_empty(),
            "Source absent from reacquired manifest"
        );
        for entry in entries {
            assert_eq!(entry["hash"].as_str(), Some(expected.as_str()));
        }
    }
    records.add(
        json!({"inbound":"after_change","response":response,"identity":identity(&state,&sources)}),
        &work,
        true,
    );
}
struct ProviderReset;
impl Drop for ProviderReset {
    fn drop(&mut self) {
        hydration::clear_test_project_search_results();
    }
}
pub(super) fn provider_cases(records: &mut Records, base: &Path) {
    for failed in [false, true] {
        let primary = base.join(format!("provider-{failed}/primary"));
        let other = base.join(format!("provider-{failed}/other"));
        let mut sources = csharp(&primary);
        sources.extend(csharp(&other));
        let roots = vec![primary.clone(), other.clone()];
        let state = state(&roots);
        let cfg = crate::cbm::config::CbmConfig {
            enabled: false,
            ..Default::default()
        };
        *state.graph_bridge_lock() = Some(crate::cbm::GraphBridge::try_create_with_roots(
            &cfg,
            &primary,
            std::slice::from_ref(&other),
        ));
        let a = crate::cbm::bridge::cbm_project_slug(&primary.canonicalize().unwrap());
        let b = crate::cbm::bridge::cbm_project_slug(&other.canonicalize().unwrap());
        let result = if failed {
            Err("frozen provider search failure".to_owned())
        } else {
            Ok(Vec::new())
        };
        hydration::set_test_project_search_results(HashMap::from([(a.clone(), result.clone())]));
        let _provider = ProviderReset;
        hydration::set_test_project_readiness(HashMap::from([(
            b,
            if failed {
                hydration::TestProjectReadiness::Failed
            } else {
                hydration::TestProjectReadiness::StillIndexing
            },
        )]));
        hydration::set_test_inbound_project_search_results(HashMap::from([(a, result)]));
        for proof in [false, true] {
            let _mode = Mode::new(false, true);
            reset();
            let response = if proof {
                inbound(&state, &primary)
            } else {
                super::super::dispatch(&state, &primary, "AbsentProvider", false)
            };
            let work = snapshot();
            assert_eq!(super::super::DEMAND.with(std::cell::Cell::get), Some(proof));
            assert!(
                !work.combined_receipts.is_empty(),
                "Provider path did not fall back to actual filesystem"
            );
            assert_eq!(
                hydration::searched_projects().is_empty(),
                !failed,
                "Busy provider must skip search; failed ready provider must attempt search"
            );
            if proof {
                assert_inbound(&response);
                assert!(work.grouped_clones > 0 && !work.proof_receipts.is_empty());
            }
            records.add(json!({"provider_failed":failed,"proof":proof,"response":response,
                "provider_calls":format!("{:?}",hydration::discovery_calls()),"identity":identity(&state,&sources)}),&work,proof);
        }
    }
}
pub(super) fn durable_case(records: &mut Records, base: &Path) {
    let root = base.join("durable");
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("wanted.rs");
    let source = "struct Wanted;\n";
    std::fs::write(&file, source).unwrap();
    for name in ["state.db", "state.db-wal", "state.db-shm"] {
        let p = root.join(name);
        if p.exists() {
            std::fs::remove_file(p).unwrap();
        }
    }
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.proxy.port = 0;
    cfg.persistence.enabled = true;
    cfg.persistence.auto_save = true;
    cfg.persistence.db_path = root.join("state.db").to_string_lossy().into_owned();
    cfg.additional_roots = vec![root.to_string_lossy().into_owned()];
    let state = crate::mcp::McpState::new(cfg);
    *state.graph_bridge_lock() = None;
    let compressed = call(
        &state,
        "compress_code_context",
        json!({"filePath":file.to_string_lossy(),"workspaceRoot":root.to_string_lossy(),"fidelity":"edit"}),
    );
    let durable = || {
        let guard = state.persistence_store_lock();
        let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
        format!(
            "{:?}",
            sqlite
                .load_durable_context(&file.to_string_lossy(), None)
                .unwrap()
                .expect("Frozen durable context")
        )
    };
    let before = durable();
    let _mode = Mode::new(false, true);
    reset();
    let response = super::super::dispatch(&state, &root, "Wanted", true);
    let work = snapshot();
    assert_eq!(
        response["result"]["structuredContent"]["coverage"]["source_complete"],
        true
    );
    assert!(work.root_manifests > 0 && !work.proof_receipts.is_empty());
    records.add(json!({"durable":true,"compressed":compressed,"response":response,"before":before,"after":durable()}),&work,true);
}
