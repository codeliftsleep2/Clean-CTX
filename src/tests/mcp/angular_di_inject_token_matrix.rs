use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(138), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"].clone()
}

fn source(fields: &str, decorator: &str) -> String {
    format!(
        "import {{ Injectable, Component, Directive, Pipe, inject }} from '@angular/core';\n@Injectable() export class AuditService {{}}\n@Injectable() export class LoggerService {{}}\n{decorator} export class AuditConsumer {{ {fields} }}\n{}",
        "// unchanged economics padding\n".repeat(100)
    )
}

fn assert_edges(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    kind: &str,
    expected: &[&str],
) {
    let answer = call(
        state,
        "workspace_query",
        json!({"workspaceRoot":root,"type":"forward_edges","domain":"angular","entity_type":kind,"name":"AuditConsumer"}),
    );
    let mut targets: Vec<_> = answer["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .map(|edge| {
            assert_eq!(edge["subject"]["name"], "AuditConsumer");
            assert_eq!(edge["subject"]["entity_type"], kind);
            assert_eq!(edge["subject"]["domain"], "angular");
            assert_eq!(edge["object"]["domain"], "angular");
            assert_eq!(edge["object"]["entity_type"], "Service");
            edge["object"]["name"].as_str().unwrap()
        })
        .collect();
    targets.sort();
    let mut expected = expected.to_vec();
    expected.sort();
    assert_eq!(targets, expected, "{answer}");
}

fn check(fields: &str, decorator: &str, kind: &str, expected: &[&str]) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("consumer.ts");
    let source = source(fields, decorator);
    std::fs::write(&path, &source).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let context = call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
    );
    let text = context["content"][0]["text"].as_str().unwrap();
    let markers: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("T @injects = ["))
        .filter_map(|line| line.strip_suffix(']'))
        .flat_map(|line| line.split(','))
        .collect();
    for target in expected {
        assert!(
            markers.contains(target),
            "dependency missing from marker surface: {text}"
        );
    }
    assert!(
        !text.contains("T @injects ?"),
        "ambiguous receiver marker survived: {text}"
    );
    if expected.is_empty() {
        assert!(
            !text.contains("T @injects"),
            "negative initializer gained DI marker: {text}"
        );
    }
    for _ in 0..2 {
        assert_edges(&state, root.path(), kind, expected);
    }
    // Defines is declaration authority; symbolic dependency endpoints are not.
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut compiler = crate::ir::compiler::IRCompiler::new();
    compiler
        .compile_focused(
            &source,
            "matrix",
            Some(path.to_str().unwrap()),
            language,
            query,
            crate::compression::Fidelity::High,
            None,
            None,
        )
        .unwrap();
    assert!(!compiler.semantic_edges.iter().any(|edge| edge.relation
        == crate::layers::meta::semantic::SemanticRelation::Defines
        && edge.subject.name == "MissingService"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn angular_di_inject_token_basic_matrix_and_modifiers() {
    for fields in [
        "private auditService = inject(AuditService);",
        "private completelyDifferentName=inject(AuditService);",
        "private readonly completelyDifferentName = inject( AuditService );",
        "public unrelated = inject(AuditService);",
        "protected unrelated = inject(\n AuditService\n);",
        "private unrelated = inject(AuditService, { optional: true });",
    ] {
        check(fields, "@Injectable()", "Service", &["AuditService"]);
    }
    check(
        "private strangeOne = inject(AuditService); private unrelatedTwo = inject(LoggerService);",
        "@Injectable()",
        "Service",
        &["AuditService", "LoggerService"],
    );
    check(
        "private first = inject(AuditService); private second = inject(AuditService);",
        "@Injectable()",
        "Service",
        &["AuditService"],
    );
}

#[test]
fn angular_di_inject_token_descendants_and_other_callees_are_negative() {
    for initializer in [
        "condition ? inject(AuditService) : otherValue",
        "wrapper(inject(AuditService))",
        "makeService(AuditService)",
        "obj.inject(AuditService)",
        "(inject(AuditService))",
        "inject?.(AuditService)",
        "inject(namespace.AuditService)",
    ] {
        check(
            &format!("private dependency = {initializer};"),
            "@Injectable()",
            "Service",
            &[],
        );
    }
}

#[test]
fn angular_di_inject_token_framework_and_constructor_preservation() {
    for (decorator, kind) in [
        ("@Component({selector:'x-audit'})", "Component"),
        ("@Directive({selector:'[audit]'})", "Directive"),
    ] {
        check(
            "private unrelated = inject(LoggerService); constructor(private different: AuditService) {}",
            decorator,
            kind,
            &["AuditService", "LoggerService"],
        );
    }
    check(
        "private unrelated = inject(LoggerService); constructor(private different: AuditService) {}",
        "@Pipe({name:'audit'})",
        "Pipe",
        &["AuditService"],
    );
}

#[test]
fn angular_di_inject_token_symbolic_unresolved_is_not_a_declaration() {
    check(
        "private unrelated = inject(MissingService);",
        "@Injectable()",
        "Service",
        &["MissingService"],
    );
    check(
        "private unrelated = inject(MY_TOKEN);",
        "@Injectable()",
        "Service",
        &["MY_TOKEN"],
    );
}

#[test]
fn angular_di_inject_token_refresh_replaces_removes_and_deletes() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("consumer.ts");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for token in [Some("AuditService"), Some("LoggerService"), None] {
        let fields = token
            .map(|token| format!("private dependency = inject({token});"))
            .unwrap_or_else(|| "private dependency = makeService(AuditService);".into());
        std::fs::write(&path, source(&fields, "@Injectable()")).unwrap();
        let refresh = call(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        assert_eq!(refresh["structuredContent"]["refreshed"], true);
        call(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}),
        );
        assert_edges(
            &state,
            root.path(),
            "Service",
            &token.into_iter().collect::<Vec<_>>(),
        );
    }
    std::fs::remove_file(&path).unwrap();
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot":root.path()}),
    );
    assert_edges(&state, root.path(), "Service", &[]);
}
