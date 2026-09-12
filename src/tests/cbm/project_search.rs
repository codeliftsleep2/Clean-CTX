use crate::cbm::GraphBridge;
use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::config::CbmConfig;
use serde_json::Value;

#[test]
fn configured_projects_are_complete_deterministic_and_non_mutating() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let bridge = GraphBridge::try_create_with_roots(
        &config,
        primary.path(),
        &[additional.path().to_path_buf()],
    );
    let active_before = bridge.project_str();

    let projects = bridge.configured_projects();
    let mut expected = vec![
        (
            cbm_project_slug(&primary.path().canonicalize().unwrap()),
            primary.path().canonicalize().unwrap(),
        ),
        (
            cbm_project_slug(&additional.path().canonicalize().unwrap()),
            additional.path().canonicalize().unwrap(),
        ),
    ];
    expected.sort_by(|left, right| left.0.cmp(&right.0));

    assert_eq!(projects, expected);
    assert_eq!(bridge.project_str(), active_before);
}

#[test]
fn additional_root_registration_matches_proxy_resolution_and_enumeration() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let bridge = GraphBridge::try_create_with_roots(
        &config,
        primary.path(),
        &[additional.path().to_path_buf()],
    );

    let additional_root = additional.path().canonicalize().unwrap();
    let expected_slug = cbm_project_slug(&additional_root);
    let resolved_slug = bridge.resolve_project_id(&additional_root.to_string_lossy());
    let configured = bridge.configured_projects();

    assert_eq!(resolved_slug, expected_slug, "proxy-style path resolution");
    assert!(
        configured
            .iter()
            .any(|(slug, root)| slug == &resolved_slug && root == &additional_root),
        "additional root must exist in both root-to-project resolution and project-to-root enumeration"
    );
}

#[test]
fn explicit_project_search_error_preserves_active_project() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create_with_roots(
        &config,
        primary.path(),
        &[additional.path().to_path_buf()],
    );
    let active_before = bridge.project_str();
    let additional_slug = cbm_project_slug(&additional.path().canonicalize().unwrap());

    assert!(
        bridge
            .search_in_project(&additional_slug, "Candidate")
            .is_err()
    );
    assert_eq!(bridge.project_str(), active_before);
}

#[test]
fn inbound_reference_query_is_path_only_exact_and_escaped() {
    let query = super::inbound_reference_query("ServiceA\\' MATCH (n) RETURN n");

    assert_eq!(
        query,
        "MATCH (caller)-[r]->(target {name: 'ServiceA\\\\\\' MATCH (n) RETURN n'}) \
         RETURN caller.file_path, type(r)"
    );
    // CBM's Cypher subset rejects `type(r)` inside `WHERE` comparisons;
    // the query must project type(r) in RETURN and filter client-side.
    assert!(
        !query.contains("WHERE type(r)"),
        "relationship type filtering must be client-side, not in WHERE"
    );
}

#[test]
fn red_r1_inbound_query_is_cbm_compatible() {
    let query = super::inbound_reference_query("ServiceA");

    // CBM's Cypher subset rejects `type(r)` inside `WHERE` comparisons
    // (parser error lands on the `type(` expression). The query must
    // project type(r) in RETURN and filter client-side instead.
    assert!(
        !query.contains("WHERE type(r)"),
        "regression: WHERE type(r) is not supported by CBM's Cypher subset"
    );
    assert!(
        query.contains("RETURN caller.file_path, type(r)"),
        "query must project caller.file_path and type(r) for client-side filtering"
    );
    assert!(
        query.contains("MATCH (caller)-[r]->(target"),
        "query must match the inbound reference pattern"
    );
}

#[test]
fn red_r2_definition_relationships_are_filtered_client_side() {
    let usage_row = vec![
        Value::String("consumer.ts".into()),
        Value::String("USAGE".into()),
    ];
    let defines_row = vec![
        Value::String("declaration.ts".into()),
        Value::String("DEFINES".into()),
    ];
    let defines_method_row = vec![
        Value::String("method_decl.ts".into()),
        Value::String("DEFINES_METHOD".into()),
    ];
    let calls_row = vec![
        Value::String("caller.ts".into()),
        Value::String("CALLS".into()),
    ];

    assert_eq!(
        super::filter_inbound_reference_row(&usage_row),
        Some("consumer.ts".to_string()),
        "USAGE relationship must be retained as a valid inbound reference"
    );
    assert_eq!(
        super::filter_inbound_reference_row(&defines_row),
        None,
        "DEFINES relationship must be discarded client-side"
    );
    assert_eq!(
        super::filter_inbound_reference_row(&defines_method_row),
        None,
        "DEFINES_METHOD relationship must be discarded client-side"
    );
    assert_eq!(
        super::filter_inbound_reference_row(&calls_row),
        Some("caller.ts".to_string()),
        "CALLS relationship must be retained as a valid inbound reference"
    );
}

#[test]
fn explicit_project_inbound_error_preserves_active_project() {
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create_with_roots(
        &config,
        primary.path(),
        &[additional.path().to_path_buf()],
    );
    let active_before = bridge.project_str();
    let additional_slug = cbm_project_slug(&additional.path().canonicalize().unwrap());

    assert!(
        bridge
            .inbound_reference_paths_in_project(&additional_slug, "ServiceA")
            .is_err()
    );
    assert_eq!(bridge.project_str(), active_before);
}
