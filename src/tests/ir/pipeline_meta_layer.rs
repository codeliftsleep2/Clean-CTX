use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

fn compile_edges(source: &str, path: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("ts").expect("TypeScript language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "α-test",
            Some(path),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("TypeScript compilation");
    compiler.semantic_edges
}

#[test]
fn canonical_path_reaches_testing_extractor_in_production_pass() {
    let edges = compile_edges(
        "describe('account', () => { it('works', () => {}); });",
        "C:/repo/account.component.spec.ts",
    );
    let edge = edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::Tests)
        .expect("filename-derived Tests edge");
    assert_eq!(edge.subject.name, "account.component.spec.ts");
    assert_eq!(
        edge.subject.file.as_deref(),
        Some("C:/repo/account.component.spec.ts")
    );
    assert_eq!(edge.object.name, "AccountComponent");
}

#[test]
fn explicit_create_component_survives_production_pass() {
    let edges = compile_edges(
        "TestBed.createComponent(AccountComponent);",
        "C:/repo/custom.spec.ts",
    );
    let tests: Vec<_> = edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::Tests)
        .collect();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].object.name, "AccountComponent");
}

#[test]
fn reverse_edge_exposes_angular_test_artifact_without_index_special_case() {
    let component = r#"
import { Component } from '@angular/core';
@Component({ selector: 'app-account', template: '' })
export class AccountComponent {}
"#;
    let spec = r#"
describe('AccountComponent', () => {
  TestBed.createComponent(AccountComponent);
  it('renders', () => {});
});
"#;
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "C:/repo/account.component.ts",
        compile_edges(component, "C:/repo/account.component.ts"),
    );
    index.add_edges(
        "C:/repo/account.component.spec.ts",
        compile_edges(spec, "C:/repo/account.component.spec.ts"),
    );
    let reverse = index.reverse_edges_by_identity("angular", "Component", "AccountComponent");
    let tests: Vec<_> = reverse
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::Tests)
        .collect();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].subject.entity_type, "TestArtifact");
    assert_eq!(tests[0].subject.name, "account.component.spec.ts");
}
