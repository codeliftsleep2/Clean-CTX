use crate::compression::Fidelity;
use crate::ir::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::layers::meta::semantic::SemanticRelation;

fn select_targets(source: &str) -> Vec<String> {
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler
        .compile_focused(
            source,
            "receiver.ts",
            Some("receiver.ts"),
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
        .filter(|edge| edge.relation == SemanticRelation::Selects)
        .map(|edge| edge.object.name)
        .collect()
}

fn source(store_import: &str, property: &str, property_type: &str) -> String {
    format!(
        r#"
import {{ Injectable }} from '@angular/core';
import {{ createSelector }} from '@ngrx/store';
import {{ {property_type} }} from '{store_import}';
const selectAuditState = (state: unknown) => state;
export const selectAudit = createSelector(selectAuditState, state => state);
@Injectable()
export class AuditService {{
  constructor(private {property}: {property_type}<AppState>) {{}}
  load() {{ return this.{property}.select(selectAudit); }}
}}
"#
    )
}

#[test]
fn arbitrary_receiver_name_with_exact_ngrx_store_authority_is_admitted() {
    assert_eq!(
        select_targets(&source("@ngrx/store", "state", "Store")),
        ["selectAudit"]
    );
}

#[test]
fn historical_store_name_with_foreign_store_authority_is_refused() {
    assert!(select_targets(&source("./my-own-store", "store", "Store")).is_empty());
}

#[test]
fn arbitrary_property_names_have_identical_store_authority() {
    for property in ["store", "state", "appStore", "foo"] {
        assert_eq!(
            select_targets(&source("@ngrx/store", property, "Store")),
            ["selectAudit"],
            "{property}"
        );
    }
}

#[test]
fn foreign_modules_and_wrong_symbol_from_ngrx_store_are_refused() {
    for module in [
        "./store",
        "@ngrx/effects",
        "@ngrx/store-devtools",
        "./ngrx-barrel",
    ] {
        assert!(
            select_targets(&source(module, "store", "Store")).is_empty(),
            "{module}"
        );
    }
    assert!(select_targets(&source("@ngrx/store", "store", "SomethingElse")).is_empty());
}

#[test]
fn fake_store_and_authoritative_selector_do_not_bootstrap_receiver_truth() {
    let fake = r#"
import { Injectable } from '@angular/core';
import { createSelector } from '@ngrx/store';
const base = (state: unknown) => state;
export const selectAudit = createSelector(base, state => state);
class FakeStore { select(value: unknown) { return value; } }
@Injectable()
export class AuditService {
  private store = new FakeStore();
  load() { return this.store.select(selectAudit); }
}
"#;
    assert!(select_targets(fake).is_empty());
}

#[test]
fn exact_member_wins_among_multiple_store_like_members_and_classes() {
    let source = r#"
import { createSelector, Store } from '@ngrx/store';
import { FakeStore } from './fake';
const base = (state: unknown) => state;
export const selectAudit = createSelector(base, state => state);
class First {
  constructor(private state: Store<AppState>, private store: FakeStore) {}
  run() { this.state.select(selectAudit); this.store.select(selectAudit); }
}
class Second {
  constructor(private state: FakeStore) {}
  run() { this.state.select(selectAudit); }
}
"#;
    assert_eq!(select_targets(source), ["selectAudit"]);
}

#[test]
fn unsupported_import_and_this_context_forms_fail_closed() {
    for source in [
        source("@ngrx/store", "state", "NgStore").replace(
            "import { NgStore } from '@ngrx/store';",
            "import { Store as NgStore } from '@ngrx/store';",
        ),
        source("@ngrx/store", "state", "Store").replace(
            "import { Store } from '@ngrx/store';",
            "import type { Store } from '@ngrx/store';",
        ),
        source("@ngrx/store", "state", "Store").replace(
            "load() { return this.state.select(selectAudit); }",
            "static load() { return this.state.select(selectAudit); }",
        ),
        source("@ngrx/store", "state", "Store").replace(
            "load() { return this.state.select(selectAudit); }",
            "load() { function nested() { return this.state.select(selectAudit); } return nested(); }",
        ),
    ] {
        assert!(select_targets(&source).is_empty(), "{source}");
    }
}

#[test]
fn current_source_mutation_recomputes_authority() {
    assert_eq!(
        select_targets(&source("@ngrx/store", "state", "Store")),
        ["selectAudit"]
    );
    assert!(select_targets(&source("./store", "state", "Store")).is_empty());
    assert_eq!(
        select_targets(&source("@ngrx/store", "foo", "Store")),
        ["selectAudit"]
    );
    assert!(select_targets(&source("@ngrx/store", "foo", "FakeStore")).is_empty());
}
