//! AQ08 mixed-source acceptance: positive facts survive nearby invalid uses.
use serde_json::{Value, json};
use std::path::Path;

fn query(state: &crate::mcp::McpState, root: &Path, mut arguments: Value) -> Value {
    arguments["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "workspace_query",
        &json!({"arguments":arguments}),
        state,
    );
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"].clone()
}

fn fixture() -> (tempfile::TempDir, crate::mcp::McpState) {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    for name in ["left.ts", "right.ts"] {
        std::fs::write(p.join(name), "export class Dependency {}").unwrap();
    }
    std::fs::write(p.join("barrel.ts"), "export { inject as resolveDependency } from '@angular/core'; export { Dependency as PublicLeft } from './left';").unwrap();
    std::fs::write(
        p.join("foreign.ts"),
        "export function inject(token:unknown) {return token;}",
    )
    .unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    (root, crate::mcp::McpState::new(config))
}

const MIXED: &str = r#"
import { Injectable as Service, inject as obtain } from '@angular/core';
import { resolveDependency as throughBarrel, PublicLeft as Left } from './barrel';
import { Dependency as Right } from './right';
import type { Dependency as Erased } from './right';
import { inject as impostor } from './foreign';
import { MissingExport } from './missing-unrelated';
const prose = 'inject(FakeToken) // not a call';
const settings = { inject: 'ordinary property', nested: [1, 2, 3] };
function wrap(value:unknown) { return value; }
function unrelated<T>(obtain:(arg:T)=>unknown, value:T) { return obtain(value); }
// @Injectable() and inject(AnotherFake) are only comment text.
@Service()
export class Owner {
  private readonly good = throughBarrel(Left, { optional: true });
  duplicate = obtain(Left);
  foreign = impostor(Right);
  erased = obtain(Erased);
  optional = obtain?.(Right);
  nested = wrap(obtain(Right));
  literal = 'throughBarrel(Right)';
  method(obtain:(token:unknown)=>unknown) { return obtain(Right); }
}
"#;

fn dependencies(state: &crate::mcp::McpState, p: &Path) -> Vec<Value> {
    query(
        state,
        p,
        json!({"type":"entities_in_file","file_path":p.join("app.ts"),"fidelity":"high"}),
    );
    let response = query(
        state,
        p,
        json!({"type":"forward_edges","domain":"angular","entity_type":"Service","name":"Owner"}),
    );
    response["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "Injects")
        .cloned()
        .collect()
}

fn exact_left(edges: &[Value], p: &Path) {
    assert_eq!(
        edges.len(),
        1,
        "unrelated bad uses must neither poison nor authorize a dependency: {edges:?}"
    );
    assert_eq!(edges[0]["object"]["name"], "Dependency");
    assert_eq!(
        edges[0]["object"]["file"],
        json!(crate::dictionary::path::canonical_identity_key(
            &p.join("left.ts").to_string_lossy()
        ))
    );
}

#[test]
fn issue_162_inject_mixed_noise_aliases_bad_neighbors_and_dependency_mutation() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    std::fs::write(p.join("app.ts"), MIXED).unwrap();
    exact_left(&dependencies(&state, p), p);
    exact_left(&dependencies(&state, p), p);
    // An unrelated endpoint changes kind and then disappears. The valid fact
    // must survive; none of the nearby rejected uses may borrow its authority.
    std::fs::write(p.join("right.ts"), "export interface Dependency {}").unwrap();
    exact_left(&dependencies(&state, p), p);
    std::fs::remove_file(p.join("right.ts")).unwrap();
    exact_left(&dependencies(&state, p), p);
    assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), MIXED);
    std::fs::write(p.join("left.ts"), "export interface Dependency {}").unwrap();
    assert!(
        dependencies(&state, p).is_empty(),
        "the valid fact must retract when its own proof changes"
    );
    std::fs::write(p.join("left.ts"), "export class Dependency {}").unwrap();
    exact_left(&dependencies(&state, p), p);
}

#[test]
fn issue_162_inject_mixed_namespace_variation_preserves_two_exact_token_owners() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = MIXED
        .replace(
            "import { Injectable",
            "import * as core from '@angular/core';\nimport { Injectable",
        )
        .replace(
            "  duplicate = obtain(Left);",
            "  duplicate = obtain(Left);\n  qualified = core.inject(Right);",
        );
    std::fs::write(p.join("app.ts"), &source).unwrap();
    let edges = dependencies(&state, p);
    assert_eq!(
        edges.len(),
        2,
        "a canonical namespace route is a valid variation, while impostor members remain refused: {edges:?}"
    );
    let mut actual: Vec<_> = edges
        .iter()
        .map(|edge| edge["object"]["file"].as_str().unwrap().to_owned())
        .collect();
    actual.sort();
    let mut expected: Vec<_> = ["left.ts", "right.ts"]
        .iter()
        .map(|file| {
            crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
        })
        .collect();
    expected.sort();
    assert_eq!(actual, expected);
}

