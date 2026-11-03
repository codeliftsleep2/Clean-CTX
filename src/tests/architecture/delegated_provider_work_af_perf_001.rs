//! AF-PERF-001 delegated provider-chain baseline; real retained admission.
use super::{
    application_config_providers, bootstrap_references, config_composition,
    delegated_work_metrics as metrics, provider_imported, provider_position::Position,
};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::time::Instant;
const SOURCE: &str = "import {Component,ApplicationConfig,mergeApplicationConfig} from '@angular/core'; import {bootstrapApplication} from '@angular/platform-browser'; import {GROUP} from './group'; import type {Routes} from '@angular/router'; @Component({providers:[...GROUP],viewProviders:[...GROUP]}) export class Host {} export const config:ApplicationConfig={providers:[...GROUP]}; export const merged:ApplicationConfig=mergeApplicationConfig(config,config); bootstrapApplication(Host,{providers:[...GROUP]}); export const routes:Routes=[{path:'route',providers:[...GROUP]}];";
fn fixture() -> (tempfile::TempDir, TypeScriptAuthority) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),"export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];} export declare function mergeApplicationConfig(...configs:ApplicationConfig[]):ApplicationConfig;").unwrap();
    for (name, api) in [
        (
            "platform-browser",
            "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;",
        ),
        (
            "router",
            "export interface Route {path?:string;providers?:unknown[];} export type Routes=Route[];",
        ),
    ] {
        let directory = p.join("node_modules/@angular").join(name);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("package.json"),
            format!(
                "{{\"name\":\"@angular/{name}\",\"version\":\"20.0.0\",\"types\":\"index.d.ts\"}}"
            ),
        )
        .unwrap();
        std::fs::write(directory.join("index.d.ts"), api).unwrap();
    }
    std::fs::write(
        p.join("tokens.ts"),
        "export class Token {} export class Target {}",
    )
    .unwrap();
    std::fs::write(p.join("group.ts"),"import {Token,Target} from './tokens'; export const GROUP=[Token,{provide:Token,useClass:Target}];").unwrap();
    let file = p.join("main.ts");
    std::fs::write(&file, SOURCE).unwrap();
    let authority = TypeScriptAuthority::capture_with_demands(
        &file,
        SOURCE,
        &[p.to_path_buf()],
        None,
        crate::angular_meta::module_metadata::DECLARATION_FIELDS,
        crate::angular_meta::bootstrap_references::DEMANDED_APIS,
    );
    assert!(
        authority.capture_current,
        "stage=fixture-current {:?}",
        authority.disposition
    );
    (root, authority)
}
fn imported(a: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for facet in &a.declaration_facets {
        if Position::Ordinary
            .owner_type(a, &facet.declaration)
            .is_some()
        {
            for position in [Position::Ordinary, Position::View] {
                edges.extend(provider_imported::project(
                    a,
                    &facet.declaration.name,
                    position,
                ));
            }
        }
    }
    edges
}
fn selected(a: &TypeScriptAuthority, index: usize) -> Vec<SemanticEdge> {
    match index {
        0 => bootstrap_references::project(a)
            .into_iter()
            .filter(|e| e.relation == SemanticRelation::DeclaresBootstrapRoot)
            .collect(),
        1 => application_config_providers::project(a)
            .into_iter()
            .filter(|e| {
                e.subject.entity_type == "ApplicationConfig"
                    && e.relation == SemanticRelation::RegistersProvider
            })
            .collect(),
        2 => config_composition::project(a),
        3 => imported(a)
            .into_iter()
            .filter(|e| e.relation == SemanticRelation::RegistersViewProvider)
            .collect(),
        _ => unreachable!(),
    }
}
fn growth(index: usize) {
    let (_root, a) = fixture();
    let edges = selected(&a, index);
    assert!(
        !edges.is_empty(),
        "stage=fixture-membership consumer={index}"
    );
    for n in [1, 2, 4, 8, 16] {
        let batch: Vec<_> = edges.iter().cycle().take(n).cloned().collect();
        metrics::reset();
        let start = Instant::now();
        assert!(
            a.admits_relation_evidence(&batch),
            "stage=exact-admission consumer={index}"
        );
        let counts = metrics::counts();
        eprintln!(
            "AF_DELEGATED consumer={index} candidates={n} constructions={counts:?} imported_position_projects={} admission_us={}",
            metrics::positions(),
            start.elapsed().as_micros()
        );
        assert_eq!(
            counts[index], 1,
            "AF-PERF-001 delegated reconstruction consumer={index} candidates={n}"
        );
        assert!(
            counts.into_iter().all(|v| v <= 1),
            "delegated helper amplification: {counts:?}"
        );
        for (other, count) in counts.into_iter().enumerate() {
            if other != index {
                assert_eq!(
                    count, 0,
                    "unreachable consumer={other} for selected={index}"
                );
            }
        }
    }
}
#[test]
fn af_delegated_001_bootstrap_growth() {
    growth(0);
}
#[test]
fn af_delegated_001_config_provider_growth() {
    growth(1);
}
#[test]
fn af_delegated_001_config_composition_growth() {
    growth(2);
}
#[test]
fn af_delegated_001_imported_view_growth() {
    growth(3);
}
#[test]
fn af_delegated_001_mixed_fresh_calls_and_mutation() {
    let (root, a) = fixture();
    let mut edges = Vec::new();
    for index in 0..4 {
        let selected = selected(&a, index);
        assert!(!selected.is_empty());
        edges.extend(selected);
    }
    edges.extend(super::route_providers::project(&a));
    // Explicit provider pairs exercise the original bootstrap/config/route/imported OR chain.
    edges.extend(
        imported(&a)
            .into_iter()
            .filter(|e| e.relation == SemanticRelation::ProviderUsesClass),
    );
    for _ in 0..2 {
        metrics::reset();
        assert!(a.admits_relation_evidence(&edges), "stage=mixed-membership");
        let counts = metrics::counts();
        eprintln!("AF_DELEGATED_MIXED constructions={counts:?}");
        assert_eq!(
            counts, [1; 4],
            "delegated complete sets must be call-local across direct and indirect candidates"
        );
    }
    let dependency = root.path().join("tokens.ts");
    let original = std::fs::read(&dependency).unwrap();
    std::fs::write(&dependency, "export class Changed {}").unwrap();
    // Production snapshot validation checks retained currentness before replay.
    // Bare legacy consumer replay intentionally does not replace that boundary.
    let retained = crate::mcp::semantic_compilation::retained::RetainedAuthority {
        typescript_authority: Some(a.clone()),
    };
    assert!(
        !retained.is_current(&a.source_hash),
        "production retained owner must refuse stale dependency"
    );
    let changed = TypeScriptAuthority::capture_with_demands(
        &root.path().join("main.ts"),
        SOURCE,
        &[root.path().to_path_buf()],
        None,
        crate::angular_meta::module_metadata::DECLARATION_FIELDS,
        crate::angular_meta::bootstrap_references::DEMANDED_APIS,
    );
    assert!(
        changed.capture_current,
        "changed dependency capture must be current"
    );
    metrics::reset();
    assert!(
        !changed.admits_relation_evidence(&edges),
        "fresh authority must refuse old target identity"
    );
    assert!(metrics::counts().into_iter().all(|n| n <= 1));
    std::fs::write(dependency, original).unwrap();
    assert!(
        retained.is_current(&a.source_hash),
        "byte repair restores retained owner currentness"
    );
    metrics::reset();
    assert!(a.admits_relation_evidence(&edges));
    assert_eq!(
        metrics::counts(),
        [1; 4],
        "repair must use fresh call-local projections"
    );
}
#[test]
fn af_delegated_001_unrelated_and_forged_short_circuit() {
    let (_root, a) = fixture();
    let unrelated = SemanticEdge {
        subject: EntityRef::new("angular", "Route", "legacy"),
        object: EntityRef::new("angular", "Guard", "legacy"),
        relation: SemanticRelation::Defines,
        layer: "angular",
        call_evidence: None,
    };
    for batch in [vec![], vec![unrelated]] {
        metrics::reset();
        assert!(a.admits_relation_evidence(&batch));
        assert_eq!(metrics::counts(), [0; 4]);
    }
    let mut bad = selected(&a, 0)[0].clone();
    bad.object.name = "forged".into();
    let later = selected(&a, 2)[0].clone();
    metrics::reset();
    assert!(!a.admits_relation_evidence(&[bad, later]));
    assert_eq!(
        metrics::counts()[2],
        0,
        "later config branch must remain unreachable"
    );
}
