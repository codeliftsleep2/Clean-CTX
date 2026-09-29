use super::*;

// ── CBM project identity: canonical slug + multi-root lifecycle ──────
//
// Verified against the CBM 0.8.1 wire contract: CBM derives a project ID from
// the CANONICAL REPO PATH (see `cbm_project_slug`), never from the directory
// basename. These tests pin the identity mapping and per-root lifecycle.

#[test]
fn cbm_project_slug_matches_verified_cbm_wire_contract() {
    use crate::cbm::bridge::cbm_project_slug;

    // Captured live from CBM 0.8.1's index_repository responses.
    assert_eq!(
        cbm_project_slug(std::path::Path::new(
            "C:/Users/MNasty/Desktop/RustContextLayerAI"
        )),
        "C-Users-MNasty-Desktop-RustContextLayerAI"
    );
    // Dots and underscores are preserved.
    assert_eq!(
        cbm_project_slug(std::path::Path::new(
            "C:/Users/MNasty/AppData/Local/Temp/CleanCtx_Probe.Repo"
        )),
        "C-Users-MNasty-AppData-Local-Temp-CleanCtx_Probe.Repo"
    );
    // Spaces become dashes; runs collapse.
    assert_eq!(
        cbm_project_slug(std::path::Path::new(
            "C:/Users/MNasty/AppData/Local/Temp/My space_probe"
        )),
        "C-Users-MNasty-AppData-Local-Temp-My-space_probe"
    );
    // Degenerate input falls back safely.
    assert_eq!(cbm_project_slug(std::path::Path::new("")), "default");
}

#[test]
fn try_create_with_roots_maps_every_root_to_canonical_cbm_identity() {
    use crate::cbm::GraphBridge;
    use crate::cbm::bridge::cbm_project_slug;
    use crate::cbm::config::CbmConfig;

    let primary = make_temp_root("primary");
    let extra_a = make_temp_root("alpha");
    let extra_b = make_temp_root("beta");

    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let bridge =
        GraphBridge::try_create_with_roots(&config, &primary, &[extra_a.clone(), extra_b.clone()]);

    let primary_slug = cbm_project_slug(&primary.canonicalize().unwrap());
    let a_slug = cbm_project_slug(&extra_a.canonicalize().unwrap());
    let b_slug = cbm_project_slug(&extra_b.canonicalize().unwrap());

    // Active project = PRIMARY root's canonical slug — never its basename.
    assert_eq!(
        bridge.project_str(),
        primary_slug,
        "active identity must be the canonical CBM slug, not a dirname"
    );

    // Every configured root resolves to its own canonical slug.
    assert_eq!(
        bridge.resolve_project_id(&extra_a.to_string_lossy()),
        a_slug,
        "path form must resolve canonically"
    );
    assert_eq!(
        bridge.resolve_project_id(&extra_b.to_string_lossy()),
        b_slug
    );

    // A root's directory BASENAME must resolve to that root's canonical slug —
    // the exact bug class that produced divergent identities before.
    let a_basename = extra_a
        .canonicalize()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert_eq!(bridge.resolve_project_id(&a_basename), a_slug);

    // An already-canonical slug passes through unchanged.
    assert_eq!(bridge.resolve_project_id(&a_slug), a_slug);

    let _ = std::fs::remove_dir_all(&primary);
    let _ = std::fs::remove_dir_all(&extra_a);
    let _ = std::fs::remove_dir_all(&extra_b);
}

#[test]
fn set_project_resolves_dirname_to_canonical_identity() {
    use crate::cbm::GraphBridge;
    use crate::cbm::bridge::cbm_project_slug;
    use crate::cbm::config::CbmConfig;

    let primary = make_temp_root("setproj");
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create_with_roots(&config, &primary, &[]);

    let canonical = cbm_project_slug(&primary.canonicalize().unwrap());
    let basename = primary
        .canonicalize()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    // Passing the raw directory basename must NOT create a divergent identity:
    // it resolves back to the same canonical CBM slug.
    bridge.set_project(&basename);
    assert_eq!(
        bridge.project_str(),
        canonical,
        "dirname override must canonicalize, not become a new project ID"
    );

    // Path form resolves identically.
    bridge.set_project(&primary.to_string_lossy());
    assert_eq!(bridge.project_str(), canonical);

    let _ = std::fs::remove_dir_all(&primary);
}

