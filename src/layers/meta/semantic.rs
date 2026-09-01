// src/layers/meta/semantic.rs
//
// Semantic relationship types shared by all meta-layers.
//
// Meta-layers discover framework-specific relationships while parsing a
// source file (injects, routes, dispatches, ...). Today those relationships
// are flattened into Phi text markers only. This module defines the
// structured SemanticEdge form so the same discovery can feed the IR
// InferenceLayer and (later) a workspace index.
//
// Identity model (semantic plan U1): an entity is identified by
// (domain, entity_type, name). The originating file is metadata, NOT part
// of the identity -- the same entity referenced from another file must
// compare equal.
//
// Semantic edges are structural facts: implicit confidence 1.0, provenance
// tracked via `layer`. They coexist with InferenceEdges (which carry
// explicit confidence + InferenceSource) in the InferenceLayer. There is no
// InferenceSource field on SemanticEdge (plan section 7, item 9).
//
// Tier discipline: this module must NEVER import from `crate::ir` -- ir
// imports from here, never the reverse (import-cycle mitigation in the
// semantic plan).

/// A structured reference to a framework entity.
///
/// `PartialEq`/`Hash` cover only (domain, entity_type, name) -- `file` is
/// excluded so identity matching works across files (plan U1/U2).
#[derive(Debug, Clone, serde::Serialize)]
pub struct EntityRef {
    /// Framework domain (e.g. "angular", "ngrx", "dotnet", "spring").
    pub domain: &'static str,
    /// Entity kind (e.g. "Component", "Service", "Action").
    pub entity_type: &'static str,
    /// Entity name (e.g. "UserComponent", "UserService").
    pub name: String,
    /// Originating file (pipeline file_id). None for local-only entities.
    pub file: Option<String>,
}

impl EntityRef {
    /// Build an entity reference with no file provenance.
    pub fn new(domain: &'static str, entity_type: &'static str, name: impl Into<String>) -> Self {
        Self {
            domain,
            entity_type,
            name: name.into(),
            file: None,
        }
    }

    /// Builder: attach the originating file (pipeline `file_id`).
    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }
}

impl PartialEq for EntityRef {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain
            && self.entity_type == other.entity_type
            && self.name == other.name
    }
}

impl Eq for EntityRef {}

impl std::hash::Hash for EntityRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.domain.hash(state);
        self.entity_type.hash(state);
        self.name.hash(state);
    }
}

/// The typed relationship between two entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum SemanticRelation {
    // ---- Angular ----
    /// Component injects a Service via constructor/DI.
    Injects,
    /// Component declares an `@Input` field.
    HasInput,
    /// Component declares an `@Output` field.
    HasOutput,
    /// Component binds a model (ngModel / forms model).
    HasModel,
    /// Component exposes a CSS selector.
    HasSelector,
    /// Component references a template.
    HasTemplate,
    /// Component references a stylesheet.
    HasStyle,
    /// NgModule declares a component/directive/pipe.
    DeclaresInModule,
    /// NgModule imports another module.
    ImportsModule,
    /// NgModule re-exports from another module.
    ExportsFromModule,
    /// A route maps to a component.
    RouteMapsTo,
    /// A route is guarded by a guard class.
    GuardedBy,
    /// A route is resolved by a resolver class.
    ResolvedBy,
    // ---- NgRx ----
    /// A component/effect dispatches an action.
    Dispatches,
    /// A component/effect selects from the store.
    Selects,
    /// An effect handles an action.
    HandlesAction,
    /// An effect calls a service method.
    CallsService,
    /// A component/effect has a store reference.
    HasStore,
    /// An action triggers a reducer state transition.
    TriggersReducer,
    /// An effect produces/dispatches a success/failure action.
    ProducesAction,
    // ---- .NET ----
    /// A controller exposes an action method.
    ControllerAction,
    /// A controller/action has a route attribute.
    HasRoute,
    /// A DbContext exposes an entity set.
    HasEntity,
    /// An entity relates to another entity.
    EntityRelationship,
    /// An AutoMapper profile maps from a source type.
    MapsFrom,
    /// An AutoMapper profile maps to a destination type.
    MapsTo,
    /// A SignalR hub method targets a client method.
    HubMethodTargets,
    // ---- Spring ----
    /// A controller autowires a service.
    Autowired,
    /// A controller endpoint maps to a handler.
    EndpointMapsTo,
    /// A configuration class produces a bean.
    BeanProduces,
    /// A properties class backs a `@ConfigurationProperties` binding.
    ConfigurationProperties,
    // ---- Generic ----
    /// A class extends another.
    Extends,
    /// A class implements an interface.
    Implements,
    /// A module/file defines an entity.
    Defines,
    /// One symbol calls another.
    Calls,
}

/// A structured semantic relationship between two entities.
///
/// Semantic edges are structural facts discovered by meta-layer parsing:
/// implicit confidence 1.0, no InferenceSource, provenance carried in
/// `layer` (e.g. "angular", "ngrx"). Duplicates are NOT removed during
/// per-file extraction -- deduplication happens at the workspace index
/// boundary (plan U2).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticEdge {
    /// The typed relationship.
    pub relation: SemanticRelation,
    /// Source entity.
    pub subject: EntityRef,
    /// Target entity.
    pub object: EntityRef,
    /// Provenance layer (e.g. "angular", "ngrx", "dotnet", "spring").
    pub layer: &'static str,
}

#[cfg(test)]
#[path = "../../tests/layers/meta/semantic.rs"]
mod tests;
