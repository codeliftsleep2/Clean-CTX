//! Diagnose the unchanged-source durable boundary without weakening its checks.
use super::angular_bootstrap_inline_controls_tests::{CALL, call, capture, setup};
use serde_json::json;

#[test]
fn issue_162_unchanged_bootstrap_durable_registration_boundary() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, mut config) = setup(CALL);
    let project = crate::tests::angular_authority_fixture::isolate_program(root.path());
    config.additional_roots = vec![project.to_string_lossy().into_owned()];
    let path = project.join("main.ts");
    let state = crate::mcp::McpState::new(config.clone());
    capture(&state, project.as_path(), "main.ts");
    call(
        &state,
        project.as_path(),
        "save_context",
        json!({"filePath":path}),
    );
    let owner = state.durable_owner_path(&path.to_string_lossy());
    let mut loaded = {
        let guard = state.persistence_store_lock();
        guard
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .load_durable_context(&owner, None)
            .unwrap()
            .unwrap()
    };
    drop(state);
    let source = std::fs::read_to_string(&path).unwrap();
    let authority = loaded.authority.typescript_authority.as_ref().unwrap();
    let stale: Vec<_> = authority
        .witnesses
        .iter()
        .filter(|witness| !witness.current())
        .collect();
    assert!(
        loaded.authority.is_current(&loaded.source_hash),
        "unchanged source authority rejected; stale witnesses: {stale:?}"
    );
    let applies = crate::layers::meta::angular_applicability::registrations_apply(
        &source,
        &path,
        loaded.fidelity,
        &config,
        authority,
    );
    loaded.authority.resolver_registration_scope = Some(applies);
    assert!(
        loaded.authority.admits_semantic_projection(
            &loaded.semantic_edges,
            crate::layers::meta::resolver_registrations::enabled(&config),
        ),
        "unchanged complete projection rejected: scope={applies}, registrations={:?}",
        loaded.authority.resolver_registrations
    );
    let stale_dependencies: Vec<_> = loaded
        .semantic_dependencies
        .iter()
        .filter(|dependency| !dependency.is_current())
        .collect();
    assert!(
        stale_dependencies.is_empty(),
        "unchanged durable dependency footprints rejected: {stale_dependencies:?}"
    );
    let identities = crate::mcp::compatibility::derive_identities(&source, &path, &config).unwrap();
    let identity_routing = identities
        .semantic_config
        .angular
        .as_ref()
        .map(|angular| (angular.enabled, angular.routing_enabled));
    let retained = loaded.authority.clone();
    crate::mcp::compatibility::validator::validate_current_context(
        loaded,
        &source,
        &path,
        &config,
        crate::compression::Fidelity::High,
    )
    .unwrap_or_else(|error| {
        panic!(
            "production validator rejected unchanged authority: {error:?}; \
             native_scope={applies}; identity_angular_routing={identity_routing:?}; \
             configured_registration_enabled={}",
            crate::layers::meta::resolver_registrations::enabled(&config)
        )
    });
    std::fs::write(project.join("added.ts"), "export class Added {}").unwrap();
    assert!(
        !retained.is_current(&crate::cache::content_hash(source.as_bytes())),
        "a real program-directory addition must still invalidate captured authority"
    );
}
