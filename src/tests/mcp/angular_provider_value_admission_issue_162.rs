use crate::layers::meta::semantic::SemanticRelation;
use serde_json::json;

#[test]
fn issue_162_provider_value_reference_refusals_repair_and_noncanonical_boundary() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class Token {}").unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function Component(metadata:unknown){return metadata;}",
    )
    .unwrap();
    let values = "export class Value {}";
    std::fs::write(p.join("values.ts"), values).unwrap();
    let source = "import {Component as View} from '@angular/core'; import {Token} from './token'; import * as values from './values'; @View({providers:[{provide:Token,useValue:values.Value}]}) export class ProviderHost {}";
    std::fs::write(p.join("app.ts"), source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let capture = || {
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            "compress_code_context",
            &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("app.ts"),"fidelity":"high"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let alias = state
            .alias_for_path(&p.join("app.ts").to_string_lossy())
            .unwrap();
        state.semantic_snapshot(&alias).unwrap()
    };
    let baseline = capture();
    let registrations: Vec<_> = baseline
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
        .collect();
    assert_eq!(
        registrations.len(),
        1,
        "nonempty native value reference control"
    );
    assert!(registrations[0].object.canonical.is_some());
    let projections: Vec<_> = baseline
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::ProviderUsesValue)
        .collect();
    assert_eq!(projections.len(), 1);
    assert_eq!(projections[0].object.name, "Value");
    assert!(
        projections[0].subject.canonical.is_none() && projections[0].object.canonical.is_none()
    );
    for invalid in [
        source.replace("from '@angular/core'", "from './foreign'"),
        source.replace("import * as values", "import type * as values"),
        source.replace("values.Value", "values.Missing"),
        source.replace(
            "useValue:values.Value",
            "useValue:values.Value,useValue:values.Value",
        ),
        source.replace("provide:Token", "provide:Token,provide:Token"),
        source.replace(
            "useValue:values.Value",
            "useValue:values.Value,useFactory:values.Value",
        ),
        source.replace("useValue:values.Value", "useValue:values.Value()"),
    ] {
        std::fs::write(p.join("app.ts"), invalid).unwrap();
        assert!(
            !capture().edges.iter().any(|edge| matches!(
                edge.relation,
                SemanticRelation::RegistersProvider | SemanticRelation::ProviderUsesValue
            )),
            "invalid entry cannot publish a registration or value projection"
        );
        std::fs::write(p.join("app.ts"), source).unwrap();
        assert_eq!(
            capture()
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
                .count(),
            1
        );
    }
    for invalid in [
        "export interface Value {}",
        "export class Value {} export class Value {}",
    ] {
        std::fs::write(p.join("values.ts"), invalid).unwrap();
        assert!(
            !capture()
                .edges
                .iter()
                .any(|edge| edge.relation == SemanticRelation::RegistersProvider)
        );
        std::fs::write(p.join("values.ts"), values).unwrap();
        assert_eq!(
            capture()
                .edges
                .iter()
                .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
                .count(),
            1
        );
    }
    let authority = baseline.authority.typescript_authority.as_ref().unwrap();
    use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
    let array = authority
        .declaration_facets
        .iter()
        .flat_map(|facet| &facet.decorators)
        .flat_map(|decorator| &decorator.arguments)
        .find_map(|argument| {
            let ExpressionValue::Object(fields) = &argument.value else {
                return None;
            };
            fields
                .iter()
                .find(|(key, _)| key == "providers")
                .map(|(_, value)| value)
        })
        .unwrap();
    let start = source.find("values.Value").unwrap();
    let span = crate::ir::invocation_evidence::SourceSpan::new(start, start + "values.Value".len())
        .unwrap();
    assert!(
        crate::angular_meta::provider_semantics::proved_reference(
            authority, array, "useValue", span
        )
        .is_some()
    );
    let shifted =
        crate::ir::invocation_evidence::SourceSpan::new(start + 1, span.end_byte()).unwrap();
    assert!(
        crate::angular_meta::provider_semantics::proved_reference(
            authority, array, "useValue", shifted
        )
        .is_none()
    );
    let mut missing = authority.clone();
    missing.reference_sites.clear();
    assert!(
        crate::angular_meta::provider_semantics::proved_reference(
            &missing, array, "useValue", span
        )
        .is_none()
    );
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
}
