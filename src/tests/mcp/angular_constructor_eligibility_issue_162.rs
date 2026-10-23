use serde_json::json;

#[test]
fn issue_162_constructor_authority_type_value_and_shadow_refusals() {
    use crate::layers::meta::semantic::SemanticRelation;
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    std::fs::write(p.join("left.ts"), "export class Dependency {}").unwrap();
    std::fs::write(
        p.join("right.ts"),
        "export class Dependency {} export interface Erased {}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (extra, parameters, generic) in [
        (
            "import type { Dependency as Bad } from './right';",
            "bad:Bad",
            "",
        ),
        ("import { Erased as Bad } from './right';", "bad:Bad", ""),
        (
            "import { Dependency as Bad } from './right';",
            "bad:Bad",
            "<Bad>",
        ),
        (
            "import * as right from './right';",
            "right:unknown, bad:right.Dependency",
            "",
        ),
        (
            "import { Dependency as Bad } from './missing';",
            "bad:Bad",
            "",
        ),
    ] {
        let source = format!(
            "import {{Injectable as Service}} from '@angular/core'; import {{Dependency as Good}} from './left'; {extra} const note='constructor(fake:Phantom)'; @Service() export class Owner{generic} {{ constructor(readonly good:Good, {parameters}) {{}} method(value:Good) {{return value;}} }}"
        );
        let file = p.join("app.ts");
        std::fs::write(&file, &source).unwrap();
        let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
            file.to_str().unwrap(),
            &source,
            crate::compression::Fidelity::High,
            &state,
            p.to_str(),
        )
        .unwrap();
        let (edges, _, _) = artifact.published_semantics();
        let canonical: Vec<_> = edges
            .iter()
            .filter(|edge| {
                edge.relation == SemanticRelation::Injects && edge.object.canonical.is_some()
            })
            .collect();
        assert_eq!(
            canonical.len(),
            1,
            "only the independent good token has runtime class proof: {}",
            json!({"source":source,"edges":edges,"facets":artifact.typescript_authority().constructor_parameters})
        );
        let token = canonical[0].object.canonical.as_ref().unwrap();
        assert!(token.declaration().unwrap().source.ends_with("left.ts"));
        assert_eq!(token.declaration().unwrap().name, "Dependency");
        assert!(
            !edges
                .iter()
                .any(|edge| edge.relation == SemanticRelation::Injects
                    && matches!(
                        edge.object.name.as_str(),
                        "Bad" | "right" | "Phantom" | "Erased"
                    )),
            "failed named/qualified proof must not fall back to a symbolic spelling"
        );
    }
}

#[test]
fn issue_162_constructor_decorator_authority_uses_exact_angular_override() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut declarations = std::fs::read_to_string(&core).unwrap();
    declarations.push_str("export declare function Inject(token:unknown):any; export declare function Optional():any;");
    std::fs::write(core, declarations).unwrap();
    for file in ["left.ts", "right.ts"] {
        std::fs::write(p.join(file), "export class Dependency {}").unwrap();
    }
    std::fs::write(
        p.join("foreign.ts"),
        "export function Inject(value:unknown) { return value; }",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (imports, decorator, annotation, wanted) in [
        (
            "import {Inject as Override, Optional as Maybe} from '@angular/core';",
            "@Maybe() @Override(Right)",
            "Left",
            "right.ts",
        ),
        (
            "import * as core from '@angular/core';",
            "@core.Inject(Right)",
            "Left",
            "right.ts",
        ),
        (
            "import {Inject as Override} from './foreign';",
            "@Override(Right)",
            "Left",
            "left.ts",
        ),
        (
            "import {Optional as Maybe} from '@angular/core';",
            "@Maybe()",
            "Left",
            "left.ts",
        ),
    ] {
        let source = format!(
            "import {{Injectable as Service}} from '@angular/core'; import {{Dependency as Left}} from './left'; import {{Dependency as Right}} from './right'; {imports} const note='@Inject(Fake) constructor(fake:Phantom)'; function unrelated<T>(value:T){{return value;}} @Service() export class Owner {{ constructor({decorator} private readonly dep:{annotation}) {{}} method(value:Right){{return unrelated(value);}} }}"
        );
        let file = p.join("app.ts");
        std::fs::write(&file, &source).unwrap();
        for _ in 0..2 {
            let args = [
                json!({"type":"entities_in_file","file_path":file,"fidelity":"high"}),
                json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
            ];
            let mut answer = serde_json::Value::Null;
            for mut arguments in args {
                arguments["workspaceRoot"] = json!(p);
                crate::protocol::captured_responses().clear();
                crate::mcp::tools::dispatch_tools_call(
                    &json!(162),
                    "workspace_query",
                    &json!({"arguments":arguments}),
                    &state,
                );
                answer = crate::protocol::captured_responses().pop().unwrap();
                assert!(answer.get("error").is_none(), "{answer}");
            }
            let edges: Vec<_> = answer["result"]["structuredContent"]["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|edge| edge["relation"] == "Injects")
                .collect();
            assert_eq!(edges.len(), 1, "one dependency for {decorator}: {answer}");
            assert_eq!(
                edges[0]["object"]["name"], "Dependency",
                "written annotation is not a canonical token"
            );
            assert_eq!(
                edges[0]["object"]["file"],
                json!(crate::dictionary::path::canonical_identity_key(
                    &p.join(wanted).to_string_lossy()
                )),
                "actual Angular Inject overrides; foreign/Optional decorators do not: {source}"
            );
        }
        assert_eq!(std::fs::read_to_string(file).unwrap(), source);
    }
}
