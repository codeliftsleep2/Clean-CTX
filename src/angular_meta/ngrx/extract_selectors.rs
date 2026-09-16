// src/angular_meta/ngrx/extract_selectors.rs
//
// NgRx selector, entity-adapter, component, store-injection and call-site
// extraction, plus the small body-shape helpers they share.
//
// Split out of `src/angular_meta/ngrx.rs` (active-file size policy): the
// module had exceeded the 615-line ceiling. This is a pure relocation -- the
// code below is byte-for-byte the previous implementation.
//
// The five entry points called by `ngrx::extract` are `pub(super)`: they are
// module-internal helpers, not public API (only `extract_ngrx_shape` is
// public), so the visibility stays as narrow as the split allows.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies `NgRxShape`, the declaration types,
// and the `Fidelity` re-export.

use super::*;

/// Extract selectors from `createSelector(...)` calls.
pub(super) fn extract_selectors(source: &str, shape: &mut NgRxShape) {
    // Multi-line aware: find each ` = createSelector(` and collect the
    // full call body (which may span multiple lines).
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find(" = createSelector(") {
        let abs_idx = search_from + idx;
        // Round-11 audit: reject when the match is inside a comment/string.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + " = createSelector(".len();
            continue;
        }
        let before = &source[..abs_idx];
        let name = before
            .split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let after_start = abs_idx + " = createSelector(".len();
        let (body, end_offset) =
            crate::angular_meta::util::collect_call_body(&source[after_start..]);

        // Extract input selectors (comma-separated, before the projection fn).
        // The projection fn is the last argument and contains `=>` — drop it.
        // Note: we must NOT filter on "state" — feature selectors like
        // `selectUserState` legitimately contain "state" and are valid inputs.
        //
        // Use depth-aware splitting so commas inside the projection function
        // or object literals (e.g. `state => ({ users, loading })`) do NOT
        // fragment the argument list.
        let inputs: Vec<String> = crate::angular_meta::util::split_top_level(&body, ',')
            .into_iter()
            .filter(|s| !s.contains("=>"))
            .collect();

        if !name.is_empty() {
            shape.selectors.push(SelectorDecl {
                name,
                inputs,
                return_type: None,
            });
        }
        search_from = after_start + end_offset;
    }
}

/// Extract entity adapter from `createEntityAdapter<T>({...})` calls.
pub(super) fn extract_entity_adapter(source: &str, shape: &mut NgRxShape) {
    // Multi-line aware: find each ` = createEntityAdapter<` and collect
    // the full call body (which may span multiple lines).
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find(" = createEntityAdapter<") {
        let abs_idx = search_from + idx;
        // Round-11 audit: reject when the match is inside a comment/string.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + " = createEntityAdapter<".len();
            continue;
        }
        let before = &source[..abs_idx];
        let _name = before
            .split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let after_start = abs_idx + " = createEntityAdapter<".len();
        // Extract the entity type from between `<` and `>`.
        // For nested generics like `EntityState<User>`, we need to find the
        // matching `>` by tracking bracket depth, not just `split('>')`.
        let entity_type = crate::angular_meta::util::extract_entity_type(&source[after_start..]);

        // The config object starts after the `>`.
        let config_start = after_start + entity_type.len() + 1; // skip `>`
        let (body, end_offset) =
            crate::angular_meta::util::collect_call_body(&source[config_start..]);

        // Extract selectId and sortComparer from the config object body.
        // The body starts with `({...})` — strip the outer parens.
        let rest = body.trim_start_matches('(').trim_end_matches(')');
        let select_id = if let Some(sid_idx) = rest.find("selectId:") {
            let after_sid = &rest[sid_idx + "selectId:".len()..];
            let sid = after_sid
                .split(',')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !sid.is_empty() { Some(sid) } else { None }
        } else {
            None
        };

        let sort_comparer = if let Some(sc_idx) = rest.find("sortComparer:") {
            let after_sc = &rest[sc_idx + "sortComparer:".len()..];
            let sc = after_sc
                .split('}')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !sc.is_empty() { Some(sc) } else { None }
        } else {
            None
        };

        // Default selectors from getSelectors()
        let selectors = vec![
            "selectAll".to_string(),
            "selectEntities".to_string(),
            "selectIds".to_string(),
            "selectTotal".to_string(),
        ];

        if !entity_type.is_empty() {
            shape.entity_adapter = Some(EntityAdapterDecl {
                entity_type,
                select_id,
                sort_comparer,
                selectors,
                data_layer: false,
            });
        }
        search_from = after_start + end_offset;
    }

    // NgRx Data `EntityCollectionServiceBase<T>` — auto-generated CRUD
    // services. Per the plan's Gotchas section: no explicit
    // createAction/createReducer; emit `Φentity:T (data-layer)` noting
    // auto-generated CRUD. The import gate already accepts `@ngrx/data`.
    let mut data_search = 0;
    while let Some(idx) = source[data_search..].find("EntityCollectionServiceBase<") {
        let abs_idx = data_search + idx;
        // Round-11 audit: reject when the match is inside a comment/string.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            data_search = abs_idx + "EntityCollectionServiceBase<".len();
            continue;
        }
        let after_start = abs_idx + "EntityCollectionServiceBase<".len();
        let entity_type = crate::angular_meta::util::extract_entity_type(&source[after_start..]);
        // Capture the length before moving `entity_type` into the struct.
        let consumed = entity_type.len() + 1; // skip `>`
        if !entity_type.is_empty() {
            shape.entity_adapter = Some(EntityAdapterDecl {
                entity_type,
                select_id: None,
                sort_comparer: None,
                selectors: Vec::new(),
                data_layer: true,
            });
        }
        data_search = after_start + consumed;
    }
}

