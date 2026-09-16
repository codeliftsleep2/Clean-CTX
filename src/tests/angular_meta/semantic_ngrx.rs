// Sibling test module for a `#[path]`-loaded test file that exceeded the
// 615-line active-file size ceiling.
//
// Declared from the parent test file with `#[path = "<file>.rs"] mod <name>;`
// -- the same nested-`#[path]` idiom already used by `src/tests/cbm/e2e.rs` --
// so this module is a DESCENDANT of that test module and `use super::*`
// inherits its entire scope (imports, helpers, fixtures). Nothing needed to be
// widened or re-imported.
//
// Pure relocation: the tests below are byte-for-byte the previously inlined
// implementations.

use super::*;

// ── Phase 4e: NgRx site-name semantic values ──────────────────────────
//   - `select('panelState')` surfaces the value `panelState`
//     (not the raw `'panelState'` source slice)
//   - `dispatch({ type: TOGGLE_PANEL })` surfaces `TOGGLE_PANEL`
//     (not the opening `{`)
//   - `dispatch(someAction())` keeps surfacing `someAction`

#[test]
fn ngrx_site_names_quoted_select_and_object_literal_dispatch() {
    let source = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { TOGGLE_PANEL, someAction } from '../store/actions';

@Component({ selector: 'widget-shell' })
export class ShellComponent {
    collapsed$ = this.store.pipe(select('panelState'));

    constructor(private store: Store) {}

    ngOnInit() {
        this.store.dispatch({ type: TOGGLE_PANEL });
        this.store.dispatch(someAction());
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    // Selects: the quoted literal's semantic value, never the raw slice.
    let selects: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Selects)
        .collect();
    assert_eq!(selects.len(), 1, "exactly one Selects edge expected");
    assert_eq!(
        selects[0].object,
        EntityRef::new("ngrx", "Selector", "panelState"),
        "select('panelState') must surface the literal value panelState, \
         not the raw 'panelState' source slice"
    );

    // Dispatches: object-literal and action-creator forms, each exactly once.
    let dispatches: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Dispatches)
        .collect();
    assert_eq!(
        dispatches.len(),
        2,
        "object-literal and action-creator dispatch each produce one edge"
    );
    assert!(
        dispatches
            .iter()
            .all(|e| e.subject == EntityRef::new("angular", "Component", "ShellComponent")),
        "dispatcher subject must be the component class"
    );
    assert!(
        dispatches
            .iter()
            .any(|e| e.object == EntityRef::new("ngrx", "Action", "TOGGLE_PANEL")),
        "dispatch({{ type: TOGGLE_PANEL }}) must surface the TOGGLE_PANEL action"
    );
    assert!(
        dispatches
            .iter()
            .any(|e| e.object == EntityRef::new("ngrx", "Action", "someAction")),
        "dispatch(someAction()) must keep surfacing the someAction action"
    );
    assert!(
        dispatches
            .iter()
            .all(|e| e.object != EntityRef::new("ngrx", "Action", "{")),
        "the object-literal opening brace must never become an action name"
    );
}

// ── Phase 4d: NgModule DeclaresInModule precision ──────────────────

#[test]
fn module_declares_component_and_pipe_with_precise_types() {
    let class_captures = vec![
        r#"
import { Component } from '@angular/core';
@Component({ selector: 'app-user' })
export class UserComponent {}"#
            .to_string(),
        r#"
import { Pipe } from '@angular/core';
@Pipe({ name: 'uppercase' })
export class UpperCasePipe {}"#
            .to_string(),
        r#"
import { NgModule } from '@angular/core';
import { UserComponent } from './user.component';
import { UpperCasePipe } from './uppercase.pipe';

@NgModule({
    declarations: [UserComponent, UpperCasePipe],
})
export class AppModule {}"#
            .to_string(),
    ];
    let source = &class_captures[2];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let declares: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::DeclaresInModule)
        .collect();
    assert_eq!(declares.len(), 2, "should declare two items");

    let declares_component = declares.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Module", "AppModule")
            && e.object == EntityRef::new("angular", "Component", "UserComponent")
    });
    assert!(
        declares_component,
        "AppModule should declare UserComponent as Component"
    );

    let declares_pipe = declares.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Module", "AppModule")
            && e.object == EntityRef::new("angular", "Pipe", "UpperCasePipe")
    });
    assert!(
        declares_pipe,
        "AppModule should declare UpperCasePipe as Pipe"
    );
}
// ── Phase 4d: NgModule exports ─────────────────────────────────────

