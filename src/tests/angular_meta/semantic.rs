// src/tests/angular_meta/semantic.rs
//
// Tests for Angular semantic edge extraction (semantic plan Phase 1).
//
// Verifies that extract_semantic_edges produces the correct SemanticEdge
// objects from Angular source files WITHOUT modifying the existing Φ
// output.

use crate::compression::Fidelity;
use crate::layers::meta::AngularMetaLayer;
use crate::layers::meta::MetaLayer;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

// ── Component → Service Injection ─────────────────────────────────────

#[test]
fn component_injects_service() {
    let source = r#"
import { Component } from '@angular/core';
import { UserService } from './user.service';

@Component({ selector: 'app-user' })
export class UserComponent {
    constructor(private userSvc: UserService) {}
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let injects: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .collect();
    assert!(!injects.is_empty(), "should have at least one Injects edge");

    let cmp_entity = EntityRef::new("angular", "Component", "UserComponent");
    let svc_entity = EntityRef::new("angular", "Service", "UserService");
    let has_injects = injects
        .iter()
        .any(|e| e.subject == cmp_entity && e.object == svc_entity);
    assert!(has_injects, "UserComponent should inject UserService");
}

// ── NgRx Effect → Action ─────────────────────────────────────────────

#[test]
fn ngrx_effect_handles_action() {
    let source = r#"
import { createAction, createEffect, ofType } from '@ngrx/effects';
import { exhaustMap } from 'rxjs';

export const loadUsers = createAction('[Users] Load Users');
export const loadUsers$ = createEffect(() =>
    this.actions$.pipe(
        ofType(loadUsers),
        exhaustMap(() => this.userService.getAll())
    )
);
"#;
    let class_captures: Vec<String> = vec![];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let handles: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::HandlesAction)
        .collect();
    assert!(
        !handles.is_empty(),
        "should have at least one HandlesAction edge"
    );

    let has_effect_action = handles.iter().any(|e| {
        e.subject == EntityRef::new("ngrx", "Effect", "loadUsers$")
            && e.object == EntityRef::new("ngrx", "Action", "loadUsers")
    });
    assert!(
        has_effect_action,
        "loadUsers$ effect should handle loadUsers action"
    );
}
// ── NgRx Dispatch → Action ───────────────────────────────────────────

#[test]
fn ngrx_dispatch_to_action() {
    let source = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { loadUsers } from './store/actions';

@Component({ selector: 'app-user' })
export class UserComponent {
    constructor(private store: Store) {}

    load() {
        this.store.dispatch(loadUsers());
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let dispatches: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Dispatches)
        .collect();
    assert!(
        !dispatches.is_empty(),
        "should have at least one Dispatches edge"
    );

    let has_dispatch = dispatches
        .iter()
        .any(|e| e.object == EntityRef::new("ngrx", "Action", "loadUsers"));
    assert!(has_dispatch, "should dispatch loadUsers action");
}

// ── Route → Component ────────────────────────────────────────────────

#[test]
fn route_maps_to_component() {
    let source = r#"
import { Routes } from '@angular/router';
import { UserListComponent } from './user-list/user-list.component';

export const routes: Routes = [
    { path: 'users', component: UserListComponent }
];
"#;
    let class_captures: Vec<String> = vec![];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let route_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::RouteMapsTo)
        .collect();
    assert!(
        !route_edges.is_empty(),
        "should have at least one RouteMapsTo edge"
    );

    let has_route = route_edges.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Route", "users")
            && e.object == EntityRef::new("angular", "Component", "UserListComponent")
    });
    assert!(has_route, "route 'users' should map to UserListComponent");
}

// ── Route → Guard ────────────────────────────────────────────────────

#[test]
fn route_guarded_by_auth_guard() {
    let source = r#"
import { Routes } from '@angular/router';
import { AuthGuard } from './auth.guard';

export const routes: Routes = [
    { path: 'admin', component: AdminComponent, canActivate: [AuthGuard] }
];
"#;
    let class_captures: Vec<String> = vec![];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let guard_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::GuardedBy)
        .collect();
    assert!(
        !guard_edges.is_empty(),
        "should have at least one GuardedBy edge"
    );

    let has_guard = guard_edges.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Route", "admin")
            && e.object == EntityRef::new("angular", "Guard", "AuthGuard")
    });
    assert!(has_guard, "route 'admin' should be guarded by AuthGuard");
}

// ── Route → Resolver ─────────────────────────────────────────────────

#[test]
fn route_resolved_by_resolver() {
    let source = r#"
import { Routes } from '@angular/router';
import { UserResolver } from './user.resolver';

export const routes: Routes = [
    { path: 'users', component: UserListComponent, resolve: { users: UserResolver } }
];
"#;
    let class_captures: Vec<String> = vec![];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let resolver_edges: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::ResolvedBy)
        .collect();
    assert!(
        !resolver_edges.is_empty(),
        "should have at least one ResolvedBy edge"
    );

    let has_resolver = resolver_edges.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Route", "users")
            && e.object == EntityRef::new("angular", "Resolver", "UserResolver")
    });
    assert!(
        has_resolver,
        "route 'users' should be resolved by UserResolver"
    );
}

