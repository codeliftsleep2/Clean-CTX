// src/angular_meta/ngrx/shape.rs
//
// NgRx shape behaviour: marker rendering, graph-edge projection, and
// semantic-edge projection for an extracted `NgRxShape`.
//
// Split out of `src/angular_meta/ngrx.rs` (active-file size policy): the
// module had exceeded the 615-line ceiling. This is a pure relocation -- the
// code below is byte-for-byte the previous `impl NgRxShape` block. It
// references none of the extraction helpers, so it has no cross-module
// dependencies.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies `NgRxShape` (and its struct fields),
// the marker enums, and the `PhiMarker` trait.

use super::*;

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
    /// [`NgRxEdgeKind`](crate::angular_meta::ngrx::NgRxEdgeKind)
    /// marker prefix. The caller feeds these into the legacy graph.
    pub fn to_graph_edges(&self) -> Vec<(String, String, crate::angular_meta::ngrx::NgRxEdgeKind)> {
        use crate::angular_meta::ngrx::NgRxEdgeKind;
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
                    format!(
                        "Φngrx:{}",
                        self.feature_name
                            .clone()
                            .unwrap_or_else(|| "Feature".to_string())
                    ),
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

    /// Convert this NgRx shape into structured semantic edges
    /// (Phase 1 of the Semantic Relationship Model).
    ///
    /// Parallel to [`to_graph_edges`](Self::to_graph_edges): the legacy
    /// graph API renders flat `(from, to, kind)` triples for the DI graph,
    /// while this method produces `SemanticEdge` objects with typed
    /// relations for the IR InferenceLayer. Reuses the same structured data
    /// fields — no re-parsing of source text.
    ///
    /// Produces:
    /// - Effect → `HandlesAction` → Action (one per `ofType` source action)
    /// - Effect → `CallsService` → ServiceMethod (from `switchMap` body)
    /// - Action → `TriggersReducer` → Reducer (from `on(action)` handlers)
    /// - Effect → `ProducesAction` → Action (from `map(successAction)`)
    /// - Component → `HasStore` → Store (from `Store<T>` DI)
    /// - Component → `Dispatches` → Action (via `this.store.dispatch(...)`)
    /// - Component → `Selects` → Selector (via `this.store.select(...)`)
    pub fn to_ngrx_semantic_edges(&self) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
        use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

        let mut edges: Vec<SemanticEdge> = Vec::new();

        // Action → TriggersReducer → Reducer (via `on(action)` handlers).
        if let Some(reducer) = &self.reducer {
            for transition in &reducer.transitions {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::TriggersReducer,
                    subject: EntityRef::new("ngrx", "Action", &transition.action_name),
                    object: EntityRef::new("ngrx", "Reducer", &reducer.name),
                    layer: "ngrx",
                    call_evidence: None,
                });
            }
        }

        // Effect → HandlesAction → Action (via `ofType(action)`).
        for effect in &self.effects {
            let sources: Vec<&str> = if !effect.source_actions.is_empty() {
                effect.source_actions.iter().map(|s| s.as_str()).collect()
            } else {
                effect.source_action.iter().map(|s| s.as_str()).collect()
            };
            for source in sources {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::HandlesAction,
                    subject: EntityRef::new("ngrx", "Effect", &effect.name),
                    object: EntityRef::new("ngrx", "Action", source),
                    layer: "ngrx",
                    call_evidence: None,
                });
            }

            // Effect → CallsService → ServiceMethod
            if let Some(service) = &effect.service_call {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::CallsService,
                    subject: EntityRef::new("ngrx", "Effect", &effect.name),
                    object: EntityRef::new("ngrx", "ServiceMethod", service),
                    layer: "ngrx",
                    call_evidence: None,
                });
            }

            // Effect → ProducesAction → Action (via `map(successAction)`).
            if let Some(success) = &effect.success_action {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::ProducesAction,
                    subject: EntityRef::new("ngrx", "Effect", &effect.name),
                    object: EntityRef::new("ngrx", "Action", success),
                    layer: "ngrx",
                    call_evidence: None,
                });
            }
        }

        // Component → HasStore → Store (via `Store<T>` DI).
        if let Some(ref component) = self.component_name {
            for store_type in &self.store_injections {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::HasStore,
                    subject: EntityRef::new("angular", "Component", component),
                    object: EntityRef::new("ngrx", "Store", store_type),
                    layer: "ngrx",
                    call_evidence: None,
                });
            }
        }

        // Component → Dispatches → Action (via `this.store.dispatch(...)`).
        // Uses the actual component class as the subject when available.
        for site in &self.dispatch_sites {
            let subject = if let Some(ref component) = self.component_name {
                EntityRef::new("angular", "Component", component)
            } else {
                EntityRef::new("angular", "DispatchSite", "store")
            };
            edges.push(SemanticEdge {
                relation: SemanticRelation::Dispatches,
                subject,
                object: EntityRef::new("ngrx", "Action", &site.action_name),
                layer: "ngrx",
                call_evidence: None,
            });
        }

        // Component → Selects → Selector (via `this.store.select(...)`).
        // Uses the actual component class as the subject when available.
        for site in &self.select_sites {
            let subject = if let Some(ref component) = self.component_name {
                EntityRef::new("angular", "Component", component)
            } else {
                EntityRef::new("angular", "SelectSite", "store")
            };
            edges.push(SemanticEdge {
                relation: SemanticRelation::Selects,
                subject,
                object: EntityRef::new("ngrx", "Selector", &site.selector_name),
                layer: "ngrx",
                call_evidence: None,
            });
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
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    if let Some(ref props) = action.props_type {
                        s.push_str(&format!(
                            "  Φaction:{} '{}' props<{}>\n",
                            action.name, action.event_string, props
                        ));
                    } else {
                        s.push_str(&format!(
                            "  Φaction:{} '{}'\n",
                            action.name, action.event_string
                        ));
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
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    if let Some(ref st) = reducer.state_type {
                        s.push_str(&format!("  Φreducer:{} → {}\n", reducer.name, st));
                    } else {
                        s.push_str(&format!("  Φreducer:{}\n", reducer.name));
                    }
                    for transition in &reducer.transitions {
                        s.push_str(&format!(
                            "    on({}) → {}\n",
                            transition.action_name, transition.state_summary
                        ));
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
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
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
                    if fidelity == Fidelity::High
                        || fidelity == Fidelity::Edit
                        || fidelity == Fidelity::Verbatim
                    {
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
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    if selector.inputs.is_empty() {
                        s.push_str(&format!("  Φselector:{}\n", selector.name));
                    } else {
                        s.push_str(&format!(
                            "  Φselector:{} = createSelector({})\n",
                            selector.name,
                            selector.inputs.join(", ")
                        ));
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
                Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
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
