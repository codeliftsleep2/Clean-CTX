use super::angular_provider_composition_admission_tests::{capture, fixture};
use crate::layers::meta::semantic::SemanticRelation;

const GROUP: &str = "import {Token,Target,Build} from './token'; export const GROUP=[Token,{provide:Token,useClass:Target},{provide:'factory',useFactory:Build},{provide:'existing',useExisting:Token},{provide:Token,useValue:Target},{provide:'literal',useValue:7}];";
const OWNER: &str = "import {Component} from '@angular/core'; import {GROUP} from './group'; import {ALIAS} from './barrel'; class Local {} @Component({providers:[...GROUP,[...ALIAS],{provide:Local,useValue:1}],viewProviders:[...GROUP,[...ALIAS],{provide:Local,useValue:1}]}) export class Host {}";

fn contributors(p: &std::path::Path) {
    std::fs::write(p.join("group.ts"), GROUP).unwrap();
    std::fs::write(
        p.join("alias.ts"),
        "import {GROUP} from './group'; export const ALIAS=GROUP;",
    )
    .unwrap();
    std::fs::write(p.join("barrel.ts"), "export {ALIAS} from './alias';").unwrap();
}

fn counts(snapshot: &crate::mcp::state::durable_semantics::SemanticSnapshot, count: usize) {
    for relation in [
        SemanticRelation::RegistersProvider,
        SemanticRelation::RegistersViewProvider,
    ] {
        let edges: Vec<_> = snapshot
            .edges
            .iter()
            .filter(|edge| edge.relation == relation)
            .collect();
        assert_eq!(edges.len(), count, "exact independently proved entries");
        assert!(edges.iter().all(|edge| edge.subject.name == "Host"
            && edge.subject.canonical.is_some()
            && edge.object.canonical.is_some()));
    }
    assert!(snapshot.authority.admits_relation_evidence(&snapshot.edges));
}

#[test]
fn issue_162_provider_imported_composition_mixed_strategies_alias_cycles_and_refusals() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    contributors(p);
    let baseline = capture(&state, p, OWNER);
    counts(&baseline, 13);
    for relation in [
        SemanticRelation::ProviderUsesClass,
        SemanticRelation::ProviderUsesFactory,
        SemanticRelation::ProviderUsesExisting,
    ] {
        let edges: Vec<_> = baseline
            .edges
            .iter()
            .filter(|edge| edge.relation == relation)
            .collect();
        assert_eq!(
            edges.len(),
            if relation == SemanticRelation::ProviderUsesClass {
                8
            } else {
                4
            }
        );
        assert!(
            edges
                .iter()
                .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
        );
    }
    assert!(
        baseline
            .edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::ProviderUsesValue)
            .all(|edge| edge.subject.canonical.is_none() && edge.object.canonical.is_none())
    );
    for invalid in [
        GROUP.replace("const GROUP=", "let GROUP="),
        GROUP.replace("const GROUP=", "var GROUP="),
        GROUP.replace("const GROUP=[", "const GROUP=build([").replace("useValue:7}];", "useValue:7}]);"),
        "export const GROUP=missing;".to_owned(),
        "export const GROUP=[...GROUP];".to_owned(),
        "import {GROUP as OTHER} from './cycle'; export const GROUP=[...OTHER];".to_owned(),
        "import {Token,Target} from './token'; export const GROUP=[{provide:Token,provide:Token,useClass:Target},{provide:Token,useExisting:Target,useValue:1},{provide:Token,useValue:Target,useValue:Target}];".to_owned(),
    ] {
        std::fs::write(p.join("cycle.ts"), "import {GROUP as OTHER} from './group'; export const GROUP=[...OTHER];").unwrap();
        std::fs::write(p.join("group.ts"), &invalid).unwrap();
        counts(&capture(&state, p, OWNER), 1);
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), OWNER);
        std::fs::write(p.join("group.ts"), GROUP).unwrap();
        counts(&capture(&state, p, OWNER), 13);
    }
    for route in [
        "export type {ALIAS} from './alias';",
        "export {ALIAS} from './missing';",
        "export * from './alias'; export * from './other';",
    ] {
        std::fs::write(p.join("other.ts"), "export const ALIAS=[];").unwrap();
        std::fs::write(p.join("barrel.ts"), route).unwrap();
        counts(&capture(&state, p, OWNER), 7);
        contributors(p);
        counts(&capture(&state, p, OWNER), 13);
    }
    // An imported alias can return to an owner-source initializer. Offsets keep
    // that actual source, while the imported route remains mandatory proof.
    let entries = GROUP.split_once("export const GROUP=").unwrap().1;
    let owner = OWNER.replace("class Local {}", &format!("import {{Token,Target,Build}} from './token'; export const LOCAL_GROUP={entries} class Local {{}}"));
    std::fs::write(
        p.join("group.ts"),
        "import {LOCAL_GROUP} from './app'; export const GROUP=LOCAL_GROUP;",
    )
    .unwrap();
    counts(&capture(&state, p, &owner), 13);
}

