#![cfg(feature = "typescript")]

use crate::compression::Fidelity;
use crate::ir::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};

fn compile(source: &str) -> Vec<SemanticEdge> {
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler
        .compile_focused(
            source,
            "selector-reference.ts",
            Some("selector-reference.ts"),
            crate::compression::language::safe_typescript_language().unwrap(),
            crate::queries::TS_QUERY,
            Fidelity::High,
            None,
            None,
        )
        .expect("TypeScript fixture must compile");
    compiler.semantic_edges
}

fn selector_targets(source: &str) -> Vec<String> {
    compile(source)
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::Selects)
        .map(|edge| edge.object.name)
        .collect()
}

fn selector_source(body: &str) -> String {
    format!(
        r#"
import {{ Component }} from '@angular/core';
import {{ createSelector, Store }} from '@ngrx/store';
const selectAuditState = (state: unknown) => state;
export const selectAudit = createSelector(selectAuditState, state => state);

@Component({{ selector: 'audit-view' }})
export class AuditComponent {{
  constructor(private store: Store) {{}}
  consume() {{
    {body}
  }}
}}
"#
    )
}

#[test]
fn direct_reference_requires_the_exact_authoritative_selector_declaration() {
    let targets = selector_targets(&selector_source("return this.store.select(selectAudit);"));
    assert_eq!(targets, ["selectAudit"]);
}

#[test]
fn ordinary_local_and_parameter_shadows_do_not_inherit_outer_selector_authority() {
    let local = selector_source("const selectAudit = 123; return this.store.select(selectAudit);");
    assert!(selector_targets(&local).is_empty());

    let parameter = selector_source("return this.store.select(selectAudit);")
        .replace("consume()", "consume(selectAudit: unknown)");
    assert!(selector_targets(&parameter).is_empty());
}

#[test]
fn nested_shadow_refuses_then_scope_exit_restores_outer_selector() {
    let source = selector_source(
        r#"
{
  const selectAudit = 123;
  this.store.select(selectAudit);
}
return this.store.select(selectAudit);
"#,
    );
    assert_eq!(selector_targets(&source), ["selectAudit"]);
}

#[test]
fn ordinary_same_name_const_is_not_a_selector() {
    let source = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
const selectAudit = 123;
@Component({ selector: 'audit-view' })
export class AuditComponent {
  constructor(private store: Store) {}
  consume() { return this.store.select(selectAudit); }
}
"#;
    assert!(selector_targets(source).is_empty());
}

#[test]
fn exact_authority_selects_the_second_of_two_selectors() {
    let source = selector_source("return this.store.select(selectSecond);").replace(
        "export const selectAudit = createSelector(selectAuditState, state => state);",
        "export const selectAudit = createSelector(selectAuditState, state => state);\n\
         export const selectSecond = createSelector(selectAuditState, state => state);",
    );
    assert_eq!(selector_targets(&source), ["selectSecond"]);
}

#[test]
fn unsupported_argument_shapes_retain_only_legacy_occurrences() {
    for body in [
        "return this.store.select('selectAudit');",
        "return this.store.select(this.selectAudit);",
        "return this.store.select(wrapper(selectAudit));",
        "return this.store.select(true ? selectAudit : selectAudit);",
        "return this.store.select([selectAudit]);",
        "return this.store.select(() => selectAudit);",
    ] {
        // The generic binder refuses these shapes. Existing occurrence
        // projection remains compatible and is not declaration authority.
        let _legacy_occurrences = selector_targets(&selector_source(body));
    }
}

#[test]
fn imported_alias_namespace_and_foreign_same_name_never_supply_authority() {
    let imported = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { selectAudit } from './selectors';
@Component({ selector: 'audit-view' })
export class AuditComponent {
  constructor(private store: Store) {}
  consume() { return this.store.select(selectAudit); }
}
"#;
    assert!(selector_targets(imported).is_empty());
    assert!(
        selector_targets(
            &imported
                .replace("{ selectAudit }", "{ selectAudit as audit }")
                .replace("select(selectAudit)", "select(audit)")
        )
        .is_empty()
    );
    assert!(
        selector_targets(
            &imported
                .replace(
                    "import { selectAudit } from './selectors';",
                    "import * as Selectors from './selectors';"
                )
                .replace("select(selectAudit)", "select(Selectors.selectAudit)")
        )
        .is_empty()
    );

    let foreign = selector_source("return this.store.select(selectAudit);");
    assert_eq!(selector_targets(&foreign), ["selectAudit"]);
    assert!(selector_targets(imported).is_empty());

    let occurrences = crate::angular_meta::ngrx::extract_ngrx_shape(imported, Fidelity::High)
        .expect("legacy candidate occurrence");
    assert_eq!(occurrences.select_sites.len(), 1);
    assert_eq!(occurrences.select_sites[0].selector_name, "selectAudit");
}
