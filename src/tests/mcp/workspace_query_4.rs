// Query-direction-aware hydration regressions (RED-23..28).

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestDiscoveryKind, TestProjectSearchResult, clear_test_project_search_results, discovery_calls,
    set_test_inbound_project_search_results, set_test_project_search_results,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

fn node(file: impl Into<String>) -> GraphNode {
    GraphNode {
        id: "candidate".into(),
        label: "Class".into(),
        name: "candidate".into(),
        file: file.into(),
        properties: HashMap::new(),
    }
}

fn state_with_roots(primary: &Path, additional: &[PathBuf]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        primary,
        additional,
    ));
    state
}

fn project_slug(root: &Path) -> String {
    cbm_project_slug(&root.canonicalize().expect("test root canonicalizes"))
}

fn configure_results(
    declaration: Vec<(String, TestProjectSearchResult)>,
    inbound: Vec<(String, TestProjectSearchResult)>,
) {
    set_test_project_search_results(declaration.into_iter().collect());
    set_test_inbound_project_search_results(inbound.into_iter().collect());
}

fn call_query(
    state: &crate::mcp::McpState,
    query_type: &str,
    name: &str,
    root: &Path,
) -> serde_json::Map<String, serde_json::Value> {
    let (result, count, attempted, hydration) = super::run_query_with_hydration(
        state,
        query_type,
        name,
        Some(&root.to_string_lossy()),
        |index| {
            let values = match query_type {
                "find_entities" => serde_json::to_value(index.find_entities_by_name(name)),
                "forward_edges" => serde_json::to_value(
                    index.forward_edges_by_identity("angular", "Service", name),
                ),
                "reverse_edges" => serde_json::to_value(
                    index.reverse_edges_by_identity("angular", "Service", name),
                ),
                "transitive_dependencies" => serde_json::to_value(
                    index.transitive_dependencies("angular", "Service", name, 1),
                ),
                other => panic!("unsupported test query: {other}"),
            }
            .unwrap_or_default();
            let count = values.as_array().map_or(0, Vec::len);
            (values, count)
        },
    );
    serde_json::json!({
        "result": result,
        "count": count,
        "hydration_attempted": attempted,
        "candidates_discovered": hydration.candidates_discovered,
        "candidates_compiled": hydration.candidates_compiled,
        "project_coverage": hydration.project_coverage,
    })
    .as_object()
    .expect("structured result")
    .clone()
}

fn write_service(root: &Path) {
    std::fs::write(
        root.join("ServiceA.ts"),
        "import { Injectable } from '@angular/core';\n@Injectable()\nexport class ServiceA {}\n",
    )
    .unwrap();
}

fn write_consumer(root: &Path, file: &str, class_name: &str) {
    std::fs::write(
        root.join(file),
        format!(
            "import {{ Component }} from '@angular/core';\nimport {{ ServiceA }} from './ServiceA';\n@Component({{ selector: 'x-{class_name}' }})\nexport class {class_name} {{\n  constructor(private service: ServiceA) {{}}\n}}\n"
        ),
    )
    .unwrap();
}

fn assert_injecting_consumers(sc: &serde_json::Map<String, serde_json::Value>, expected: &[&str]) {
    let subjects: HashSet<_> = sc["result"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .filter_map(|edge| edge["subject"]["name"].as_str())
        .collect();
    assert_eq!(subjects, expected.iter().copied().collect());
}

#[test]
fn red23_reverse_edges_discovers_consumer_files_in_additional_root() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    write_service(additional.path());
    write_consumer(additional.path(), "ComponentA.ts", "ComponentA");
    write_consumer(additional.path(), "ComponentB.ts", "ComponentB");
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        vec![
            (primary_slug.clone(), Ok(Vec::new())),
            (additional_slug.clone(), Ok(vec![node("ServiceA.ts")])),
        ],
        vec![
            (primary_slug, Ok(Vec::new())),
            (
                additional_slug,
                Ok(vec![node("ComponentA.ts"), node("ComponentB.ts")]),
            ),
        ],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", primary.path());

    assert_eq!(sc["candidates_discovered"], 2);
    assert_eq!(sc["candidates_compiled"], 2);
    assert_injecting_consumers(&sc, &["ComponentA", "ComponentB"]);
    clear_test_project_search_results();
}

