//! Narrow real MCP publication/SQLite restore/snapshot/query admission bounds.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use crate::angular_meta::admission_work_metrics as metrics;
use crate::mcp::tool_handlers::query::work_metrics;
use serde_json::json;
use std::time::Instant;
fn measured<T>(phase: &str, requires_admission: bool, action: impl FnOnce() -> T) -> T {
    metrics::reset_operations();
    work_metrics::reset();
    #[cfg(feature = "rust")]
    crate::mcp::tool_handlers::hydration::reset_test_scan_calls();
    crate::angular_meta::detect_metrics::reset_ast_parse_count();
    let start = Instant::now();
    let result = action();
    let counts = metrics::operations();
    let query = work_metrics::snapshot();
    #[cfg(feature = "rust")]
    let discovery_scans = Some(crate::mcp::tool_handlers::hydration::test_scan_calls());
    #[cfg(not(feature = "rust"))]
    let discovery_scans: Option<usize> = None;
    eprintln!(
        "AF_LIFECYCLE phase={phase} us={} admissions={} per_operation_max={:?} witness_checks={} angular_detection_parses={} discovery_scans={discovery_scans:?} query_work={query:?}",
        start.elapsed().as_micros(),
        counts.calls,
        counts.maximum,
        counts.witnesses,
        crate::angular_meta::detect_metrics::ast_parse_count()
    );
    if requires_admission {
        assert!(
            counts.calls > 0,
            "phase={phase} must exercise actual retained admission"
        );
        assert_eq!(
            counts.maximum[9], 1,
            "phase={phase} must validate nonempty resolver proof"
        );
        assert!(
            counts.witnesses > 0,
            "phase={phase} must observe existing witness validation"
        );
    } else {
        // Restored exact forward queries consume current indexed edges; they
        // must not reconstruct complete projections merely to return them.
        assert_eq!(
            counts.calls, 0,
            "phase={phase} unexpectedly repeated retained admission"
        );
        assert_eq!(
            counts.maximum, [0; 14],
            "phase={phase} unexpectedly projected indexed facts"
        );
    }
    assert!(
        counts.maximum.into_iter().all(|n| n <= 1),
        "phase={phase} repeated complete projection: {counts:?}"
    );
    result
}
#[test]
fn af_perf_001_lifecycle_publication_restore_snapshot_query() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = super::angular_route_resolver_semantic_tests::fixture("direct");
    let p = root.path();
    let original = std::fs::read_to_string(p.join("one.ts")).unwrap();
    let prefix = original
        .split("export const routes:Routes=")
        .next()
        .unwrap();
    std::fs::write(p.join("one.ts"),format!("{prefix} export const routes:Routes=[{{path:'bounded',component:Eager,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen),canActivate:[Gate],canMatch:[Gate],canDeactivate:[Gate],canLoad:[Gate],canActivateChild:[Gate],resolve:{{x:Gate,y:Gate}},providers:[Token]}}];")).unwrap();
    std::fs::remove_file(p.join("two.ts")).unwrap();
    let state = crate::mcp::McpState::new(config.clone());
    let authority = measured("publication", true, || capture(&state, p, "one.ts"));
    let expected = crate::angular_meta::route_resolvers::project(&authority);
    assert_eq!(
        expected.len(),
        1,
        "two mapping supports select one exact target"
    );
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let state = crate::mcp::McpState::new(config);
    measured("sqlite-restore", true, || {
        call(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("one.ts")}),
        )
    });
    let alias = state
        .alias_for_path(&p.join("one.ts").to_string_lossy())
        .unwrap();
    let snapshot = measured("snapshot-currentness", true, || {
        state.semantic_snapshot(&alias).unwrap()
    });
    assert!(
        snapshot
            .edges
            .iter()
            .any(|e| e.relation == expected[0].relation
                && e.subject.canonical == expected[0].subject.canonical
                && e.object.canonical == expected[0].object.canonical)
    );
    use crate::layers::meta::semantic::SemanticRelation;
    for relation in [
        SemanticRelation::GuardedBy,
        SemanticRelation::GuardedByCanMatch,
        SemanticRelation::GuardedByCanDeactivate,
        SemanticRelation::GuardedByCanLoad,
        SemanticRelation::GuardedByCanActivateChild,
        SemanticRelation::RouteMapsTo,
        SemanticRelation::LoadsChildren,
        SemanticRelation::RegistersProvider,
    ] {
        assert!(
            snapshot
                .edges
                .iter()
                .any(|e| e.relation == relation && e.subject.canonical.is_some()),
            "restored native sibling must remain reachable: {relation:?}"
        );
    }

    for phase in ["query", "warm-query"] {
        let answer = measured(phase, false, || {
            call(
                &state,
                p,
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"bounded","withinPath":p.join("one.ts")}),
            )
        });
        assert_eq!(
            answer["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["relation"] == "ResolvedBy")
                .count(),
            1
        );
    }
    let dependency = p.join("guard-a.ts");
    let bytes = std::fs::read(&dependency).unwrap();
    std::fs::write(&dependency, "export const Changed=1;").unwrap();
    assert!(
        !authority.admits_relation_evidence(&expected),
        "fresh admission refuses consumed target mutation"
    );
    std::fs::write(dependency, bytes).unwrap();
    assert!(authority.admits_relation_evidence(&expected));
}
