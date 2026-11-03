//! AF-PERF-001: real retained-admission work, independent of wall-clock timing.
use crate::angular_meta::{
    admission_work_metrics as metrics, route_components, route_guards, route_lazy_components,
    route_load_children, route_providers,
};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::time::Instant;
fn fixture() -> (tempfile::TempDir, TypeScriptAuthority) {
    let (root, _) = super::angular_route_resolver_semantic_tests::fixture("direct");
    let file = root.path().join("one.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    let a = TypeScriptAuthority::capture_with_declaration_fields(
        &file,
        &source,
        &[root.path().to_path_buf()],
        None,
        crate::angular_meta::module_metadata::DECLARATION_FIELDS,
    );
    assert!(
        a.capture_current,
        "stage=fixture-current {:?}",
        a.disposition
    );
    (root, a)
}
fn selected(a: &TypeScriptAuthority, index: usize) -> Vec<SemanticEdge> {
    match index {
        0 => route_guards::project(a),
        1..=4 => route_guards::project_kinds(a)
            .into_iter()
            .filter(|e| e.relation == route_guards::KINDS[index - 1].1)
            .collect(),
        5 => route_components::project(a),
        6 => route_lazy_components::project(a),
        7 => route_load_children::project(a),
        8 => route_providers::project(a),
        _ => unreachable!(),
    }
}
fn check_growth(index: usize) {
    let (_root, a) = fixture();
    let edges = selected(&a, index);
    assert!(
        !edges.is_empty(),
        "stage=fixture-membership consumer={index}"
    );
    for n in [1, 2, 4, 8, 16] {
        let batch: Vec<_> = edges.iter().cycle().take(n).cloned().collect();
        metrics::reset();
        let started = Instant::now();
        assert!(
            a.admits_relation_evidence(&batch),
            "stage=exact-admission consumer={index}"
        );
        let counts = metrics::counts();
        eprintln!(
            "AF_WORK consumer={index} candidates={n} projections={counts:?} admission_us={}",
            started.elapsed().as_micros()
        );
        assert_eq!(
            counts[index], 1,
            "AF-PERF-001 projection-per-edge consumer={index} candidates={n}"
        );
        for (other, count) in counts.into_iter().enumerate() {
            // Lazy selection legitimately attempts the eager branch first.
            if other != index && !(index == 6 && other == 5) {
                assert_eq!(count, 0, "unreachable consumer={other} selected={index}");
            }
        }
        if index == 6 {
            assert_eq!(counts[5], 1, "lazy fallback eager projection bound");
        }
    }
}
#[test]
fn af_perf_001_activate_growth() {
    check_growth(0);
}
#[test]
fn af_perf_001_match_growth() {
    check_growth(1);
}
#[test]
fn af_perf_001_deactivate_growth() {
    check_growth(2);
}
#[test]
fn af_perf_001_load_growth() {
    check_growth(3);
}
#[test]
fn af_perf_001_activate_child_growth() {
    check_growth(4);
}
#[test]
fn af_perf_001_eager_growth() {
    check_growth(5);
}
#[test]
fn af_perf_001_lazy_growth() {
    check_growth(6);
}
#[test]
fn af_perf_001_children_growth() {
    check_growth(7);
}
#[test]
fn af_perf_001_providers_growth() {
    check_growth(8);
}
#[test]
fn af_perf_001_unrelated_and_short_circuit_zero_work() {
    let (_root, a) = fixture();
    let legacy = SemanticEdge {
        subject: EntityRef::new("angular", "Route", "legacy"),
        object: EntityRef::new("angular", "Component", "legacy"),
        relation: SemanticRelation::RouteMapsTo,
        layer: "angular",
        call_evidence: None,
    };
    for batch in [vec![], vec![legacy]] {
        metrics::reset();
        assert!(a.admits_relation_evidence(&batch));
        assert_eq!(metrics::counts(), [0; 9]);
    }
    let mut forged = selected(&a, 0)[0].clone();
    forged.object.name = "forged".into();
    let later = selected(&a, 7)[0].clone();
    metrics::reset();
    assert!(!a.admits_relation_evidence(&[forged, later]));
    assert_eq!(
        metrics::counts()[7],
        0,
        "all() refusal must not evaluate later consumers"
    );
}
#[test]
fn af_perf_001_mixed_fresh_calls_and_dependency_refusal() {
    let (root, a) = fixture();
    let mut edges = Vec::new();
    for i in 0..9 {
        edges.extend(selected(&a, i));
    }
    assert!(a.admits_relation_evidence(&edges), "stage=mixed-fixture");
    for _ in 0..2 {
        metrics::reset();
        assert!(a.admits_relation_evidence(&edges));
        let counts = metrics::counts();
        eprintln!("AF_MIXED projections={counts:?}");
        assert_eq!(
            counts, [1; 9],
            "one projection per family in each fresh invocation"
        );
    }
    let dependency = root.path().join("node_modules/@angular/router/index.d.ts");
    let original = std::fs::read_to_string(&dependency).unwrap();
    std::fs::write(&dependency, format!("{original} // changed dependency")).unwrap();
    metrics::reset();
    assert!(
        !a.admits_relation_evidence(&edges),
        "new call must refuse stale dependencies"
    );
    std::fs::write(&dependency, original).unwrap();
    assert!(
        a.admits_relation_evidence(&edges),
        "byte repair restores proof"
    );
}