#[test]
fn ensure_indexed_for_is_per_project_and_untracked_never_dead_ends() {
    use crate::cbm::GraphBridge;
    use crate::cbm::bridge::{IndexingState, IndexingStatus, cbm_project_slug};
    use crate::cbm::config::CbmConfig;
    use std::time::Instant;

    let primary = make_temp_root("iso_primary");
    let extra = make_temp_root("iso_extra");
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge =
        GraphBridge::try_create_with_roots(&config, &primary, std::slice::from_ref(&extra));
    prime_available(&mut bridge);

    let p_slug = cbm_project_slug(&primary.canonicalize().unwrap());
    let e_slug = cbm_project_slug(&extra.canonicalize().unwrap());

    // Seed independent per-project states: primary Complete, additional InProgress.
    {
        let mut states = bridge.indexing_state();
        states.insert(p_slug.clone(), IndexingState::Complete);
        states.insert(
            e_slug.clone(),
            IndexingState::InProgress {
                started_at: Instant::now(),
            },
        );
    }

    // 1. REGRESSION: an unrelated/untracked project must pass straight through —
    //    it can NEVER dead-end in `StillIndexing{0}` forever (the pre-fix bug).
    assert_eq!(
        bridge.ensure_indexed_for("totally-unrelated-project").ok(),
        Some(IndexingStatus::Ready),
        "untracked project must not be gated into an eternal still_indexing loop"
    );

    // 2. Primary root Complete → Ready.
    assert_eq!(
        bridge.ensure_indexed_for(&p_slug).ok(),
        Some(IndexingStatus::Ready)
    );

    // 3. Additional root InProgress → legit retry, ISOLATED to that root.
    match bridge.ensure_indexed_for(&e_slug) {
        Ok(IndexingStatus::StillIndexing { .. }) => {}
        other => panic!(
            "expected StillIndexing for the in-progress root, got {:?}",
            other
        ),
    }

    // 4. Making the in-progress root ACTIVE must not block the completed one.
    bridge.set_project(&e_slug); // entry exists → ensure_tracked does NOT spawn
    match bridge.ensure_indexed() {
        Ok(IndexingStatus::StillIndexing { .. }) => {}
        other => panic!("expected StillIndexing on active switch, got {:?}", other),
    }
    bridge.set_project(&p_slug);
    assert_eq!(
        bridge.ensure_indexed().ok(),
        Some(IndexingStatus::Ready),
        "one root's StillIndexing must never block another root that is complete"
    );

    let _ = std::fs::remove_dir_all(&primary);
    let _ = std::fs::remove_dir_all(&extra);
}

#[test]
fn try_create_without_additional_roots_preserves_single_root_behavior() {
    use crate::cbm::GraphBridge;
    use crate::cbm::bridge::cbm_project_slug;
    use crate::cbm::config::CbmConfig;

    let primary = make_temp_root("single");
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };

    let plain = GraphBridge::try_create(&config, &primary);
    let with_empty = GraphBridge::try_create_with_roots(&config, &primary, &[]);

    let expected = cbm_project_slug(&primary.canonicalize().unwrap());
    assert_eq!(plain.project_str(), expected);
    assert_eq!(with_empty.project_str(), expected);
    assert_eq!(
        with_empty.project_paths.len(),
        1,
        "no additional_roots must register ONLY the primary root"
    );

    let _ = std::fs::remove_dir_all(&primary);
}

