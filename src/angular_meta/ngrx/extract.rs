// src/angular_meta/ngrx/extract.rs
//
// NgRx shape extraction: import-gated parsing of actions, reducers, effects,
// and effect/action wiring from Angular TypeScript source.
//
// Split out of `src/angular_meta/ngrx.rs` (active-file size policy): the
// module had exceeded the 615-line ceiling. This is a pure relocation -- the
// code below is byte-for-byte the previous implementation. `ngrx` re-exports
// `extract_ngrx_shape`, so the established path
// (`crate::angular_meta::ngrx::extract_ngrx_shape`) keeps resolving.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies `NgRxShape`, the declaration types,
// and the `Fidelity` re-export.

use super::extract_selectors::{
    extract_call_sites, extract_component_name, extract_entity_adapter, extract_selectors,
    extract_store_injections,
};
use super::*;

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Extract the NgRx shape from a source file.
///
/// Returns `None` when the file has no NgRx imports (zero overhead).
/// Returns `Some(NgRxShape)` with detected actions, reducers, effects,
/// selectors, entity adapters, and store usage.
pub fn extract_ngrx_shape(source: &str, _fidelity: Fidelity) -> Option<NgRxShape> {
    // Import gate: skip non-NgRx files
    if !has_ngrx_imports(source) {
        return None;
    }

    let mut shape = NgRxShape::default();

    // Extract feature name from createFeature or StoreModule.forFeature
    extract_feature_name(source, &mut shape);

    // Extract action creators
    extract_actions(source, &mut shape);

    // Extract reducer
    extract_reducer(source, &mut shape);

    // Extract effects
    extract_effects(source, &mut shape);

    // Extract selectors
    extract_selectors(source, &mut shape);

    // Extract entity adapter
    extract_entity_adapter(source, &mut shape);

    // Extract the enclosing component class name (for Component -> Store
    // graph edges). Must run before `extract_store_injections` so the
    // component name is available when wiring the edge.
    extract_component_name(source, &mut shape);

    // Extract store injections
    extract_store_injections(source, &mut shape);

    // Extract dispatch/select call sites
    extract_call_sites(source, &mut shape);

    if shape.is_empty() {
        return None;
    }

    Some(shape)
}

/// Extract the feature name from `createFeature({name: '...'})` or
/// `StoreModule.forFeature('...', ...)`.
fn extract_feature_name(source: &str, shape: &mut NgRxShape) {
    // Pattern: `createFeature({ name: 'featureName', ... })`
    if let Some(idx) = source.find("createFeature({") {
        // Round-11 audit: reject when the match is inside a comment/string
        // (e.g. a `// createFeature({ name: 'x' })` trailing comment).
        if crate::angular_meta::util::is_inside_comment_or_string(source, idx) {
            return;
        }
        let rest = &source[idx + "createFeature({".len()..];
        if let Some(name_idx) = rest.find("name:") {
            let after_name = &rest[name_idx + "name:".len()..];
            let name = after_name
                .trim_start()
                .trim_start_matches('\'')
                .trim_start_matches('"')
                .split('\'')
                .next()
                .unwrap_or("")
                .split('"')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() {
                shape.feature_name = Some(name);
                return;
            }
        }
    }

    // Pattern: `StoreModule.forFeature('featureName', ...)`
    if let Some(idx) = source.find("StoreModule.forFeature(") {
        // Round-11 audit: reject when the match is inside a comment/string.
        if crate::angular_meta::util::is_inside_comment_or_string(source, idx) {
            return;
        }
        let rest = &source[idx + "StoreModule.forFeature(".len()..];
        let name = rest
            .trim_start()
            .trim_start_matches('\'')
            .trim_start_matches('"')
            .split('\'')
            .next()
            .unwrap_or("")
            .split('"')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if !name.is_empty() {
            shape.feature_name = Some(name);
        }
    }
}