/// Extract the enclosing component class name from a `@Component`
/// decorator. The class name is the identifier after `export class`
/// (or `class`) that follows the decorator.
pub(super) fn extract_component_name(source: &str, shape: &mut NgRxShape) {
    // Find `@Component(` decorator, then the class declaration after it.
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("@Component(") {
        let abs_idx = search_from + idx;
        // Round-11 audit: reject when the decorator match is inside a
        // comment/string (e.g. a `// @Component({...})` trailing comment).
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + "@Component(".len() + 1;
            continue;
        }
        // Round-10 audit: use the shared string-aware `find_matching_brace`
        // primitive instead of a hand-rolled depth counter. The old scan
        // ignored string literals — an `@Component({ template: '<div>)</div>' })`
        // with a `)` inside the template string would prematurely terminate
        // the scan, breaking the class-name lookup. The shared primitive
        // (Round-8 centralization) handles strings/templates correctly.
        let after_component = &source[abs_idx + "@Component".len()..];
        // `after_component` starts with `(` (the decorator open paren).
        let close_rel = match crate::angular_meta::util::find_matching_brace(after_component, '(') {
            Some(close) => close,
            None => {
                search_from = abs_idx + "@Component(".len() + 1;
                continue;
            }
        };
        let after_close = &after_component[close_rel + 1..];
        // Look for `export class Name` or `class Name` after the decorator.
        let class_idx = after_close.find("class ").map(|i| i + "class ".len());
        if let Some(class_start) = class_idx {
            let after_class = &after_close[class_start..];
            let name = after_class
                .split_whitespace()
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !name.is_empty() {
                shape.component_name = Some(name);
                return;
            }
        }
        search_from = abs_idx + "@Component(".len() + 1;
    }
}

/// Extract store injections from constructor parameters.
pub(super) fn extract_store_injections(source: &str, shape: &mut NgRxShape) {
    // Track absolute byte offsets so matches inside trailing comments or
    // string literals are rejected (Round-11 audit).
    let mut line_start = 0usize;
    for line in source.split('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            line_start += line.len() + 1;
            continue;
        }

        let leading = line.len() - line.trim_start().len();
        let trimmed_abs = line_start + leading;

        // Pattern: `private store: Store<AppState>` or `store: Store<AppState>`
        if let Some(idx) = trimmed.find(": Store<") {
            // Round-11 audit: reject when the match is inside a comment/string.
            if crate::angular_meta::util::is_inside_comment_or_string(source, trimmed_abs + idx) {
                line_start += line.len() + 1;
                continue;
            }
            let after = &trimmed[idx + ": Store<".len()..];
            let state_type = after
                .split('>')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if !state_type.is_empty() {
                shape.store_injections.push(state_type);
            }
        }
        line_start += line.len() + 1;
    }
}

