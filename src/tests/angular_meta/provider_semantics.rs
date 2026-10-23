use crate::compression::Fidelity;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use crate::layers::meta::{AngularMetaLayer, MetaLayer};

fn edges(source: &str) -> Vec<SemanticEdge> {
    let captures = source
        .split("/* class */")
        .skip(1)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    AngularMetaLayer::new().extract_semantic_edges(source, &captures, Fidelity::High, None)
}

fn production_edges(source: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("ts").expect("TypeScript language");
    let mut compiler = crate::ir::compiler::IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "issue-138-provider-production",
            Some("C:/fixture/issue138.ts"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("TypeScript production compilation");
    let directly_projected = super::project(
        "C:/fixture/issue138.ts",
        source,
        "AuditModule",
        crate::angular_meta::decorators::ClassKind::Module,
    );
    assert!(
        directly_projected
            .iter()
            .any(|edge| { edge.relation == SemanticRelation::RegistersProvider }),
        "structured provider interpretation rejected the production fixture: {:?}",
        directly_projected
    );
    assert!(
        compiler.semantic_edges.iter().any(|edge| {
            matches!(
                edge.relation,
                SemanticRelation::RegistersProvider
                    | SemanticRelation::ProviderUsesClass
                    | SemanticRelation::ProviderUsesFactory
                    | SemanticRelation::ProviderUsesValue
            )
        }),
        "Angular registry integration dropped provider semantics: {:?}",
        compiler.semantic_edges
    );
    compiler.semantic_edges
}

fn has_edge(
    edges: &[SemanticEdge],
    relation: SemanticRelation,
    subject_type: &str,
    subject: &str,
    object_type: &str,
    object: &str,
) -> bool {
    edges.iter().any(|edge| {
        edge.relation == relation
            && edge.subject.entity_type == subject_type
            && edge.subject.name == subject
            && edge.object.entity_type == object_type
            && edge.object.name == object
    })
}

#[test]
fn provider_structural_authorities_are_joined() {
    let source = "import { NgModule } from '@angular/core'; class ApiClient {} @NgModule({ providers: [ApiClient] }) class AppModule {}";
    let language = crate::compression::language::safe_typescript_language().unwrap();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error(), "fixture must parse");
    let class = super::find_named_class(tree.root_node(), "AppModule", source.as_bytes())
        .expect("AppModule class authority");
    let providers = super::provider_array("standalone-typescript", source, class)
        .expect("Angular decorator/import/providers context authority");
    let mut cursor = providers.walk();
    let entry = providers
        .named_children(&mut cursor)
        .next()
        .expect("provider array entry");
    let binding = super::parse_provider(
        "standalone-typescript",
        source,
        tree.root_node(),
        entry,
        None,
    )
    .expect("provider token/strategy endpoint authority");
    assert_eq!(binding.token, "ApiClient");
    assert_eq!(binding.endpoint, "ApiClient");
}
#[test]
fn ngmodule_use_class_preserves_scope_token_and_implementation() {
    let source = r#"
import { Injectable, InjectionToken, NgModule } from '@angular/core';
const API = new InjectionToken<string>('API');
/* class */@Injectable() class ApiClient {}
/* class */@NgModule({ providers: [{ provide: API, useClass: ApiClient }] })
class AppModule {}
"#;
    let edges = edges(source);
    assert!(has_edge(
        &edges,
        SemanticRelation::RegistersProvider,
        "Module",
        "AppModule",
        "ProviderToken",
        "API",
    ));
    assert!(has_edge(
        &edges,
        SemanticRelation::ProviderUsesClass,
        "ProviderToken",
        "API",
        "ProviderClass",
        "ApiClient",
    ));
    assert!(!edges.iter().any(|edge| {
        edge.relation == SemanticRelation::Injects && edge.subject.name == "AppModule"
    }));
}

#[test]
fn provider_relations_reach_the_production_compiler_boundary() {
    let source = r#"
import { NgModule } from '@angular/core';
class AuditService {}
class AlternateService {}
const API_TOKEN = 'API_TOKEN';
@NgModule({ providers: [
  AuditService,
  { provide: API_TOKEN, useClass: AlternateService }
] })
export class AuditModule {}
"#;
    let edges = production_edges(source);
    assert!(has_edge(
        &edges,
        SemanticRelation::RegistersProvider,
        "Module",
        "AuditModule",
        "ProviderToken",
        "AuditService",
    ));
    assert!(has_edge(
        &edges,
        SemanticRelation::ProviderUsesClass,
        "ProviderToken",
        "API_TOKEN",
        "ProviderClass",
        "AlternateService",
    ));
}

#[test]
fn component_provider_strategies_remain_distinct() {
    let source = r#"
import { Component, InjectionToken } from '@angular/core';
const CLASS_TOKEN = new InjectionToken<object>('class');
const FACTORY_TOKEN = new InjectionToken<object>('factory');
const VALUE_TOKEN = new InjectionToken<number>('value');
class Implementation {}
function createImplementation() { return new Implementation(); }
/* class */@Component({
  selector: 'app-root',
  providers: [
    { provide: CLASS_TOKEN, useClass: Implementation },
    { provide: FACTORY_TOKEN, useFactory: createImplementation },
    { provide: VALUE_TOKEN, useValue: 7 }
  ]
})
class AppComponent {}
"#;
    let edges = edges(source);
    assert!(has_edge(
        &edges,
        SemanticRelation::ProviderUsesClass,
        "ProviderToken",
        "CLASS_TOKEN",
        "ProviderClass",
        "Implementation",
    ));
    assert!(has_edge(
        &edges,
        SemanticRelation::ProviderUsesFactory,
        "ProviderToken",
        "FACTORY_TOKEN",
        "ProviderFactory",
        "createImplementation",
    ));
    assert!(has_edge(
        &edges,
        SemanticRelation::ProviderUsesValue,
        "ProviderToken",
        "VALUE_TOKEN",
        "ProviderValue",
        "7",
    ));
}

#[test]
fn shorthand_provider_requires_and_reuses_declaration_authority() {
    let source = r#"
import { Injectable, NgModule } from '@angular/core';
/* class */@Injectable() class ApiClient {}
/* class */@NgModule({ providers: [ApiClient, MissingClient] }) class AppModule {}
"#;
    let edges = edges(source);
    assert!(has_edge(
        &edges,
        SemanticRelation::RegistersProvider,
        "Module",
        "AppModule",
        "ProviderToken",
        "ApiClient",
    ));
    assert!(!edges.iter().any(|edge| edge.object.name == "MissingClient"));
}

#[test]
fn provider_spelling_outside_proven_angular_decorator_is_negative() {
    let source = r#"
class Fake { static config = { providers: [{ provide: TOKEN, useClass: Impl }] }; }
/* class */class Plain {}
"#;
    assert!(edges(source).iter().all(|edge| {
        !matches!(
            edge.relation,
            SemanticRelation::RegistersProvider
                | SemanticRelation::ProviderUsesClass
                | SemanticRelation::ProviderUsesFactory
                | SemanticRelation::ProviderUsesValue
        )
    }));
}
