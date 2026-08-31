// src/layers/meta/mod.rs
//
// MetaLayer trait + per-framework implementations.
// Each implementation is gated by its Cargo feature so the framework-specific
// code is only compiled when the feature is enabled.
//
// # Preferred pattern for new meta-layers
//
// New meta-layers SHOULD follow the decentralized .NET pattern:
//   - Define the MetaLayer struct + MetaLayer trait impl in the meta-layer's
//     own module (e.g. `src/dotnet_meta/mod.rs`)
//   - Register it in `src/layers/registry.rs` with a `#[cfg(feature = "...")]` guard
//   - The decentralized approach keeps the meta-layer's code co-located with
//     its detection, extraction, and marker logic
//
// Legacy: Angular and Spring Boot meta-layers have their trait impls directly
// in this file for historical reasons. New meta-layers should NOT follow
// this pattern.

pub mod semantic;

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::layers::meta::semantic::SemanticEdge;
use std::path::Path;

/// Structured output of a single meta-layer pass.
///
/// This replaces the previous render-then-reparse anti-pattern where
/// `enrich` wrote a rendered `String` and the pipeline re-parsed it back
/// into a structured `MetaBlock` by string-splitting on `// ---` headers.
///
/// Each layer produces its structured block (if any) plus the rendered
/// text. The pipeline uses the structured block directly; the rendered
/// text is retained for consumers that only need the flat string
/// (e.g. the IR compiler's `TypeAlias` enrichment).
#[derive(Debug, Clone, Default)]
pub struct MetaLayerOutput {
    /// The layer that produced this output (e.g. "angular").
    pub layer_name: &'static str,
    /// The fully-rendered `Φ` block text (empty when the layer produced
    /// no markers for this file).
    pub rendered: String,
    /// Structured Angular block (decorators + RxJS/NgRx/Signals/Routing
    /// sections). `None` when the file is not Angular.
    pub angular_block: Option<crate::angular_meta::MetaBlock>,
    /// Structured Spring Boot block. `None` when not a Spring file.
    pub spring_block: Option<crate::spring_meta::MetaBlock>,
    /// Structured .NET block. `None` when not a .NET file.
    pub dotnet_block: Option<crate::dotnet_meta::MetaBlock>,
}

/// A meta-layer that enriches compressed output with framework-specific
/// context (e.g. Angular decorators, Spring Boot annotations).
///
/// Meta-layers are purely additive: they append framework-specific markers
/// to the compressed output without modifying the base compression.
pub trait MetaLayer: Send + Sync {
    /// Unique identifier (e.g. "angular", "spring_boot").
    fn name(&self) -> &'static str;

    /// Check if this meta-layer applies to the given source file.
    /// Called before `enrich` to avoid unnecessary work.
    ///
    /// `config` is optional so the layer can honor its per-framework
    /// `enabled` flag (a disabled layer returns `false`, skipping
    /// detection entirely — zero overhead).
    fn is_applicable(&self, source: &str, path: &Path, config: Option<&CleanCtxConfig>) -> bool;

    /// Enrich the compressed output with framework-specific markers.
    ///
    /// Returns `Some(MetaLayerOutput)` when the layer produced markers for
    /// this file, `None` when it produced none (zero overhead).
    ///
    /// # Arguments
    /// - `source`: The full source text of the file being compressed.
    /// - `class_captures`: The canonical source spans of each type capture
    ///   (class/interface/enum/record/struct), in document order. Each span
    ///   is the exact text of that type's declaration including leading
    ///   decorators/annotations/attributes and the full body. The meta-layer
    ///   must NEVER inspect text outside the owning class span — this is the
    ///   per-class metadata invariant.
    /// - `fidelity`: The fidelity level controlling verbosity.
    /// - `config`: The project config (optional) so per-layer `enabled`
    ///   flags and sub-layer settings are honored.
    fn enrich(
        &self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
        config: Option<&CleanCtxConfig>,
    ) -> Option<MetaLayerOutput>;

