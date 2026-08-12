// src/tests/angular_meta/ngrx.rs
//
// Unit tests for the NgRx Meta-Layer (Phase 2 of the Angular
// Ecosystem Deepening).

use crate::angular_meta::phi::PhiMarker;
use crate::angular_meta::ngrx::{
    expand_phi, expand_phi_in_line, extract_ngrx_shape, has_ngrx_imports,
    NgRxKind,
};
use crate::compression::Fidelity;

// ── Import gate ────────────────────────────────────────────────────

#[test]
fn detects_ngrx_store_import() {
    let src = "import { createAction } from '@ngrx/store';";
    assert!(has_ngrx_imports(src));
}

#[test]
fn detects_ngrx_effects_import() {
    let src = "import { createEffect } from '@ngrx/effects';";
    assert!(has_ngrx_imports(src));
}

#[test]
fn detects_ngrx_entity_import() {
    let src = "import { createEntityAdapter } from '@ngrx/entity';";
    assert!(has_ngrx_imports(src));
}

#[test]
fn rejects_non_ngrx_imports() {
    let src = "import { Component } from '@angular/core';";
    assert!(!has_ngrx_imports(src));
}

// ── Action extraction ──────────────────────────────────────────────

#[test]
fn extracts_action_with_event_string() {
    let src = r#"
import { createAction } from '@ngrx/store';

export const loadUsers = createAction('[User] Load Users');
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.actions.len(), 1);
    assert_eq!(shape.actions[0].name, "loadUsers");
    assert_eq!(shape.actions[0].event_string, "[User] Load Users");
    assert!(shape.actions[0].props_type.is_none());
}

#[test]
fn extracts_action_with_props() {
    let src = r#"
import { createAction, props } from '@ngrx/store';

export const loadUsersSuccess = createAction(
  '[User] Load Users Success',
  props<{ users: User[] }>()
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.actions.len(), 1);
    assert_eq!(shape.actions[0].name, "loadUsersSuccess");
    assert_eq!(shape.actions[0].event_string, "[User] Load Users Success");
    assert!(shape.actions[0].props_type.as_deref().unwrap_or("").contains("users"));
}

#[test]
fn extracts_multiple_actions() {
    let src = r#"
import { createAction, props } from '@ngrx/store';

export const loadUsers = createAction('[User] Load Users');
export const loadUsersSuccess = createAction('[User] Load Users Success', props<{ users: User[] }>());
export const loadUsersFailure = createAction('[User] Load Users Failure', props<{ error: string }>());
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.actions.len(), 3);
    assert_eq!(shape.actions[0].name, "loadUsers");
    assert_eq!(shape.actions[1].name, "loadUsersSuccess");
    assert_eq!(shape.actions[2].name, "loadUsersFailure");
}

// ── Reducer extraction ─────────────────────────────────────────────

#[test]
fn extracts_reducer_with_transitions() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';

export const userReducer = createReducer(
  initialState,
  on(loadUsers, (state) => ({ ...state, loading: true })),
  on(loadUsersSuccess, (state, { users }) => ({ ...state, loading: false, users }))
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let reducer = shape.reducer.expect("should have reducer");
    assert_eq!(reducer.name, "userReducer");
    assert_eq!(reducer.transitions.len(), 2);
    assert_eq!(reducer.transitions[0].action_name, "loadUsers");
    assert_eq!(reducer.transitions[1].action_name, "loadUsersSuccess");
}

// ── Round-3 audit: destructured action props must not leak into the
// state summary. `(state, { users }) => ({ ...state, users })` should
// produce `...state, users`, NOT `users }`.
#[test]
fn reducer_summary_skips_destructured_props() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';

export const userReducer = createReducer(
  initialState,
  on(loadUsersSuccess, (state, { users }) => ({ ...state, loading: false, users }))
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let reducer = shape.reducer.expect("should have reducer");
    assert_eq!(reducer.transitions.len(), 1);
    let summary = &reducer.transitions[0].state_summary;
    assert!(
        summary.contains("...state"),
        "summary should contain the returned state object, got: {}",
        summary
    );
    assert!(
        !summary.starts_with("users"),
        "summary must not start with the destructured prop, got: {}",
        summary
    );
}