#[test]
fn issue_162_inject_mixed_object_members_cannot_borrow_namespace_role() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = MIXED.replace(
        "  duplicate = obtain(Left);",
        "  duplicate = obtain(Left);\n  fake = settings.inject(Right);\n  optionalMember = settings?.inject(Right);",
    );
    std::fs::write(p.join("app.ts"), &source).unwrap();
    exact_left(&dependencies(&state, p), p);
    exact_left(&dependencies(&state, p), p);
}

#[test]
fn issue_162_inject_mixed_symbolic_compatibility_does_not_promote_bad_bindings() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = fixture();
    let p = root.path();
    let source = MIXED.replace(
        "  duplicate = obtain(Left);",
        "  duplicate = obtain(Left);\n  symbolic = obtain(MissingService);",
    );
    std::fs::write(p.join("app.ts"), &source).unwrap();
    for _ in 0..2 {
        let edges = dependencies(&state, p);
        assert_eq!(
            edges.len(),
            2,
            "preserve the genuine left fact and one symbolic dependency, without admitting erased, foreign, nested or shadowed neighbors: {edges:?}"
        );
        let symbolic: Vec<_> = edges
            .iter()
            .filter(|edge| edge["object"]["name"] == "MissingService")
            .collect();
        assert_eq!(symbolic.len(), 1, "symbolic dependency must remain visible");
        assert_eq!(
            symbolic[0]["object"]["file"],
            json!(crate::dictionary::path::canonical_identity_key(
                &p.join("app.ts").to_string_lossy()
            )),
            "the legacy public file is asserting-source provenance"
        );
        let canonical: Vec<_> = edges
            .into_iter()
            .filter(|edge| edge["object"]["name"] != "MissingService")
            .collect();
        exact_left(&canonical, p);
    }
}

#[test]
fn issue_162_inject_symbolic_payload_has_no_declaration_or_canonical_endpoint() {
    use crate::layers::meta::semantic::SemanticRelation;
    let (root, state) = fixture();
    let p = root.path();
    let source = MIXED.replace(
        "  duplicate = obtain(Left);",
        "  duplicate = obtain(Left);\n  symbolic = obtain(MissingService);",
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
    let authority = artifact.typescript_authority();
    assert!(
        !authority
            .declarations
            .iter()
            .any(|id| id.name == "MissingService")
    );
    assert!(
        authority
            .unbound_value_sites
            .iter()
            .any(|(_, _, name)| name == "MissingService")
    );
    let (edges, _, _) = artifact.published_semantics();
    assert!(
        !edges
            .iter()
            .any(|edge| edge.relation == SemanticRelation::Defines
                && (edge.subject.name == "MissingService" || edge.object.name == "MissingService"))
    );
    let symbolic: Vec<_> = edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Injects && edge.object.name == "MissingService"
        })
        .collect();
    assert_eq!(symbolic.len(), 1);
    assert_eq!(
        crate::dictionary::path::canonical_identity_key(
            symbolic[0].object.file.as_deref().unwrap()
        ),
        crate::dictionary::path::canonical_identity_key(&file.to_string_lossy()),
        "the compilation pipeline attaches asserting-source provenance"
    );
    assert!(symbolic[0].object.canonical.is_none());
    assert!(symbolic[0].subject.canonical.is_none());

    // Inspect the actual evaluator boundary before the pipeline attaches file
    // provenance. File metadata above is not canonical declaration evidence.
    use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
    let (language, query) = crate::compression::language::language_for_extension("ts").unwrap();
    let mut lowered = PassContext::new(
        source.clone(),
        file.to_string_lossy().into_owned(),
        crate::compression::Fidelity::High,
    );
    lowered.language = Some(language);
    lowered.query_string = query.into();
    CoreIRPass::new().run(&mut lowered).unwrap();
    let calls = artifact.invocation_evidence().unwrap();
    let view = calls.view(calls.source_id(), &source).unwrap();
    let direct = crate::angular_meta::inject_authority::project(
        &source,
        authority,
        &lowered.angular_initializers,
        &view,
    );
    let direct_symbolic: Vec<_> = direct
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Injects && edge.object.name == "MissingService"
        })
        .collect();
    assert_eq!(direct_symbolic.len(), 1);
    assert!(direct_symbolic[0].object.file.is_none());
    assert!(direct_symbolic[0].object.canonical.is_none());
    assert!(direct_symbolic[0].subject.canonical.is_none());
}
