use crate::compression::Fidelity;
use crate::ir::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::layers::meta::semantic::SemanticRelation;

fn service_selects(source: &str) -> Vec<(String, String)> {
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler
        .compile_focused(
            source,
            "service-select.ts",
            Some("service-select.ts"),
            crate::compression::language::safe_typescript_language().unwrap(),
            crate::queries::TS_QUERY,
            Fidelity::High,
            None,
            None,
        )
        .expect("fixture compiles");
    compiler
        .semantic_edges
        .into_iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Selects
                && edge.subject.domain == "angular"
                && edge.subject.entity_type == "Service"
        })
        .map(|edge| (edge.subject.name, edge.object.name))
        .collect()
}

fn source(decorator: &str) -> String {
    format!(
        r#"
import {{ Injectable }} from '@angular/core';
import {{ createSelector, Store }} from '@ngrx/store';
const base = (state: unknown) => state;
export const selectAudit = createSelector(base, state => state);
{decorator}
export class AuditService {{
  constructor(private state: Store<AppState>) {{}}
  load() {{ return this.state.select(selectAudit); }}
}}
"#
    )
}

#[test]
fn authoritative_service_store_and_selector_publish_service_ownership() {
    assert_eq!(
        service_selects(&source("@Injectable()")),
        [("AuditService".into(), "selectAudit".into())]
    );
}

#[test]
fn ordinary_class_with_authoritative_operation_is_not_a_service_owner() {
    assert!(service_selects(&source("")).is_empty());
}

#[test]
fn component_and_fake_service_name_are_not_service_authority() {
    let component = source("@Injectable()")
        .replace("@Injectable()", "@Component({ selector: 'audit' })")
        .replace("import { Injectable }", "import { Component }")
        .replace("AuditService", "AuditComponent");
    assert!(service_selects(&component).is_empty());
    assert!(service_selects(&source("")).is_empty());
}

#[test]
fn all_three_authorities_are_independently_required() {
    let valid = source("@Injectable()");
    assert_eq!(service_selects(&valid).len(), 1);
    assert!(service_selects(&valid.replace("@ngrx/store';", "./foreign-store';")).is_empty());
    assert!(
        service_selects(&valid.replace(
            "constructor(private state: Store<AppState>) {}",
            "constructor(private state: FakeStore) {}"
        ))
        .is_empty()
    );
    assert!(
        service_selects(&valid.replace("select(selectAudit)", "select(notASelector)")).is_empty()
    );
}

#[test]
fn multiple_services_and_selectors_preserve_exact_ownership() {
    let source = r#"
import { Injectable } from '@angular/core';
import { createSelector, Store } from '@ngrx/store';
const base = (state: unknown) => state;
export const selectFirst = createSelector(base, state => state);
export const selectSecond = createSelector(base, state => state);
@Injectable()
export class FirstService {
  constructor(private state: Store<AppState>) {}
  load() { this.state.select(selectFirst); return this.state.select(selectSecond); }
}
@Injectable()
export class SecondService {
  constructor(private foo: Store<AppState>) {}
  load() { return this.foo.select(selectSecond); }
}
export class Helper {
  constructor(private state: Store<AppState>) {}
  load() { return this.state.select(selectFirst); }
}
"#;
    let mut edges = service_selects(source);
    edges.sort();
    assert_eq!(
        edges,
        [
            ("FirstService".into(), "selectFirst".into()),
            ("FirstService".into(), "selectSecond".into()),
            ("SecondService".into(), "selectSecond".into()),
        ]
    );
}

#[test]
fn same_selector_can_have_multiple_exact_service_owners() {
    let source = source("@Injectable()")
        .replace("export class AuditService", "export class FirstService")
        + &source("@Injectable()")
            .replace(
                "export const selectAudit = createSelector(base, state => state);",
                "",
            )
            .replace("const base = (state: unknown) => state;", "")
            .replace("export class AuditService", "export class SecondService")
            .replace("import { Injectable } from '@angular/core';", "")
            .replace("import { createSelector, Store } from '@ngrx/store';", "");
    let mut edges = service_selects(&source);
    edges.sort();
    assert_eq!(
        edges,
        [
            ("FirstService".into(), "selectAudit".into()),
            ("SecondService".into(), "selectAudit".into()),
        ]
    );
}

#[test]
fn nested_arrow_static_and_constructor_contexts_do_not_inherit_ownership() {
    let valid = source("@Injectable()");
    for body in [
        "load() { function inner() { return this.state.select(selectAudit); } return inner(); }",
        "load() { const helper = () => this.state.select(selectAudit); return helper(); }",
        "static load() { return this.state.select(selectAudit); }",
        "load() { return selectAudit; }",
    ] {
        let candidate = valid.replace("load() { return this.state.select(selectAudit); }", body);
        assert!(service_selects(&candidate).is_empty(), "{body}");
    }
}

#[test]
fn source_mutation_retracts_and_restores_service_ownership() {
    let valid = source("@Injectable()");
    assert_eq!(service_selects(&valid).len(), 1);
    assert!(service_selects(&valid.replace("@Injectable()", "")).is_empty());
    assert!(service_selects(&valid.replace("@ngrx/store';", "./foreign';")).is_empty());
    assert!(
        service_selects(&valid.replace("createSelector(base, state => state)", "ordinaryValue"))
            .is_empty()
    );
    assert_eq!(
        service_selects(&valid.replace("state", "appStore")).len(),
        1
    );
}
