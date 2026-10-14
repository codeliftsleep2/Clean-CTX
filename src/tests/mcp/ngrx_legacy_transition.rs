//! The Action and source-bound type association precede reducer linkage.
use crate::compression::Fidelity;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

pub(super) const PRIMARY: &str = "import { Action } from '@ngrx/store'; export const SET_AUDIT = '[Audit] Set Audit'; export class AuditAction implements Action { readonly type = SET_AUDIT; } export function auditReducer(state: any, action: Action) { switch(action.type) { case SET_AUDIT: return state; default: return state; } }";

pub(super) fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(140), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"].clone()
}

#[test]
fn ngrx_legacy_transition_public_primary() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("actions.ts");
    let source = format!("{PRIMARY}\n{}", "// economics padding\n".repeat(100));
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut context = PassContext::new(source.clone(), "alias".into(), Fidelity::High);
    context.canonical_path = Some(path.to_string_lossy().into_owned());
    context.language = Some(language.clone());
    context.query_string = query.into();
    CoreIRPass::new().run(&mut context).unwrap();
    assert_eq!(context.ngrx_class_actions.len(), 1);
    assert_eq!(context.ngrx_class_actions[0].name, "AuditAction");
    assert_eq!(context.ngrx_action_type_bindings.len(), 1);
    let binding = &context.ngrx_action_type_bindings[0];
    assert_eq!(binding.declaration_name, "SET_AUDIT");
    assert_eq!(binding.file, path.to_string_lossy());
    // Independent lexical oracle: one program const, one top-level function,
    // its typed Action parameter, and an immediate-body switch/direct case.
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(&source, None).unwrap();
    let program = tree.root_node();
    let mut cursor = program.walk();
    let children: Vec<_> = program
        .named_children(&mut cursor)
        .filter(|n| n.kind() != "comment")
        .collect();
    assert_eq!(children.len(), 4);
    let constant = children[1]
        .child_by_field_name("declaration")
        .unwrap()
        .named_child(0)
        .unwrap();
    assert_eq!(
        (constant.start_byte(), constant.end_byte()),
        (
            binding.declaration_span.start_byte(),
            binding.declaration_span.end_byte()
        )
    );
    let owner = children[3].child_by_field_name("declaration").unwrap();
    assert_eq!(owner.kind(), "function_declaration");
    assert_eq!(
        owner
            .child_by_field_name("name")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "auditReducer"
    );
    let parameter = owner
        .child_by_field_name("parameters")
        .unwrap()
        .named_child(1)
        .unwrap();
    assert_eq!(
        parameter
            .child_by_field_name("pattern")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "action"
    );
    assert_eq!(
        parameter
            .child_by_field_name("type")
            .unwrap()
            .named_child(0)
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "Action"
    );
    let switch = owner
        .child_by_field_name("body")
        .unwrap()
        .named_child(0)
        .unwrap();
    assert_eq!(switch.kind(), "switch_statement");
    let discriminant = switch
        .child_by_field_name("value")
        .unwrap()
        .named_child(0)
        .unwrap();
    assert_eq!(discriminant.kind(), "member_expression");
    assert_eq!(
        discriminant
            .child_by_field_name("object")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "action"
    );
    assert_eq!(
        discriminant
            .child_by_field_name("property")
            .unwrap()
            .utf8_text(source.as_bytes())
            .unwrap(),
        "type"
    );
    let case = switch
        .child_by_field_name("body")
        .unwrap()
        .named_child(0)
        .unwrap();
    let reference = case.child_by_field_name("value").unwrap();
    assert_eq!(reference.kind(), "identifier");
    assert_eq!(reference.utf8_text(source.as_bytes()).unwrap(), "SET_AUDIT");
    // The fixture has no local const/import/parameter that could shadow SET_AUDIT;
    // its program declaration is the only eligible source binding.
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
    );
    let entities = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}),
    );
    assert!(
        entities["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["domain"] == "ngrx"
                && e["entity_type"] == "Action"
                && e["name"] == "AuditAction"
                && e["file"] == path.to_string_lossy().as_ref())
    );
    let answer = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"ngrx","entity_type":"Action","name":"AuditAction"}),
    );
    let edges: Vec<_> = answer["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "TriggersReducer")
        .collect();
    assert_eq!(
        edges.len(),
        1,
        "source-bound legacy transition is missing: {answer}"
    );
    let edge = edges[0];
    for (role, kind, name) in [
        ("subject", "Action", "AuditAction"),
        ("object", "Reducer", "auditReducer"),
    ] {
        assert_eq!(edge[role]["domain"], "ngrx");
        assert_eq!(edge[role]["entity_type"], kind);
        assert_eq!(edge[role]["name"], name);
        assert_eq!(edge[role]["file"], path.to_string_lossy().as_ref());
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
}
