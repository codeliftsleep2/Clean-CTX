//! Intended A3 budgets at the real dirty-generation public-query boundary.
use super::{assert_published_exact_call, assert_save_call, query};
use crate::work_observations_af_perf_002 as work;

fn measure(calls: usize, extras: usize) {
    let root = tempfile::tempdir().unwrap();
    let body = "target.Save();".repeat(calls);
    let methods: String = (0..extras)
        .map(|i| format!("public void Other{i}() {{}} "))
        .collect();
    std::fs::write(
        root.path().join("Caller.cs"),
        format!("class Caller {{ void Run(Target target) {{ {body} }} }}"),
    )
    .unwrap();
    std::fs::write(
        root.path().join("Target.cs"),
        format!("class Target {{ public void Save() {{}} {methods} }}"),
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    query(&state, root.path(), "TypeRef", "Target");
    state.mark_csharp_authority_dirty();
    state.reset_test_csharp_reconciliation_count();
    state.reset_test_full_semantic_parses();
    work::reset();
    let started = std::time::Instant::now();
    let response = query(&state, root.path(), "Method", "Save");
    assert_save_call(&response);
    assert_published_exact_call(&state, root.path());
    let counts = work::snapshot();
    let catalog = counts.csharp_catalog_population_max;
    assert_eq!(catalog, extras + 2, "admitted catalog fixture");
    assert_eq!(
        counts.csharp_resolution_calls, calls,
        "actual invocation dimension"
    );
    assert_eq!(state.test_csharp_reconciliation_count(), 1);
    assert_eq!(state.test_full_semantic_parses(), 0);
    eprintln!(
        "AF002_A3_BUDGET C={calls} D={catalog} builds={} build_visits={} build_probes={} lookups={} candidate_visits={:?} capacities={:?} payload_bytes={} owners={} us={} memory_accounting=logical-payload-capacity-not-allocator-bytes",
        counts.csharp_lookup_builds,
        counts.csharp_lookup_build_visits,
        counts.csharp_lookup_build_probes,
        counts.csharp_lookup_probes,
        counts.csharp_declaration_probes,
        counts.csharp_lookup_capacities,
        counts.csharp_lookup_payload_bytes,
        counts.artifact_projection_owners,
        started.elapsed().as_micros()
    );
    let total = counts.csharp_lookup_build_visits
        + counts.csharp_lookup_build_probes
        + counts.csharp_lookup_probes
        + counts.csharp_declaration_probes.iter().sum::<usize>();
    // This fixture has one demanding owner and a unique direct target. The
    // second retained owner has no invocation and must not build a lookup.
    assert!(
        counts.csharp_lookup_builds <= 1,
        "AF002_A3 repeated lookup construction"
    );
    assert!(
        counts.csharp_lookup_build_visits <= catalog,
        "AF002_A3 repeated catalog construction visits"
    );
    assert!(
        counts.csharp_lookup_build_probes <= catalog,
        "instance-only fixture requires one construction entry per declaration"
    );
    assert!(
        total <= 2 * catalog + 2 * calls,
        "AF002_A3 full-catalog amplification C={calls} D={catalog} total={total} budget={}",
        2 * catalog + 2 * calls
    );
}

#[test]
fn af002_a3_budget_invocation_growth() {
    let _serial = crate::protocol::handler_response_serial();
    for calls in [1, 2, 4, 8, 16, 64, 256] {
        measure(calls, 1);
    }
}

#[test]
fn af002_a3_budget_catalog_growth_and_joint() {
    let _serial = crate::protocol::handler_response_serial();
    for extras in [1, 2, 4, 8, 16, 64, 256] {
        measure(16, extras);
    }
}

#[test]
fn af002_a3_budget_no_demand_builds_nothing() {
    use crate::ir::opcodes::CoreOp;
    let instructions = vec![
        CoreOp::DefClass("C1".into(), "Quiet".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "Idle".into()),
        CoreOp::DefMethod("C1".into(), "M2".into(), "Other".into()),
    ];
    let authority = crate::ir::csharp_exact_calls::declaration_authority(&instructions);
    assert_eq!(authority.len(), 2);
    work::reset();
    let projected = crate::ir::csharp_exact_calls::project_with_authority(
        "",
        &instructions,
        &[],
        "Quiet.cs",
        &authority,
    );
    let counts = work::snapshot();
    assert_eq!(projected.len(), 2); // Declaration registration remains independent of call demand.
    assert_eq!(counts.csharp_lookup_builds, 0);
    assert_eq!(counts.csharp_lookup_build_visits, 0);
    assert_eq!(counts.csharp_lookup_build_probes, 0);
    assert_eq!(counts.csharp_lookup_probes, 0);
    assert_eq!(counts.csharp_declaration_probes, [0, 0]);
}

#[test]
fn a3_gap_single_call_catalog_growth() {
    let _serial = crate::protocol::handler_response_serial();
    for extras in [1, 2, 4, 8, 16, 64, 256] {
        measure(1, extras);
    }
}