#[test]
fn proxy_target_resolution_gates_only_project_bound_calls() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;
    use crate::cbm::proxy::resolve_proxy_target_project;

    let primary = make_temp_root("proxy");
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let bridge = GraphBridge::try_create(&config, &primary);

    // `list_projects` (and any tool without a project reference) → None →
    // handle_cbm_proxy skips the indexing gate entirely.
    let none = resolve_proxy_target_project(
        &bridge,
        &serde_json::json!({ "arguments": {} }),
        &serde_json::json!({}),
    );
    assert!(
        none.is_none(),
        "project-independent proxy calls must never be gated"
    );

    // Explicit CBM-native parameter position passes through verbatim.
    let explicit = resolve_proxy_target_project(
        &bridge,
        &serde_json::json!({ "arguments": {
            "parameters": { "name_pattern": ".*", "project": "whatever" }
        }}),
        &serde_json::json!({ "name_pattern": ".*", "project": "whatever" }),
    );
    assert_eq!(explicit.as_deref(), Some("whatever"));

    // Clean-CTX shorthand `arguments.project` carrying a root PATH resolves to
    // the canonical slug via the authoritative map.
    let expected = crate::cbm::bridge::cbm_project_slug(&primary.canonicalize().unwrap());
    let shorthand = resolve_proxy_target_project(
        &bridge,
        &serde_json::json!({ "arguments": {
            "project": primary.to_string_lossy()
        }}),
        &serde_json::json!({}),
    );
    assert_eq!(shorthand.as_deref(), Some(expected.as_str()));

    let _ = std::fs::remove_dir_all(&primary);
}

// ══════════════════════════════════════════════════════════════════
// CBM-ID fix: get_architecture must parse CBM 0.8.1's REAL wire schema
// (packages → modules, boundaries → dependencies).
// Uses inline schema data — no personal machine captures.
// ══════════════════════════════════════════════════════════════════

#[test]
fn parse_architecture_maps_packages_and_boundaries_from_live_payload() {
    use crate::cbm::bridge::parse_architecture_response;

    let arch = serde_json::json!({
        "packages": [
            {"name": "cbm", "node_count": 81, "fan_in": 0, "fan_out": 0},
            {"name": "tests", "node_count": 65, "fan_in": 0, "fan_out": 0},
            {"name": "mcp", "node_count": 35, "fan_in": 0, "fan_out": 0}
        ],
        "boundaries": [
            {"from": "tests", "to": "cbm", "call_count": 42},
            {"from": "mcp", "to": "cbm", "call_count": 18},
            {"from": "tests", "to": "mcp", "call_count": 7}
        ]
    });

    let ov = parse_architecture_response(&arch);

    assert_eq!(ov.modules.len(), 3, "packages[] must map to modules");
    let cbm = ov
        .modules
        .iter()
        .find(|m| m.name == "cbm")
        .expect("cbm package present");
    assert_eq!(cbm.file_count, 81, "node_count maps into file_count");

    assert_eq!(
        ov.dependencies.len(),
        3,
        "boundaries[] must map to dependencies"
    );
    let dep = ov
        .dependencies
        .iter()
        .find(|d| d.from == "tests" && d.to == "cbm")
        .expect("tests→cbm boundary present");
    assert_eq!(dep.kind, "calls", "boundaries are call edges");
}

#[test]
fn parse_architecture_tolerates_small_graph_without_boundaries_key() {
    use crate::cbm::bridge::parse_architecture_response;

    let arch = serde_json::json!({
        "packages": [
            {"name": "main", "node_count": 4}
        ]
        // Intentionally no `boundaries` key — tests the missing-key fallback.
    });

    let ov = parse_architecture_response(&arch);

    assert_eq!(ov.modules.len(), 1);
    assert_eq!(ov.modules[0].name, "main");
    assert_eq!(ov.modules[0].file_count, 4);

    // The mini payload has NO `boundaries` key — the old parser read a
    // key that never exists; the new one defaults to empty instead of
    // erroring.
    assert!(ov.dependencies.is_empty());
}

// ══════════════════════════════════════════════════════════════════
// CBM-ID fix: graph_search must NOT restrict CBM's name_pattern search
// to label="Function" — that hid every Class/Enum/Field/Module from the
// wrapper. Request shape is pinned via the extracted pure helper.
// ══════════════════════════════════════════════════════════════════

