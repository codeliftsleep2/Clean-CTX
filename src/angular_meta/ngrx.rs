// src/angular_meta/ngrx.rs
//
// NgRx Meta-Layer — Phase 2 of the Angular Ecosystem Deepening.
//
// Detects and compresses NgRx store artifacts — actions, reducers,
// effects, selectors, entity adapters — in Angular TypeScript files.
// Outputs a `// --- Φ NgRx Meta ---` block.
//
// # Purely additive
//
// The NgRx meta-layer never modifies existing TS compression output.
// It only appends a `Φ NgRx Meta` block below the existing compacted
// class. Non-NgRx files pay zero overhead (import-gate detection).
//
// # Marker architecture
//
// This module defines its own `NgRxKind` sub-enum (not added to the
// existing `PhiLineKind` in `markers.rs`) to avoid a 41-variant
// monolith. The `expand_phi_in_line` function is chained into the
// existing Angular expansion via `markers.rs`.

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;

// ---------------------------------------------------------------------------
// NgRxKind — single source of truth for NgRx marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind for NgRx constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NgRxKind {
    NgRx,
    Action,
    Reducer,
    Effect,
    Selector,
    Entity,
    Store,
    Dispatch,
    Select,
}

impl PhiMarker for NgRxKind {
    /// The `Φ` marker prefix for this kind (e.g. `"Φaction:"`).
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::NgRx => "Φngrx:",
            Self::Action => "Φaction:",
            Self::Reducer => "Φreducer:",
            Self::Effect => "Φeffect:",
            Self::Selector => "Φselector:",
            Self::Entity => "Φentity:",
            Self::Store => "Φstore:",
            Self::Dispatch => "Φdispatch:",
            Self::Select => "Φselect:",
        }
    }

    /// The human-readable expansion (e.g. `"createAction"`).
    /// Does NOT include the trailing space.
    fn expansion(self) -> &'static str {
        match self {
            Self::NgRx => "NgRx",
            Self::Action => "createAction",
            Self::Reducer => "createReducer",
            Self::Effect => "createEffect",
            Self::Selector => "createSelector",
            Self::Entity => "createEntityAdapter",
            Self::Store => "Store",
            Self::Dispatch => "dispatch",
            Self::Select => "select",
        }
    }

    /// All variants in a canonical order (longer prefixes first to
    /// prevent partial-match issues in string replacement).
    fn all_in_expand_order() -> &'static [NgRxKind] {
        &[
            Self::NgRx,       // Φngrx:     (6 chars)
            Self::Action,     // Φaction:   (8 chars)
            Self::Reducer,    // Φreducer:  (9 chars)
            Self::Effect,     // Φeffect:   (8 chars)
            Self::Selector,   // Φselector: (10 chars)
            Self::Entity,     // Φentity:   (8 chars)
            Self::Store,      // Φstore:    (7 chars)
            Self::Dispatch,   // Φdispatch: (10 chars)
            Self::Select,     // Φselect:   (8 chars)
        ]
    }

    /// Look up an [`NgRxKind`] by its marker token string (without
    /// the trailing colon). Returns `None` for unknown tokens.
    fn from_token(token: &str) -> Option<NgRxKind> {
        match token {
            "Φngrx" => Some(Self::NgRx),
            "Φaction" => Some(Self::Action),
            "Φreducer" => Some(Self::Reducer),
            "Φeffect" => Some(Self::Effect),
            "Φselector" => Some(Self::Selector),
            "Φentity" => Some(Self::Entity),
            "Φstore" => Some(Self::Store),
            "Φdispatch" => Some(Self::Dispatch),
            "Φselect" => Some(Self::Select),
            _ => None,
        }
    }

    /// Returns the token string (without trailing `:`) for a given kind.
    fn token(self) -> &'static str {
        match self {
            Self::NgRx => "Φngrx",
            Self::Action => "Φaction",
            Self::Reducer => "Φreducer",
            Self::Effect => "Φeffect",
            Self::Selector => "Φselector",
            Self::Entity => "Φentity",
            Self::Store => "Φstore",
            Self::Dispatch => "Φdispatch",
            Self::Select => "Φselect",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single NgRx action creator.
#[derive(Debug, Clone)]
pub struct ActionDecl {
    pub name: String,
    pub event_string: String,
    pub props_type: Option<String>,
}

/// A single reducer transition (on(action) → state change).
#[derive(Debug, Clone)]
pub struct ReducerTransition {
    pub action_name: String,
    pub state_summary: String,
}

/// A reducer declaration.
#[derive(Debug, Clone)]
pub struct ReducerDecl {
    pub name: String,
    pub state_type: Option<String>,
    pub transitions: Vec<ReducerTransition>,
}

/// A single NgRx effect.
#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub name: String,
    /// The primary source action (first in `ofType(...)`).
    pub source_action: Option<String>,
    /// All source actions from `ofType(a, b, c)` — one graph edge is
    /// emitted per action so multi-action effects wire every trigger.
    pub source_actions: Vec<String>,
    pub service_call: Option<String>,
    pub success_action: Option<String>,
    pub failure_action: Option<String>,
    pub no_dispatch: bool,
}

