//! Issue #139 bounded NgModule value composition and kind authority.
use crate::angular_meta::module_metadata::project;
use crate::layers::meta::semantic::SemanticRelation;

#[test]
fn spreads_compose_bounded_values_and_kinds_come_from_declarations() {
    let root = tempfile::tempdir().unwrap();
    let module_path = root.path().join("app.module.ts");
    std::fs::write(
        root.path().join("shared.ts"),
        r#"import { Directive, Pipe } from '@angular/core';
@Directive({selector:'[shared]'}) export class SharedDirective {}
@Pipe({name:'shared'}) export class SharedPipe {}
export const SHARED = [SharedDirective, SharedPipe];"#,
    )
    .unwrap();
    let source = r#"import { NgModule, Component } from '@angular/core';
import { SHARED } from './shared';
@Component({selector:'app-local'}) class LocalComponent {}
const LOCAL = [LocalComponent];
@NgModule({ declarations: [...LOCAL, ...SHARED], exports: [LocalComponent] })
export class AppModule {}"#;
    std::fs::write(&module_path, source).unwrap();
    let raw = &source[source.find("@NgModule").unwrap()..];
    let result = project("AppModule", raw, source, Some(&module_path), &[]);
    assert!(result.unresolved_spreads.is_empty());

    let actual: Vec<_> = result
        .edges
        .iter()
        .map(|edge| {
            (
                edge.relation,
                edge.object.entity_type,
                edge.object.name.as_str(),
            )
        })
        .collect();
    assert!(actual.contains(&(
        SemanticRelation::DeclaresInModule,
        "Component",
        "LocalComponent"
    )));
    assert!(actual.contains(&(
        SemanticRelation::DeclaresInModule,
        "Directive",
        "SharedDirective"
    )));
    assert!(actual.contains(&(SemanticRelation::DeclaresInModule, "Pipe", "SharedPipe")));
}

#[test]
fn unresolved_spread_is_evidence_and_never_an_entity() {
    let source = r#"import { NgModule } from '@angular/core';
@NgModule({ declarations: [...missing] }) export class AppModule {}"#;
    let raw = &source[source.find("@NgModule").unwrap()..];
    let result = project("AppModule", raw, source, None, &[]);
    assert_eq!(result.unresolved_spreads, vec!["missing"]);
    assert!(
        result
            .edges
            .iter()
            .all(|edge| edge.object.name != "...missing")
    );

    let captures = vec![raw.to_string()];
    let block = crate::angular_meta::run_meta_layer_with_config_and_path(
        source,
        &captures,
        crate::compression::Fidelity::High,
        None,
        std::path::Path::new("app.module.ts"),
    )
    .expect("NgModule should render metadata evidence");
    let rendered = block.render();
    assert!(rendered.contains("Φmodspread:missing"), "{rendered}");
}
#[test]
fn direct_imports_and_router_calls_use_independent_kind_authority() {
    let root = tempfile::tempdir().unwrap();
    let module_path = root.path().join("app.module.ts");
    std::fs::write(
        root.path().join("feature.ts"),
        r#"import { Component, NgModule } from '@angular/core';
@Component({standalone:true, selector:'x-feature'}) export class FeatureComponent {}
@NgModule({}) export class FeatureModule {}"#,
    )
    .unwrap();
    let source = r#"import { NgModule } from '@angular/core';
import { RouterModule } from '@angular/router';
import { FeatureComponent, FeatureModule } from './feature';
const routes = [];
@NgModule({ imports: [FeatureComponent, FeatureModule, RouterModule.forChild(routes)] })
export class AppModule {}"#;
    std::fs::write(&module_path, source).unwrap();
    let raw = &source[source.find("@NgModule").unwrap()..];
    let result = project("AppModule", raw, source, Some(&module_path), &[]);
    assert!(result.edges.iter().any(|edge| {
        edge.relation == SemanticRelation::ImportsModule
            && edge.object.entity_type == "Component"
            && edge.object.name == "FeatureComponent"
    }));
    assert!(result.edges.iter().any(|edge| {
        edge.relation == SemanticRelation::ImportsModule
            && edge.object.entity_type == "Module"
            && edge.object.name == "FeatureModule"
    }));
    assert!(result.edges.iter().any(|edge| {
        edge.relation == SemanticRelation::ImportsModule
            && edge.object.entity_type == "Module"
            && edge.object.name == "RouterModule"
    }));
}
