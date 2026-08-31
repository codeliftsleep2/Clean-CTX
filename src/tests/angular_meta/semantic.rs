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
