// Semantic-completeness hydration regressions (RED-C1..C6).

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestProjectSearchResult, clear_test_project_search_results,
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
                "reverse_edges" => serde_json::to_value(
                    index.reverse_edges_by_identity("angular", "Service", name),
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
    })
    .as_object()
    .expect("structured result")
    .clone()
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

fn consumer_names(sc: &serde_json::Map<String, serde_json::Value>) -> HashSet<&str> {
    sc["result"]
        .as_array()
        .expect("edges")
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .filter_map(|edge| edge["subject"]["name"].as_str())
        .collect()
}

#[test]
fn red_c1_reverse_edges_returns_all_inbound_consumers() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    let files: Vec<_> = (0..9).map(|i| format!("Consumer{i}.ts")).collect();
    for (i, file) in files.iter().enumerate() {
        write_consumer(root.path(), file, &format!("Consumer{i}"));
    }
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(Vec::new()))],
        vec![(slug, Ok(files.iter().map(node).collect()))],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["candidates_discovered"], 9);
    assert_eq!(sc["candidates_compiled"], 9);
    assert_eq!(sc["count"], 9);
    assert_eq!(consumer_names(&sc).len(), 9);
    clear_test_project_search_results();
}

#[test]
fn red_c2_find_entities_is_not_silently_truncated() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    let files: Vec<_> = (0..7).map(|i| format!("Shared{i}.ts")).collect();
    for file in &files {
        std::fs::write(
            root.path().join(file),
            "export class SharedDeclaration {}\n",
        )
        .unwrap();
    }
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(files.iter().map(node).collect()))],
        vec![(slug, Ok(Vec::new()))],
    );

    let sc = call_query(&state, "find_entities", "SharedDeclaration", root.path());

    assert_eq!(sc["candidates_discovered"], 7);
    assert_eq!(sc["candidates_compiled"], 7);
    assert_eq!(sc["count"], 7);
    clear_test_project_search_results();
}

#[test]
fn red_c3_multi_project_reverse_edges_is_complete() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let primary_files: Vec<_> = (0..4).map(|i| format!("Primary{i}.ts")).collect();
    let additional_files: Vec<_> = (0..4).map(|i| format!("Additional{i}.ts")).collect();
    for (i, file) in primary_files.iter().enumerate() {
        write_consumer(primary.path(), file, &format!("Primary{i}"));
    }
    for (i, file) in additional_files.iter().enumerate() {
        write_consumer(additional.path(), file, &format!("Additional{i}"));
    }
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        vec![
            (primary_slug.clone(), Ok(Vec::new())),
            (additional_slug.clone(), Ok(Vec::new())),
        ],
        vec![
            (primary_slug, Ok(primary_files.iter().map(node).collect())),
            (
                additional_slug,
                Ok(additional_files.iter().map(node).collect()),
            ),
        ],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", primary.path());

    assert_eq!(sc["candidates_discovered"], 8);
    assert_eq!(sc["candidates_compiled"], 8);
    assert_eq!(sc["count"], 8);
    assert_eq!(consumer_names(&sc).len(), 8);
    clear_test_project_search_results();
}

#[test]
fn red_c4_duplicate_candidates_compile_once() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    write_consumer(root.path(), "OnlyConsumer.ts", "OnlyConsumer");
    let absolute = root
        .path()
        .join("OnlyConsumer.ts")
        .to_string_lossy()
        .into_owned();
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(Vec::new()))],
        vec![(
            slug,
            Ok(vec![
                node("OnlyConsumer.ts"),
                node(absolute),
                node("OnlyConsumer.ts"),
            ]),
        )],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["candidates_discovered"], 3);
    assert_eq!(sc["candidates_compiled"], 1);
    assert_eq!(sc["count"], 1);
    clear_test_project_search_results();
}

#[test]
fn red_c5_already_indexed_candidates_are_excluded_without_losing_results() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    let files: Vec<_> = (0..7).map(|i| format!("Consumer{i}.ts")).collect();
    for (i, file) in files.iter().enumerate() {
        write_consumer(root.path(), file, &format!("Consumer{i}"));
    }
    let state = state_with_roots(root.path(), &[]);
    let indexed = root.path().join(&files[0]).to_string_lossy().into_owned();
    assert!(crate::mcp::tool_handlers::hydration::compile_candidate(
        &state,
        &indexed,
        Some(&root.path().to_string_lossy()),
    ));
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(Vec::new()))],
        vec![(slug, Ok(files.iter().map(node).collect()))],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["candidates_discovered"], 7);
    assert_eq!(sc["candidates_compiled"], 6);
    assert_eq!(sc["count"], 7);
    clear_test_project_search_results();
}

#[test]
fn red_c6_candidate_count_is_not_semantic_result_count() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    write_consumer(root.path(), "RealConsumer.ts", "RealConsumer");
    let mut files = vec!["RealConsumer.ts".to_string()];
    for i in 0..7 {
        let file = format!("GraphOnly{i}.ts");
        std::fs::write(
            root.path().join(&file),
            format!("export class GraphOnly{i} {{}}\n"),
        )
        .unwrap();
        files.push(file);
    }
    let state = state_with_roots(root.path(), &[]);
    let slug = project_slug(root.path());
    configure_results(
        vec![(slug.clone(), Ok(Vec::new()))],
        vec![(slug, Ok(files.iter().map(node).collect()))],
    );

    let sc = call_query(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(sc["candidates_discovered"], 8);
    assert_eq!(sc["candidates_compiled"], 8);
    assert_eq!(sc["count"], 1);
    assert_eq!(consumer_names(&sc), HashSet::from(["RealConsumer"]));
    clear_test_project_search_results();
}
