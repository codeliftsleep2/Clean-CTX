use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({"arguments": args}), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("response")
}

fn state_for(
    extension: &str,
    source: &str,
) -> (tempfile::TempDir, std::path::PathBuf, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join(format!("overloads.{extension}"));
    std::fs::write(&path, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let provided = dispatch(
        &state,
        "provide_code_context",
        json!({
            "filePath": path, "workspaceRoot": root.path(), "fidelity": "edit"
        }),
    );
    assert!(provided.get("error").is_none(), "{provided}");
    (root, path, state)
}

fn replace(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    path: &std::path::Path,
    target: &str,
    old: &str,
    new: &str,
) -> Value {
    dispatch(
        state,
        "apply_edit",
        json!({
            "filePath": path, "workspaceRoot": root,
            "operations": [{"type":"replace_body", "target":target,
                "expectedOldText":old, "newText":new}]
        }),
    )
}

#[test]
fn issue_136_csharp_three_overloads_are_independently_editable() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Service {\n  int Run(int value) { return 1; }\n  int Run(string value) { return 2; }\n  int Run(bool value) { return 3; }\n}\n";
    for (target, old, new, untouched) in [
        (
            "Service.Run(int)",
            "{ return 1; }",
            "{ return 11; }",
            "return 2;",
        ),
        (
            "Service.Run(string)",
            "{ return 2; }",
            "{ return 22; }",
            "return 3;",
        ),
        (
            "Service.Run(bool)",
            "{ return 3; }",
            "{ return 33; }",
            "return 1;",
        ),
    ] {
        let (root, path, state) = state_for("cs", source);
        let response = replace(&state, root.path(), &path, target, old, new);
        assert!(response.get("error").is_none(), "{target}: {response}");
        let disk = std::fs::read_to_string(path).unwrap();
        assert!(disk.contains(new));
        assert!(disk.contains(untouched));
    }
}

#[test]
fn issue_136_typescript_overloads_are_signature_addressed() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Service {\n  run(value: string) { return 's'; }\n  run(value: number) { return 'n'; }\n}\n";
    let (root, path, state) = state_for("ts", source);
    let response = replace(
        &state,
        root.path(),
        &path,
        "Service.run(number)",
        "{ return 'n'; }",
        "{ return 'number'; }",
    );
    assert!(response.get("error").is_none(), "{response}");
    let disk = std::fs::read_to_string(path).unwrap();
    assert!(disk.contains("return 's';"));
    assert!(disk.contains("return 'number';"));
}

#[test]
fn issue_136_legacy_selector_refuses_same_owner_ambiguity_without_writing() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Service {\n  int Run(int value) { return 1; }\n  int Run(string value) { return 2; }\n}\n";
    let (root, path, state) = state_for("cs", source);
    let response = replace(
        &state,
        root.path(),
        &path,
        "Service.Run",
        "{ return 2; }",
        "{ return 9; }",
    );
    assert_eq!(response["error"]["data"]["kind"], "ambiguous_target");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn issue_136_found_property_is_uneditable_not_missing_and_source_is_unchanged() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Service {\n  int Value { get; set; }\n  int Run() { return 1; }\n}\n";
    let (root, path, state) = state_for("cs", source);
    let response = replace(&state, root.path(), &path, "Service.Value", "", "{}");
    assert_eq!(response["error"]["data"]["kind"], "uneditable_target");
    assert_eq!(
        response["error"]["data"]["targetKind"],
        "property_declaration"
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn issue_136_missing_target_remains_distinct_from_uneditable() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Service { int Run() { return 1; } }";
    let (root, path, state) = state_for("cs", source);
    let response = replace(&state, root.path(), &path, "Service.Missing", "", "{}");
    assert_eq!(response["error"]["data"]["kind"], "not_found");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn issue_136_named_uneditable_constructs_report_their_parser_kind() {
    let _serial = crate::protocol::handler_response_serial();
    let cases = [
        (
            "cs",
            "class S { int Value { get; set; } }",
            "S.Value",
            "property_declaration",
        ),
        (
            "cs",
            "class S { int value; }",
            "S.value",
            "field_declaration",
        ),
        (
            "cs",
            "class S { public static S operator +(S a, S b) => a; }",
            "S.operator",
            "operator_declaration",
        ),
        (
            "cs",
            "record Audit(int Value);",
            "Audit",
            "record_declaration",
        ),
        (
            "cs",
            "interface IAudit { void Run(); }",
            "IAudit.Run",
            "method_declaration",
        ),
        (
            "ts",
            "class S { run = () => 1; }",
            "S.run",
            "public_field_definition",
        ),
        (
            "ts",
            "type Audit = { value: string };",
            "Audit",
            "type_alias_declaration",
        ),
        (
            "ts",
            "it('works', () => { expect(true); });",
            "it",
            "spec_test_callback",
        ),
    ];
    for (extension, source, target, kind) in cases {
        let (root, path, state) = state_for(extension, source);
        let response = replace(&state, root.path(), &path, target, "", "{}");
        assert_eq!(
            response["error"]["data"]["kind"], "uneditable_target",
            "{extension}/{target}: {response}"
        );
        assert_eq!(response["error"]["data"]["targetKind"], kind);
        assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    }
}

#[test]
fn issue_136_expression_bodied_method_is_edited_with_its_body_authority() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class S { int Value(int input) => input + 1; }";
    let (root, path, state) = state_for("cs", source);
    let response = replace(
        &state,
        root.path(),
        &path,
        "S.Value(int)",
        " input + 1;",
        " input + 2;",
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "class S { int Value(int input) => input + 2; }"
    );
}

#[test]
fn issue_136_bare_class_name_does_not_silently_select_constructor() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Service { Service() {} }";
    let (root, path, state) = state_for("cs", source);
    let response = replace(
        &state,
        root.path(),
        &path,
        "Service",
        "{}",
        "{ changed(); }",
    );
    assert_eq!(response["error"]["data"]["kind"], "ambiguous_target");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn issue_136_typescript_getter_and_setter_use_signature_discriminators() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class S {\n  get value() { return 1; }\n  set value(input: number) { this.saved = input; }\n}\n";
    let (root, path, state) = state_for("ts", source);
    let response = replace(
        &state,
        root.path(),
        &path,
        "S.value(number)",
        "{ this.saved = input; }",
        "{ this.saved = input + 1; }",
    );
    assert!(response.get("error").is_none(), "{response}");
    let disk = std::fs::read_to_string(path).unwrap();
    assert!(disk.contains("get value() { return 1; }"));
    assert!(disk.contains("this.saved = input + 1;"));
}

#[test]
fn issue_136_nested_and_outer_same_name_members_remain_owner_qualified() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "class Outer {\n  int Run(int value) { return 1; }\n  class Inner { int Run(int value) { return 2; } }\n}\n";
    let (root, path, state) = state_for("cs", source);
    let response = replace(
        &state,
        root.path(),
        &path,
        "Inner.Run(int)",
        "{ return 2; }",
        "{ return 22; }",
    );
    assert!(response.get("error").is_none(), "{response}");
    let disk = std::fs::read_to_string(path).unwrap();
    assert!(disk.contains("Outer {\n  int Run(int value) { return 1; }"));
    assert!(disk.contains("Inner { int Run(int value) { return 22; }"));
}