/// Extract action creators from `createAction(...)` calls.
///
/// Handles both the explicit and generic forms:
/// - `const load = createAction('[X] Event')`
/// - `const load = createAction('[X] Event', (u: any) => ({ u }))`
/// - `const load = createAction<{id: string}>('[X] Event')` (generic form)
fn extract_actions(source: &str, shape: &mut NgRxShape) {
    // Multi-line aware: find each ` = createAction(` or
    // ` = createAction<` (generic form) and collect the full call body.
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find(" = createAction") {
        let abs_idx = search_from + idx;
        // Round-11 audit: reject when the match is inside a comment/string.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + " = createAction".len();
            continue;
        }
        let before = &source[..abs_idx];
        let name = before
            .split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // The text between `createAction` and `(` — either empty or a
        // generic `<T>` param. `createAction(` → `(` directly;
        // `createAction<{id: string}>(` → `<{id: string}>`.
        let after_create = &source[abs_idx + " = createAction".len()..];
        // Skip if `createAction` is not followed by `(` (e.g. `createActionName`).
        let open_paren = match after_create.find('(') {
            Some(open) => open,
            None => {
                search_from = abs_idx + " = createAction".len() + 1;
                continue;
            }
        };
        let after_paren = abs_idx + " = createAction".len() + open_paren + 1;
        let generic_param = {
            let between = &after_create[..open_paren];
            if between.starts_with('<') {
                between.trim_end_matches('>').trim().to_string()
            } else {
                String::new()
            }
        };

        // Collect the full call body (up to matching close paren).
        // `end_offset` is the offset just past the close paren — the
        // standardized contract (Round-8 structural audit).
        let (body, end_offset) =
            crate::angular_meta::util::collect_call_body(&source[after_paren..]);

        // Extract event string (first quoted string)
        let event_string =
            crate::angular_meta::util::extract_first_quoted(&body).unwrap_or_default();

        // Extract props type. Prefer the explicit `props<T>()` form;
        // fall back to the generic `createAction<T>(` parameter.
        let props_type = if body.contains("props<") {
            let props_idx = body.find("props<").unwrap_or(0);
            let after_props = &body[props_idx + "props<".len()..];
            after_props.split('>').next().map(|s| s.trim().to_string())
        } else if !generic_param.is_empty() {
            Some(generic_param)
        } else {
            None
        };

        if !name.is_empty() {
            shape.actions.push(ActionDecl {
                name,
                event_string,
                props_type,
            });
        }
        // Advance past the whole call (including the closing paren).
        search_from = after_paren + end_offset;
    }
}

