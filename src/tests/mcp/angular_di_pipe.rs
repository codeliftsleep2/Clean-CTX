//! Pipe admission consumes the existing constructor evidence; extraction is frozen.
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{tool}: {response}");
    response["result"].clone()
}

fn check(decorator: &str, kind: &str, parameters: &str, expected: &[&str]) {
    check_fields(decorator, kind, parameters, "", expected);
}

fn check_fields(decorator: &str, kind: &str, parameters: &str, fields: &str, expected: &[&str]) {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("consumer.ts");
    let field_prefix = if fields.is_empty() {
        String::new()
    } else {
        format!("{fields} ")
    };
    let consumer = format!(
        "{decorator}\nexport class AuditConsumer {{ {field_prefix}constructor({parameters}) {{}} transform(value: string): string {{ return value; }} }}"
    );
    let source = format!(
        "import {{ Injectable, Component, Directive, Pipe{} }} from '@angular/core';\n@Injectable() export class AuditService {{}}\n@Injectable() export class LoggerService {{}}\n{consumer}\n{}",
        if fields.is_empty() { "" } else { ", inject" },
        "// unchanged economics padding\n".repeat(100)
    );
    crate::edit::apply::verify_syntax(&source, "ts").unwrap();
    let evidence = crate::angular_meta::decorators::extract_graph_entries(&consumer).unwrap();
    assert_eq!(evidence.0, "AuditConsumer");
    let mut written = if kind == "Pipe" {
        crate::angular_meta::constructor_injects::extract_constructor_injects(&consumer)
            .unwrap_or_default()
    } else {
        evidence.3.clone()
    };
    written.sort();
    written.dedup();
    let mut expected_names: Vec<_> = expected.iter().map(|name| name.to_string()).collect();
    expected_names.sort();
    assert_eq!(
        written, expected_names,
        "dependency evidence must exist before admission"
    );
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
    assert!(
        text.contains("C AuditConsumer"),
        "core owner missing: {text}"
    );
    if !expected.is_empty() {
        assert!(
            text.contains("T @injects"),
            "constructor marker missing: {text}"
        );
    }
    let entities = call(
        &state,
        "workspace_query",
        json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}),
    );
    let entities = entities["structuredContent"]["entities"]
        .as_array()
        .unwrap();
    for (entity_kind, name) in [
        (kind, "AuditConsumer"),
        ("Service", "AuditService"),
        ("Service", "LoggerService"),
    ] {
        assert_eq!(
            entities
                .iter()
                .filter(|entity| entity["domain"] == "angular"
                    && entity["entity_type"] == entity_kind
                    && entity["name"] == name
                    && entity["file"] == path.to_string_lossy().as_ref())
                .count(),
            1,
            "independent occurrence: {entities:?}"
        );
    }
    for _ in 0..2 {
        let answer = call(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"angular","entity_type":kind,"name":"AuditConsumer"}),
        );
        let injects: Vec<_> = answer["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "Injects")
            .collect();
        assert_eq!(
            injects.len(),
            expected.len(),
            "exact admitted dependencies: {answer}"
        );
        let mut targets: Vec<_> = injects
            .iter()
            .map(|edge| edge["object"]["name"].as_str().unwrap().to_string())
            .collect();
        targets.sort();
        assert_eq!(targets, expected_names);
        for edge in injects {
            assert_eq!(edge["subject"]["domain"], "angular");
            assert_eq!(edge["subject"]["entity_type"], kind);
            assert_eq!(edge["subject"]["name"], "AuditConsumer");
            assert_eq!(edge["subject"]["file"], path.to_string_lossy().as_ref());
            assert_eq!(edge["object"]["domain"], "angular");
            assert_eq!(edge["object"]["entity_type"], "Service");
            assert_eq!(edge["object"]["file"], path.to_string_lossy().as_ref());
        }
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}

#[test]
fn angular_di_pipe_constructor_admission() {
    check(
        "@Pipe({name:'audit'})",
        "Pipe",
        "private readonly completelyDifferentName: AuditService",
        &["AuditService"],
    );
}

#[test]
fn angular_di_pipe_existing_consumer_kinds() {
    for (decorator, kind) in [
        ("@Injectable()", "Service"),
        ("@Component({selector:'x-audit'})", "Component"),
        ("@Directive({selector:'[audit]'})", "Directive"),
    ] {
        check(
            decorator,
            kind,
            "private completelyDifferentName: AuditService",
            &["AuditService"],
        );
    }
}

#[test]
fn angular_di_pipe_multiple_dependencies() {
    check(
        "@Pipe({name:'audit'})",
        "Pipe",
        "audit: AuditService, private completelyDifferentName: LoggerService",
        &["AuditService", "LoggerService"],
    );
}

#[test]
fn angular_di_pipe_same_target_deduplicated() {
    check(
        "@Pipe({name:'audit'})",
        "Pipe",
        "private first: AuditService, private second: AuditService",
        &["AuditService"],
    );
}

#[test]
fn angular_di_pipe_empty_constructor_no_injections() {
    check("@Pipe({name:'audit'})", "Pipe", "", &[]);
}

#[test]
fn angular_di_pipe_constructor_modifier_controls() {
    for parameters in [
        "audit: AuditService",
        "public audit: AuditService",
        "readonly audit: AuditService",
        "private\nreadonly different:\n AuditService",
    ] {
        check(
            "@Pipe({name:'audit'})",
            "Pipe",
            parameters,
            &["AuditService"],
        );
    }
}

#[test]
fn angular_di_pipe_does_not_admit_unrepaired_functional_evidence() {
    check_fields(
        "@Pipe({name:'audit'})",
        "Pipe",
        "",
        "private receiver = inject(LoggerService);",
        &[],
    );
    check_fields(
        "@Pipe({name:'audit'})",
        "Pipe",
        "private audit: AuditService",
        "private unrelatedReceiver = inject(LoggerService);",
        &["AuditService"],
    );
}

#[test]
fn angular_di_pipe_refresh_dependency_and_removal() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("pipe.ts");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for target in [Some("AuditService"), Some("LoggerService"), None] {
        let params = target
            .map(|name| format!("private receiver: {name}"))
            .unwrap_or_default();
        std::fs::write(&path, format!("import {{ Pipe, Injectable }} from '@angular/core';\n@Injectable() class AuditService {{}}\n@Injectable() class LoggerService {{}}\n@Pipe({{name:'audit'}}) class AuditPipe {{constructor({params}) {{}} }}")).unwrap();
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
        let result = call(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(),"type":"forward_edges","domain":"angular","entity_type":"Pipe","name":"AuditPipe"}),
        );
        let names: Vec<_> = result["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "Injects")
            .map(|edge| edge["object"]["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, target.into_iter().collect::<Vec<_>>(), "{result}");
    }
}
