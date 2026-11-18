//! Qualified owner identity must come from the class declaration, not decorators.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

fn check(declaration: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("owners.ts");
    let source = format!(
        "import {{ Injectable }} from '@angular/core';\nclass Alpha {{ read() {{ return 'ALPHA_BODY'; }} }}\n{declaration}\n{}",
        "// unchanged economics padding\n".repeat(100)
    );
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    assert!(source.contains("class OwnerProbe"));
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    {
        let (selector, included, excluded) = ("OwnerProbe.read", "OWNER_BODY", "ALPHA_BODY");
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(
            &json!(1),
            "provide_code_context",
            &json!({"arguments": {
                "workspaceRoot":root.path(), "filePath":path, "fidelity":"edit",
                "focusMethods":[selector]
            }}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{selector}: {response}");
        assert_eq!(
            response["result"]["_meta"]["content_kind"],
            "skeleton_with_focused_verbatim_bodies"
        );
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(included), "{selector}: {text}");
        assert!(!text.contains(excluded), "wrong owner's body: {text}");
        assert!(
            text.contains("OwnerProbe"),
            "missing class identity: {text}"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    }
}

#[test]
fn decorated_owner_identity_plain() {
    check("class OwnerProbe { read() { return 'OWNER_BODY'; } }");
}
#[test]
fn decorated_owner_identity_exported_plain() {
    check("export class OwnerProbe { read() { return 'OWNER_BODY'; } }");
}
#[test]
fn decorated_owner_identity_non_exported() {
    check("@Injectable()\nclass OwnerProbe { read() { return 'OWNER_BODY'; } }");
}
#[test]
fn decorated_owner_identity_exported() {
    check("@Injectable()\nexport class OwnerProbe { read() { return 'OWNER_BODY'; } }");
}
#[test]
fn decorated_owner_identity_multiline_custom() {
    check(
        "@Audit({\n label: 'class Fake',\n nested: { name: 'Other' }\n})\nclass OwnerProbe { read() { return 'OWNER_BODY'; } }",
    );
}
#[test]
fn decorated_owner_identity_multiple_custom() {
    check("@First()\n@Second('Alpha')\nclass OwnerProbe { read() { return 'OWNER_BODY'; } }");
}

#[test]
fn decorated_owner_identity_same_line_and_generic() {
    check("@Audit() class OwnerProbe<T> { read() { return 'OWNER_BODY'; } }");
}

#[test]
fn decorated_owner_identity_structural_member_ownership() {
    use crate::compression::Fidelity;
    use crate::ir::{compiler::IRCompiler, hierarchical::ir_to_hierarchical};
    let source = "class Alpha { value: string; read() { return 'ALPHA_BODY'; } }\n@Audit({name:'Fake'})\nabstract class OwnerProbe<T> { value: number; read() { return 'OWNER_BODY'; } }";
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(
        crate::ir::layers::typescript::TypeScriptLayer::new(),
    ));
    for fidelity in [
        Fidelity::Low,
        Fidelity::Medium,
        Fidelity::High,
        Fidelity::Edit,
    ] {
        let ir = compiler
            .compile(
                source,
                "owners.ts",
                crate::compression::language::safe_typescript_language().unwrap(),
                crate::queries::TS_QUERY,
                fidelity,
                None,
            )
            .unwrap();
        let hierarchy = ir_to_hierarchical(&ir);
        assert_eq!(hierarchy.classes.len(), 2);
        for owner in ["Alpha", "OwnerProbe"] {
            let class = hierarchy
                .classes
                .iter()
                .find(|class| class.name == owner)
                .unwrap();
            assert_eq!(class.methods.len(), 1);
            assert_eq!(class.methods[0].name, "read");
            assert_eq!(class.fields.len(), 1);
            assert_eq!(class.fields[0].name, "value");
        }
        let selectors = ["Alpha.read".into(), "OwnerProbe.read".into()]
            .into_iter()
            .collect();
        assert_eq!(
            crate::ir::focus::resolve_focus_method_ids(&hierarchy, &selectors)
                .unwrap()
                .len(),
            2
        );
        let bare = ["read".into()].into_iter().collect();
        assert!(matches!(
            crate::ir::focus::resolve_focus_method_ids(&hierarchy, &bare),
            Err(crate::ir::focus::FocusResolutionError::AmbiguousBareMethod(
                _
            ))
        ));
    }
}

#[test]
fn decorated_owner_identity_registered_edit_exact_target() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("owners.ts");
    let source = "class Alpha { read() { return 'ALPHA_BODY'; } }\n@Audit()\nclass OwnerProbe { value: number; read() { return 'OWNER_BODY'; } }\n";
    std::fs::write(&path, source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (tool, arguments) in [
        (
            "provide_code_context",
            json!({"workspaceRoot":root.path(), "filePath":path, "fidelity":"edit"}),
        ),
        (
            "apply_edit",
            json!({"workspaceRoot":root.path(), "filePath":path, "operations":[{
                "type":"replace_body", "target":"OwnerProbe.read",
                "expectedOldText":"{ return 'OWNER_BODY'; }", "newText":"{ return 'OWNER_CHANGED'; }"
            }]}),
        ),
    ] {
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(&json!(1), tool, &json!({"arguments":arguments}), &state);
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{tool}: {response}");
    }
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        source.replace("OWNER_BODY", "OWNER_CHANGED")
    );
}