/// Extract the reducer from `createReducer(...)` calls.
///
/// Handles both forms:
/// - Standalone: `export const userReducer = createReducer(...)`
/// - Inline (NgRx 15+ `createFeature`): `createFeature({ name: 'users',
///   reducer: createReducer(...) })` — per the plan's Gotchas section,
///   the inline `: createReducer(` form must also be recognized, not
///   just the ` = createReducer(` assignment form.
fn extract_reducer(source: &str, shape: &mut NgRxShape) {
    // Multi-line aware: find each ` = createReducer(` or the inline
    // `: createReducer(` (inside createFeature) and collect the full
    // call body (which may span multiple lines).
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("createReducer(") {
        let abs_idx = search_from + idx;
        // Round-11 audit: reject when the match is inside a comment/string
        // (e.g. a `// createReducer(...)` trailing comment).
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + "createReducer(".len();
            continue;
        }
        // Reject `myCreateReducer(` / `obj.createReducer(` — the bare
        // pattern would otherwise match inside a longer identifier or a
        // method call. A genuine `createReducer(` call is preceded by
        // whitespace, `=`, `:`, `(`, `,`, `;`, or the start of the file.
        let prev = source[..abs_idx].chars().last();
        if let Some(c) = prev {
            if c.is_alphanumeric() || matches!(c, '_' | '$' | '.') {
                search_from = abs_idx + "createReducer(".len();
                continue;
            }
        }
        let before = &source[..abs_idx];
        // Determine if this is ` = createReducer(` (assignment) or
        // `: createReducer(` (inline in createFeature). The feature name
        // is the fallback name for the inline form.
        let is_inline = before.trim_end().ends_with(':');
        let name = if is_inline {
            // Inline form: use the enclosing feature name if available,
            // else `createFeature`'s name field as the reducer name.
            shape
                .feature_name
                .clone()
                .unwrap_or_else(|| "featureReducer".to_string())
        } else {
            // Strip the trailing ` = ` (the assignment operator) so the
            // last whitespace token is the actual variable name. The bare
            // `createReducer(` match includes the ` = ` in `before`.
            let before_ident = before.trim_end().trim_end_matches('=').trim();
            before_ident
                .split_whitespace()
                .last()
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        // The match pattern is `createReducer(` — `idx` points at the `C`.
        // Advance past the `createReducer(` to collect the call body.
        let after_start = abs_idx + "createReducer(".len();
        let (body, end_offset) =
            crate::angular_meta::util::collect_call_body(&source[after_start..]);

        // Extract state type from the first argument's type annotation
        // (e.g. `initialState: UserState` or `initialState`).
        // The first argument is the initial state; its type annotation
        // (if present) is the state type.
        //
        // Round-7 audit: use depth-aware splitting so an inline object
        // literal initialState (`createReducer({ users: [], ... }, ...)`)
        // is treated as a single argument — the old naive `body.split(',')`
        // would fragment it and mis-parse `[]` as the state type. We also
        // guard against mistaking object-literal property colons for a
        // type annotation.
        let state_type = crate::angular_meta::util::split_top_level(&body, ',')
            .into_iter()
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .filter(|s| !s.starts_with('{') && !s.starts_with('['))
            .and_then(|first_arg| {
                // Look for `: Type` in the first argument (e.g. `initialState: UserState`).
                if let Some(colon_idx) = first_arg.find(':') {
                    let ty = first_arg[colon_idx + 1..].trim().to_string();
                    if !ty.is_empty() { Some(ty) } else { None }
                } else {
                    None
                }
            });

        // Extract transitions from `on(action, ...)` calls
        let mut transitions = Vec::new();
        let mut on_search = 0;
        while let Some(on_idx) = body[on_search..].find("on(") {
            let abs_on = on_search + on_idx;
            // Round-11 audit: reject `on(` matches inside comments or
            // string literals within the reducer body (e.g. a
            // `// on(someAction)` comment inside the reducer, or an
            // `onPress(` string) — they are not real transitions.
            if crate::angular_meta::util::is_inside_comment_or_string(&body, abs_on) {
                on_search = abs_on + "on(".len() + 1;
                continue;
            }
            let after_on = &body[abs_on + "on(".len()..];
            let action_name = after_on
                .split(',')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            // Extract state summary (the handler body)
            let state_summary = extract_state_summary(after_on);

            if !action_name.is_empty() {
                transitions.push(ReducerTransition {
                    action_name,
                    state_summary,
                });
            }
            on_search = abs_on + "on(".len() + 1;
        }

        if !name.is_empty() {
            shape.reducer = Some(ReducerDecl {
                name,
                state_type,
                transitions,
            });
        }
        search_from = after_start + end_offset;
    }
}

