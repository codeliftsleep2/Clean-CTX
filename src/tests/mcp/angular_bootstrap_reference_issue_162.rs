//! Investigation only: relation labels/occurrence owner are provisional hypotheses.
use crate::ir::invocation_evidence::{ArgumentQualifier, SourceSpan, WrittenCallee};
use crate::ir::typescript_resolution::{Namespace, TypeScriptAuthority};
use serde_json::{Value, json};
use std::path::Path;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("registered response");
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    response["result"]["structuredContent"].clone()
}
fn capture(state: &crate::mcp::McpState, root: &Path, file: &str) -> TypeScriptAuthority {
    call(
        state,
        root,
        "compress_code_context",
        json!({"filePath":root.join(file),"fidelity":"high"}),
    );
    let alias = state
        .alias_for_path(&root.join(file).to_string_lossy())
        .expect("captured alias");
    state
        .semantic_snapshot(&alias)
        .expect("retained snapshot")
        .authority
        .typescript_authority
        .expect("native authority")
}
fn identity(path: &str) -> String {
    crate::dictionary::path::canonical_identity_key(path)
}
fn setup(callers: &[(String, String)]) -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Component(metadata:unknown):any; export interface ApplicationConfig {providers:unknown[];}").unwrap();
    let platform = p.join("node_modules/@angular/platform-browser");
    std::fs::create_dir_all(&platform).unwrap();
    std::fs::write(
        platform.join("package.json"),
        r#"{"name":"@angular/platform-browser","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        platform.join("index.d.ts"),
        "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;",
    )
    .unwrap();
    std::fs::write(
        p.join("api.ts"),
        "export {bootstrapApplication as start} from '@angular/platform-browser';",
    )
    .unwrap();
    std::fs::write(
        p.join("fake.ts"),
        "export function bootstrapApplication(root:unknown,config?:unknown){}",
    )
    .unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from '@angular/platform-browser'; export * from './fake';",
    )
    .unwrap();
    for side in ["left", "right"] {
        std::fs::write(p.join(format!("{side}-token.ts")), "export class Token {}").unwrap();
        std::fs::write(p.join(format!("{side}-root.ts")),format!(
            "import {{Component}} from '@angular/core'; import {{Token}} from './{side}-token'; @Component({{providers:[Token]}}) export class Root {{}}" )).unwrap();
        std::fs::write(p.join(format!("{side}-config.ts")),format!(
            "import type {{ApplicationConfig}} from '@angular/core'; import {{Token}} from './{side}-token'; export const appConfig:ApplicationConfig={{providers:[Token]}};" )).unwrap();
    }
    // Complete caller contents before any program/native authority capture.
    // Placeholder-to-final edits also invalidate project content witnesses.
    for (file, source) in callers {
        std::fs::write(p.join(file), source).unwrap();
    }
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for side in ["left", "right"] {
        let authority = capture(&state, p, &format!("{side}-root.ts"));
        let root_id = authority
            .declarations
            .iter()
            .find(|id| id.name == "Root" && id.source == authority.source)
            .expect("root declaration");
        assert_eq!(
            crate::angular_meta::class_role_authority::declaration_role(&authority, root_id),
            Some(crate::angular_meta::decorators::ClassKind::Component)
        );
        let authority = capture(&state, p, &format!("{side}-config.ts"));
        assert!(crate::angular_meta::application_config_providers::has_owner(&authority));
        assert!(!crate::angular_meta::application_config_providers::project(&authority).is_empty());
    }
    for (kind, name, suffix) in [
        ("Component", "Root", "root"),
        ("ApplicationConfig", "appConfig", "config"),
    ] {
        let answer = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":kind,"name":name}),
        );
        let mut actual: Vec<_> = answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider")
            .map(|e| {
                (
                    identity(e["subject"]["file"].as_str().unwrap()),
                    identity(e["object"]["file"].as_str().unwrap()),
                )
            })
            .collect();
        actual.sort();
        let mut expected: Vec<_> = ["left", "right"]
            .into_iter()
            .map(|side| {
                (
                    identity(&p.join(format!("{side}-{suffix}.ts")).to_string_lossy()),
                    identity(&p.join(format!("{side}-token.ts")).to_string_lossy()),
                )
            })
            .collect();
        expected.sort();
        assert_eq!(
            actual, expected,
            "nonempty independent native {kind} prerequisite: {answer}"
        );
    }
    (root, state)
}
fn api(form: &str) -> (&'static str, &'static str) {
    match form {
        "direct" => (
            "import {bootstrapApplication} from '@angular/platform-browser';",
            "bootstrapApplication",
        ),
        "alias" => (
            "import {bootstrapApplication as launch} from '@angular/platform-browser';",
            "launch",
        ),
        "barrel" => ("import {start as launch} from './api';", "launch"),
        "namespace" => (
            "import * as platform from '@angular/platform-browser';",
            "platform.bootstrapApplication",
        ),
        _ => panic!("unknown API form"),
    }
}
fn prove_calls(
    state: &crate::mcp::McpState,
    p: &Path,
    file: &str,
    source: &str,
    expression: &str,
    side: &str,
    repeats: usize,
    config: bool,
    native: bool,
) {
    let authority = capture(state, p, file);
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        &p.join(file).to_string_lossy(),
        source,
        crate::compression::Fidelity::High,
        state,
        Some(&p.to_string_lossy()),
    )
    .expect("default compiler candidate");
    let evidence = artifact
        .invocation_evidence()
        .expect("captured invocation inventory");
    assert_eq!(evidence.source_hash(), authority.source_hash);
    let view = evidence.view(evidence.source_id(), source).unwrap();
    let spans: Vec<_> = source
        .match_indices(expression)
        .map(|(start, _)| (start, start + expression.len()))
        .collect();
    assert_eq!(spans.len(), repeats);
    let calls: Vec<_> = view
        .occurrences()
        .iter()
        .filter(|c| spans.contains(&(c.span().start_byte(), c.span().end_byte())))
        .collect();
    assert_eq!(calls.len(), repeats, "exact call spans required");
    for occurrence in calls {
        let callee = match occurrence.callee() {
            WrittenCallee::Free { terminal } => *terminal,
            WrittenCallee::Member { receiver, terminal } => {
                SourceSpan::new(receiver.start_byte(), terminal.end_byte()).unwrap()
            }
            other => panic!("unexpected callee {other:?}"),
        };
        let native_api = authority.value_reference_at(callee).is_some_and(|id| {
            crate::angular_meta::class_role_authority::public_entry_api(
                &authority,
                id,
                "@angular/platform-browser",
                "@angular/platform-browser",
                "bootstrapApplication",
                Namespace::Value,
            )
        });
        assert_eq!(
            native_api, native,
            "exact native API prerequisite; file={file}; proof={authority:?}"
        );
        assert_eq!(occurrence.arguments().len(), if config { 2 } else { 1 });
        for (position, argument) in occurrence.arguments().iter().enumerate() {
            assert!(matches!(argument.qualifier(), ArgumentQualifier::Plain));
            assert!(
                occurrence.span().start_byte() <= argument.span().start_byte()
                    && argument.span().end_byte() <= occurrence.span().end_byte()
            );
            // Refused callees have no bootstrap argument role. Native positive
            // controls and all association candidates still require exact targets.
            if !native {
                continue;
            }
            let target = authority
                .value_reference_at(argument.span())
                .expect("resolved runtime argument");
            let suffix = if position == 0 { "root" } else { "config" };
            assert_eq!(
                identity(&target.source.to_string_lossy()),
                identity(&p.join(format!("{side}-{suffix}.ts")).to_string_lossy())
            );
            assert_eq!(
                target.name,
                if position == 0 { "Root" } else { "appConfig" }
            );
            let target_alias = state
                .alias_for_path(&p.join(format!("{side}-{suffix}.ts")).to_string_lossy())
                .unwrap();
            let snapshot = state
                .semantic_snapshot(&target_alias)
                .expect("target prerequisite must remain current after caller capture");
            let target_proof = snapshot.authority.typescript_authority.as_ref().unwrap();
            let expected: Vec<_> = target_proof
                .declarations
                .iter()
                .filter(|id| id.source == target_proof.source && id.name == target.name)
                .collect();
            assert_eq!(expected.len(), 1, "unique native argument declaration");
            assert_eq!(
                target, expected[0],
                "exact source/hash/span/kind argument identity"
            );
        }
    }
}
fn question(form: &str, root_relation: bool, repeats: usize, config: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let (import, callee) = api(form);
    let expression = format!("{callee}(Root{})", if config { ",appConfig" } else { "" });
    let callers: Vec<_> = ["left", "right"].into_iter().map(|side| {
        (format!("{side}-main.ts"), format!(
            "{import} import {{Root}} from './{side}-root'; import {{appConfig}} from './{side}-config'; {}",
            format!("{expression};").repeat(repeats)))
    }).collect();
    let (directory, state) = setup(&callers);
    let p = directory.path();
    for (side, (file, source)) in ["left", "right"].into_iter().zip(&callers) {
        prove_calls(
            &state,
            p,
            file,
            source,
            &expression,
            side,
            repeats,
            config,
            true,
        );
    }
    // Provisional labels are test hypotheses only; no production enum/schema added.
    let (kind, name, relation, suffix) = if root_relation {
        ("Component", "Root", "DeclaresBootstrapRoot", "root")
    } else {
        (
            "ApplicationConfig",
            "appConfig",
            "UsesApplicationConfig",
            "config",
        )
    };
    for _ in 0..2 {
        let answer = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"reverse_edges","domain":"angular","entity_type":kind,"name":name}),
        );
        let found: Vec<_> = answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == relation)
            .collect();
        assert_eq!(
            found.len(),
            2 * repeats,
            "stage=bootstrap-association; provisional={relation}; form={form}; repeats={repeats}; public={answer}"
        );
        let mut actual: Vec<_> = found
            .iter()
            .map(|e| {
                assert_eq!(e["subject"]["entity_type"], "ApplicationBootstrap");
                assert!(
                    e["subject"].get("canonical").is_none()
                        && e["object"].get("canonical").is_none()
                );
                (
                    identity(e["subject"]["file"].as_str().unwrap()),
                    identity(e["object"]["file"].as_str().unwrap()),
                )
            })
            .collect();
        actual.sort();
        let mut expected = Vec::new();
        for side in ["left", "right"] {
            for _ in 0..repeats {
                expected.push((
                    identity(&p.join(format!("{side}-main.ts")).to_string_lossy()),
                    identity(&p.join(format!("{side}-{suffix}.ts")).to_string_lossy()),
                ));
            }
        }
        expected.sort();
        assert_eq!(actual, expected, "call occurrence/public multiplicity");
    }
}

