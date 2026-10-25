use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use serde_json::json;

#[test]
fn issue_162_provider_strategy_pair_refuses_crossed_and_missing_occurrences() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    for name in ["left.ts", "right.ts"] {
        std::fs::write(
            p.join(name),
            "export class Token {} export class Implementation {}",
        )
        .unwrap();
    }
    std::fs::write(
        p.join("app.ts"),
        r#"
import {Component as View} from '@angular/core';
import * as left from './left';
import * as right from './right';
const text = 'provide:Phantom';
function noise<T>(left:T){return left;}
@View({providers:[
 {provide:left.Token,useClass:left.Implementation},
 {provide:right.Token,useClass:right.Implementation},
 {provide:left.Token,useClass:left.Implementation},
 {provide:'audit.api',useClass:left.Implementation}
]}) export class ProviderHost {}
"#,
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
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
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let pairs: Vec<SemanticEdge> = snapshot
        .edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::ProviderUsesClass)
        .cloned()
        .collect();
    assert_eq!(
        pairs.len(),
        4,
        "nonempty declaration, repeated and string pairs"
    );
    assert!(snapshot.authority.admits_relation_evidence(&pairs));
    assert_eq!(pairs[0].subject.canonical, pairs[2].subject.canonical);
    assert_eq!(pairs[0].object.canonical, pairs[2].object.canonical);
    assert_eq!(pairs[3].subject.name, "\"audit.api\"");
    for change in 0..6 {
        let mut forged = pairs[0].clone();
        match change {
            0 => forged.object = pairs[1].object.clone(),
            1 => forged.subject = pairs[1].subject.clone(),
            2 => forged.subject = pairs[3].subject.clone(),
            3 => {
                forged
                    .object
                    .canonical
                    .as_mut()
                    .unwrap()
                    .declaration_mut()
                    .unwrap()
                    .source_hash = "forged".into()
            }
            4 => forged.object.canonical = None,
            _ => forged.layer = "typescript",
        }
        // The string token actually relates to left, so cross it with right.
        if change == 2 {
            forged.object = pairs[1].object.clone();
        }
        assert!(
            !snapshot.authority.admits_relation_evidence(&[forged]),
            "forgery {change}"
        );
    }
    let mut missing = snapshot.authority.clone();
    let authority = missing.typescript_authority.as_mut().unwrap();
    for facet in &mut authority.declaration_facets {
        for decorator in &mut facet.decorators {
            for argument in &mut decorator.arguments {
                remove_strategy(argument);
            }
        }
    }
    assert!(
        !missing.admits_relation_evidence(&pairs),
        "endpoint inventory cannot replace provider occurrences"
    );
}

fn remove_strategy(
    expression: &mut crate::ir::typescript_resolution::declaration_facets::ExpressionFacet,
) {
    use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
    match &mut expression.value {
        ExpressionValue::Object(fields) => {
            fields.retain(|(name, _)| name != "useClass");
            for (_, value) in fields {
                remove_strategy(value);
            }
        }
        ExpressionValue::Array(entries) => {
            for entry in entries {
                remove_strategy(entry);
            }
        }
        _ => {}
    }
}