#[test]
fn module_exports_entity() {
    let class_captures = vec![
        r#"
import { Component } from '@angular/core';
@Component({ selector: 'app-shared' })
export class SharedComponent {}"#
            .to_string(),
        r#"
import { NgModule } from '@angular/core';
import { SharedComponent } from './shared.component';

@NgModule({
    exports: [SharedComponent],
})
export class SharedModule {}"#
            .to_string(),
    ];
    let source = &class_captures[1];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let exports: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::ExportsFromModule)
        .collect();
    assert!(!exports.is_empty(), "should have ExportsFromModule edges");

    let has_export = exports.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Module", "SharedModule")
            && e.object == EntityRef::new("angular", "Component", "SharedComponent")
    });
    assert!(has_export, "SharedModule should export SharedComponent");
}

// ── Phase 4d: Cross-file resolution via WorkspaceIndex ─────────────

#[test]
fn pipe_and_ngrx_edges_in_workspace_index() {
    use crate::workspace::index::WorkspaceIndex;

    let mut idx = WorkspaceIndex::new();

    let pipe_edge = SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: EntityRef::new("angular", "Pipe", "UpperCasePipe"),
        object: EntityRef::new("angular", "PipeName", "uppercase"),
        layer: "angular",
        call_evidence: None,
    };
    let reducer_edge = SemanticEdge {
        relation: SemanticRelation::TriggersReducer,
        subject: EntityRef::new("ngrx", "Action", "loadUsers"),
        object: EntityRef::new("ngrx", "Reducer", "usersReducer"),
        layer: "ngrx",
        call_evidence: None,
    };
    let produces_edge = SemanticEdge {
        relation: SemanticRelation::ProducesAction,
        subject: EntityRef::new("ngrx", "Effect", "loadUsers$"),
        object: EntityRef::new("ngrx", "Action", "loadUsersSuccess"),
        layer: "ngrx",
        call_evidence: None,
    };
    let store_edge = SemanticEdge {
        relation: SemanticRelation::HasStore,
        subject: EntityRef::new("angular", "Component", "UserComponent"),
        object: EntityRef::new("ngrx", "Store", "AppState"),
        layer: "ngrx",
        call_evidence: None,
    };

    idx.add_edges(
        "app.ts",
        vec![pipe_edge, reducer_edge, produces_edge, store_edge],
    );

    let pipe_entities = idx.entities_by_identity("angular", "Pipe", "UpperCasePipe");
    assert_eq!(pipe_entities.len(), 1, "Pipe entity must be registered");

    let action_forward = idx.forward_edges_by_identity("ngrx", "Action", "loadUsers");
    assert!(
        action_forward
            .iter()
            .any(|e| e.relation == SemanticRelation::TriggersReducer),
        "Action must have outgoing TriggersReducer edge"
    );

    let action_reverse = idx.reverse_edges_by_identity("ngrx", "Action", "loadUsersSuccess");
    assert!(
        action_reverse
            .iter()
            .any(|e| e.relation == SemanticRelation::ProducesAction),
        "Action must have incoming ProducesAction edge"
    );

    let store_forward = idx.forward_edges_by_identity("angular", "Component", "UserComponent");
    assert!(
        store_forward
            .iter()
            .any(|e| e.relation == SemanticRelation::HasStore),
        "Component must have outgoing HasStore edge"
    );
}