/// A single NgRx selector.
#[derive(Debug, Clone)]
pub struct SelectorDecl {
    pub name: String,
    pub inputs: Vec<String>,
    pub return_type: Option<String>,
}

/// An entity adapter declaration.
#[derive(Debug, Clone)]
pub struct EntityAdapterDecl {
    pub entity_type: String,
    pub select_id: Option<String>,
    pub sort_comparer: Option<String>,
    pub selectors: Vec<String>,
    /// True when this is an NgRx Data `EntityCollectionServiceBase<T>`
    /// service (auto-generated CRUD — no explicit createAction/
    /// createReducer) rather than a manual `createEntityAdapter<T>(...)`.
    pub data_layer: bool,
}

/// A store dispatch call site.
#[derive(Debug, Clone)]
pub struct DispatchSite {
    pub action_name: String,
}

/// A store select call site.
#[derive(Debug, Clone)]
pub struct SelectSite {
    pub selector_name: String,
}

/// The complete NgRx shape extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct NgRxShape {
    pub feature_name: Option<String>,
    /// The enclosing component class name (e.g. `UserComponent`) when
    /// the file has a `@Component` decorator. Used to wire the
    /// `Component -> Store` graph edge to the actual component, not
    /// just the `Φstore:` marker.
    pub component_name: Option<String>,
    pub actions: Vec<ActionDecl>,
    pub reducer: Option<ReducerDecl>,
    pub effects: Vec<EffectDecl>,
    pub selectors: Vec<SelectorDecl>,
    pub entity_adapter: Option<EntityAdapterDecl>,
    pub store_injections: Vec<String>,
    pub dispatch_sites: Vec<DispatchSite>,
    pub select_sites: Vec<SelectSite>,
}

