// src/tests/angular_meta/ngrx.rs
//
// Unit tests for the NgRx Meta-Layer (Phase 2 of the Angular
// Ecosystem Deepening).

use crate::angular_meta::ngrx::{
    NgRxKind, expand_phi, expand_phi_in_line, extract_ngrx_shape, has_ngrx_imports,
};
use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;

// ── Import gate ────────────────────────────────────────────────────

#[test]
fn detects_ngrx_store_import() {
    let src = "import { Store } from '@ngrx/store';";
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
fn detects_ngrx_data_import() {
    let src = "import { EntityCollectionServiceBase } from '@ngrx/data';";
    assert!(has_ngrx_imports(src));
}

#[test]
fn rejects_non_ngrx_imports() {
    let src = "import { Component } from '@angular/core';";
    assert!(!has_ngrx_imports(src));
}

// ── Action extraction ──────────────────────────────────────────────

#[test]
fn extracts_action_creator() {
    let src = r#"
import { createAction, props } from '@ngrx/store';

export const loadUsers = createAction(
  '[Users] Load Users',
  props<{ page: number }>()
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.actions.len(), 1);
    assert_eq!(shape.actions[0].name, "loadUsers");
    assert_eq!(shape.actions[0].event_string, "[Users] Load Users");
    assert!(
        shape.actions[0]
            .props_type
            .as_deref()
            .unwrap_or("")
            .contains("page")
    );
}

// ── Reducer extraction ─────────────────────────────────────────────

#[test]
fn extracts_reducer_with_transitions() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';
import { loadUsers, loadUsersSuccess, loadUsersFailure } from './user.actions';

export interface UserState {
  users: User[];
  loading: boolean;
}

export const initialState: UserState = {
  users: [],
  loading: false,
};

export const userReducer = createReducer(
  initialState: UserState,
  on(loadUsers, (state) => ({ ...state, loading: true })),
  on(loadUsersSuccess, (state) => ({ ...state, loading: false, users: [] })),
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(shape.reducer.is_some());
    let reducer = shape.reducer.as_ref().unwrap();
    assert_eq!(reducer.name, "userReducer");
    assert_eq!(reducer.state_type.as_deref(), Some("UserState"));
    // Two transitions: one for loadUsers, one for loadUsersSuccess
    assert_eq!(
        reducer.transitions.len(),
        2,
        "transitions: {:?}",
        reducer.transitions
    );
    assert_eq!(reducer.transitions[0].action_name, "loadUsers");
    assert!(
        reducer.transitions[0]
            .state_summary
            .contains("loading: true")
    );
}

#[test]
fn reducer_summary_skips_destructured_props() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';
import { selectUser } from './user.actions';

export const userReducer = createReducer(
  initialState,
  on(selectUser, (state, { id }) => ({ ...state, selectedId: id })),
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    let reducer = shape.reducer.as_ref().unwrap();
    // Should have one transition, with the summary being the returned object
    // (after the `=>`), not the destructured parameter.
    assert_eq!(reducer.transitions.len(), 1);
    assert!(
        !reducer.transitions[0].state_summary.contains("{ id }"),
        "state summary should NOT include destructured props: {}",
        reducer.transitions[0].state_summary
    );
    assert!(
        reducer.transitions[0].state_summary.contains("selectedId"),
        "state summary should contain the returned field: {}",
        reducer.transitions[0].state_summary
    );
}

// ── Round-7+8 audit: inline object-literal initialState ────────────
//
// An object literal initialState (e.g. `createReducer({ users: [], ...}, ...)`)
// must NOT be fragmented by naive comma-splitting. The depth-aware
// `split_top_level` + object-literal guard prevents this.

#[test]
fn reducer_with_object_literal_initial_state() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';

export const userReducer = createReducer(
  { users: [], loading: false },
  on(loadUsers, (state) => ({ ...state, loading: true })),
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(shape.reducer.is_some(), "reducer should be detected");
    // Must not extract state type from the object literal (no `: Type`).
    assert!(shape.reducer.as_ref().unwrap().state_type.is_none());
}

#[test]
fn extracts_reducer_with_entity_adapter() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';
import { createEntityAdapter } from '@ngrx/entity';

export interface User {
  id: number;
  name: string;
}

export const userAdapter = createEntityAdapter<User>();

export const userReducer = createReducer(
  userAdapter.getInitialState(),
  on(loadUsersSuccess, (state, { users }) => userAdapter.setAll(users, state)),
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(shape.reducer.is_some());
    assert!(shape.entity_adapter.is_some());
}

// ── Effect extraction ──────────────────────────────────────────────

#[test]
fn extracts_effect_with_of_type_and_service_call() {
    let src = r#"
import { createEffect, Actions, ofType } from '@ngrx/effects';
import { loadUsers, loadUsersSuccess, loadUsersFailure } from './user.actions';

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
    let effect = &shape.effects[0];
    assert_eq!(effect.name, "loadUsers$");
    assert_eq!(effect.source_action.as_deref(), Some("loadUsers"));
    assert!(effect.service_call.is_some());
    assert!(!effect.no_dispatch);
}

// ── Round-6 audit: action-creator detection via barrel imports ─────
//
// Some projects re-export NgRx creators from a local barrel file
// (e.g. `index.ts`). The `has_ngrx_imports` gate falls back to
// scanning for `createAction(` / etc. so these files are still enriched.

#[test]
fn barrel_import_fallback_detects_ngrx() {
    let src = r#"
import { createAction, props } from './store';

export const loadUsers = createAction(
  '[Users] Load Users',
  props<{ page: number }>()
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx via barrel");
    assert_eq!(shape.actions.len(), 1);
    assert_eq!(shape.actions[0].name, "loadUsers");
}

// ── Round-9 audit: array-map inside effect body must NOT be a success action ──
//
// The `find_effect_map_action` heuristics looked for the first `map(` whose
// argument contained `=> actionName(`. An array `.map(u => u.name)` inside
// the `switchMap` callback body would match as "success action called `u`".
// We now require plausible action-creator names (uppercase-start or
// Success/Failure suffix).

#[test]
fn array_map_inside_effect_is_not_a_success_action() {
    let src = r#"
import { createEffect, Actions, ofType } from '@ngrx/effects';
import { loadUsers, loadUsersSuccess } from './user.actions';

export class UserEffects {
  loadUsers$ = createEffect(() =>
    this.actions$.pipe(
      ofType(loadUsers),
      switchMap(({ ids }) => this.userService.getUsers().pipe(
        map(users => readUsersSuccess({
          users: users.map(u => u.name).filter(n => n.length > 0)
        }))
      ))
    )
  );
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.effects.len(), 1);
    let effect = &shape.effects[0];
    // The array `.map(u => u.name)` inside the switchMap callback must NOT
    // be treated as a success action returning `u` — the genuine success
    // action is `readUsersSuccess` (PascalCase), not `u`.
    assert_eq!(
        effect.source_action.as_deref(),
        Some("loadUsers"),
        "the ofType source action must be captured, got: {:?}",
        effect.source_action
    );
    assert_eq!(
        effect.success_action.as_deref(),
        Some("readUsersSuccess"),
        "the success action should be readUsersSuccess, got: {:?}",
        effect.success_action
    );
}

// ── Round-10 audit: component name with template containing `)` ────
//
// The old `@Component` decorator scanner used a hand-rolled depth counter
// that ignored string literals — template HTML containing a `)` (e.g.
// `<div>)</div>`) would prematurely terminate the decorator body scan,
// missing the class declaration and emitting `null` for the component
// name. The shared string-aware `find_matching_brace` fixes this.

#[test]
fn captures_component_name_when_template_contains_paren() {
    let src = r#"
import { Component } from '@angular/core';
import { Store } from '@ngrx/store';
import { Observable } from 'rxjs';

@Component({
  selector: 'app-user',
  template: '<div>)</div>',
})
export class UserComponent {
  users$: Observable<User[]> = this.store.select(selectUsers);
  constructor(private store: Store<AppState>) {}
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(
        shape.component_name.as_deref(),
        Some("UserComponent"),
        "component name must not be truncated by `)` in template string, got: {:?}",
        shape.component_name
    );
    assert!(
        !shape.store_injections.is_empty(),
        "store injection should be detected"
    );
}

// ── Round-11 audit: comment/string patterns must NOT create phantom artifacts ──
//
// Global scans for local patterns like `createAction(`, `createReducer(`,
// `createEffect(`, etc. would match inside trailing comments or string
// literals. The shared `is_inside_comment_or_string` helper rejects these.

#[test]
fn ignores_action_in_trailing_comment() {
    let src = r#"
import { createAction, props } from '@ngrx/store';

export const loadUsers = createAction('[Users] Load');  // phantomAction = createAction('')
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(
        shape.actions.len(),
        1,
        "trailing-comment action must not be extracted, got: {:?}",
        shape.actions
    );
    assert_eq!(shape.actions[0].name, "loadUsers");
}

#[test]
fn ignores_reducer_on_in_comment_inside_body() {
    let src = r#"
import { createReducer, on } from '@ngrx/store';

export const userReducer = createReducer(
  initialState,
  // on(phantomAction, (s) => ({ ...s })),
  on(loadUsers, (s) => ({ ...s, loading: true })),
);
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(shape.reducer.is_some());
    assert_eq!(
        shape.reducer.as_ref().unwrap().transitions.len(),
        1,
        "comment-line on() transition must not be counted, got: {:?}",
        shape.reducer.as_ref().unwrap().transitions
    );
    assert_eq!(
        shape.reducer.as_ref().unwrap().transitions[0].action_name,
        "loadUsers"
    );
}

#[test]
fn ignores_effect_in_trailing_comment() {
    let src = r#"
import { createEffect, Actions, ofType } from '@ngrx/effects';

export class UserEffects {
  loadUsers$ = createEffect(() =>
    this.actions$.pipe(ofType(loadUsers))
  );  // phantomEffect$ = createEffect(() => of())
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(
        shape.effects.len(),
        1,
        "trailing-comment effect must not be extracted, got: {:?}",
        shape.effects
    );
    assert_eq!(shape.effects[0].name, "loadUsers$");
}

// ── Entity adapter extraction ──────────────────────────────────────

#[test]
fn extracts_entity_adapter() {
    let src = r#"
import { createEntityAdapter } from '@ngrx/entity';

export interface User {
  id: number;
  name: string;
}

export const userAdapter = createEntityAdapter<User>({
  selectId: (user) => user.id,
  sortComparer: (a, b) => a.name.localeCompare(b.name),
});
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(shape.entity_adapter.is_some());
    let adapter = shape.entity_adapter.unwrap();
    assert_eq!(adapter.entity_type, "User");
}

// ── NgRx Data EntityCollectionServiceBase ──────────────────────────

#[test]
fn detects_entity_collection_service_base_data_layer() {
    let src = r#"
import { EntityCollectionServiceBase } from '@ngrx/data';

@Injectable({ providedIn: 'root' })
export class UserDataService extends EntityCollectionServiceBase<User> {
  constructor(serviceElementsFactory: HttpEntityCollectionServiceElementsFactory) {
    super('User', serviceElementsFactory);
  }
}
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert!(
        shape.entity_adapter.is_some(),
        "entity_adapter should be Some"
    );
    let adapter = shape.entity_adapter.as_ref().unwrap();
    assert_eq!(adapter.entity_type, "User");
    assert!(
        adapter.data_layer,
        "NgRx Data service must be marked as data-layer"
    );
}

// ── Inline reducer in createFeature ────────────────────────────────

#[test]
fn extracts_inline_reducer_in_create_feature() {
    let src = r#"
import { createFeature, createReducer, on } from '@ngrx/store';

export const userFeature = createFeature({
  name: 'users',
  reducer: createReducer(
    initialState,
    on(loadUsers, (state) => ({ ...state, loading: true })),
  ),
});
"#;
    let shape = extract_ngrx_shape(src, Fidelity::Medium).expect("should detect NgRx");
    assert_eq!(shape.feature_name.as_deref(), Some("users"));
    assert!(shape.reducer.is_some());
    // The inline reducer uses the enclosing feature name.
    assert_eq!(shape.reducer.as_ref().unwrap().name, "users");
}

// ── Non-NgRx no-op ─────────────────────────────────────────────────

#[test]
fn no_ngrx_import_produces_none() {
    let src = r#"
export class PlainService {
  private data: string[] = [];
}
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
    let line = "  Φaction:loadUsers '[Users] Load'";
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
