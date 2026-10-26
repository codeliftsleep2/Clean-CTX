use serde_json::{Value, json};

fn position_question(both: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("token.ts"), "export class Token {}").unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let call = |tool: &str, mut args: Value| {
        args["workspaceRoot"] = json!(p);
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(162),
            tool,
            &json!({"arguments":args}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        response["result"]["structuredContent"].clone()
    };
    let prefix = "import {Component as View} from '@angular/core'; import {Token} from './token';";
    let ordinary = "providers:[{provide:Token,useValue:1}]";
    let view = "viewProviders:[{provide:Token,useValue:1}]";
    let mut ordinary_facts = Value::Null;
    for control in [true, false] {
        let metadata = if control {
            ordinary.to_owned()
        } else if both {
            format!("{ordinary},{view}")
        } else {
            view.to_owned()
        };
        let source = format!("{prefix} @View({{{metadata}}}) export class ProviderHost {{}}");
        std::fs::write(p.join("app.ts"), &source).unwrap();
        call(
            "compress_code_context",
            json!({"filePath":p.join("app.ts"),"fidelity":"high"}),
        );
        let alias = state
            .alias_for_path(&p.join("app.ts").to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let authority = snapshot.authority.typescript_authority.as_ref().unwrap();
        let owner = authority
            .declaration_facets
            .iter()
            .find(|facet| facet.declaration.name == "ProviderHost")
            .unwrap();
        assert!(
            matches!(
                crate::angular_meta::class_role_authority::declaration_role(
                    authority,
                    &owner.declaration
                ),
                Some(crate::angular_meta::decorators::ClassKind::Component)
            ),
            "exact native Component prerequisite"
        );
        let response = call(
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"ProviderHost"}),
        );
        // Do not preselect a new relation name or schema field before the carrier decision.
        let facts: Vec<_> = response["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| {
                edge["subject"]["entity_type"] == "Component"
                    && edge["subject"]["name"] == "ProviderHost"
                    && edge["object"]["entity_type"] == "ProviderToken"
                    && edge["object"]["name"] == "Token"
            })
            .collect();
        let retained_fields: Vec<_> = owner
            .decorators
            .iter()
            .flat_map(|decorator| &decorator.arguments)
            .filter_map(|argument| match &argument.value {
                crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Object(
                    fields,
                ) => Some(fields),
                _ => None,
            })
            .flatten()
            .map(|(field, _)| field.as_str())
            .collect();
        assert!(
            !facts.is_empty(),
            "source-declared provider position must be publicly answerable; control={control}, both={both}, retained fields={retained_fields:?}, public facts={facts:?}"
        );
        if control {
            assert_eq!(
                facts.len(),
                1,
                "nonempty established ordinary registration control"
            );
            ordinary_facts = json!(facts);
        }
        for edge in &facts {
            assert_eq!(
                crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap()
                ),
                crate::dictionary::path::canonical_identity_key(
                    &p.join("token.ts").to_string_lossy()
                )
            );
        }
        if !control && both {
            assert_ne!(
                json!(facts),
                ordinary_facts,
                "same token in both arrays cannot have an indistinguishable public answer from providers alone; retained fields={retained_fields:?}"
            );
        }
        assert!(
            snapshot
                .edges
                .iter()
                .any(|edge| edge.subject.name == "ProviderHost"
                    && edge.object.name == "Token"
                    && edge.subject.canonical.is_some()
                    && edge.object.canonical.is_some()),
            "local registration requires canonical owner/token proof"
        );
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
    }
}

#[test]
fn issue_162_view_provider_position_view_only_public_registration() {
    position_question(false);
}

#[test]
fn issue_162_view_provider_position_same_token_both_arrays_remain_distinct() {
    position_question(true);
}
