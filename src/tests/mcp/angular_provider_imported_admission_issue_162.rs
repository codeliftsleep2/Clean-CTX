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
    let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
    let owner = authority
        .declaration_facets
        .iter()
        .find(|facet| facet.declaration.name == "Host")
        .unwrap();
    use crate::angular_meta::decorators::ClassKind;
    let kind =
        crate::angular_meta::class_role_authority::declaration_role(authority, &owner.declaration)
            .unwrap();
    let owner_type = match kind {
        ClassKind::Component => "Component",
        ClassKind::Directive => "Directive",
        ClassKind::Module => "Module",
        _ => panic!("unexpected owner"),
    };
    for relation in [
        SemanticRelation::RegistersProvider,
        SemanticRelation::RegistersViewProvider,
    ] {
        let edges: Vec<_> = snapshot
            .edges
            .iter()
            .filter(|edge| edge.relation == relation)
            .collect();
        assert_eq!(
            edges.len(),
            if relation == SemanticRelation::RegistersViewProvider && kind != ClassKind::Component {
                0
            } else {
                count
            },
            "exact independently proved entries"
        );
        assert!(
            edges
                .iter()
                .all(|edge| edge.subject.entity_type == owner_type)
        );
        assert!(edges.iter().all(|edge| edge.subject.name == "Host"
            && edge.subject.canonical.is_some()
            && edge.object.canonical.is_some()));
    }
    assert!(snapshot.authority.admits_relation_evidence(&snapshot.edges));
}

#[test]
fn issue_162_provider_imported_composition_mixed_strategies_alias_cycles_and_refusals() {
    mixed_case("Component");
}

#[test]
fn issue_162_provider_owner_composition_mixed_strategies_alias_cycles_and_refusals() {
    for api in ["Directive", "NgModule"] {
        mixed_case(api);
    }
}

fn mixed_case(api: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), "export declare function Component(metadata:unknown):any; export declare function Directive(metadata:unknown):any; export declare function NgModule(metadata:unknown):any;").unwrap();
    let owner_source = OWNER.replace("Component", api);
    contributors(p);
    let baseline = capture(&state, p, &owner_source);
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
            (if relation == SemanticRelation::ProviderUsesClass {
                4
            } else {
                2
            }) * if api == "Component" { 2 } else { 1 }
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
        counts(&capture(&state, p, &owner_source), 1);
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), owner_source);
        std::fs::write(p.join("group.ts"), GROUP).unwrap();
        counts(&capture(&state, p, &owner_source), 13);
    }
    for route in [
        "export type {ALIAS} from './alias';",
        "export {ALIAS} from './missing';",
        "export * from './alias'; export * from './other';",
    ] {
        std::fs::write(p.join("other.ts"), "export const ALIAS=[];").unwrap();
        std::fs::write(p.join("barrel.ts"), route).unwrap();
        counts(&capture(&state, p, &owner_source), 7);
        contributors(p);
        counts(&capture(&state, p, &owner_source), 13);
    }
    // An imported alias can return to an owner-source initializer. Offsets keep
    // that actual source, while the imported route remains mandatory proof.
    let entries = GROUP.split_once("export const GROUP=").unwrap().1;
    let owner = owner_source.replace("class Local {}", &format!("import {{Token,Target,Build}} from './token'; export const LOCAL_GROUP={entries} class Local {{}}"));
    std::fs::write(
        p.join("group.ts"),
        "import {LOCAL_GROUP} from './app'; export const GROUP=LOCAL_GROUP;",
    )
    .unwrap();
    counts(&capture(&state, p, &owner), 13);
}

#[test]
fn issue_162_provider_imported_composition_source_hash_span_and_occurrence_refusal() {
    proof_case("Component");
}

#[test]
fn issue_162_provider_owner_composition_source_hash_span_and_occurrence_refusal() {
    for api in ["Directive", "NgModule"] {
        proof_case(api);
    }
}

fn proof_case(api: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"), "export declare function Component(metadata:unknown):any; export declare function Directive(metadata:unknown):any; export declare function NgModule(metadata:unknown):any;").unwrap();
    let owner_source = OWNER.replace("Component", api);
    contributors(p);
    let snapshot = capture(&state, p, &owner_source);
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
        "container",
        "owner-native",
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
            "container" => {
                let owner = forged
                    .declaration_facets
                    .iter_mut()
                    .find(|facet| facet.declaration.name == "Host")
                    .unwrap();
                for decorator in &mut owner.decorators {
                    decorator.arguments.clear();
                }
                assert!(
                    crate::angular_meta::class_role_authority::declaration_role(
                        &forged,
                        &authority
                            .declaration_facets
                            .iter()
                            .find(|facet| facet.declaration.name == "Host")
                            .unwrap()
                            .declaration
                    )
                    .is_some(),
                    "owner identity survives missing metadata membership"
                );
            }
            "owner-native" => forged.package_exports.clear(),
            "owner-sites" => forged.reference_sites.clear(),
            _ => forged.witnesses.clear(),
        }
        assert!(
            !forged.admits_relation_evidence(&snapshot.edges),
            "independent endpoints cannot repair {case} occurrence proof"
        );
    }
    let registrations: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
        .cloned()
        .collect();
    if api != "Component" {
        let mut wrong_position = registrations.clone();
        for edge in &mut wrong_position {
            edge.relation = SemanticRelation::RegistersViewProvider;
        }
        assert!(
            !authority.admits_relation_evidence(&wrong_position),
            "ordinary owner cannot gain view position authority"
        );
    }
    let mut relabeled = registrations;
    for edge in &mut relabeled {
        edge.subject.entity_type = if api == "Component" {
            "Directive"
        } else {
            "Component"
        };
    }
    assert!(
        !authority.admits_relation_evidence(&relabeled),
        "actual canonical owner role cannot be relabeled"
    );
    // Numeric offsets coincide by construction; source identity must discriminate.
    std::fs::write(
        p.join("other.ts"),
        GROUP.replace(
            "provide:Token,useClass:Target",
            "provide:Token,useClass:Token",
        ),
    )
    .unwrap();
    let source = owner_source
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
    let direct = format!(
        "import {{{api}}} from '@angular/core'; import {{Token}} from './token'; @{api}({{providers:[{{provide:Token,useValue:1}}]}}) export class Host {{}}"
    );
    let direct = capture(&state, p, &direct);
    let registrations: Vec<_> = direct
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
        .collect();
    assert_eq!(
        registrations.len(),
        1,
        "one canonical direct ordinary registration"
    );
    assert!(
        registrations
            .iter()
            .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
    );
    assert_eq!(
        direct
            .edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::RegistersViewProvider)
            .count(),
        0,
        "direct control declares no viewProviders field"
    );
    assert!(direct.authority.admits_relation_evidence(&direct.edges));
    let mut authority = direct
        .authority
        .typescript_authority
        .as_ref()
        .unwrap()
        .clone();
    authority.demanded_occurrences.clear();
    assert!(
        authority.admits_relation_evidence(&direct.edges),
        "valid direct canonical membership does not require imported/composed occurrence proof"
    );
}