#[test]
fn issue_162_bootstrap_reference_direct_root() {
    question("direct", true, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_direct_config() {
    question("direct", false, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_alias_root() {
    question("alias", true, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_alias_config() {
    question("alias", false, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_barrel_root() {
    question("barrel", true, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_barrel_config() {
    question("barrel", false, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_namespace_root() {
    question("namespace", true, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_namespace_config() {
    question("namespace", false, 1, true);
}
#[test]
fn issue_162_bootstrap_reference_repeated_root_occurrences() {
    question("direct", true, 2, true);
}
#[test]
fn issue_162_bootstrap_reference_repeated_config_occurrences() {
    question("direct", false, 2, true);
}
#[test]
fn issue_162_bootstrap_reference_omitted_config_keeps_root() {
    question("direct", true, 1, false);
}
#[test]
fn issue_162_bootstrap_reference_native_api_refusal_controls() {
    let _serial = crate::protocol::handler_response_serial();
    let variants = [
        "import {bootstrapApplication} from './fake';",
        "import type {bootstrapApplication} from '@angular/platform-browser';",
        "import {bootstrapApplication} from './missing';",
        "import {bootstrapApplication} from './ambiguous';",
        "function bootstrapApplication(root:unknown,config:unknown){}",
    ];
    let expression = "bootstrapApplication(Root,appConfig)";
    let mut callers: Vec<_> = variants.into_iter().enumerate().map(|(index, import)|
        (format!("negative-{index}.ts"), format!(
            "{import} import {{Root}} from './left-root'; import {{appConfig}} from './left-config'; {expression};")))
        .collect();
    callers.push(("shadow-main.ts".into(),"import {bootstrapApplication} from '@angular/platform-browser'; import {Root} from './left-root'; import {appConfig} from './left-config'; function run(bootstrapApplication:unknown){bootstrapApplication(Root,appConfig);}".into()));
    callers.push(("positive-main.ts".into(), "import {bootstrapApplication} from '@angular/platform-browser'; import {Root} from './left-root'; import {appConfig} from './left-config'; bootstrapApplication(Root,appConfig);".into()));
    let (directory, state) = setup(&callers);
    let p = directory.path();
    for (file, source) in &callers {
        prove_calls(
            &state,
            p,
            file,
            source,
            expression,
            "left",
            1,
            true,
            file == "positive-main.ts",
        );
    }
}
