// Multi-root hydration regressions for workspace_query (RED-15..22).

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestProjectReadiness, TestProjectSearchResult, clear_test_project_search_results,
    searched_projects, set_test_project_readiness, set_test_project_search_results,
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
    let (entities, count, hydration) = super::run_query_with_hydration(
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
    // The response projection itself, so these regressions assert the contract
    // the handler actually produces and cannot drift from it.
    let mut structured = serde_json::json!({ "entities": entities, "count": count })
        .as_object()
        .cloned()
        .expect("structured result");
    if let Some(discovery) = super::discovery_field(&hydration) {
        structured.insert("discovery".to_string(), discovery);
    }
    structured
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

fn configure_readiness(entries: &[(&str, TestProjectReadiness)]) {
    set_test_project_readiness(
        entries
            .iter()
            .map(|(project, readiness)| ((*project).to_string(), *readiness))
            .collect(),
    );
}

/// The optional `discovery` diagnostic of a response, when one was emitted.
fn discovery(sc: &serde_json::Map<String, serde_json::Value>) -> Option<&serde_json::Value> {
    sc.get("discovery")
}

/// One reported exceptional project entry, looked up by project identity.
fn coverage_entry<'a>(
    sc: &'a serde_json::Map<String, serde_json::Value>,
    project: &str,
) -> &'a serde_json::Value {
    discovery(sc).unwrap_or_else(|| panic!("expected a discovery diagnostic: {sc:?}"))["projects"]
        .as_array()
        .expect("reported project entries")
        .iter()
        .find(|entry| entry["project"] == project)
        .expect("project coverage entry")
}

/// Whether the response reports this project as an exceptional entry.
fn reports_project(sc: &serde_json::Map<String, serde_json::Value>, project: &str) -> bool {
    discovery(sc)
        .and_then(|discovery| discovery["projects"].as_array())
        .is_some_and(|entries| entries.iter().any(|entry| entry["project"] == project))
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
    assert_eq!(sc["discovery"]["discovered"], 1);
    assert_eq!(sc["discovery"]["compiled"], 1);
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
fn red17_merged_candidates_are_exhaustive_after_dedup_and_index_exclusion() {
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

    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(sc["discovery"]["discovered"], 9);
    assert_eq!(sc["discovery"]["compiled"], 7);
    assert_eq!(added.len(), 7, "all unique unindexed candidates compile");
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
    assert_eq!(sc["discovery"]["discovered"], 1);
    assert_eq!(sc["discovery"]["compiled"], 1);
    assert!(entities.iter().any(|entity| entity["name"] == "Survivor"));
    clear_test_project_search_results();
}

#[test]
fn additional_root_registration_is_reached_by_hydration_discovery() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (&primary_slug, Ok(Vec::new())),
        (&additional_slug, Ok(Vec::new())),
    );

    let sc = call_find(&state, "Absent", primary.path());

    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert!(
        discovery(&sc).is_none(),
        "both projects were searched while ready and nothing was discovered, so there is no \
         diagnostic to report: {sc:?}"
    );
    assert!(!reports_project(&sc, &primary_slug));
    assert!(!reports_project(&sc, &additional_slug));
    clear_test_project_search_results();
}

#[test]
fn red19_still_indexing_project_with_queryable_graph_is_searched() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        additional.path().join("Persisted.ts"),
        "export class Persisted {}",
    )
    .unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (&primary_slug, Ok(Vec::new())),
        (&additional_slug, Ok(vec![node("Persisted.ts")])),
    );
    configure_readiness(&[(&additional_slug, TestProjectReadiness::StillIndexing)]);

    let sc = call_find(&state, "Persisted", primary.path());

    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(sc["discovery"]["discovered"], 1);
    assert_eq!(sc["discovery"]["compiled"], 1);
    assert_eq!(sc["count"], 1);
    assert_eq!(coverage_entry(&sc, &additional_slug)["status"], "searched");
    assert_eq!(
        coverage_entry(&sc, &additional_slug)["readiness"],
        "still_indexing"
    );
    clear_test_project_search_results();
}

#[test]
fn red20_failed_readiness_with_queryable_graph_is_searched() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        additional.path().join("PersistedAfterFailure.ts"),
        "export class PersistedAfterFailure {}",
    )
    .unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (&primary_slug, Ok(Vec::new())),
        (&additional_slug, Ok(vec![node("PersistedAfterFailure.ts")])),
    );
    configure_readiness(&[(&additional_slug, TestProjectReadiness::Failed)]);

    let sc = call_find(&state, "PersistedAfterFailure", primary.path());

    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(sc["discovery"]["discovered"], 1);
    assert_eq!(sc["discovery"]["compiled"], 1);
    assert_eq!(sc["count"], 1);
    assert_eq!(coverage_entry(&sc, &additional_slug)["status"], "searched");
    assert_eq!(coverage_entry(&sc, &additional_slug)["readiness"], "failed");
    clear_test_project_search_results();
}

