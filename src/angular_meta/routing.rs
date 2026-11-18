// src/angular_meta/routing.rs
//
// Routing Meta-Layer — Phase 4 of the Angular Ecosystem Deepening.
//
// Detects and compresses Angular Router constructs — `Routes` arrays,
// `RouterModule.forRoot`/`forChild`, lazy `loadComponent`/`loadChildren`,
// route guards, and resolvers — in Angular TypeScript files.
//
// # Purely additive
//
// The Routing meta-layer never modifies existing TS compression output.
// It only appends a `// --- Φ Routing Meta ---` block below the existing
// compacted class. Non-Routing files pay zero overhead (import-gate
// detection via `@angular/router`).

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;
use std::path::Path;

mod extract_guards;
#[cfg(test)]
mod extraction_metrics;
mod structured;

// Guard/resolver extraction moved to `routing/extract_guards.rs`; the entry
// helpers stay module-internal (`pub(super)`) and are used here.
use extract_guards::{extract_guards, extract_resolvers};

// ---------------------------------------------------------------------------
// RouteKind — single source of truth for Routing marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind for Angular Router constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteKind {
    Route,
    Guard,
    Resolver,
}

impl PhiMarker for RouteKind {
    /// The `Φ` marker prefix for this kind.
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Route => "Φroute:",
            Self::Guard => "Φguard:",
            Self::Resolver => "Φresolver:",
        }
    }

    /// The human-readable expansion.
    fn expansion(self) -> &'static str {
        match self {
            Self::Route => "Route",
            Self::Guard => "Guard",
            Self::Resolver => "Resolver",
        }
    }

    /// All variants in a canonical order.
    fn all_in_expand_order() -> &'static [RouteKind] {
        &[
            Self::Resolver, // Φresolver: (10 chars)
            Self::Guard,    // Φguard:    (7 chars)
            Self::Route,    // Φroute:    (7 chars)
        ]
    }

    /// Look up a [`RouteKind`] by its marker token string.
    fn from_token(token: &str) -> Option<RouteKind> {
        match token {
            "Φroute" => Some(Self::Route),
            "Φguard" => Some(Self::Guard),
            "Φresolver" => Some(Self::Resolver),
            _ => None,
        }
    }

    /// Returns the token string.
    fn token(self) -> &'static str {
        match self {
            Self::Route => "Φroute",
            Self::Guard => "Φguard",
            Self::Resolver => "Φresolver",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single route declaration.
#[derive(Debug, Clone)]
pub struct RouteDecl {
    /// The route path (e.g. `"users"`, `""`, `"**"`).
    pub path: String,
    /// The eagerly-loaded component (e.g. `UserListComponent`).
    pub component: Option<String>,
    /// The lazily-loaded component (e.g. `() => import('./user-detail')`).
    pub load_component: Option<String>,
    /// The lazily-loaded children module (e.g. `() => import('./users')`).
    pub load_children: Option<String>,
    /// A declaration-proven lazy Component target.
    pub resolved_load_component: Option<LazyRouteTarget>,
    /// A declaration-proven lazy Module or route-configuration target.
    pub resolved_load_children: Option<LazyRouteTarget>,
    /// Guard names applied to this route (e.g. `AuthGuard`).
    pub guards: Vec<String>,
    /// Resolver names applied to this route (e.g. `UserResolver`).
    pub resolvers: Vec<String>,
}

/// A declaration-proven lazy route target.
#[derive(Debug, Clone)]
pub struct LazyRouteTarget {
    pub entity_type: &'static str,
    pub name: String,
}

impl LazyRouteTarget {
    fn new(entity_type: &'static str, name: String) -> Self {
        Self { entity_type, name }
    }
}

/// A standalone guard declaration (class or function).
#[derive(Debug, Clone)]
pub struct GuardDecl {
    /// The guard name (e.g. `AuthGuard`).
    pub name: String,
    /// The guard kind (e.g. `CanActivate`, `CanLoad`, `CanDeactivate`).
    pub kind: String,
}

/// A standalone resolver declaration.
#[derive(Debug, Clone)]
pub struct ResolverDecl {
    /// The resolver name (e.g. `UserResolver`).
    pub name: String,
}

/// The complete Routing shape extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct RouteShape {
    pub routes: Vec<RouteDecl>,
    pub semantic_dependencies: Vec<crate::layers::meta::semantic::SemanticDependency>,
    pub guards: Vec<GuardDecl>,
    pub resolvers: Vec<ResolverDecl>,
}

