// src/tests/angular_meta/graph_ngrx.rs
//
// Unit tests for NgRx cross-layer graph edges (Phase 5 of the Angular
// Ecosystem Deepening). These verify that NgRx store artifacts — actions,
// reducers, effects, selectors, and components — are wired into the
// `AngularGraph` so the LLM can trace
// `dispatch(loadUsers)` → `loadUsers$ effect` → `UserService.getUsers()`
// → `.NET UserController.GetAll()` as a single semantic chain.

use crate::angular_meta::graph::{AngularGraph, AngularGraphBuilder, NgRxEdgeKind};
use crate::angular_meta::ngrx::extract_ngrx_shape;
use crate::compression::Fidelity;

/// Build a graph from a set of NgRx edges.
fn build_graph_with_edges(edges: Vec<(String, String, NgRxEdgeKind)>) -> AngularGraph {
    let mut builder = AngularGraphBuilder::new();
    for (from, to, kind) in edges {
        builder.add_ngrx_edge(&from, &to, kind);
    }
    builder.build()
}

// ── Action → Effect edge ─────────────────────────────────────────────

#[test]
fn action_to_effect_edge_from_of_type() {
    let src = r#"
import { createAction } from '@ngrx/store';
import { createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

export const loadUsers = createAction('[User] Load Users');

export const loadUsers$ = createEffect(() =>
  this.actions$.pipe(
    ofType(loadUsers),
    switchMap(() => this.userService.getUsers()),
    map(users => loadUsersSuccess({ users }))
  )
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let edges = shape.to_graph_edges();

    // Action → Effect edge via ofType(loadUsers).
    assert!(
        edges.iter().any(|(from, to, kind)| {
            from == "Φaction:loadUsers"
                && to == "Φeffect:loadUsers$"
                && *kind == NgRxEdgeKind::ActionEffect
        }),
        "expected Action → Effect edge, got: {:?}",
        edges
    );
}

// ── Effect → Service edge ────────────────────────────────────────────

#[test]
fn effect_to_service_edge_from_switch_map() {
    let src = r#"
import { createAction } from '@ngrx/store';
import { createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

export const loadUsers = createAction('[User] Load Users');

export const loadUsers$ = createEffect(() =>
  this.actions$.pipe(
    ofType(loadUsers),
    switchMap(() => this.userService.getUsers()),
    map(users => ({ type: '[User] Load Users Success', users }))
  )
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let edges = shape.to_graph_edges();

    // Effect → Service edge: switchMap(() => this.userService.getUsers()).
    assert!(
        edges.iter().any(|(from, to, kind)| {
            from == "Φeffect:loadUsers$"
                && to.contains("userService.getUsers")
                && *kind == NgRxEdgeKind::EffectService
        }),
        "Effect → Service edge missing, got: {:?}",
        edges
    );
}

// ── Effect → Action edge ─────────────────────────────────────────────

#[test]
fn effect_to_action_edge_from_map() {
    let src = r#"
import { createAction } from '@ngrx/store';
import { createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

export const loadUsers = createAction('[User] Load Users');
export const loadUsersSuccess = createAction('[User] Load Users Success');

export const loadUsers$ = createEffect(() =>
  this.actions$.pipe(
    ofType(loadUsers),
    switchMap(() => this.userService.getUsers()),
    map(users => loadUsersSuccess({ users }))
  )
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let edges = shape.to_graph_edges();

    // Effect → Action edge: map(users => loadUsersSuccess(...)).
    assert!(
        edges.iter().any(|(from, to, kind)| {
            from == "Φeffect:loadUsers$"
                && to == "Φaction:loadUsersSuccess"
                && *kind == NgRxEdgeKind::EffectAction
        }),
        "Effect → Action edge missing, got: {:?}",
        edges
    );
}

// ── Component → Store edge ───────────────────────────────────────────

#[test]
fn component_to_store_edge_from_di() {
    let src = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';

@Component({ selector: 'app-user', template: '' })
export class UserComponent {
  constructor(private store: Store<AppState>) {}
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let edges = shape.to_graph_edges();

    // Component → Store edge: Store<AppState> DI injection. The `from`
    // node is the actual component class (Φcmp:UserComponent), which the
    // workspace graph pass resolves to `UserComponent@αN` using the file
    // alias — not the `Φstore:` marker.
    assert!(
        edges.iter().any(|(from, to, kind)| {
            from == "Φcmp:UserComponent"
                && to == "Φngrx:Feature"
                && *kind == NgRxEdgeKind::ComponentStore
        }),
        "Component → Store edge missing, got: {:?}",
        edges
    );
}

// ── Component → Selector edge ────────────────────────────────────────

#[test]
fn component_to_selector_edge_from_select() {
    let src = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';

@Component({ selector: 'app-user', template: '' })
export class UserComponent {
  constructor(private store: Store<AppState>) {}

  users$ = this.store.select(selectAllUsers);
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let edges = shape.to_graph_edges();

    // Component → Selector edge: store.select(selectAllUsers). The `from`
    // node is the actual component class (Φcmp:UserComponent), matching
    // the ComponentStore edge semantics.
    assert!(
        edges.iter().any(|(from, to, kind)| {
            from == "Φcmp:UserComponent"
                && to == "Φselector:selectAllUsers"
                && *kind == NgRxEdgeKind::ComponentSelector
        }),
        "Component → Selector edge missing, got: {:?}",
        edges
    );
}

// ── CBM absent graceful skip ─────────────────────────────────────────

#[test]
fn effect_endpoint_edge_graceful_skip_when_cbm_absent() {
    // When CBM is unavailable, `resolve_cross_language_endpoint` returns
    // None and no EffectEndpoint edge is added. This test verifies the
    // graph builder accepts EffectService edges without requiring an
    // EffectEndpoint resolution — the workspace pass silently skips.
    let graph = build_graph_with_edges(vec![(
        "Φeffect:loadUsers$".to_string(),
        "UserService.getUsers".to_string(),
        NgRxEdgeKind::EffectService,
    )]);

    // The graph should contain the EffectService edge but no
    // EffectEndpoint edge (CBM absent → graceful skip).
    let edges = graph.ngrx_edges();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].kind, NgRxEdgeKind::EffectService);
    assert!(
        !edges.iter().any(|e| e.kind == NgRxEdgeKind::EffectEndpoint),
        "EffectEndpoint edge should not exist when CBM is absent"
    );
}

// ── Multi-feature workspace ──────────────────────────────────────────

#[test]
fn multi_feature_workspace_edges() {
    let src = r#"
import { createAction } from '@ngrx/store';
import { createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

export const loadUsers = createAction('[User] Load Users');
export const loadUsersSuccess = createAction('[User] Load Users Success');
export const loadOrders = createAction('[Order] Load Orders');
export const loadOrdersSuccess = createAction('[Order] Load Orders Success');

export const loadUsers$ = createEffect(() =>
  this.actions$.pipe(
    ofType(loadUsers),
    switchMap(() => this.userService.getUsers()),
    map(users => loadUsersSuccess({ users }))
  )
);

export const loadOrders$ = createEffect(() =>
  this.actions$.pipe(
    ofType(loadOrders),
    switchMap(() => this.orderService.getOrders()),
    map(orders => loadOrdersSuccess({ orders }))
  )
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let edges = shape.to_graph_edges();

    // Two Action → Effect edges (one per feature).
    let action_effect: Vec<_> = edges
        .iter()
        .filter(|(_, _, k)| *k == NgRxEdgeKind::ActionEffect)
        .collect();
    assert_eq!(action_effect.len(), 2, "expected 2 Action → Effect edges");

    // Two Effect → Service edges.
    let effect_service: Vec<_> = edges
        .iter()
        .filter(|(_, _, k)| *k == NgRxEdgeKind::EffectService)
        .collect();
    assert_eq!(effect_service.len(), 2, "expected 2 Effect → Service edges");

    // Two Effect → Action edges.
    let effect_action: Vec<_> = edges
        .iter()
        .filter(|(_, _, k)| *k == NgRxEdgeKind::EffectAction)
        .collect();
    assert_eq!(effect_action.len(), 2, "expected 2 Effect → Action edges");
}