//! Issue #139 declaration-authority query regressions.
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn carrier(domain: &'static str, kind: &'static str, name: &str) -> SemanticEdge {
    let entity = EntityRef::new(domain, kind, name);
    SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: entity.clone(),
        object: entity,
        layer: domain,
        call_evidence: None,
    }
}

fn declaration_query(state: &crate::mcp::McpState, name: &str, domain: &str, kind: &str) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(139),
        "workspace_query",
        &json!({"arguments":{
            "type":"find_entities",
            "name":name,
            "domain":domain,
            "entity_type":kind,
            "declarations_only":true
        }}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

fn entities(response: &Value) -> &[Value] {
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .unwrap()
}

#[test]
fn declaration_mode_excludes_builtin_and_reference_occurrences() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    {
        let mut index = state.workspace_index_lock();
        index.add_edges(
            "service.ts",
            vec![
                carrier("builtin", "Class", "AuditService"),
                carrier("angular", "Service", "AuditService"),
            ],
        );
        index.add_edges(
            "consumer-a.ts",
            vec![SemanticEdge {
                relation: SemanticRelation::Injects,
                subject: EntityRef::new("angular", "Component", "A"),
                object: EntityRef::new("angular", "Service", "AuditService"),
                layer: "angular",
                call_evidence: None,
            }],
        );
        index.add_edges(
            "consumer-b.ts",
            vec![SemanticEdge {
                relation: SemanticRelation::Injects,
                subject: EntityRef::new("angular", "Directive", "B"),
                object: EntityRef::new("angular", "Service", "AuditService"),
                layer: "angular",
                call_evidence: None,
            }],
        );
    }

    let response = declaration_query(&state, "AuditService", "angular", "Service");
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(entities(&response).len(), 1, "{response}");
    assert_eq!(entities(&response)[0]["file"], "service.ts");
}

#[test]
fn declaration_mode_preserves_real_class_spelling_and_distinct_files() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    state
        .workspace_index_lock()
        .add_edges("first.ts", vec![carrier("builtin", "Class", "Class")]);
    state
        .workspace_index_lock()
        .add_edges("second.ts", vec![carrier("builtin", "Class", "Class")]);

    let response = declaration_query(&state, "Class", "builtin", "Class");
    assert!(response.get("error").is_none(), "{response}");
    let mut files: Vec<_> = entities(&response)
        .iter()
        .map(|entity| entity["file"].as_str().unwrap())
        .collect();
    files.sort_unstable();
    assert_eq!(files, vec!["first.ts", "second.ts"]);
}

#[test]
fn occurrence_mode_remains_the_default() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    state.workspace_index_lock().add_edges(
        "consumer.ts",
        vec![SemanticEdge {
            relation: SemanticRelation::Injects,
            subject: EntityRef::new("angular", "Component", "Consumer"),
            object: EntityRef::new("angular", "Service", "OnlyReferenced"),
            layer: "angular",
            call_evidence: None,
        }],
    );
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(139),
        "workspace_query",
        &json!({"arguments":{"type":"find_entities","name":"OnlyReferenced"}}),
        &state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert_eq!(entities(&response).len(), 1, "{response}");
}
