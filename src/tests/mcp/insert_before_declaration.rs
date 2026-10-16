use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({"arguments": args}), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("MCP response")
}

fn fixture(extension: &str) -> &'static str {
    match extension {
        "cs" => {
            "// π unchanged\nusing System;\npublic class Placement {\n  public int First() { return 1; }\n  public int Middle() { return 2; }\n  [Obsolete]\n  public int Last() { return 3; }\n}\n"
        }
        _ => {
            "// π unchanged\nexport class Placement {\n  public first(): number { return 1; }\n  public middle(): number { return 2; }\n  @audit\n  public last(): number { return 3; }\n}\n"
        }
    }
}

fn check(extension: &str, target: &str, boundary: &str, comment: bool) {
    let _serial = crate::protocol::handler_response_serial();
    for eol in ["\n", "\r\n"] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(format!("placement.{extension}"));
        let source = fixture(extension).replace('\n', eol);
        std::fs::write(&path, source.as_bytes()).unwrap();
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
        let unit = if comment {
            "/*PLACEMENT_PROBE*/"
        } else if extension == "cs" {
            "public int Added() { return 9; }\n  "
        } else {
            "public added(): number { return 9; }\n  "
        };
        let offset = source.find(boundary).unwrap();
        let expected = format!(
            "{}{}{}",
            &source[..offset],
            unit.replace('\n', eol),
            &source[offset..]
        );
        let response = dispatch(
            &state,
            "apply_edit",
            json!({
                "filePath": path, "workspaceRoot": root.path(),
                "operations": [{"type":"insert_before", "anchor":format!("Placement.{target}"), "unitText":unit}]
            }),
        );
        assert!(
            response.get("error").is_none(),
            "{extension}/{target}: {response}"
        );
        assert_eq!(
            std::fs::read(&path).unwrap(),
            expected.as_bytes(),
            "exact source {extension}/{target}"
        );
        assert_eq!(
            response["result"]["structuredContent"]["operations"][0]["startByte"],
            offset as u64
        );
        crate::edit::apply::verify_syntax(&expected, extension).unwrap();
    }
}

#[test]
fn insert_before_declaration_csharp_first() {
    check("cs", "First", "public int First", false);
}
#[test]
fn insert_before_declaration_csharp_middle() {
    check("cs", "Middle", "public int Middle", false);
}
#[test]
fn insert_before_declaration_csharp_attribute() {
    check("cs", "Last", "[Obsolete]", false);
}
#[test]
fn insert_before_declaration_typescript_first() {
    check("ts", "first", "public first", false);
}
#[test]
fn insert_before_declaration_typescript_middle() {
    check("ts", "middle", "public middle", false);
}
#[test]
fn insert_before_declaration_typescript_decorator() {
    check("ts", "last", "@audit", false);
}
#[test]
fn insert_before_declaration_parseable_comment_csharp() {
    check("cs", "Last", "[Obsolete]", true);
}
#[test]
fn insert_before_declaration_parseable_comment_typescript() {
    check("ts", "last", "@audit", true);
}

#[test]
fn insert_before_declaration_without_authority_refuses_body_fallback() {
    use crate::edit::{apply, locate::UnitTable, ops::EditOperation};
    use crate::ir::opcodes::CoreOp;
    let source = "class C { m() {} }";
    let table = UnitTable::from_instructions(&[
        CoreOp::DefClass("C1".into(), "C".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "m".into()),
        CoreOp::Body("M1".into(), "{}".into(), Some(14), Some(16)),
    ]);
    assert!(
        apply::apply(
            source,
            &table,
            &[EditOperation::InsertBefore {
                anchor: "C.m".into(),
                unit_text: "/*must refuse*/".into()
            }]
        )
        .is_err(),
        "body-only record must not grant declaration insertion authority"
    );
}