/// Extract dispatch and select call sites.
///
/// Handles (multi-line aware — Round-7 audit):
/// - `this.store.dispatch(action)` and bare `store.dispatch(action)`
/// - `this.store.select(selector)` and bare `store.select(selector)`
/// - `store.pipe(select(selector))` (the modern RxJS-pipe selector form)
///
/// The old line-based scan missed multi-line calls
/// (`this.store.dispatch(\n  loadUsersSuccess({ users })\n)`) and used a
/// naive `split(')')` that truncated nested selectors
/// (`store.select(selectUser({ id }))`). We now scan the whole source and
/// use `collect_call_body` so string-aware paren matching handles nested
/// args and multi-line bodies.
pub(super) fn extract_call_sites(source: &str, shape: &mut NgRxShape) {
    for (pattern, kind) in [
        ("this.store.dispatch(", SiteKind::Dispatch),
        ("store.dispatch(", SiteKind::Dispatch),
        ("this.store.select(", SiteKind::Select),
        ("store.select(", SiteKind::Select),
        ("this.store.pipe(select(", SiteKind::PipeSelect),
        ("store.pipe(select(", SiteKind::PipeSelect),
    ] {
        let mut search_from = 0;
        while let Some(idx) = source[search_from..].find(pattern) {
            let abs_idx = search_from + idx;

            // Skip bare matches that are actually part of a `this.`-prefixed
            // call — `store.dispatch(` matches inside `this.store.dispatch(`
            // at a later offset (Round-7 audit: prevents double-counting).
            if !pattern.starts_with("this.") && source[..abs_idx].ends_with('.') {
                search_from = abs_idx + pattern.len();
                continue;
            }

            // Skip matches inside comment lines (`// ...` or `* ...`).
            let line_start = source[..abs_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_trim = source[line_start..abs_idx].trim_start();
            if line_trim.starts_with("//") || line_trim.starts_with('*') {
                search_from = abs_idx + pattern.len();
                continue;
            }

            // Round-11 audit: reject matches inside trailing comments, block
            // comments, or string literals.
            if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
                search_from = abs_idx + pattern.len();
                continue;
            }

            // Collect the full call body (up to matching close paren).
            // `collect_call_body` is string-aware and multi-line capable.
            let after_start = abs_idx + pattern.len();
            let (body, end_offset) =
                crate::angular_meta::util::collect_call_body(&source[after_start..]);

            // The semantic site name depends on the call-site flavour:
            //   - Dispatch action creator (`someAction()`) -> `someAction`
            //   - Dispatch object literal (`{ type: TOGGLE_PANEL }`) ->
            //     `TOGGLE_PANEL`
            //   - Select / PipeSelect identifier or quoted literal
            //     (`selectAllUsers` / `'panelState'`) -> semantic value
            let site_name = match kind {
                SiteKind::Dispatch => dispatch_action_name(&body),
                SiteKind::Select | SiteKind::PipeSelect => select_selector_name(&body),
            };

            if let Some(name) = site_name {
                match kind {
                    SiteKind::Dispatch => {
                        shape
                            .dispatch_sites
                            .push(DispatchSite { action_name: name });
                    }
                    SiteKind::Select | SiteKind::PipeSelect => {
                        shape.select_sites.push(SelectSite {
                            selector_name: name,
                        });
                    }
                }
            }

            search_from = after_start + end_offset;
        }
    }
}

/// First token of a call body slice under the established site-name split
/// semantics: the segment before the first `(`, `,`, or whitespace.
///
/// Examples:
///   `someAction()`       -> `someAction`
///   `selectAllUsers`     -> `selectAllUsers`
///   `selectUser({ id })` -> `selectUser`
fn first_body_token(body: &str) -> Option<String> {
    let token = body
        .trim_start()
        .split(['(', ',', ' ', '\t', '\n', '\r'])
        .next()
        .map(|s| s.trim())
        .unwrap_or_default();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// Derive the semantic selector name for a `select(...)` / `pipe(select(...))`
/// call body (the raw slice between the outer parens).
///
/// Supported forms (the established contract, plus quoted literals):
///   `selectAllUsers`   -> `selectAllUsers` (identifier)
///   `'panelState'`     -> `panelState`    (quoted string literal)
///   `"panelState"`     -> `panelState`
///
/// A whole-token quoted literal is surfaced as its unquoted content so the
/// semantic value is the literal's value rather than a raw source slice.
fn select_selector_name(body: &str) -> Option<String> {
    let token = first_body_token(body)?;
    if token.len() >= 2
        && ((token.starts_with('\'') && token.ends_with('\''))
            || (token.starts_with('"') && token.ends_with('"')))
    {
        return Some(token[1..token.len() - 1].to_string());
    }
    Some(token)
}

/// Derive the action name for a `dispatch(...)` call body (raw slice between
/// the outer parens).
///
/// Supported forms:
///   `someAction()`           -> `someAction`    (action creator)
///   `{ type: TOGGLE_PANEL }` -> `TOGGLE_PANEL`  (object literal)
///
/// The object-literal form requires a bare-identifier-valued `type`
/// property; anything else yields `None` so unsupported object literals are
/// skipped rather than emitting punctuation or fabricated names.
fn dispatch_action_name(body: &str) -> Option<String> {
    let trimmed = body.trim_start();
    if trimmed.starts_with('{') {
        return object_literal_action_name(trimmed);
    }
    first_body_token(body)
}

/// Extract the action name from the established NgRx object-literal action
/// form `{ type: TOGGLE_PANEL }`.
///
/// Only the `type` property with a bare identifier value is accepted. No
/// general object/expression parsing is performed — anything outside this
/// shape returns `None`.
fn object_literal_action_name(trimmed_body: &str) -> Option<String> {
    let close = crate::meta_util::find_matching_brace(trimmed_body, '{')?;
    if close <= 1 {
        return None;
    }
    let inner = &trimmed_body[1..close];
    for part in crate::meta_util::split_top_level(inner, ',') {
        let (key, value) = match part.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        let key = key.trim_matches(|c| c == '"' || c == '\'');
        if key != "type" {
            continue;
        }
        // The established form requires a bare identifier value.
        return is_identifier(value).then_some(value.to_string());
    }
    None
}

/// A valid bare identifier (JavaScript-style, Unicode-aware). Used to
/// validate object-literal `type` values so punctuation, member expressions,
/// and other non-action shapes never become action identities.
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// The flavour of store call site extracted by [`extract_call_sites`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SiteKind {
    Dispatch,
    Select,
    PipeSelect,
}
