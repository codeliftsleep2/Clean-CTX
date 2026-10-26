use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

pub(super) fn capture(
    state: &crate::mcp::McpState,
    p: &std::path::Path,
    source: &str,
) -> crate::mcp::state::durable_semantics::SemanticSnapshot {
    std::fs::write(p.join("app.ts"), source).unwrap();
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "compress_code_context",
        &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("app.ts"),"fidelity":"high"}}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    state
        .semantic_snapshot(
            &state
                .alias_for_path(&p.join("app.ts").to_string_lossy())
                .unwrap(),
        )
        .unwrap()
}

pub(super) fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("token.ts"),
        "export class Token {} export class Target {} export function Build(){}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

#[test]
fn issue_162_provider_composition_mixed_alias_cycles_strategies_and_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = "import {Component} from '@angular/core'; import {Token,Target,Build} from './token'; class Local {} const GROUP=[{provide:Token,useClass:Target},{provide:Token,useFactory:Build},{provide:'api.alias',useExisting:Token},{provide:Token,useValue:Target}]; const ALIAS=GROUP; const CYCLE=[...CYCLE]; @Component({providers:[...GROUP,[...ALIAS],{provide:Local,useValue:1},...CYCLE],viewProviders:[...GROUP,[...ALIAS],{provide:Local,useValue:1},...CYCLE]}) export class Host {}";
    let baseline = capture(&state, p, source);
    for relation in [
        SemanticRelation::RegistersProvider,
        SemanticRelation::RegistersViewProvider,
    ] {
        let registrations: Vec<_> = baseline
            .edges
            .iter()
            .filter(|edge| edge.relation == relation)
            .collect();
        assert_eq!(
            registrations.len(),
            9,
            "repeated proved paths plus independent direct sibling"
        );
        assert!(
            registrations
                .iter()
                .all(|edge| edge.subject.canonical.is_some() && edge.object.canonical.is_some())
        );
    }
    assert!(baseline.authority.admits_relation_evidence(&baseline.edges));
    for invalid in [
        source.replace("const GROUP=", "let GROUP="),
        source.replace("const GROUP=", "var GROUP="),
        source
            .replace("const ALIAS=GROUP", "const ALIAS=missing")
            .replace("...GROUP", "...missing"),
        source
            .replace("...GROUP", "...GROUP.slice()")
            .replace("...ALIAS", "...ALIAS.slice()"),
        source
            .replace("const GROUP=[", "const GROUP=build([")
            .replace("useValue:Target}];", "useValue:Target}]);"),
        source
            .replace(
                "const CYCLE=[...CYCLE]",
                "const CYCLE=[...OTHER]; const OTHER=[...CYCLE]",
            )
            .replace("...GROUP", "...CYCLE")
            .replace("...ALIAS", "...CYCLE"),
    ] {
        let refused = capture(&state, p, &invalid);
        for relation in [
            SemanticRelation::RegistersProvider,
            SemanticRelation::RegistersViewProvider,
        ] {
            let registrations: Vec<_> = refused
                .edges
                .iter()
                .filter(|edge| edge.relation == relation)
                .collect();
            assert_eq!(
                registrations.len(),
                1,
                "unsupported/cyclic path retains only proved Local sibling: {invalid}"
            );
            assert_eq!(registrations[0].object.name, "Local");
        }
        assert!(refused.authority.admits_relation_evidence(&refused.edges));
        assert_eq!(
            capture(&state, p, source)
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersViewProvider)
                .count(),
            9
        );
    }
}

#[test]
fn issue_162_provider_composition_refuses_missing_crossed_and_shifted_native_paths() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let source = "import {Component} from '@angular/core'; import {Token,Target} from './token'; const GROUP=[{provide:Token,useValue:1}]; const OTHER=[{provide:Target,useValue:2}]; @Component({providers:GROUP,viewProviders:OTHER}) export class Host {}";
    let baseline = capture(&state, root.path(), source);
    let ordinary: Vec<_> = baseline
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
        .cloned()
        .collect();
    assert_eq!(
        ordinary.len(),
        1,
        "nonempty exact composed registration baseline"
    );
    let authority = baseline.authority.typescript_authority.as_ref().unwrap();
    assert!(authority.admits_relation_evidence(&baseline.edges));
    use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
    let other = authority
        .declaration_facets
        .iter()
        .find(|facet| facet.declaration.name == "OTHER")
        .unwrap();
    for case in ["missing", "crossed", "shifted", "reference", "native"] {
        let mut forged = authority.clone();
        match case {
            "missing" => {
                forged
                    .declaration_facets
                    .iter_mut()
                    .find(|facet| facet.declaration.name == "GROUP")
                    .unwrap()
                    .initializer = None
            }
            "crossed" => {
                forged
                    .declaration_facets
                    .iter_mut()
                    .find(|facet| facet.declaration.name == "GROUP")
                    .unwrap()
                    .initializer = other.initializer.clone()
            }
            "native" => forged.reference_sites.clear(),
            _ => {
                let owner = forged
                    .declaration_facets
                    .iter_mut()
                    .find(|facet| facet.declaration.name == "Host")
                    .unwrap();
                for argument in owner
                    .decorators
                    .iter_mut()
                    .flat_map(|decorator| &mut decorator.arguments)
                {
                    if let ExpressionValue::Object(fields) = &mut argument.value {
                        let value = &mut fields
                            .iter_mut()
                            .find(|(field, _)| field == "providers")
                            .unwrap()
                            .1;
                        if case == "shifted" {
                            value.start += 1;
                        } else {
                            value.value = ExpressionValue::Reference(
                                crate::ir::typescript_resolution::Resolution::Resolved(
                                    other.declaration.clone(),
                                ),
                            );
                        }
                    }
                }
            }
        }
        assert!(
            !forged.admits_relation_evidence(&ordinary),
            "independent endpoints cannot repair {case} path proof"
        );
    }
}
