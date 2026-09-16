// src/tests/workspace/index_support.rs
//
// Shared edge-construction helpers for the WorkspaceIndex test modules.
// Loaded through `#[path]` from `workspace::index` so every index test
// module can use the same builders.
//
// The `#[path]`-loaded module is compiled in both lib and test targets. In
// the lib target the `#[cfg(test)]` gate prevents this from causing
// dead-code warnings; clippy across all targets requires this allow.
#![allow(dead_code)]

use crate::layers::meta::semantic::*;
use crate::workspace::index::WorkspaceIndex;

/// Helper: build a simple Inject edge.
pub fn inject_edge(component: &str, service: &str, file: Option<&str>) -> SemanticEdge {
    let mut subj = EntityRef::new("angular", "Component", component);
    let mut obj = EntityRef::new("angular", "Service", service);
    if let Some(f) = file {
        subj.file = Some(f.to_string());
        obj.file = Some(f.to_string());
    }
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: subj,
        object: obj,
        layer: "angular",
    }
}

/// Helper: build a RouteMapsTo edge.
pub fn route_edge(path: &str, component: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::RouteMapsTo,
        subject: EntityRef::new("angular", "Route", path),
        object: EntityRef::new("angular", "Component", component),
        layer: "angular",
    }
}

/// Helper: build a ControllerAction edge (dotnet controller shape).
pub fn controller_action_edge(controller: &str, action: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::ControllerAction,
        subject: EntityRef::new("dotnet", "Controller", controller),
        object: EntityRef::new("dotnet", "Action", action),
        layer: "dotnet",
    }
}

/// Helper: build a HasRoute edge (dotnet controller shape).
pub fn has_route_edge(controller: &str, route: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::HasRoute,
        subject: EntityRef::new("dotnet", "Controller", controller),
        object: EntityRef::new("dotnet", "Route", route),
        layer: "dotnet",
    }
}

/// Helper: create an ImportsModule edge (dependency relation).
pub fn imports_module_edge(from: &str, to: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::ImportsModule,
        subject: EntityRef::new("angular", "Module", from),
        object: EntityRef::new("angular", "Module", to),
        layer: "angular",
    }
}

/// Helper: create a HandlesAction edge (dependency relation).
pub fn handles_action_edge(effect: &str, action: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::HandlesAction,
        subject: EntityRef::new("ngrx", "Effect", effect),
        object: EntityRef::new("ngrx", "Action", action),
        layer: "ngrx",
    }
}

/// Helper: create a ConfigurationProperties edge (dependency relation).
pub fn config_props_edge(config: &str, prefix: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::ConfigurationProperties,
        subject: EntityRef::new("spring", "Configuration", config),
        object: EntityRef::new("spring", "Properties", prefix),
        layer: "spring",
    }
}

/// The canonical file identities used by the occurrence-aware edge tests.
pub const FILE_A: &str = "project-a/loading.component.ts";
pub const FILE_B: &str = "project-b/loading.component.ts";
pub const FILE_C: &str = "project-c/loading.component.ts";

/// Deterministic, provenance-carrying view of an edge result set:
/// `asserting source file | subject -> object`.
///
/// Sorted so compile order can never influence a comparison.
pub fn evidence(edges: Vec<&SemanticEdge>) -> Vec<String> {
    let mut out: Vec<String> = edges
        .iter()
        .map(|edge| {
            format!(
                "{}|{}->{}",
                edge.subject.file.clone().unwrap_or_default(),
                edge.subject.name,
                edge.object.name
            )
        })
        .collect();
    out.sort();
    out
}

/// Build a WorkspaceIndex by inserting `(file, edges)` entries in the given
/// order, so compile-order independence can be asserted directly.
pub fn index_in_order(entries: &[(String, Vec<SemanticEdge>)]) -> WorkspaceIndex {
    let mut idx = WorkspaceIndex::new();
    for (file, edges) in entries {
        idx.add_edges(file, edges.clone());
    }
    idx
}