// ── Round-3 audit: braced arrow bodies in effects.
// `switchMap(() => { return svc.getAll(); })` must yield `svc.getAll`,
// not `{ return svc.getAll(); }`.
#[test]
fn effect_service_call_strips_braced_arrow_body() {
    let src = r#"
import { Injectable } from '@angular/core';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

@Injectable()
export class UserEffects {
  loadUsers$ = createEffect(() =>
    this.actions$.pipe(
      ofType(loadUsers),
      switchMap(() => {
        return this.userService.getUsers();
      }),
      map(users => loadUsersSuccess({ users }))
    )
  );
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.effects.len(), 1);
    let svc = shape.effects[0].service_call.as_deref().unwrap_or("");
    assert!(
        svc.contains("getUsers"),
        "service call should contain getUsers, got: {}",
        svc
    );
    assert!(
        !svc.contains('{') && !svc.contains('}'),
        "service call must not contain braces, got: {}",
        svc
    );
}

// ── Round-3 audit: barrel-import fallback. A file that calls NgRx
// creators without importing from @ngrx/* directly (re-exported via a
// local barrel) must still be detected.
#[test]
fn barrel_import_fallback_detects_creator_calls() {
    let src = r#"
import { createAction } from './store-barrel';

export const loadUsers = createAction('[User] Load Users');
"#;
    assert!(has_ngrx_imports(src), "barrel-imported createAction must be detected");
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx via barrel");
    assert_eq!(shape.actions.len(), 1);
    assert_eq!(shape.actions[0].name, "loadUsers");
}

// ── Round-3 audit: generic `createAction<T>(` form.
#[test]
fn extracts_generic_create_action() {
    let src = r#"
import { createAction } from '@ngrx/store';

export const loadUser = createAction<{ id: string }>('[User] Load User');
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.actions.len(), 1);
    assert_eq!(shape.actions[0].name, "loadUser");
    assert_eq!(shape.actions[0].event_string, "[User] Load User");
    assert!(
        shape.actions[0].props_type.as_deref().unwrap_or("").contains("id"),
        "generic type param should be captured as props_type"
    );
}

// ── Round-3 audit: `store.pipe(select(...))` and bare `store` forms.
#[test]
fn extracts_pipe_select_and_bare_store_sites() {
    let src = r#"
import { Store } from '@ngrx/store';

export class UserComponent {
  constructor(private store: Store<AppState>) {}

  ngOnInit(): void {
    store.dispatch(loadUsers());
    store.pipe(select(selectAllUsers));
  }
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.dispatch_sites.len(), 1);
    assert_eq!(shape.dispatch_sites[0].action_name, "loadUsers");
    assert_eq!(shape.select_sites.len(), 1);
    assert_eq!(shape.select_sites[0].selector_name, "selectAllUsers");
}