#[test]
fn red21_search_failure_is_local_and_other_project_still_hydrates() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        additional.path().join("SurvivesSearchFailure.ts"),
        "export class SurvivesSearchFailure {}",
    )
    .unwrap();
    let state = state_with_roots(primary.path(), &[additional.path().to_path_buf()]);
    let primary_slug = project_slug(primary.path());
    let additional_slug = project_slug(additional.path());
    configure_results(
        (&primary_slug, Err("search failed".into())),
        (&additional_slug, Ok(vec![node("SurvivesSearchFailure.ts")])),
    );

    let sc = call_find(&state, "SurvivesSearchFailure", primary.path());

    assert_both_projects_searched(&primary_slug, &additional_slug);
    assert_eq!(
        coverage_entry(&sc, &primary_slug)["status"],
        "search_failed"
    );
    assert!(
        coverage_entry(&sc, &primary_slug).get("reason").is_none(),
        "a failed search reports its status once, never twice as a synonymous reason"
    );
    assert_eq!(sc["count"], 1);
    clear_test_project_search_results();
}

#[test]
fn red22_project_coverage_distinguishes_every_discovery_outcome() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let primary = tempfile::TempDir::new().unwrap();
    let search_fails = tempfile::TempDir::new().unwrap();
    let unavailable = tempfile::TempDir::new().unwrap();
    let never_registered = primary.path().join("not-present-at-startup");
    let additional = vec![
        search_fails.path().to_path_buf(),
        unavailable.path().to_path_buf(),
        never_registered.clone(),
    ];
    let state = state_with_roots(primary.path(), &additional);
    let primary_slug = project_slug(primary.path());
    let failed_slug = project_slug(search_fails.path());
    let unavailable_slug = project_slug(unavailable.path());
    let unregistered_slug = crate::cbm::bridge::cbm_project_slug(&never_registered);
    set_test_project_search_results(HashMap::from([
        (primary_slug.clone(), Ok(Vec::new())),
        (failed_slug.clone(), Err("search failed".into())),
        (unavailable_slug.clone(), Ok(Vec::new())),
    ]));
    configure_readiness(&[(&unavailable_slug, TestProjectReadiness::Unavailable)]);

    let sc = call_find(&state, "Absent", primary.path());

    assert!(
        !reports_project(&sc, &primary_slug),
        "the primary project was searched while ready: it is expected coverage and is not \
         reported: {sc:?}"
    );
    assert_eq!(coverage_entry(&sc, &failed_slug)["status"], "search_failed");
    assert_eq!(coverage_entry(&sc, &unavailable_slug)["status"], "skipped");
    assert_eq!(
        coverage_entry(&sc, &unavailable_slug)["reason"],
        "cbm_unavailable"
    );
    assert_eq!(coverage_entry(&sc, &unregistered_slug)["status"], "skipped");
    assert_eq!(
        coverage_entry(&sc, &unregistered_slug)["reason"],
        "additional_root_not_registered"
    );
    assert!(
        discovery(&sc)
            .unwrap_or_else(|| panic!("expected a diagnostic: {sc:?}"))
            .get("projects_truncated")
            .is_none(),
        "nothing was truncated, and a zero would restate absence"
    );
    clear_test_project_search_results();
}

#[test]
fn workspace_query_schema_declares_sparse_discovery_diagnostics() {
    let tools = crate::mcp::tools::tool_list();
    let workspace_query = tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query tool");
    let properties = workspace_query["outputSchema"]["properties"]
        .as_object()
        .expect("output properties");

    // One optional sparse object replaced the ten flat diagnostic fields.
    for removed in [
        "hydration_attempted",
        "discovery_provider",
        "discovery_status",
        "discovery_completed",
        "fallback_occurred",
        "fallback_reason",
        "candidates_discovered",
        "candidates_compiled",
        "project_coverage",
        "project_coverage_truncated",
    ] {
        assert!(
            !properties.contains_key(removed),
            "the flat diagnostic '{removed}' must not be advertised any more"
        );
    }
    let discovery = properties["discovery"]
        .as_object()
        .expect("the sparse discovery property");
    assert_eq!(discovery["type"], "object");
    let fields = discovery["properties"]
        .as_object()
        .expect("discovery properties");
    for field in [
        "provider",
        "status",
        "fallback_reason",
        "discovered",
        "compiled",
        "projects",
        "projects_truncated",
    ] {
        assert!(fields.contains_key(field), "missing schema field: {field}");
    }
    assert!(
        !fields.contains_key("attempted") && !fields.contains_key("fallback"),
        "the schema must not advertise redundant booleans"
    );
    assert_eq!(fields["projects"]["type"], "array");
    assert_eq!(
        fields["projects"]["items"]["properties"]["status"]["enum"],
        serde_json::json!(["searched", "search_failed", "skipped"])
    );
}