/// Extract a state change summary from an `on()` handler.
///
/// The handler is the text after the action name (first comma). It is
/// typically an arrow function: `(state, { users }) => ({ ...state, users })`
/// or `(state) => ({ ...state, loading: true })`.
///
/// We must NOT capture the destructured action-props object (`{ users }`)
/// in the arrow parameters — that is the first `{` in the raw text. We
/// instead locate the `=>` and extract the object literal **after** it,
/// which is the actual returned state shape.
fn extract_state_summary(after_on: &str) -> String {
    // Find the `=>` that separates the arrow params from the body.
    let arrow_idx = match after_on.find("=>") {
        Some(i) => i,
        None => return String::new(),
    };
    let after_arrow = &after_on[arrow_idx + 2..];

    // Skip a leading `(` (e.g. `=> ({ ...state })`).
    let after_arrow = after_arrow.trim_start();
    let after_arrow = after_arrow.strip_prefix('(').unwrap_or(after_arrow);

    // Find the first `{` after the arrow — this is the returned object.
    let open_idx = match after_arrow.find('{') {
        Some(i) => i,
        None => return String::new(),
    };
    // Use the shared string-aware matching primitive for the brace depth
    // scan (Round-8 structural audit: no per-layer hand-rolled scanners).
    let rest = &after_arrow[open_idx..];
    let close_rel = match crate::angular_meta::util::find_matching_brace(rest, '{') {
        Some(close) => close,
        None => return String::new(),
    };
    let inner = &rest[1..close_rel];
    let mut summary = inner.to_string();
    // Truncate long summaries
    if summary.len() > 60 {
        summary.truncate(57);
        summary.push_str("...");
    }
    summary
}