#[test]
fn red24_compiled_declaration_is_not_sufficient_for_reverse_edges() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    write_consumer(root.path(), "OnlyConsumer.ts", "OnlyConsumer");
    let state = state_with_roots(root.path(), &[]);
    let service_path = root
        .path()
        .join("ServiceA.ts")
        .to_string_lossy()
        .into_owned();
    assert!(crate::mcp::tool_handlers::hydration::compile_candidate(
        &state,
        &service_path,
        Some(&root.path().to_string_lossy()),
    ));
    assert!(
        state
            .workspace_index_read()
            .reverse_edges_by_identity("angular", "Service", "ServiceA")
            .is_empty()
    );
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug, Ok(vec![node("OnlyConsumer.ts")]))],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["candidates_compiled"], 1);
    assert_injecting_consumers(&sc, &["OnlyConsumer"]);
    clear_test_project_search_results();
}

#[test]
fn red25_declaration_oriented_queries_keep_name_discovery() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for query_type in ["find_entities", "forward_edges", "transitive_dependencies"] {
        let root = tempfile::TempDir::new().unwrap();
        let state = state_with_roots(root.path(), &[]);
        let slug = project_slug(root.path());
        configure_results(
            vec![(slug.clone(), Ok(Vec::new()))],
            vec![(slug.clone(), Err("inbound discovery must not run".into()))],
        );

        let _ = call_query(&state, query_type, "ServiceA", root.path());

        assert_eq!(
            discovery_calls(),
            vec![(slug, TestDiscoveryKind::Declaration)],
            "wrong discovery strategy for {query_type}"
        );
        clear_test_project_search_results();
    }
}

#[test]
fn red26_zero_inbound_references_is_a_successful_empty_search() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(vec![node("ServiceA.ts")]))],
        vec![(slug.clone(), Ok(Vec::new()))],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["count"], 0);
    assert_eq!(sc["candidates_discovered"], 0);
    assert_eq!(sc["candidates_compiled"], 0);
    assert_eq!(sc["project_coverage"][0]["status"], "searched");
    assert_eq!(
        discovery_calls(),
        vec![(slug, TestDiscoveryKind::InboundReference)]
    );
    clear_test_project_search_results();
}

#[test]
fn red27_multi_project_inbound_discovery_is_exhaustive() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    let primary_files = ["07.ts", "01.ts", "05.ts"];
    let additional_files = ["06.ts", "02.ts", "04.ts", "03.ts"];
    for (root, files) in [
        (primary.path(), primary_files.as_slice()),
        (additional.path(), additional_files.as_slice()),
    ] {
        for file in files {
            std::fs::write(
                root.join(file),
                format!("export class C{} {{}}", &file[..2]),
            )
            .unwrap();
        }
    }
    configure_results(
        vec![
            (primary_slug.clone(), Ok(Vec::new())),
            (additional_slug.clone(), Ok(Vec::new())),
        ],
        vec![
            (
                primary_slug.clone(),
                Ok(primary_files.iter().map(|file| node(*file)).collect()),
            ),
            (
                additional_slug.clone(),
                Ok(additional_files.iter().map(|file| node(*file)).collect()),
            ),
        ],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", primary.path());

    assert_eq!(sc["candidates_discovered"], 7);
    assert_eq!(sc["candidates_compiled"], 7);
    let indexed: HashSet<_> = state
        .workspace_index_read()
        .file_map()
        .keys()
        .cloned()
        .collect();
    let mut expected: Vec<_> = primary_files
        .iter()
        .map(|file| primary.path().join(file))
        .chain(
            additional_files
                .iter()
                .map(|file| additional.path().join(file)),
        )
        .map(|path| crate::dictionary::path::canonical_identity_key(&path.to_string_lossy()))
        .collect();
    expected.sort();
    assert_eq!(indexed, expected.into_iter().collect());
    let calls: HashSet<_> = discovery_calls().into_iter().collect();
    assert_eq!(
        calls,
        HashSet::from([
            (primary_slug, TestDiscoveryKind::InboundReference),
            (additional_slug, TestDiscoveryKind::InboundReference),
        ])
    );
    clear_test_project_search_results();
}

#[test]
fn red28_cbm_relationship_cardinality_never_becomes_semantic_count() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    write_consumer(root.path(), "RealConsumer.ts", "RealConsumer");
    std::fs::write(
        root.path().join("GraphOnlyA.ts"),
        "export class GraphOnlyA {}",
    )
    .unwrap();
    std::fs::write(
        root.path().join("GraphOnlyB.ts"),
        "export class GraphOnlyB {}",
    )
    .unwrap();
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(Vec::new()))],
        vec![(
            slug,
            Ok(["RealConsumer.ts", "GraphOnlyA.ts", "GraphOnlyB.ts"]
                .into_iter()
                .map(node)
                .collect()),
        )],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["candidates_discovered"], 3);
    assert_eq!(sc["count"], 1, "only Clean-CTX semantic edges count");
    assert_injecting_consumers(&sc, &["RealConsumer"]);
    clear_test_project_search_results();
}