    /// Extract structured semantic edges for the given source file.
    ///
    /// Returns an empty vector by default -- meta-layers are NOT required
    /// to produce semantic edges. Semantic edges are structural facts
    /// (implicit confidence 1.0) discovered by the same per-framework
    /// helpers that produce the Phi markers; `enrich()` output is
    /// unchanged. No plugin feeds these yet (Phase 1+); the default keeps
    /// every existing layer behavior-identical.
    fn extract_semantic_edges(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Vec<SemanticEdge> {
        Vec::new()
    }
}

// ── Angular Meta-Layer ────────────────────────────────────────────────

#[cfg(feature = "angular")]
pub struct AngularMetaLayer;

#[cfg(feature = "angular")]
impl AngularMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "angular")]
impl Default for AngularMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "angular")]
impl MetaLayer for AngularMetaLayer {
    fn name(&self) -> &'static str {
        "angular"
    }

    fn is_applicable(&self, source: &str, _path: &Path, config: Option<&CleanCtxConfig>) -> bool {
        // Honor the per-framework `enabled` flag. When the config is
        // absent or the "angular" entry is missing, the layer is on
        // (default behaviour).
        let enabled = config
            .and_then(|c| c.meta_layers.get("angular"))
            .map(|m| m.enabled)
            .unwrap_or(true);
        if !enabled {
            return false;
        }

        // Delegate to the existing Angular detection heuristic, plus the
        // Angular Ecosystem Deepening import gates. Pure NgRx actions/
        // selectors files, routing config files (app.routes.ts), and
        // standalone RxJS services have no @Component/@Injectable
        // decorators — they must still pass the gate so the pipeline
        // enriches them. The per-layer extractors apply their own
        // import-gate checks internally, so false positives here are
        // cheap (zero output, zero overhead).
        crate::angular_meta::detect::is_angular_file(source)
            || crate::angular_meta::rx::has_rxjs_imports(source)
            || crate::angular_meta::ngrx::has_ngrx_imports(source)
            || crate::angular_meta::signals::has_signal_imports(source)
            || crate::angular_meta::routing::has_router_imports(source)
    }

    fn enrich(
        &self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
        config: Option<&CleanCtxConfig>,
    ) -> Option<MetaLayerOutput> {
        // Honor the per-framework meta-layer config (enabled flags,
        // min_pipe_operators, include_dispatch_sites, etc.). When the
        // config is absent or the "angular" entry is missing, all
        // sub-layers run with their defaults.
        let meta_config = config.and_then(|c| c.meta_layers.get("angular"));
        let block = crate::angular_meta::run_meta_layer_with_config(
            source,
            class_captures,
            fidelity,
            meta_config,
        )?;
        if block.is_empty() {
            return None;
        }
        Some(MetaLayerOutput {
            layer_name: self.name(),
            rendered: block.render(),
            angular_block: Some(block),
            spring_block: None,
            dotnet_block: None,
        })
    }

    fn extract_semantic_edges(
        &self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
        config: Option<&CleanCtxConfig>,
    ) -> Vec<SemanticEdge> {
        let meta_config = config.and_then(|c| c.meta_layers.get("angular"));
        let mut edges: Vec<SemanticEdge> = Vec::new();

        // 1. Decorator-based edges from class captures
        let is_angular = crate::angular_meta::detect::is_angular_file(source);
        if is_angular {
            // Build a name→entity_type map from all class captures to support
            // precise DeclaresInModule/ExportsFromModule entity types.
            let mut decl_types: std::collections::HashMap<String, &'static str> =
                std::collections::HashMap::new();
            for raw_class in class_captures {
                if let Some((class_name, kind, _, _, _)) =
                    crate::angular_meta::decorators::extract_graph_entries(raw_class)
                {
                    let entity_type = match kind {
                        crate::angular_meta::decorators::ClassKind::Component => "Component",
                        crate::angular_meta::decorators::ClassKind::Service => "Service",
                        crate::angular_meta::decorators::ClassKind::Directive => "Directive",
                        crate::angular_meta::decorators::ClassKind::Pipe => "Pipe",
                        crate::angular_meta::decorators::ClassKind::Module => "Module",
                    };
                    decl_types.insert(class_name, entity_type);
                }
            }

            for raw_class in class_captures {
                // extract_graph_entries gives structured class metadata
                // (kind, selector, injects, pipe_name) — no re-parsing of the source.
                if let Some((class_name, kind, selector, injects, pipe_name)) =
                    crate::angular_meta::decorators::extract_graph_entries(raw_class)
                {
                    edges.extend(crate::angular_meta::semantic::class_to_semantic_edges(
                        &class_name,
                        kind,
                        selector.as_deref(),
                        &injects,
                        pipe_name.as_deref(),
                        raw_class,
                        fidelity,
                        &decl_types,
                    ));
                }
            }
        }

        // 2. NgRx semantic edges — the shape extraction already parses the
        //    file for NgRx artifacts; reuse its structured output.
        let ngrx_enabled = meta_config.map(|c| c.ngrx.enabled).unwrap_or(true);
        if ngrx_enabled {
            if let Some(shape) = crate::angular_meta::ngrx::extract_ngrx_shape(source, fidelity) {
                edges.extend(shape.to_ngrx_semantic_edges());
            }
        }

        // 3. Routing semantic edges — same reuse of existing shape extraction.
        let routing_enabled = meta_config.map(|c| c.routing.enabled).unwrap_or(true);
        if routing_enabled {
            if let Some(shape) = crate::angular_meta::routing::extract_route_shape(source, fidelity)
            {
                edges.extend(shape.to_semantic_edges());
            }
        }

        // 4. RxJS: observables and subjects describe data-flow wiring, not
        //    cross-entity semantic relationships. The NgRx effect extraction
        //    already covers the action→effect→service pipeline, which is the
        //    primary semantic chain. RxJS entries are retained as structural
        //    metadata (Φ markers); semantic-edge projection of observables
        //    is deferred to Phase 4 (WorkspaceIndex) if needed.

        edges
    }
}

