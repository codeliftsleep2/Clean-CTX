//! ANG-R2 initial production transport and exact shared-projection preservation.
use super::angular_route_resolver_semantic_tests::fixture;
use super::typescript_route_guard_field_support::capture;
use crate::angular_meta::route_resolvers::{project_with_registrations, replay_metrics};
use crate::mcp::McpState;

#[test]
fn issue_162_resolver_registration_transport_exact_shared_projection() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let state = McpState::new(config);
    let authority = capture(&state, root.path(), "one.ts");
    let alias = state
        .alias_for_path(&root.path().join("one.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let produced = snapshot
        .authority
        .resolver_registrations
        .as_ref()
        .expect("default compiler/artifact/snapshot must retain produced registrations");
    replay_metrics::reset();
    let expected = project_with_registrations(&authority);
    assert_eq!(
        replay_metrics::counts(),
        (1, 9),
        "one shared resolver evaluation"
    );
    assert_eq!(
        expected.edges.len(),
        8,
        "unqualified memberships stay deduplicated"
    );
    assert_eq!(
        expected.registrations.records.len(),
        9,
        "distinct keys survive"
    );
    assert_eq!(
        produced, &expected.registrations,
        "actual production transport"
    );
    assert_eq!(produced.format_version, 1);
    assert!(produced.semantic_omissions.is_empty());
    for record in &produced.records {
        let (mapping, target) = authority.resolve_mapping_at(record.evidence.pair).unwrap();
        assert_eq!(record.key, mapping.name);
        assert_eq!(record.resolver, target.into());
        assert_eq!(record.route.source, authority.source);
        assert_eq!(record.route.source_hash, authority.source_hash);
        assert_eq!((record.route.start, record.route.end), mapping.owner);
        assert_eq!(record.evidence.source, mapping.source);
        assert_eq!(record.evidence.source_hash, mapping.source_hash);
        assert_eq!(record.evidence.field_pair, mapping.field_pair);
        assert_eq!(record.evidence.field_key, mapping.field_key);
        assert_eq!(record.evidence.object, mapping.object);
        assert_eq!(record.evidence.key, mapping.key);
        assert_eq!(record.evidence.value, mapping.value);
    }
}

#[test]
fn issue_162_resolver_registration_transport_duplicate_key_refuses_locally() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let file = root.path().join("one.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, source.replace("data1:", "data0:")).unwrap();
    let state = McpState::new(config);
    let authority = capture(&state, root.path(), "one.ts");
    let projected = project_with_registrations(&authority);
    assert_eq!(projected.registrations.records.len(), 7);
    assert_eq!(projected.edges.len(), 7);
    assert_eq!(projected.registrations.semantic_omissions.len(), 2);
    for omission in &projected.registrations.semantic_omissions {
        assert_eq!(omission.reason, crate::layers::meta::resolver_registrations::ResolverOmissionReason::DuplicateMappingKey);
        assert!(omission.route.is_some());
        assert!(
            omission.resolver.is_none(),
            "refusal cannot invent target association"
        );
        assert_eq!(omission.source_hash, authority.source_hash);
    }
}
