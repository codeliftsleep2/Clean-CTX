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

fn compile_csharp_edges(source: &str, path: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
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
        .expect("C# compilation");
    compiler.semantic_edges
}

fn compile_java_edges(source: &str, path: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("java").expect("Java language");
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
        .expect("Java compilation");
    compiler.semantic_edges
}

fn compile_rust_edges(source: &str, path: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("rs").expect("Rust language");
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
        .expect("Rust compilation");
    compiler.semantic_edges
}

#[test]
fn production_meta_pass_constructs_one_shared_lexical_index() {
    crate::meta_util::reset_construction_count();

    let _ = compile_edges(
        "import { signal } from '@angular/core'; const count = signal(0);",
        "C:/repo/state.ts",
    );

    assert_eq!(
        crate::meta_util::construction_count(),
        1,
        "MetaLayerPass must construct one lexical index for both dispatches"
    );
}

#[test]
fn production_testing_family_reuses_meta_pass_lexical_index() {
    crate::meta_util::reset_construction_count();

    let _ = compile_edges(
        "describe('account', () => { it('works', () => {}); });",
        "C:/repo/account.component.spec.ts",
    );

    assert_eq!(
        crate::meta_util::construction_count(),
        1,
        "Angular testing markers and semantics must reuse the meta-pass lexical index"
    );
}

#[test]
fn production_meta_pass_uses_one_combined_registry_evaluation() {
    crate::layers::registry::reset_context_route_counts();

    let _ = compile_edges(
        "import { signal } from '@angular/core'; const count = signal(0);",
        "C:/repo/state.ts",
    );

    assert_eq!(
        crate::layers::registry::context_route_counts(),
        (0, 0, 1),
        "MetaLayerPass must replace separate marker/semantic routes with one evaluation"
    );
}

#[test]
fn production_angular_evaluation_extracts_ngrx_shape_once() {
    crate::angular_meta::ngrx::reset_extraction_count();

    let _ = compile_edges(
        "import { createAction } from '@ngrx/store'; export const load = createAction('[X] Load');",
        "C:/repo/actions.ts",
    );

    assert_eq!(
        crate::angular_meta::ngrx::extraction_count(),
        1,
        "Angular evaluation must share one NgRx shape between markers and semantics"
    );
}

#[test]
fn production_angular_evaluation_extracts_routing_shape_once() {
    crate::angular_meta::routing::reset_extraction_count();

    let _ = compile_edges(
        "import { Routes } from '@angular/router'; export const routes: Routes = [{ path: 'x', component: XComponent }];",
        "C:/repo/app.routes.ts",
    );

    assert_eq!(
        crate::angular_meta::routing::extraction_count(),
        1,
        "Angular evaluation must share one routing shape between markers and semantics"
    );
}

#[test]
fn production_angular_layer_runs_framework_detection_once() {
    crate::angular_meta::reset_detection_count();

    let _ = compile_edges(
        "import { Component } from '@angular/core'; @Component({ template: '' }) export class AppComponent {}",
        "C:/repo/app.component.ts",
    );

    assert_eq!(
        crate::angular_meta::detection_count(),
        1,
        "Angular decorator evidence must not be reparsed during evaluation"
    );
}

#[test]
fn production_dotnet_layer_uses_combined_evaluation_override() {
    crate::dotnet_meta::reset_evaluation_count();

    let _ = compile_csharp_edges(
        "[ApiController] public class UsersController : ControllerBase { [HttpGet] public IActionResult Get() => Ok(); }",
        "C:/repo/UsersController.cs",
    );

    assert_eq!(
        crate::dotnet_meta::evaluation_count(),
        1,
        ".NET must own one explicit combined evaluation per compilation"
    );
}

#[test]
fn production_dotnet_layer_runs_framework_detection_once() {
    crate::dotnet_meta::reset_detection_count();

    let _ = compile_csharp_edges(
        "[ApiController] public class UsersController : ControllerBase { }",
        "C:/repo/UsersController.cs",
    );

    assert_eq!(
        crate::dotnet_meta::detection_count(),
        1,
        ".NET applicability evidence must not be reparsed during evaluation"
    );
}

#[test]
fn production_spring_layer_uses_combined_evaluation_override() {
    crate::spring_meta::reset_evaluation_count();

    let _ = compile_java_edges(
        "@RestController class UsersController { @GetMapping public String get() { return \"ok\"; } }",
        "C:/repo/UsersController.java",
    );

    assert_eq!(
        crate::spring_meta::evaluation_count(),
        1,
        "Spring must own one explicit combined evaluation per compilation"
    );
}

#[test]
fn production_spring_layer_runs_framework_detection_once() {
    crate::spring_meta::reset_detection_count();

    let _ = compile_java_edges(
        "@RestController class UsersController { @GetMapping String get() { return \"ok\"; } }",
        "C:/repo/UsersController.java",
    );

    assert_eq!(
        crate::spring_meta::detection_count(),
        1,
        "Spring applicability evidence must not be reparsed during evaluation"
    );
}

#[test]
fn production_builtin_layer_uses_combined_evaluation_override() {
    crate::layers::meta::builtin::reset_evaluation_count();

    let _ = compile_rust_edges("pub struct Account { id: u64 }", "C:/repo/account.rs");

    assert_eq!(
        crate::layers::meta::builtin::evaluation_count(),
        1,
        "builtin must own one explicit combined evaluation per compilation"
    );
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