#[test]
fn extracts_reducer_with_entity_adapter() {
    let src = r#"
import { createEntityAdapter, createReducer, on } from '@ngrx/store';

export const userAdapter = createEntityAdapter<User>({
  selectId: (user) => user.id,
  sortComparer: false,
});

export const userEntityReducer = createReducer(
  initialState,
  on(loadUsersSuccess, (state, { users }) => userAdapter.setAll(users, state)),
  on(addUser, (state, { user }) => userAdapter.addOne(user, state))
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let reducer = shape.reducer.expect("should have reducer");
    assert_eq!(reducer.name, "userEntityReducer");
    assert_eq!(reducer.transitions.len(), 2);
}

// ── Effect extraction ──────────────────────────────────────────────

#[test]
fn extracts_effect_with_source_and_service() {
    let src = r#"
import { Injectable } from '@angular/core';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { of } from 'rxjs';
import { catchError, map, switchMap } from 'rxjs/operators';

@Injectable()
export class UserEffects {
  loadUsers$ = createEffect(() =>
    this.actions$.pipe(
      ofType(loadUsers),
      switchMap(() => this.userService.getUsers().pipe(
        map(users => loadUsersSuccess({ users })),
        catchError(error => of(loadUsersFailure({ error })))
      ))
    )
  );
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.effects.len(), 1);
    assert_eq!(shape.effects[0].name, "loadUsers$");
    assert_eq!(shape.effects[0].source_action.as_deref(), Some("loadUsers"));
    assert!(shape.effects[0].service_call.as_deref().unwrap_or("").contains("getUsers"));
    assert_eq!(shape.effects[0].success_action.as_deref(), Some("loadUsersSuccess"));
    assert_eq!(shape.effects[0].failure_action.as_deref(), Some("loadUsersFailure"));
}

// ── Round-9 audit: array-transform `map(` must NOT be a success action ──
//
// The `find_effect_map_action` heuristic scans for `map(...)` returning an
// action creator. But a `map(` inside the switchMap callback body can be
// an ARRAY transform (`users.map(u => u.name)`), not an RxJS operator. The
// old heuristic returned the FIRST `=> ...(` it found — capturing `u` (the
// projection variable) as a bogus success action. We now require the
// returned identifier to be a plausible action name (PascalCase or
// Success/Failure/Error suffix), filtering out lowercase projection vars.

#[test]
fn array_map_inside_effect_is_not_a_success_action() {
    let src = r#"
import { Injectable } from '@angular/core';
import { Actions, createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

@Injectable()
export class UserEffects {
  refreshNames$ = createEffect(() =>
    this.actions$.pipe(
      ofType(refreshNames),
      switchMap(() =>
        this.userService.getUsers().pipe(
          map(users => users.map(u => u.name)), // array transform — NOT an action
          map(names => refreshNamesSuccess({ names }))
        )
      )
    )
  );
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.effects.len(), 1);
    assert_eq!(
        shape.effects[0].success_action.as_deref(),
        Some("refreshNamesSuccess"),
        "array-map projection variable must NOT be captured as a success action"
    );
}

#[test]
fn extracts_no_dispatch_effect() {
    let src = r#"
import { Actions, createEffect, ofType } from '@ngrx/effects';

export class LogEffects {
  logActions$ = createEffect(() =>
    this.actions$.pipe(
      ofType(loadUsers),
      tap(action => console.log(action))
    ),
    { dispatch: false }
  );
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.effects.len(), 1);
    assert!(shape.effects[0].no_dispatch);
}

// ── Selector extraction ────────────────────────────────────────────

#[test]
fn extracts_selector_with_inputs() {
    let src = r#"
import { createSelector } from '@ngrx/store';

export const selectAllUsers = createSelector(
  selectUserState,
  (state) => state.users
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.selectors.len(), 1);
    assert_eq!(shape.selectors[0].name, "selectAllUsers");
    assert!(!shape.selectors[0].inputs.is_empty());
}

#[test]
fn extracts_multiple_selectors() {
    let src = r#"
import { createSelector } from '@ngrx/store';

export const selectAllUsers = createSelector(selectUserState, (state) => state.users);
export const selectLoading = createSelector(selectUserState, (state) => state.loading);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.selectors.len(), 2);
    assert_eq!(shape.selectors[0].name, "selectAllUsers");
    assert_eq!(shape.selectors[1].name, "selectLoading");
}

// ── Round-6 audit: depth-aware selector input splitting ────────────
//
// A projection function returning an object literal with commas (e.g.
// `state => ({ users, loading })`) must NOT fragment the input-selector
// list. The old naive `body.split(',')` would split on the commas inside
// the object literal, producing garbage inputs.

#[test]
fn selector_projection_with_object_literal_commas() {
    let src = r#"
import { createSelector } from '@ngrx/store';

export const selectUserSummary = createSelector(
  selectUserState,
  selectLoadingState,
  (userState, loadingState) => ({
    users: userState.users,
    loading: loadingState.loading,
    total: userState.users.length,
  })
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.selectors.len(), 1);
    assert_eq!(shape.selectors[0].name, "selectUserSummary");
    // The two input selectors must be captured intact — the commas inside
    // the returned object literal must NOT leak into the inputs list.
    assert_eq!(shape.selectors[0].inputs.len(), 2, "inputs: {:?}", shape.selectors[0].inputs);
    assert!(shape.selectors[0].inputs.iter().any(|i| i.contains("selectUserState")));
    assert!(shape.selectors[0].inputs.iter().any(|i| i.contains("selectLoadingState")));
}

// ── Round-7 audit: multi-line dispatch + object-literal reducer ────
//
// 1. `this.store.dispatch(\n  action()\n)` spans multiple lines — the
//    old line-based scan missed it entirely.
// 2. `createReducer({ users: [], ... }, ...)` uses an inline object
//    literal initialState — the old naive `body.split(',')` mis-parsed
//    `[]` as the state type.

#[test]
fn extracts_multi_line_dispatch_site() {
    let src = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { loadUsersSuccess } from './user.actions';

@Component({ selector: 'app-user' })
export class UserComponent {
  constructor(private store: Store<AppState>) {}

  onLoad() {
    this.store.dispatch(
      loadUsersSuccess({ users: [] })
    );
  }
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.dispatch_sites.len(), 1, "dispatch sites: {:?}", shape.dispatch_sites);
    assert_eq!(shape.dispatch_sites[0].action_name, "loadUsersSuccess");
}

#[test]
fn reducer_with_object_literal_initial_state() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';
import { loadUsersSuccess } from './user.actions';

export const initialState = {
  users: [],
  loading: false,
  error: null,
};

export const userReducer = createReducer(
  initialState,
  on(loadUsersSuccess, (state, { users }) => ({ ...state, users }))
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let reducer = shape.reducer.expect("should have a reducer");
    assert_eq!(reducer.name, "userReducer");
    // The object-literal initialState must NOT be mis-parsed as a state type.
    assert!(reducer.state_type.is_none(), "state_type should be None for object literal, got: {:?}", reducer.state_type);
    assert_eq!(reducer.transitions.len(), 1);
    assert_eq!(reducer.transitions[0].action_name, "loadUsersSuccess");
}

// ── Entity adapter extraction ──────────────────────────────────────

#[test]
fn extracts_entity_adapter() {
    let src = r#"
import { createEntityAdapter } from '@ngrx/entity';

export const userAdapter = createEntityAdapter<User>({
  selectId: (user) => user.id,
  sortComparer: false,
});
"#;
    let shape = extract_ngrx_shape(src, Fidelity::High).expect("should detect NgRx");
    let entity = shape.entity_adapter.expect("should have entity adapter");
    assert_eq!(entity.entity_type, "User");
    assert!(entity.select_id.as_deref().unwrap_or("").contains("user.id"));
}

// ── NgRx Data EntityCollectionServiceBase (data-layer gotcha) ──────
//
// Per the plan's Gotchas section: NgRx Data `EntityCollectionServiceBase<T>`
// services have no explicit createAction/createReducer — CRUD is
// auto-generated. We emit `Φentity:T (data-layer)` noting this.

#[test]
fn detects_entity_collection_service_base_data_layer() {
    let src = r#"
import { Injectable } from '@angular/core';
import { EntityCollectionServiceBase, EntityCollectionServiceElementsFactory } from '@ngrx/data';

@Injectable({ providedIn: 'root' })
export class UserService extends EntityCollectionServiceBase<User> {
  constructor(serviceElementsFactory: EntityCollectionServiceElementsFactory) {
    super('User', serviceElementsFactory);
  }
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let entity = shape.entity_adapter.as_ref().expect("should have data-layer entity");
    assert_eq!(entity.entity_type, "User");
    assert!(entity.data_layer, "NgRx Data service must be flagged data_layer");
    // No actions/reducers emitted for auto-generated CRUD.
    assert!(shape.actions.is_empty());
    assert!(shape.reducer.is_none());

    let rendered = shape.render(Fidelity::Medium);
    assert!(
        rendered.contains("Φentity:User (data-layer)"),
        "rendered: {rendered}"
    );
}

#[test]
fn entity_collection_service_base_renders_data_layer_at_all_fidelities() {
    let src = r#"
import { EntityCollectionServiceBase, EntityCollectionServiceElementsFactory } from '@ngrx/data';

export class OrderService extends EntityCollectionServiceBase<Order> {
  constructor(serviceElementsFactory: EntityCollectionServiceElementsFactory) {
    super('Order', serviceElementsFactory);
  }
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Low).expect("should detect NgRx");
    let rendered = shape.render(Fidelity::Low);
    assert!(
        rendered.contains("Φentity:Order (data-layer)"),
        "Low fidelity rendered: {rendered}"
    );

    let shape_high = extract_ngrx_shape(src, Fidelity::High).expect("should detect NgRx");
    let rendered_high = shape_high.render(Fidelity::High);
    assert!(
        rendered_high.contains("Φentity:Order (data-layer)"),
        "High fidelity rendered: {rendered_high}"
    );
}

// ── Store injection ────────────────────────────────────────────────

#[test]
fn extracts_store_injection() {
    let src = r#"
import { Store } from '@ngrx/store';

export class UserComponent {
  constructor(private store: Store<AppState>) {}
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.store_injections.len(), 1);
    assert_eq!(shape.store_injections[0], "AppState");
}

// ── Dispatch/select call sites ─────────────────────────────────────

#[test]
fn extracts_dispatch_and_select_sites() {
    let src = r#"
import { Store } from '@ngrx/store';

export class UserComponent {
  constructor(private store: Store<AppState>) {}

  ngOnInit(): void {
    this.store.dispatch(loadUsers());
    this.store.select(selectAllUsers);
  }
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.dispatch_sites.len(), 1);
    assert_eq!(shape.dispatch_sites[0].action_name, "loadUsers");
    assert_eq!(shape.select_sites.len(), 1);
    assert_eq!(shape.select_sites[0].selector_name, "selectAllUsers");
}

// ── Round-4 audit: component name capture ──────────────────────────

#[test]
fn captures_component_name_from_decorator() {
    let src = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';

@Component({ selector: 'app-user', template: '' })
export class UserComponent {
  constructor(private store: Store<AppState>) {}
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(
        shape.component_name.as_deref(),
        Some("UserComponent"),
        "component name should be captured from @Component decorator"
    );
}

// ── Round-10 audit: string-aware @Component decorator scan ─────────
//
// The old component-name extractor used a hand-rolled depth counter that
// ignored string literals. A `template: '<div>)</div>'` (with a `)` inside
// the template string) prematurely terminated the decorator scan, so the
// class name after the decorator was never found. The shared string-aware
// `find_matching_brace` primitive (Round-8 centralization) handles this.

#[test]
fn captures_component_name_when_template_contains_paren() {
    let src = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';

@Component({
  selector: 'app-user',
  template: '<div>)</div>',
})
export class UserComponent {
  constructor(private store: Store<AppState>) {}
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(
        shape.component_name.as_deref(),
        Some("UserComponent"),
        "component name should be captured despite ')' inside the template string"
    );
}

// ── Round-4 audit: inline reducer in createFeature ─────────────────

#[test]
fn extracts_inline_reducer_in_create_feature() {
    let src = r#"
import { createFeature, createReducer, on } from '@ngrx/store';

export const userFeature = createFeature({
  name: 'users',
  reducer: createReducer(
    initialState,
    on(loadUsers, (state) => ({ ...state, loading: true }))
  )
});
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let reducer = shape.reducer.expect("should have reducer from inline createFeature");
    assert_eq!(reducer.name, "users", "inline reducer should use the feature name");
    assert_eq!(reducer.transitions.len(), 1);
    assert_eq!(reducer.transitions[0].action_name, "loadUsers");
}

// ── Round-4 audit: ofType multi-action ─────────────────────────────

#[test]
fn extracts_first_action_from_multi_of_type() {
    let src = r#"
import { createAction } from '@ngrx/store';
import { createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

export const loadUsers = createAction('[User] Load Users');
export const loadUsersFailed = createAction('[User] Load Users Failed');

export const loadUsers$ = createEffect(() =>
  this.actions$.pipe(
    ofType(loadUsers, loadUsersFailed),
    switchMap(() => this.userService.getUsers()),
    map(users => loadUsersSuccess({ users }))
  )
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.effects.len(), 1);
    assert_eq!(
        shape.effects[0].source_action.as_deref(),
        Some("loadUsers"),
        "ofType multi-action should take the first action as primary source"
    );
    assert_eq!(
        shape.effects[0].source_actions,
        vec!["loadUsers".to_string(), "loadUsersFailed".to_string()],
        "all ofType actions should be retained for per-action graph edges"
    );

    // Phase 3 completion criterion: one Action → Effect edge per ofType action.
    let edges = shape.to_graph_edges();
    let action_effect: Vec<&(String, String, crate::angular_meta::graph::NgRxEdgeKind)> = edges
        .iter()
        .filter(|(_, _, k)| *k == crate::angular_meta::graph::NgRxEdgeKind::ActionEffect)
        .collect();
    assert_eq!(action_effect.len(), 2, "should emit one edge per ofType action");
    assert!(
        action_effect.iter().any(|(from, _, _)| from == "Φaction:loadUsers"),
        "should have edge from loadUsers"
    );
    assert!(
        action_effect.iter().any(|(from, _, _)| from == "Φaction:loadUsersFailed"),
        "should have edge from loadUsersFailed"
    );
}

// ── Round-5 audit: reducer identifier-boundary guard ───────────────

#[test]
fn rejects_reducer_like_identifiers() {
    // `myCreateReducer(...)` and `helper.createReducer(...)` must NOT be
    // treated as NgRx reducers - the bare `createReducer(` pattern would
    // otherwise match inside a longer identifier or a method call.
    let src = r#"
import { createAction } from '@ngrx/store';

export function myCreateReducer(state: any) { return state; }
export function helper() { return something.createReducer(initialState); }
export const loadUsers = createAction('[User] Load Users');
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(
        shape.reducer.is_none(),
        "reducer-like identifiers must not be extracted as reducers"
    );
    assert_eq!(shape.actions.len(), 1, "real createAction should still be extracted");
}

// ── No-NgRx no-op ──────────────────────────────────────────────────

#[test]
fn no_ngrx_imports_produces_none() {
    let src = r#"
import { Component } from '@angular/core';

@Component({ selector: 'app-plain' })
export class PlainComponent {}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium);
    assert!(shape.is_none(), "non-NgRx file should return None");
}

// ── Marker round-trip ──────────────────────────────────────────────

#[test]
fn expand_phi_round_trip() {
    assert_eq!(expand_phi("Φngrx"), Some("NgRx"));
    assert_eq!(expand_phi("Φaction"), Some("createAction"));
    assert_eq!(expand_phi("Φreducer"), Some("createReducer"));
    assert_eq!(expand_phi("Φeffect"), Some("createEffect"));
    assert_eq!(expand_phi("Φselector"), Some("createSelector"));
    assert_eq!(expand_phi("Φentity"), Some("createEntityAdapter"));
    assert_eq!(expand_phi("Φstore"), Some("Store"));
    assert_eq!(expand_phi("Φdispatch"), Some("dispatch"));
    assert_eq!(expand_phi("Φselect"), Some("select"));
    assert_eq!(expand_phi("Φunknown"), None);
}

#[test]
fn expand_phi_in_line_rewrites_ngrx_markers() {
    let line = "  Φaction:loadUsers '[User] Load Users'";
    let expanded = expand_phi_in_line(line);
    assert!(expanded.contains("createAction loadUsers"));
}

#[test]
fn ngrx_kind_marker_prefixes_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for kind in NgRxKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        assert!(seen.insert(prefix), "duplicate prefix: {}", prefix);
    }
}