// ── Φ Output Unchanged ───────────────────────────────────────────────

#[test]
fn semantic_extraction_does_not_alter_phi_output() {
    let source = r#"
import { Component } from '@angular/core';
import { UserService } from './user.service';

@Component({ selector: 'app-user', template: '<p>Hello</p>' })
export class UserComponent {
    constructor(private userSvc: UserService) {}
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();

    // Call extract_semantic_edges first — must not mutate any shared state
    // that would alter subsequent enrich() output.
    let _edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    // Then enrich — output must be deterministic and unchanged.
    // Fidelity::High is required to emit Φinjects: markers (F-ANG-23).
    let output = layer.enrich(source, &class_captures, Fidelity::High, None);
    let rendered = output.map(|o| o.rendered).unwrap_or_default();

    assert!(
        rendered.contains("Φcmp:"),
        "enrich() must still produce Φcmp: markers after semantic extraction"
    );
    assert!(
        rendered.contains("Φinjects:"),
        "enrich() must still produce Φinjects: markers after semantic extraction"
    );
}
// ── Phase 4d: Pipe semantic representation ──────────────────────────

#[test]
fn pipe_entity_has_pipe_name() {
    let source = r#"
import { Pipe } from '@angular/core';

@Pipe({ name: 'uppercase' })
export class UpperCasePipe {
    transform(value: string): string {
        return value.toUpperCase();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let defines: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Defines)
        .collect();
    assert!(
        !defines.is_empty(),
        "should have a Defines edge for the pipe"
    );

    let has_pipe = defines.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Pipe", "UpperCasePipe")
            && e.object == EntityRef::new("angular", "PipeName", "uppercase")
    });
    assert!(
        has_pipe,
        "UpperCasePipe should define pipe name 'uppercase'"
    );
}

#[test]
fn pipe_entity_is_registered() {
    let source = r#"
import { Pipe } from '@angular/core';

@Pipe({ name: 'lowercase' })
export class LowerCasePipe {
    transform(value: string): string {
        return value.toLowerCase();
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);
    let pipe_entity = EntityRef::new("angular", "Pipe", "LowerCasePipe");
    let has_pipe_subject = edges.iter().any(|e| e.subject == pipe_entity);
    assert!(
        has_pipe_subject,
        "Pipe entity must appear as an edge subject"
    );
}

// ── Phase 4d: NgRx Action → Reducer ────────────────────────────────

#[test]
fn ngrx_action_triggers_reducer() {
    let source = r#"
import { createAction, createReducer, on } from '@ngrx/store';

export const loadUsers = createAction('[Users] Load Users');
export const loadUsersSuccess = createAction('[Users] Load Users Success');

export const usersReducer = createReducer(
    initialState,
    on(loadUsers, (state) => ({ ...state, loading: true })),
    on(loadUsersSuccess, (state, { users }) => ({ ...state, users, loading: false }))
);
"#;
    let class_captures: Vec<String> = vec![];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let triggers: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::TriggersReducer)
        .collect();
    assert!(!triggers.is_empty(), "should have TriggersReducer edges");

    let has_load_users = triggers.iter().any(|e| {
        e.subject == EntityRef::new("ngrx", "Action", "loadUsers")
            && e.object == EntityRef::new("ngrx", "Reducer", "usersReducer")
    });
    assert!(
        has_load_users,
        "loadUsers action should trigger usersReducer"
    );
}

#[test]
fn ngrx_effect_produces_action() {
    let source = r#"
import { createAction, createEffect, ofType } from '@ngrx/effects';
import { exhaustMap, map } from 'rxjs';

export const loadUsers = createAction('[Users] Load Users');
export const loadUsersSuccess = createAction('[Users] Load Users Success');

export const loadUsers$ = createEffect(() =>
    this.actions$.pipe(
        ofType(loadUsers),
        exhaustMap(() =>
            this.userService.getAll().pipe(
                map((users) => loadUsersSuccess({ users }))
            )
        )
    )
);
"#;
    let class_captures: Vec<String> = vec![];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let produces: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::ProducesAction)
        .collect();
    assert!(!produces.is_empty(), "should have ProducesAction edges");

    let has_success = produces.iter().any(|e| {
        e.subject == EntityRef::new("ngrx", "Effect", "loadUsers$")
            && e.object == EntityRef::new("ngrx", "Action", "loadUsersSuccess")
    });
    assert!(
        has_success,
        "loadUsers$ effect should produce loadUsersSuccess action"
    );
}
// ── Phase 4d: NgRx Component → Store ───────────────────────────────

#[test]
fn ngrx_component_has_store() {
    let source = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { AppState } from './store/state';

@Component({ selector: 'app-user' })
export class UserComponent {
    constructor(private store: Store<AppState>) {}
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let has_store: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::HasStore)
        .collect();
    assert!(!has_store.is_empty(), "should have HasStore edges");

    let has_component_store = has_store.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Component", "UserComponent")
            && e.object == EntityRef::new("ngrx", "Store", "AppState")
    });
    assert!(
        has_component_store,
        "UserComponent should have a Store<AppState>"
    );
}