impl NgRxShape {
    /// Returns `true` if there are no NgRx artifacts to emit.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
            && self.reducer.is_none()
            && self.effects.is_empty()
            && self.selectors.is_empty()
            && self.entity_adapter.is_none()
            && self.store_injections.is_empty()
            && self.dispatch_sites.is_empty()
            && self.select_sites.is_empty()
    }

    /// Convert this NgRx shape into cross-layer graph edges (Phase 5
    /// of the Angular Ecosystem Deepening).
    ///
    /// Returns `(from, to, kind)` triples where `kind` is the
    /// [`NgRxEdgeKind`](crate::angular_meta::graph::NgRxEdgeKind)
    /// marker prefix. The caller (workspace graph pass) resolves
    /// service names to file aliases and feeds these into the
    /// `AngularGraphBuilder`.
    pub fn to_graph_edges(&self) -> Vec<(String, String, crate::angular_meta::graph::NgRxEdgeKind)> {
        use crate::angular_meta::graph::NgRxEdgeKind;
        let mut edges = Vec::new();

        // Action → Reducer (via `on(action)` handlers).
        if let Some(reducer) = &self.reducer {
            for transition in &reducer.transitions {
                edges.push((
                    format!("Φaction:{}", transition.action_name),
                    format!("Φreducer:{}", reducer.name),
                    NgRxEdgeKind::ActionReducer,
                ));
            }
        }

        // Action → Effect (via `ofType(action)`). One edge per source
        // action so `ofType(loadUsers, loadUsersFailed)` wires both
        // triggers to the effect (Phase 3 completion criterion).
        for effect in &self.effects {
            let sources: Vec<&str> = if !effect.source_actions.is_empty() {
                effect.source_actions.iter().map(|s| s.as_str()).collect()
            } else {
                effect.source_action.iter().map(|s| s.as_str()).collect()
            };
            for source in sources {
                edges.push((
                    format!("Φaction:{}", source),
                    format!("Φeffect:{}", effect.name),
                    NgRxEdgeKind::ActionEffect,
                ));
            }
            // Effect → Action (via `map(successAction)`).
            if let Some(success) = &effect.success_action {
                edges.push((
                    format!("Φeffect:{}", effect.name),
                    format!("Φaction:{}", success),
                    NgRxEdgeKind::EffectAction,
                ));
            }
            // Effect → Service (via `switchMap(() => svc.method())`).
            if let Some(service) = &effect.service_call {
                edges.push((
                    format!("Φeffect:{}", effect.name),
                    service.clone(),
                    NgRxEdgeKind::EffectService,
                ));
            }
        }

        // Component → Store (via `Store<T>` DI). The `from` node is the
        // actual component class (e.g. `Φcmp:UserComponent`), which the
        // workspace graph pass resolves to `UserComponent@αN` using the
        // file alias. This wires the component into the graph, not just
        // the `Φstore:` marker.
        if let Some(ref component) = self.component_name {
            // One ComponentStore edge per Store<T> DI injection. The
            // `store` type is not part of the edge (the `to` node is the
            // feature), so we iterate for count only.
            for _store in &self.store_injections {
                edges.push((
                    format!("Φcmp:{}", component),
                    format!("Φngrx:{}", self.feature_name.clone().unwrap_or_else(|| "Feature".to_string())),
                    NgRxEdgeKind::ComponentStore,
                ));
            }
        }

        // Component → Selector (via `store.select(selector)`). The `from`
        // node is the actual component class (when known), matching the
        // `ComponentStore` edge semantics. Falls back to the `Φselect:`
        // marker when no component is detected (e.g. services).
        if let Some(ref component) = self.component_name {
            for site in &self.select_sites {
                edges.push((
                    format!("Φcmp:{}", component),
                    format!("Φselector:{}", site.selector_name),
                    NgRxEdgeKind::ComponentSelector,
                ));
            }
        } else {
            for site in &self.select_sites {
                edges.push((
                    format!("Φselect:{}", site.selector_name),
                    format!("Φselector:{}", site.selector_name),
                    NgRxEdgeKind::ComponentSelector,
                ));
            }
        }

        edges
    }

    /// Render the full `Φ NgRx Meta` block at the given fidelity.
    pub fn render(&self, fidelity: Fidelity) -> String {
        self.render_with_config(fidelity, None)
    }

    /// Render the full `Φ NgRx Meta` block at the given fidelity,
    /// honoring the NgRx sub-layer config flags (when provided):
    /// - `include_dispatch_sites`: emit `Φdispatch:` call sites
    /// - `include_select_sites`: emit `Φselect:` call sites
    /// - `entity_selectors`: include entity adapter default selectors
    ///
    /// When `config` is `None`, all flags default to `true` (the same
    /// behaviour as [`render`](Self::render)).
    pub fn render_with_config(
        &self,
        fidelity: Fidelity,
        config: Option<&crate::config::NgRxConfig>,
    ) -> String {
        if self.is_empty() {
            return String::new();
        }

        let include_dispatch = config.map(|c| c.include_dispatch_sites).unwrap_or(true);
        let include_select = config.map(|c| c.include_select_sites).unwrap_or(true);
        let include_entity_selectors = config.map(|c| c.entity_selectors).unwrap_or(true);

        let mut s = String::new();
        s.push_str("// --- Φ NgRx Meta ---\n");

        // Feature name (all fidelities)
        if let Some(ref feature) = self.feature_name {
            s.push_str(&format!("  Φngrx:{}\n", feature));
        }

        // Actions (all fidelities)
        for action in &self.actions {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φaction:{}\n", action.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    if let Some(ref props) = action.props_type {
                        s.push_str(&format!("  Φaction:{} '{}' props<{}>\n",
                            action.name, action.event_string, props));
                    } else {
                        s.push_str(&format!("  Φaction:{} '{}'\n",
                            action.name, action.event_string));
                    }
                }
            }
        }

        // Reducer (Medium+)
        if let Some(ref reducer) = self.reducer {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φreducer:{}\n", reducer.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    if let Some(ref st) = reducer.state_type {
                        s.push_str(&format!("  Φreducer:{} → {}\n", reducer.name, st));
                    } else {
                        s.push_str(&format!("  Φreducer:{}\n", reducer.name));
                    }
                    for transition in &reducer.transitions {
                        s.push_str(&format!("    on({}) → {}\n",
                            transition.action_name, transition.state_summary));
                    }
                }
            }
        }

        // Effects (Medium+)
        for effect in &self.effects {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φeffect:{}\n", effect.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    let mut line = format!("  Φeffect:{}", effect.name);
                    if let Some(ref src) = effect.source_action {
                        line.push_str(&format!(" ← {}", src));
                    }
                    if let Some(ref svc) = effect.service_call {
                        line.push_str(&format!(" → {}", svc));
                    }
                    if effect.no_dispatch {
                        line.push_str(" (no-dispatch)");
                    }
                    s.push_str(&line);
                    s.push('\n');
                    if fidelity == Fidelity::High {
                        if let Some(ref success) = effect.success_action {
                            s.push_str(&format!("    → {}\n", success));
                        }
                        if let Some(ref failure) = effect.failure_action {
                            s.push_str(&format!("    → {}\n", failure));
                        }
                    }
                }
            }
        }

        // Selectors (Medium+)
        for selector in &self.selectors {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φselector:{}\n", selector.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    if selector.inputs.is_empty() {
                        s.push_str(&format!("  Φselector:{}\n", selector.name));
                    } else {
                        s.push_str(&format!("  Φselector:{} = createSelector({})\n",
                            selector.name, selector.inputs.join(", ")));
                    }
                }
            }
        }

        // Entity adapter (High)
        if let Some(ref entity) = self.entity_adapter {
            match fidelity {
                Fidelity::Low | Fidelity::Medium => {
                    let mut line = format!("  Φentity:{}", entity.entity_type);
                    // NgRx Data auto-generated CRUD services are noted at
                    // every fidelity level (per the plan's Gotchas section).
                    if entity.data_layer {
                        line.push_str(" (data-layer)");
                    }
                    s.push_str(&line);
                    s.push('\n');
                }
                Fidelity::High => {
                    let mut line = format!("  Φentity:{}", entity.entity_type);
                    if entity.data_layer {
                        line.push_str(" (data-layer)");
                    }
                    if let Some(ref sid) = entity.select_id {
                        line.push_str(&format!(" selectId=({})", sid));
                    }
                    if let Some(ref sc) = entity.sort_comparer {
                        line.push_str(&format!(" sortComparer=({})", sc));
                    }
                    s.push_str(&line);
                    s.push('\n');
                    if include_entity_selectors && !entity.selectors.is_empty() {
                        s.push_str(&format!("    selectors: {}\n", entity.selectors.join(", ")));
                    }
                }
            }
        }

        // Store injections (Medium+)
        for store in &self.store_injections {
            s.push_str(&format!("  Φstore:{}\n", store));
        }

        // Dispatch/select sites (Medium+)
        if fidelity != Fidelity::Low {
            if include_dispatch {
                for site in &self.dispatch_sites {
                    s.push_str(&format!("  Φdispatch:{}\n", site.action_name));
                }
            }
            if include_select {
                for site in &self.select_sites {
                    s.push_str(&format!("  Φselect:{}\n", site.selector_name));
                }
            }
        }

        s
    }
}