/// Extract effects from `createEffect(() => ...)` calls.
fn extract_effects(source: &str, shape: &mut NgRxShape) {
    // Multi-line aware: find each ` = createEffect(` and collect the
    // full call body (which may span multiple lines).
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find(" = createEffect(") {
        let abs_idx = search_from + idx;
        // Round-11 audit: reject when the match is inside a comment/string.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + " = createEffect(".len();
            continue;
        }
        let before = &source[..abs_idx];
        let name = before
            .split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let after_start = abs_idx + " = createEffect(".len();
        let (after, end_offset) =
            crate::angular_meta::util::collect_call_body(&source[after_start..]);

        // Check for `{ dispatch: false }` option
        let no_dispatch = after.contains("dispatch: false");

        // Extract source action from `ofType(...)`.
        // Multiple actions are supported: `ofType(loadUsers, loadUsersFailed)`.
        // We take the first action as the primary source and emit one edge
        // per action via `to_graph_edges` below (`source_action` is the
        // primary; additional actions are stored in `source_actions`).
        let source_actions: Vec<String> = if let Some(ot_idx) = after.find("ofType(") {
            let after_ot = &after[ot_idx + "ofType(".len()..];
            // Collect the ofType(...) body with the shared string-aware
            // primitive, then depth-split on commas (Round-8 audit).
            let (of_body, _) = crate::angular_meta::util::collect_call_body(after_ot);
            crate::angular_meta::util::split_top_level(&of_body, ',')
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };
        let source_action = source_actions.first().cloned();

        // Extract service call from `switchMap`/`mergeMap`/`concatMap`/
        // `exhaustMap` (`() => service.method()` or
        // `() => { return service.method(); }` — braced arrow body).
        let service_call = ["switchMap(", "mergeMap(", "concatMap(", "exhaustMap("]
            .iter()
            .find_map(|op| {
                let sm_idx = after.find(op)?;
                let after_sm = &after[sm_idx + op.len()..];
                // Look for `=> service.method()` or `=> this.service.method()`
                let arrow_idx = after_sm.find("=>")?;
                let after_arrow = &after_sm[arrow_idx + 2..];
                let after_arrow = after_arrow.trim_start();

                // Strip a braced arrow body: `{ return svc.m(); }` → `svc.m();`.
                // We don't just naively split on `(` because a preceding
                // `{ return ` must be peeled first.
                let mut body = after_arrow.to_string();
                if body.starts_with('{') {
                    // Peels `{ return ` (single statement) — take everything
                    // after `return ` up to the closing `}`.
                    body = body.trim_start_matches('{').to_string();
                    if let Some(ret_idx) = body.find("return ") {
                        body = body[ret_idx + "return ".len()..].to_string();
                    }
                }

                let call = body
                    .split('(')
                    .next()
                    .map(|s| s.trim().trim_end_matches(';').trim().to_string())
                    .unwrap_or_default();
                if call.is_empty() { None } else { Some(call) }
            });

        // Extract success action from `map(...)` returning an action.
        // We scan for `map(` occurrences and pick the one whose argument
        // contains `=> actionName(` — this avoids false positives from
        // nested `map(` calls inside the switchMap callback body (e.g.
        // `users.map(u => u.name)`), which are array transformations,
        // not RxJS operators.
        let success_action = find_effect_map_action(&after);

        // Extract failure action from `catchError(...)`
        // The pattern is usually `catchError(error => of(loadUsersFailure({ error })))`.
        // The actual action is the first identifier that looks like an
        // action (ends in uppercase launch or contains "Failure"/"Error"),
        // nested inside the `of(...)` call.
        let failure_action = if let Some(ce_idx) = after.find("catchError(") {
            let after_ce = &after[ce_idx + "catchError(".len()..];
            // Look for `of(` which wraps the failure action.
            if let Some(of_idx) = after_ce.find("of(") {
                let after_of = &after_ce[of_idx + "of(".len()..];
                // The action is the identifier before the first `(`.
                let action = after_of
                    .split('(')
                    .next()
                    .map(|s| s.trim().trim_end_matches(')').to_string())
                    .unwrap_or_default();
                if !action.is_empty() {
                    Some(action)
                } else {
                    None
                }
            } else if let Some(arrow_idx) = after_ce.find("=> ") {
                let after_arrow = &after_ce[arrow_idx + 3..];
                let action = after_arrow
                    .split('(')
                    .next()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if !action.is_empty() {
                    Some(action)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if !name.is_empty() {
            shape.effects.push(EffectDecl {
                name,
                source_action,
                source_actions,
                service_call,
                success_action,
                failure_action,
                no_dispatch,
            });
        }
        search_from = after_start + end_offset;
    }
}

/// Find the success action from a `map(...)` operator in an effect body.
///
/// The effect body is the text inside `createEffect(() => ...)`. We scan
/// for `map(` occurrences and pick the one whose argument contains an
/// arrow function returning an action creator call (e.g.
/// `map(users => loadUsersSuccess({ users }))`).
///
/// This avoids false positives from nested `map(` calls inside the
/// `switchMap` callback body (e.g. `users.map(u => u.name)`), which are
/// array transformations, not RxJS operators.
///
/// Round-9 audit: the old heuristic returned the FIRST `=> ...(` after any
/// `map(`, which could capture an array-transform `users.map(u => u.name)`
/// as a "success action" called `u`. We now require the returned identifier
/// to be a plausible action-creator name — it must start with an uppercase
/// letter or contain `Success`/`Failure`/`Error`/`$`, and must be followed
/// by `(` (an action creator call). This filters out lowercase projection
/// variables like `u`, `users`, `result`.
fn find_effect_map_action(effect_body: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(idx) = effect_body[search_from..].find("map(") {
        let abs_idx = search_from + idx;
        let after_map = &effect_body[abs_idx + "map(".len()..];

        // Look for `=> actionName(` or `=> actionName` in the map argument.
        if let Some(arrow_idx) = after_map.find("=> ") {
            let after_arrow = &after_map[arrow_idx + 3..];
            let action = after_arrow
                .split('(')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            // Round-9 audit: require a plausible action-creator name. An
            // array transform (`users.map(u => u.name)`) yields a lowercase
            // `u` — reject it. A genuine success action is either
            // PascalCase (`loadUsersSuccess`) or contains an action suffix.
            let is_plausible_action = !action.is_empty()
                && (action.starts_with(|c: char| c.is_uppercase())
                    || action.contains("Success")
                    || action.contains("Failure")
                    || action.contains("Error")
                    || action.ends_with('$'));
            if is_plausible_action {
                return Some(action);
            }
        }

        // Advance past this `map(` occurrence.
        search_from = abs_idx + "map(".len() + 1;
    }
    None
}
