//! Deterministic admission-work regressions and one fixed direct-authority measurement.
use crate::angular_meta::route_resolvers::{project, replay_metrics};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::time::Instant;
const SOURCE: &str = "import type {Routes} from '@angular/router';function A(){return true;}function B(){return false;}export const routes:Routes=[{path:'same',resolve:{first:A,second:B,again:A}},{path:'same',resolve:{first:A}},{path:'',resolve:{second:B}}];";
fn fixture() -> (tempfile::TempDir, TypeScriptAuthority, Vec<SemanticEdge>) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    let package = p.join("node_modules/@angular/router");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(p.join("tsconfig.json"),r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/router","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.d.ts"),
        "export interface Route {path?:string;resolve?:unknown;}export type Routes=Route[];",
    )
    .unwrap();
    let file = p.join("routes.ts");
    std::fs::write(&file, SOURCE).unwrap();
    let a = TypeScriptAuthority::capture_with_declaration_fields(
        &file,
        SOURCE,
        &[p.to_path_buf()],
        None,
        crate::angular_meta::module_metadata::DECLARATION_FIELDS,
    );
    assert!(a.capture_current, "fixture capture: {:?}", a.disposition);
    assert_eq!(a.resolve_mappings.len(), 5);
    let edges = project(&a);
    assert_eq!(
        edges.len(),
        4,
        "fixture must prove nonempty exact membership"
    );
    (root, a, edges)
}
#[test]
fn issue_162_replay_p0_unrelated_admission_has_zero_resolver_projections() {
    let (_root, a, _edges) = fixture();
    for relation in [SemanticRelation::Defines, SemanticRelation::ResolvedBy] {
        let edge = SemanticEdge {
            subject: EntityRef::new("angular", "Route", "legacy"),
            object: EntityRef::new("angular", "Resolver", "legacy"),
            relation,
            layer: "angular",
            call_evidence: None,
        };
        replay_metrics::reset();
        assert!(a.admits_relation_evidence(&[edge]));
        assert_eq!(
            replay_metrics::counts(),
            (0, 0),
            "noncanonical compatibility must not trigger canonical resolver projection"
        );
    }
    replay_metrics::reset();
    assert!(a.admits_relation_evidence(&[]));
    assert_eq!(replay_metrics::counts(), (0, 0));
}
#[test]
fn issue_162_replay_p0_canonical_batch_projects_once_independent_of_edge_count() {
    let (_root, a, edges) = fixture();
    for n in [1, 2, 4, 8, 16] {
        let batch = edges.iter().cycle().take(n).cloned().collect::<Vec<_>>();
        replay_metrics::reset();
        assert!(a.admits_relation_evidence(&batch));
        let (projects, mappings) = replay_metrics::counts();
        eprintln!("P0_WORK edges={n} projections={projects} mapping_replays={mappings}");
        assert_eq!(
            projects, 1,
            "architecture defect: projection must be call-local, not per edge"
        );
    }
}
#[test]
fn issue_162_replay_p0_projection_does_not_repeat_mapping_proof() {
    let (_root, a, _edges) = fixture();
    replay_metrics::reset();
    assert_eq!(project(&a).len(), 4);
    assert_eq!(
        replay_metrics::counts(),
        (1, 5),
        "each of the five exact mappings is replayed once; demanded reference must not replay again"
    );
}
#[test]
fn issue_162_replay_p0_fixed_measurement() {
    let started = Instant::now();
    let (_root, a, edges) = fixture();
    let fixture_us = started.elapsed().as_micros();
    replay_metrics::reset();
    let started = Instant::now();
    assert!(a.admits_relation_evidence(&edges));
    let elapsed = started.elapsed().as_micros();
    let (projections, mappings) = replay_metrics::counts();
    eprintln!(
        "P0_FIXED fixture_bytes={} files=4 objects=3 mappings=5 edges=4 fixture_us={fixture_us} admission_us={elapsed} projections={projections} mapping_replays={mappings} witnesses={}",
        SOURCE.len(),
        a.witnesses.len()
    );
    // Timing is descriptive. Deterministic work-count tests establish the invariant.
}

#[test]
fn issue_162_replay_p0_independent_runtime_and_forged_refusal() {
    let (root, a, edges) = fixture();
    let edge = edges
        .iter()
        .find(|e| e.subject.name == "same" && e.object.name == "B")
        .unwrap();
    let mapping = &a.resolve_mappings[1];
    assert_eq!(a.resolve_mapping_at(mapping.pair).unwrap().1.name, "B");
    for mode in 0..8 {
        let mut b = a.clone();
        let mut candidate = edge.clone();
        match mode {
            0 => b
                .reference_sites
                .retain(|(s, e, _)| (*s, *e) != mapping.value),
            1 => b
                .demanded_occurrences
                .retain(|o| (o.start, o.end) != mapping.value),
            2 => b.resolve_mappings.clear(),
            3 => b.capture_current = false,
            4 => candidate.object.name = "forged".into(),
            5 => candidate.layer = "typescript",
            6 => candidate.subject.canonical = None,
            _ => {
                let mut borrowed = b.resolve_mappings[0].clone();
                borrowed.value = mapping.value;
                borrowed.source_hash = "crossed-hash".into();
                b.resolve_mappings.push(borrowed);
            }
        }
        assert!(
            !b.admits_relation_evidence(&[candidate]),
            "independent refusal {mode}"
        );
    }
    // Host::seed supplies immutable owner bytes; the outer caller compares
    // the current owner hash. Here mutate an actually consumed dependency.
    let dependency = root.path().join("node_modules/@angular/router/index.d.ts");
    let original = std::fs::read_to_string(&dependency).unwrap();
    std::fs::write(&dependency, format!("{original} // dependency changed")).unwrap();
    assert!(
        !a.admits_relation_evidence(&edges),
        "consumed dependency witness must refuse stale proof"
    );
}