#[test]
fn issue_162_provider_imported_composition_source_hash_span_and_occurrence_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    contributors(p);
    let snapshot = capture(&state, p, OWNER);
    counts(&snapshot, 13);
    let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
    let encoded = serde_json::to_string(authority).unwrap();
    let roundtrip: crate::ir::typescript_resolution::TypeScriptAuthority =
        serde_json::from_str(&encoded).unwrap();
    assert!(roundtrip.admits_relation_evidence(&snapshot.edges));
    for case in [
        "missing",
        "duplicate",
        "source",
        "hash",
        "span",
        "initializer",
        "owner-sites",
        "witness",
    ] {
        let mut forged = authority.clone();
        match case {
            "missing" => forged.demanded_occurrences.clear(),
            "duplicate" => {
                let sites = forged.demanded_occurrences.clone();
                forged.demanded_occurrences.extend(sites);
            }
            "source" => {
                for site in &mut forged.demanded_occurrences {
                    site.source = forged.source.clone();
                }
            }
            "hash" => {
                for site in &mut forged.demanded_occurrences {
                    site.source_hash.push_str("forged");
                }
            }
            "span" => {
                for site in &mut forged.demanded_occurrences {
                    site.start += 1;
                }
            }
            "initializer" => {
                for facet in &mut forged.declaration_facets {
                    if facet.declaration.name == "GROUP" {
                        facet.initializer = None;
                    }
                }
            }
            "owner-sites" => forged.reference_sites.clear(),
            _ => forged.witnesses.clear(),
        }
        assert!(
            !forged.admits_relation_evidence(&snapshot.edges),
            "independent endpoints cannot repair {case} occurrence proof"
        );
    }
    // Numeric offsets coincide by construction; source identity must discriminate.
    std::fs::write(
        p.join("other.ts"),
        GROUP.replace(
            "provide:Token,useClass:Target",
            "provide:Token,useClass:Token",
        ),
    )
    .unwrap();
    let source = OWNER
        .replace(
            "import {GROUP} from './group';",
            "import {GROUP} from './group'; import {GROUP as OTHER} from './other';",
        )
        .replace("...GROUP,[...ALIAS]", "...GROUP,[...OTHER]");
    let snapshot = capture(&state, p, &source);
    counts(&snapshot, 13);
    let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
    let groups: Vec<_> = authority
        .declaration_facets
        .iter()
        .filter(|facet| facet.declaration.name == "GROUP")
        .collect();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].declaration.start, groups[1].declaration.start);
    assert_ne!(groups[0].declaration.source, groups[1].declaration.source);
    let mut crossed = authority.clone();
    let group = groups
        .iter()
        .find(|facet| facet.declaration.source.ends_with("group.ts"))
        .unwrap();
    let other_initializer = groups
        .iter()
        .find(|facet| facet.declaration.source.ends_with("other.ts"))
        .unwrap()
        .initializer
        .clone();
    crossed
        .declaration_facets
        .iter_mut()
        .find(|facet| facet.declaration == group.declaration)
        .unwrap()
        .initializer = other_initializer;
    assert!(
        !crossed.admits_relation_evidence(&snapshot.edges),
        "equal numeric spans cannot cross contributor proof"
    );
    let class_targets: std::collections::BTreeSet<_> = snapshot
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::ProviderUsesClass)
        .map(|edge| edge.object.name.as_str())
        .collect();
    assert_eq!(class_targets, ["Target", "Token"].into_iter().collect());
}
