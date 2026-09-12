use crate::cbm::GraphBridge;
use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::config::CbmConfig;

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
         WHERE type(r) <> 'DEFINES' AND type(r) <> 'DEFINES_METHOD' \
         RETURN caller.file_path"
    );
    assert!(
        !query.contains("type(r),"),
        "relationship data is not projected"
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