// ── Phase 4d: NgRx Component → Selector with correct entity ────────

#[test]
fn ngrx_component_selects_selector() {
    let source = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { selectAllUsers } from './store/selectors';

@Component({ selector: 'app-user' })
export class UserComponent {
    constructor(private store: Store) {}

    ngOnInit() {
        this.store.select(selectAllUsers);
    }
}
"#;
    let class_captures = vec![source.to_string()];
    let layer = AngularMetaLayer::new();
    let edges = layer.extract_semantic_edges(source, &class_captures, Fidelity::High, None);

    let selects: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Selects)
        .collect();
    assert!(!selects.is_empty(), "should have Selects edges");

    let has_component_select = selects.iter().any(|e| {
        e.subject == EntityRef::new("angular", "Component", "UserComponent")
            && e.object == EntityRef::new("ngrx", "Selector", "selectAllUsers")
    });
    assert!(
        has_component_select,
        "UserComponent should select selectAllUsers as Component"
    );
}

// ── HasSelector literal selector-value regression ────────────────────
// The HasSelector object's EntityRef.name MUST be the exact CSS selector
// string declared in the @Component decorator, without any artificial
// encoding (no bracket marker, no normalization). Three distinct selector
// forms MUST remain distinct.

#[test]
fn has_selector_preserves_literal_selector_forms() {
    // Element selector: selector: 'app-widget'
    let element_source = r#"
import { Component } from '@angular/core';
@Component({ selector: 'app-widget' })
export class WidgetComponent {}
"#;
    // Attribute selector: selector: '[app-widget]'
    let attribute_source = r#"
import { Component } from '@angular/core';
@Component({ selector: '[app-widget]' })
export class WidgetComponent {}
"#;
    // Class selector: selector: '.app-widget'
    let class_source = r#"
import { Component } from '@angular/core';
@Component({ selector: '.app-widget' })
export class WidgetComponent {}
"#;

    let layer = AngularMetaLayer::new();

    let element_edges = layer.extract_semantic_edges(
        element_source,
        &[element_source.to_string()],
        Fidelity::High,
        None,
    );
    let attribute_edges = layer.extract_semantic_edges(
        attribute_source,
        &[attribute_source.to_string()],
        Fidelity::High,
        None,
    );
    let class_edges = layer.extract_semantic_edges(
        class_source,
        &[class_source.to_string()],
        Fidelity::High,
        None,
    );

    // Element selector: stored verbatim, no wrapping.
    let element_sel = element_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::HasSelector)
        .expect("element selector edge must exist");
    assert_eq!(
        element_sel.object,
        EntityRef::new("angular", "Component", "app-widget"),
        "element selector 'app-widget' must be stored as the bare string"
    );

    // Attribute selector: stored verbatim, including its square brackets.
    let attribute_sel = attribute_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::HasSelector)
        .expect("attribute selector edge must exist");
    assert_eq!(
        attribute_sel.object,
        EntityRef::new("angular", "Component", "[app-widget]"),
        "attribute selector '[app-widget]' must be stored verbatim, not double-bracketed"
    );

    // Class selector: stored verbatim, including its leading dot.
    let class_sel = class_edges
        .iter()
        .find(|e| e.relation == SemanticRelation::HasSelector)
        .expect("class selector edge must exist");
    assert_eq!(
        class_sel.object,
        EntityRef::new("angular", "Component", ".app-widget"),
        "class selector '.app-widget' must be stored verbatim"
    );

    // The three forms MUST remain distinct semantic identities.
    assert_ne!(
        element_sel.object, attribute_sel.object,
        "element selector 'app-widget' and attribute selector '[app-widget]' must be distinct"
    );
    assert_ne!(
        element_sel.object, class_sel.object,
        "element selector 'app-widget' and class selector '.app-widget' must be distinct"
    );
    assert_ne!(
        attribute_sel.object, class_sel.object,
        "attribute selector '[app-widget]' and class selector '.app-widget' must be distinct"
    );
}

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
    };
    let reducer_edge = SemanticEdge {
        relation: SemanticRelation::TriggersReducer,
        subject: EntityRef::new("ngrx", "Action", "loadUsers"),
        object: EntityRef::new("ngrx", "Reducer", "usersReducer"),
        layer: "ngrx",
    };
    let produces_edge = SemanticEdge {
        relation: SemanticRelation::ProducesAction,
        subject: EntityRef::new("ngrx", "Effect", "loadUsers$"),
        object: EntityRef::new("ngrx", "Action", "loadUsersSuccess"),
        layer: "ngrx",
    };
    let store_edge = SemanticEdge {
        relation: SemanticRelation::HasStore,
        subject: EntityRef::new("angular", "Component", "UserComponent"),
        object: EntityRef::new("ngrx", "Store", "AppState"),
        layer: "ngrx",
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
