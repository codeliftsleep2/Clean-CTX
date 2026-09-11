// Multi-root bounded-hydration regressions for workspace_query (RED-15..18).

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestProjectSearchResult, clear_test_project_search_results, searched_projects,
    set_test_project_search_results,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

static TEST_SERIALIZE: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
    let bridge = GraphBridge::try_create_with_roots(&bridge_config, primary, additional);
    *state.graph_bridge_lock() = Some(bridge);
    state
}

fn project_slug(root: &Path) -> String {
    cbm_project_slug(&root.canonicalize().expect("test root canonicalizes"))
}

fn call_find(
    state: &crate::mcp::McpState,
    name: &str,
    workspace_root: &Path,
) -> serde_json::Map<String, serde_json::Value> {
    let (entities, count, hydration_attempted, candidates_discovered, candidates_compiled) =
        super::run_query_with_hydration(
            state,
            "find_entities",
            name,
            Some(&workspace_root.to_string_lossy()),
            |index| {
                let entities = index.find_entities_by_name(name);
                let count = entities.len();
                (serde_json::to_value(entities).unwrap_or_default(), count)
            },
        );
    serde_json::json!({
        "entities": entities,
        "count": count,
        "hydration_attempted": hydration_attempted,
        "candidates_discovered": candidates_discovered,
        "candidates_compiled": candidates_compiled,
    })
    .as_object()
    .expect("structured result")
    .clone()
}

fn configure_results(
    primary: (&str, TestProjectSearchResult),
    additional: (&str, TestProjectSearchResult),
) {
    set_test_project_search_results(HashMap::from([
        (primary.0.to_string(), primary.1),
        (additional.0.to_string(), additional.1),
    ]));
}

fn assert_both_projects_searched(primary: &str, additional: &str) {
    let searched: HashSet<_> = searched_projects().into_iter().collect();
    assert_eq!(searched.len(), 2, "each configured project searched once");
    assert!(searched.contains(primary), "primary project searched");
    assert!(searched.contains(additional), "additional project searched");
}

#[test]
fn red15_additional_root_only_discovery() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        additional.path().join("AdditionalOnly.ts"),
        "export class AdditionalOnly {}",
    )
    .unwrap();

    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (&primary_slug, Ok(Vec::new())),
        (&additional_slug, Ok(vec![node("AdditionalOnly.ts")])),
    );

    let sc = call_find(&state, "AdditionalOnly", primary.path());
    let entities = sc["entities"].as_array().expect("entities array");
    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(sc["candidates_discovered"], 1);
    assert_eq!(sc["candidates_compiled"], 1);
    assert!(
        entities.iter().any(|entity| {
            entity["entity_type"] == "Class" && entity["name"] == "AdditionalOnly"
        })
    );
    clear_test_project_search_results();
}

#[test]
fn red16_active_project_remains_unchanged_on_every_exit_path() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for (case, primary_result, additional_result) in [
        ("success", Ok(Vec::new()), Ok(vec![node("Candidate.ts")])),
        (
            "partial failure",
            Err("primary unavailable".into()),
            Ok(vec![node("Candidate.ts")]),
        ),
        ("empty", Ok(Vec::new()), Ok(Vec::new())),
        (
            "all errors",
            Err("primary unavailable".into()),
            Err("additional unavailable".into()),
        ),
    ] {
        let primary = tempfile::TempDir::new().unwrap();
        let additional = tempfile::TempDir::new().unwrap();
        std::fs::write(
            additional.path().join("Candidate.ts"),
            "export class Candidate {}",
        )
        .unwrap();
        let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
        let primary_slug = project_slug(primary.path());
        let additional_slug = project_slug(additional.path());
        configure_results(
            (&primary_slug, primary_result),
            (&additional_slug, additional_result),
        );

        let _ = call_find(&state, "Candidate", primary.path());
        let active_after = state
            .graph_bridge_lock()
            .as_ref()
            .expect("bridge")
            .project_str();
        assert_eq!(active_after, primary_slug, "active project changed: {case}");
        assert_both_projects_searched(&primary_slug, &additional_slug);
        clear_test_project_search_results();
    }
}

#[test]
fn red17_global_cap_applies_after_merged_dedup_and_index_exclusion() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);

    let indexed = primary.path().join("00Indexed.ts");
    std::fs::write(&indexed, "export class AlreadyIndexed {}").unwrap();
    let indexed_str = indexed.to_string_lossy().into_owned();
    crate::mcp::tool_handlers::hydration::compile_candidate(
        &state,
        &indexed_str,
        Some(&primary.path().to_string_lossy()),
    );
    let before: HashSet<_> = state
        .workspace_index_read()
        .file_map()
        .keys()
        .cloned()
        .collect();

    let primary_names = ["05.ts", "01.ts", "03.ts"];
    let additional_names = ["06.ts", "02.ts", "04.ts", "07.ts"];
    for name in primary_names {
        std::fs::write(
            primary.path().join(name),
            format!("export class C{} {{}}", &name[..2]),
        )
        .unwrap();
    }
    for name in additional_names {
        std::fs::write(
            additional.path().join(name),
            format!("export class C{} {{}}", &name[..2]),
        )
        .unwrap();
    }

    let duplicate_absolute = additional
        .path()
        .join("02.ts")
        .to_string_lossy()
        .into_owned();
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (
            &primary_slug,
            Ok(vec![
                node("05.ts"),
                node(indexed_str.clone()),
                node(duplicate_absolute.clone()),
                node("01.ts"),
                node("03.ts"),
            ]),
        ),
        (
            &additional_slug,
            Ok(vec![
                node("06.ts"),
                node("02.ts"),
                node("04.ts"),
                node("07.ts"),
            ]),
        ),
    );

    let sc = call_find(&state, "C", primary.path());
    let after: HashSet<_> = state
        .workspace_index_read()
        .file_map()
        .keys()
        .cloned()
        .collect();
    let added: HashSet<_> = after.difference(&before).cloned().collect();

    let mut expected: Vec<_> = primary_names
        .iter()
        .map(|name| primary.path().join(name))
        .chain(
            additional_names
                .iter()
                .map(|name| additional.path().join(name)),
        )
        .map(|path| crate::dictionary::path::canonical_identity_key(&path.to_string_lossy()))
        .collect();
    expected.sort();
    expected.dedup();
    expected.truncate(5);

    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(sc["candidates_discovered"], 9);
    assert_eq!(sc["candidates_compiled"], 5);
    assert_eq!(added.len(), 5, "cap must be global, not per project");
    assert_eq!(added, expected.into_iter().collect());
    clear_test_project_search_results();
}

#[test]
fn red18_per_project_failure_degrades_locally() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        additional.path().join("Survivor.ts"),
        "export class Survivor {}",
    )
    .unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (&primary_slug, Err("primary unavailable".into())),
        (&additional_slug, Ok(vec![node("Survivor.ts")])),
    );

    let sc = call_find(&state, "Survivor", primary.path());
    let entities = sc["entities"].as_array().expect("entities array");
    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(sc["candidates_discovered"], 1);
    assert_eq!(sc["candidates_compiled"], 1);
    assert!(entities.iter().any(|entity| entity["name"] == "Survivor"));
    clear_test_project_search_results();
}
