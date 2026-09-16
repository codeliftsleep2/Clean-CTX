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

mod extract;
mod extract_selectors;
mod shape;

// Re-exported so the established public path
// (`crate::angular_meta::ngrx::extract_ngrx_shape`) is unchanged by the split.
pub use extract::extract_ngrx_shape;

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
            Self::NgRx,     // Φngrx:     (6 chars)
            Self::Action,   // Φaction:   (8 chars)
            Self::Reducer,  // Φreducer:  (9 chars)
            Self::Effect,   // Φeffect:   (8 chars)
            Self::Selector, // Φselector: (10 chars)
            Self::Entity,   // Φentity:   (8 chars)
            Self::Store,    // Φstore:    (7 chars)
            Self::Dispatch, // Φdispatch: (10 chars)
            Self::Select,   // Φselect:   (8 chars)
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
// NgRxEdgeKind — legacy graph edge kind, kept for backward compatibility
// with to_graph_edges(). Superseded by SemanticRelation for phases 4+.
// ---------------------------------------------------------------------------

/// The kind of an NgRx cross-layer graph edge (Phase 5 of the Angular
/// Ecosystem Deepening). These wire NgRx store artifacts — actions,
/// reducers, effects, selectors, and components — into the DI graph so
/// the LLM can trace `dispatch(loadUsers)` → `loadUsers$ effect` →
/// `UserService.getUsers()` → `.NET UserController.GetAll()` as a single
/// semantic chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NgRxEdgeKind {
    /// `Φaction:loadUsers` → `Φreducer:users` (via `on(loadUsers)` handler).
    ActionReducer,
    /// `Φaction:loadUsers` → `Φeffect:loadUsers$` (via `ofType(loadUsers)`).
    ActionEffect,
    /// `Φeffect:loadUsers$` → `UserService@α3` (via `switchMap(() => svc.getAll())`).
    EffectService,
    /// `Φeffect:loadUsers$` → `Φaction:loadUsersSuccess` (via `map(...)`).
    EffectAction,
    /// `UserComponent@α7` → `Φngrx:UserFeature` (via `Store<AppState>` DI).
    ComponentStore,
    /// `UserComponent@α7` → `Φselector:selectAllUsers` (via `store.select(...)`).
    ComponentSelector,
    /// `Φeffect:loadUsers$` → `UserController.GetAll@α12` (CBM cross-language).
    EffectEndpoint,
}

impl NgRxEdgeKind {
    /// The `Φ` marker prefix for this edge kind.
    pub fn marker_prefix(self) -> &'static str {
        match self {
            Self::ActionReducer => "Φact→red:",
            Self::ActionEffect => "Φact→eff:",
            Self::EffectService => "Φeff→svc:",
            Self::EffectAction => "Φeff→act:",
            Self::ComponentStore => "Φcmp→store:",
            Self::ComponentSelector => "Φcmp→sel:",
            Self::EffectEndpoint => "Φeff→endpoint:",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

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