#[cfg(not(feature = "angular"))]
pub struct AngularMetaLayer;

#[cfg(not(feature = "angular"))]
impl AngularMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "angular"))]
impl Default for AngularMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "angular"))]
impl MetaLayer for AngularMetaLayer {
    fn name(&self) -> &'static str {
        "angular"
    }

    fn is_applicable(&self, _source: &str, _path: &Path, _config: Option<&CleanCtxConfig>) -> bool {
        false
    }

    fn enrich(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Option<MetaLayerOutput> {
        // No-op when feature is disabled
        None
    }
}

// ── Spring Boot Meta-Layer ────────────────────────────────────────────

#[cfg(feature = "spring_boot")]
pub struct SpringBootMetaLayer;

#[cfg(feature = "spring_boot")]
impl SpringBootMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "spring_boot")]
impl Default for SpringBootMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "spring_boot")]
impl MetaLayer for SpringBootMetaLayer {
    fn name(&self) -> &'static str {
        "spring_boot"
    }

    fn is_applicable(&self, source: &str, _path: &Path, _config: Option<&CleanCtxConfig>) -> bool {
        // Delegate to the existing Spring Boot detection heuristic
        crate::spring_meta::detect::is_spring_file(source)
    }

    fn enrich(
        &self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Option<MetaLayerOutput> {
        let block = crate::spring_meta::run_meta_layer(source, class_captures, fidelity)?;
        if block.is_empty() {
            return None;
        }
        Some(MetaLayerOutput {
            layer_name: self.name(),
            rendered: block.render(),
            angular_block: None,
            spring_block: Some(block),
            dotnet_block: None,
        })
    }

    fn extract_semantic_edges(
        &self,
        _source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Vec<SemanticEdge> {
        let mut edges = Vec::new();
        for raw_class in class_captures {
            edges.extend(crate::spring_meta::semantic::extract_spring_semantic_edges(
                raw_class, fidelity,
            ));
        }
        edges
    }
}

#[cfg(not(feature = "spring_boot"))]
pub struct SpringBootMetaLayer;

#[cfg(not(feature = "spring_boot"))]
impl SpringBootMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "spring_boot"))]
impl Default for SpringBootMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "spring_boot"))]
impl MetaLayer for SpringBootMetaLayer {
    fn name(&self) -> &'static str {
        "spring_boot"
    }

    fn is_applicable(&self, _source: &str, _path: &Path, _config: Option<&CleanCtxConfig>) -> bool {
        false
    }

    fn enrich(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Option<MetaLayerOutput> {
        // No-op when feature is disabled
        None
    }
}