impl RouteShape {
    /// Returns `true` if there are no routing artifacts to emit.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty() && self.guards.is_empty() && self.resolvers.is_empty()
    }

    /// Convert this Routing shape into structured semantic edges
    /// (Phase 1 of the Semantic Relationship Model).
    ///
    /// Produces:
    /// - Route → `RouteMapsTo` → Component (one per route with a component)
    /// - Route → `GuardedBy` → Guard (one per guard per route)
    /// - Route → `ResolvedBy` → Resolver (one per resolver per route)
    pub fn to_semantic_edges(&self) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
        use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

        let mut edges = Vec::new();
        for route in &self.routes {
            let route_entity = EntityRef::new("angular", "Route", &route.path);

            // Route → RouteMapsTo → Component
            if let Some(ref comp) = route.component {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::RouteMapsTo,
                    subject: route_entity.clone(),
                    object: EntityRef::new("angular", "Component", comp),
                    layer: "angular",
                    call_evidence: None,
                });
            }

            if let Some(target) = &route.resolved_load_component {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::RouteMapsTo,
                    subject: route_entity.clone(),
                    object: EntityRef::new("angular", target.entity_type, &target.name),
                    layer: "angular",
                    call_evidence: None,
                });
            }

            if let Some(target) = &route.resolved_load_children {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::LoadsChildren,
                    subject: route_entity.clone(),
                    object: EntityRef::new("angular", target.entity_type, &target.name),
                    layer: "angular",
                    call_evidence: None,
                });
            }

            // Route → GuardedBy → Guard
            for guard in &route.guards {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::GuardedBy,
                    subject: route_entity.clone(),
                    object: EntityRef::new("angular", "Guard", guard),
                    layer: "angular",
                    call_evidence: None,
                });
            }

            // Route → ResolvedBy → Resolver
            for resolver in &route.resolvers {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::ResolvedBy,
                    subject: route_entity.clone(),
                    object: EntityRef::new("angular", "Resolver", resolver),
                    layer: "angular",
                    call_evidence: None,
                });
            }
        }

        // Guard declarations
        for guard in &self.guards {
            edges.push(SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: EntityRef::new("angular", "Guard", &guard.name),
                object: EntityRef::new("angular", "Guard", &guard.kind),
                layer: "angular",
                call_evidence: None,
            });
        }

        // Resolver declarations
        for resolver in &self.resolvers {
            edges.push(SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: EntityRef::new("angular", "Resolver", &resolver.name),
                object: EntityRef::new("angular", "Guard", "Resolve"),
                layer: "angular",
                call_evidence: None,
            });
        }

        edges
    }

    /// Render the full `Φ Routing Meta` block at the given fidelity.
    pub fn render(&self, fidelity: Fidelity) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut s = String::new();
        s.push_str("// --- Φ Routing Meta ---\n");

        // Routes
        for route in &self.routes {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φroute:{}\n", route.path));
                }
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    let mut parts: Vec<String> = Vec::new();
                    if let Some(ref c) = route.component {
                        parts.push(format!("component={}", c));
                    }
                    if let Some(ref lc) = route.load_component {
                        parts.push(format!("loadComponent={}", lc));
                    }
                    if let Some(ref lch) = route.load_children {
                        parts.push(format!("loadChildren={}", lch));
                    }
                    if !route.guards.is_empty() {
                        parts.push(format!("guards=[{}]", route.guards.join(",")));
                    }
                    if !route.resolvers.is_empty() {
                        parts.push(format!("resolvers=[{}]", route.resolvers.join(",")));
                    }
                    if parts.is_empty() {
                        s.push_str(&format!("  Φroute:{}\n", route.path));
                    } else {
                        s.push_str(&format!("  Φroute:{} {}\n", route.path, parts.join(" ")));
                    }
                }
            }
        }

        // Guards
        for guard in &self.guards {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φguard:{}\n", guard.name));
                }
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    s.push_str(&format!("  Φguard:{} {}\n", guard.name, guard.kind));
                }
            }
        }

        // Resolvers
        for resolver in &self.resolvers {
            s.push_str(&format!("  Φresolver:{}\n", resolver.name));
        }

        s
    }
}

// ---------------------------------------------------------------------------
// Detection — import gate
// ---------------------------------------------------------------------------

/// Check whether the source file uses Angular Router.
/// Returns true if the file imports from `@angular/router`.
pub fn has_router_imports(source: &str) -> bool {
    source.contains("@angular/router")
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Extract the Routing shape from a source file.
pub fn extract_route_shape(source: &str, fidelity: Fidelity) -> Option<RouteShape> {
    extract_route_shape_at(source, fidelity, None)
}

/// Extract routes with optional declaration-path authority for lazy targets.
pub fn extract_route_shape_at(
    source: &str,
    fidelity: Fidelity,
    declaring_path: Option<&Path>,
) -> Option<RouteShape> {
    let lexical_regions = crate::meta_util::LexicalRegions::new(source);
    extract_route_shape_at_with_regions(source, fidelity, &lexical_regions, declaring_path)
}

pub(crate) fn extract_route_shape_with_regions(
    source: &str,
    fidelity: Fidelity,
    lexical_regions: &crate::meta_util::LexicalRegions,
) -> Option<RouteShape> {
    extract_route_shape_at_with_regions(source, fidelity, lexical_regions, None)
}

pub(crate) fn extract_route_shape_at_with_regions(
    source: &str,
    _fidelity: Fidelity,
    lexical_regions: &crate::meta_util::LexicalRegions,
    declaring_path: Option<&Path>,
) -> Option<RouteShape> {
    #[cfg(test)]
    extraction_metrics::record_extraction();

    if !has_router_imports(source) {
        return None;
    }
    let (routes, semantic_dependencies) = structured::extract(source, declaring_path);
    let mut shape = RouteShape {
        routes,
        semantic_dependencies,
        ..RouteShape::default()
    };

    extract_guards(source, &mut shape, lexical_regions);
    extract_resolvers(source, &mut shape, lexical_regions);

    (!shape.is_empty()).then_some(shape)
}

#[cfg(test)]
pub(crate) use extraction_metrics::{extraction_count, reset_extraction_count};
// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Expand every recognised Routing `Φ` marker in a line back to its
/// human-readable form.
pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<RouteKind>(line)
}

/// Expand a single Routing `Φ` marker token.
pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<RouteKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/routing.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/angular_meta/routing_issue_139.rs"]
mod routing_issue_139;