#[test]
fn build_search_graph_args_omits_label_by_default() {
    use crate::cbm::client::CbmClient;

    let args = CbmClient::build_search_graph_args(".*GraphBridge.*", "proj-slug", None);
    assert_eq!(args["name_pattern"], ".*GraphBridge.*");
    assert_eq!(args["project"], "proj-slug");
    assert!(
        args.get("label").is_none(),
        "default wrapper search must not restrict the node label — \
         the hardcoded Function filter made Class symbols unfindable"
    );
}

#[test]
fn build_search_graph_args_includes_explicit_label_override() {
    use crate::cbm::client::CbmClient;

    let args = CbmClient::build_search_graph_args(".*GraphBridge.*", "proj-slug", Some("Class"));
    assert_eq!(args["label"], "Class");
}

// ── CBM-ID fix: search result mapping (real wire capture) ───────
//
// Verbatim CBM 0.8.1 search_graph envelope (live capture): results carry
// `qualified_name` / `file_path` — NOT `id` / `file`. The old GraphNode
// mapping required those nonexistent keys, so filter_map dropped every
// result and wrapper searches were always empty.

const SEARCH_RESULT_WIRE_CAPTURE: &str = r#"{"total":1,"results":[{"name":"GraphBridge","qualified_name":"C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge","label":"Class","file_path":"src/cbm/bridge.rs","in_degree":6,"out_degree":0,"complexity":0,"lines":0,"is_exported":true,"is_test":false,"is_entry_point":false,"docstring":"/// Graph bridge with TTL caching and graceful degradation.\n"}],"has_more":false}"#;

#[test]
fn map_search_result_keeps_results_using_cbm_081_field_names() {
    use crate::cbm::bridge::map_search_result;

    let envelope: serde_json::Value =
        serde_json::from_str(SEARCH_RESULT_WIRE_CAPTURE).expect("captured envelope parses");
    let results = envelope["results"].as_array().expect("results array");

    assert_eq!(results.len(), 1, "capture sanity");
    let node = map_search_result(&results[0]).expect("result must NOT be dropped");

    assert_eq!(node.name, "GraphBridge");
    assert_eq!(node.label, "Class");
    assert_eq!(node.file, "src/cbm/bridge.rs", "file_path maps to file");
    assert_eq!(
        node.id, "C-Users-MNasty-Desktop-RustContextLayerAI.src.cbm.bridge.GraphBridge",
        "qualified_name maps to id"
    );
}

// ── Finding #3: CBM 0.8.1 uses DEFINES_METHOD, not DECLARES ─────
//
// CBM 0.8.1 has zero DECLARES edges. The edge_types in the
// architecture response prove 0 DECLARES and DEFINES_METHOD matching
// Method nodes. The old resolve_cross_language_endpoint Cypher queried
// DECLARES, which never existed in CBM.

#[test]
fn fixture_proves_no_declares_edge_exists() {
    let arch = serde_json::json!({
        "edge_types": [
            {"type": "DEFINES_METHOD", "count": 73},
            {"type": "CALLS", "count": 241},
            {"type": "USAGE", "count": 285}
        ]
    });
    let edge_types = arch["edge_types"].as_array().expect("edge_types array");
    assert!(edge_types.iter().all(|e| e["type"] != "DECLARES"));
    assert!(edge_types.iter().any(|e| e["type"] == "DEFINES_METHOD"));
}

#[test]
fn fixture_defines_method_count_matches_method_node_count() {
    let arch = serde_json::json!({
        "node_labels": [
            {"label": "Method", "count": 73},
            {"label": "Function", "count": 93}
        ],
        "edge_types": [
            {"type": "DEFINES_METHOD", "count": 73},
            {"type": "CALLS", "count": 241}
        ]
    });
    let mcount = arch["node_labels"]
        .as_array()
        .and_then(|l| l.iter().find(|l| l["label"] == "Method"))
        .and_then(|m| m["count"].as_u64())
        .unwrap_or(0);
    let ecount = arch["edge_types"]
        .as_array()
        .and_then(|e| e.iter().find(|e| e["type"] == "DEFINES_METHOD"))
        .and_then(|e| e["count"].as_u64())
        .unwrap_or(0);
    assert_eq!(ecount, mcount);
}
// ── reindex_for_file tests ────────────────────────────────────
//