// ---------------------------------------------------------------------------
// Detection — import gate
// ---------------------------------------------------------------------------

/// Check whether the source file has NgRx imports.
/// Returns true if the file imports from `@ngrx/store`, `@ngrx/effects`,
/// or `@ngrx/entity`.
pub fn has_ngrx_imports(source: &str) -> bool {
    source.contains("from '@ngrx/store'")
        || source.contains("from \"@ngrx/store\"")
        || source.contains("from '@ngrx/effects'")
        || source.contains("from \"@ngrx/effects\"")
        || source.contains("from '@ngrx/entity'")
        || source.contains("from \"@ngrx/entity\"")
        || source.contains("from '@ngrx/data'")
        || source.contains("from \"@ngrx/data\"")
        // Barrel-import fallback (per the plan's Gotchas section): some
        // projects re-export NgRx creators from a local `index.ts` barrel.
        // If the @ngrx import isn't visible but the file directly calls an
        // NgRx creator function, treat it as NgRx so the extraction pass runs.
        || (source.contains("createAction(")
            || source.contains("createReducer(")
            || source.contains("createEffect(")
            || source.contains("createSelector(")
            || source.contains("createEntityAdapter(")
            || source.contains("createFeature("))
}

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
            let name = after_name.trim_start()
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
        let name = rest.trim_start()
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
        let name = before.split_whitespace()
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
        let (body, end_offset) = crate::angular_meta::util::collect_call_body(&source[after_paren..]);

        // Extract event string (first quoted string)
        let event_string = crate::angular_meta::util::extract_first_quoted(&body).unwrap_or_default();

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
            shape.feature_name.clone().unwrap_or_else(|| "featureReducer".to_string())
        } else {
            // Strip the trailing ` = ` (the assignment operator) so the
            // last whitespace token is the actual variable name. The bare
            // `createReducer(` match includes the ` = ` in `before`.
            let before_ident = before.trim_end().trim_end_matches('=').trim();
            before_ident.split_whitespace()
                .last()
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        // The match pattern is `createReducer(` — `idx` points at the `C`.
        // Advance past the `createReducer(` to collect the call body.
        let after_start = abs_idx + "createReducer(".len();
        let (body, end_offset) = crate::angular_meta::util::collect_call_body(&source[after_start..]);

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
                    if !ty.is_empty() {
                        Some(ty)
                    } else {
                        None
                    }
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
            let action_name = after_on.split(',').next()
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
        let name = before.split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let after_start = abs_idx + " = createEffect(".len();
        let (after, end_offset) = crate::angular_meta::util::collect_call_body(&source[after_start..]);

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
            let (of_body, _) =
                crate::angular_meta::util::collect_call_body(after_ot);
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

                let call = body.split('(').next()
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
                let action = after_of.split('(').next()
                    .map(|s| s.trim().trim_end_matches(')').to_string())
                    .unwrap_or_default();
                if !action.is_empty() {
                    Some(action)
                } else {
                    None
                }
            } else if let Some(arrow_idx) = after_ce.find("=> ") {
                let after_arrow = &after_ce[arrow_idx + 3..];
                let action = after_arrow.split('(').next()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if !action.is_empty() { Some(action) } else { None }
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
            let action = after_arrow.split('(').next()
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

/// Extract selectors from `createSelector(...)` calls.
fn extract_selectors(source: &str, shape: &mut NgRxShape) {
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
        let name = before.split_whitespace()
            .last()
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let after_start = abs_idx + " = createSelector(".len();
        let (body, end_offset) = crate::angular_meta::util::collect_call_body(&source[after_start..]);

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
fn extract_entity_adapter(source: &str, shape: &mut NgRxShape) {
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
        let _name = before.split_whitespace()
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
        let (body, end_offset) = crate::angular_meta::util::collect_call_body(&source[config_start..]);

        // Extract selectId and sortComparer from the config object body.
        // The body starts with `({...})` — strip the outer parens.
        let rest = body.trim_start_matches('(').trim_end_matches(')');
        let select_id = if let Some(sid_idx) = rest.find("selectId:") {
            let after_sid = &rest[sid_idx + "selectId:".len()..];
            let sid = after_sid.split(',').next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if !sid.is_empty() { Some(sid) } else { None }
        } else {
            None
        };

        let sort_comparer = if let Some(sc_idx) = rest.find("sortComparer:") {
            let after_sc = &rest[sc_idx + "sortComparer:".len()..];
            let sc = after_sc.split('}').next()
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
fn extract_component_name(source: &str, shape: &mut NgRxShape) {
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
            let name = after_class.split_whitespace()
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
fn extract_store_injections(source: &str, shape: &mut NgRxShape) {
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
            if crate::angular_meta::util::is_inside_comment_or_string(
                source, trimmed_abs + idx,
            ) {
                line_start += line.len() + 1;
                continue;
            }
            let after = &trimmed[idx + ": Store<".len()..];
            let state_type = after.split('>').next()
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
fn extract_call_sites(source: &str, shape: &mut NgRxShape) {
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
            let (body, end_offset) = crate::angular_meta::util::collect_call_body(&source[after_start..]);

            // The action/selector name is the first identifier in the body
            // (e.g. `loadUsersSuccess({ users })` → `loadUsersSuccess`;
            // `selectUser({ id })` → `selectUser`).
            let first_ident = body.trim_start()
                .split(['(', ',', ' ', '\t', '\n', '\r'])
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if !first_ident.is_empty() {
                match kind {
                    SiteKind::Dispatch => {
                        shape.dispatch_sites.push(DispatchSite { action_name: first_ident });
                    }
                    SiteKind::Select | SiteKind::PipeSelect => {
                        shape.select_sites.push(SelectSite { selector_name: first_ident });
                    }
                }
            }

            search_from = after_start + end_offset;
        }
    }
}

/// The flavour of store call site extracted by [`extract_call_sites`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SiteKind {
    Dispatch,
    Select,
    PipeSelect,
}

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Expand every recognised NgRx `Φ` marker in a line back to its
/// human-readable form. Used by the decompressor.
///
/// This is chained into the existing Angular `expand_phi_in_line` in
/// `markers.rs` via the [`PHI_EXPANDERS`](crate::angular_meta::phi::PHI_EXPANDERS)
/// registry.
pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<NgRxKind>(line)
}

/// Expand a single NgRx `Φ` marker token to its human-readable form.
/// Returns `None` for unknown markers.
pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<NgRxKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/ngrx.rs"]
mod tests;