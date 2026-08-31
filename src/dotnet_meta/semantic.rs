// src/dotnet_meta/semantic.rs
//
// .NET-specific semantic edge construction helpers.
//
// These follow the same scanning patterns as the existing .NET meta-layer
// extractors (aspnet, efcore, signalr, automapper) but produce SemanticEdge
// objects instead of Phi marker strings.
//
// Phase 3 contract: zero duplication of existing Phi output. Semantic edges
// are a separate projection of the same framework information.

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Extract .NET semantic edges from a single class capture.
/// Reuses the same string-scanning patterns as the existing .NET extractors.
pub fn extract_dotnet_semantic_edges(
    raw_class: &str,
    class_name: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();

    // ── ASP.NET Core: Controller -> ControllerAction, Controller -> HasRoute ──
    let is_controller = raw_class.contains("[ApiController]")
        || raw_class.contains("[Controller]")
        || raw_class.contains(": ControllerBase")
        || raw_class.contains(": Controller");

    if is_controller {
        let controller = EntityRef::new("dotnet", "Controller", class_name);

        // Controller -> HasRoute -> route template
        if let Some(route) = extract_route(raw_class) {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasRoute,
                subject: controller.clone(),
                object: EntityRef::new("dotnet", "Route", &route),
                layer: "dotnet",
            });
        }

        // Controller -> ControllerAction -> action method (at Medium+ fidelity)
        if fidelity != Fidelity::Low {
            let actions = extract_actions(raw_class);
            for (method_name, _params, _return_type) in &actions {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::ControllerAction,
                    subject: controller.clone(),
                    object: EntityRef::new("dotnet", "Action", method_name),
                    layer: "dotnet",
                });
            }
        }
    }

    // ── EF Core: DbContext -> HasEntity (via DbSet<T>) ──
    let is_dbcontext = raw_class.contains(": DbContext");
    if is_dbcontext {
        let dbcontext = EntityRef::new("dotnet", "DbContext", class_name);
        let entities = extract_dbset_entities(raw_class);
        for entity_name in &entities {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasEntity,
                subject: dbcontext.clone(),
                object: EntityRef::new("dotnet", "Entity", entity_name),
                layer: "dotnet",
            });
        }
    }

    // ── AutoMapper: Profile -> MapsFrom/MapsTo (via CreateMap<TSource, TDest>) ──
    let is_profile = raw_class.contains(": Profile");
    if is_profile {
        let profile = EntityRef::new("dotnet", "MapperProfile", class_name);
        let mappings = extract_create_map_mappings(raw_class);
        for (source, dest) in &mappings {
            edges.push(SemanticEdge {
                relation: SemanticRelation::MapsFrom,
                subject: profile.clone(),
                object: EntityRef::new("dotnet", "Entity", source),
                layer: "dotnet",
            });
            edges.push(SemanticEdge {
                relation: SemanticRelation::MapsTo,
                subject: profile.clone(),
                object: EntityRef::new("dotnet", "Entity", dest),
                layer: "dotnet",
            });
        }
    }

    // ── SignalR: Hub -> HubMethodTargets ──
    let is_hub = raw_class.contains(": Hub<") || raw_class.contains(": Hub");
    if is_hub && fidelity != Fidelity::Low {
        let hub = EntityRef::new("dotnet", "Hub", class_name);
        let methods = extract_hub_methods(raw_class);
        for method_name in &methods {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HubMethodTargets,
                subject: hub.clone(),
                object: EntityRef::new("dotnet", "HubMethod", method_name),
                layer: "dotnet",
            });
        }
    }

    edges
}
// ── Private Helpers (mirroring existing .NET extractor patterns) ──────────

/// Extract route template from [Route("...")] attribute.
fn extract_route(source: &str) -> Option<String> {
    if let Some(pos) = source.find("[Route(") {
        let start = pos + "[Route(".len();
        let rest = &source[start..];
        if let Some(quote_end) = rest.find('"') {
            let end = rest[quote_end + 1..].find('"')?;
            return Some(rest[quote_end + 1..quote_end + 1 + end].to_string());
        }
    }
    None
}

/// Extract action method names with HTTP verb attributes.
fn extract_actions(class_source: &str) -> Vec<(String, String, Option<String>)> {
    let verb_patterns = [
        "[HttpGet",
        "[HttpPost",
        "[HttpPut",
        "[HttpDelete",
        "[HttpPatch",
        "[HttpHead",
        "[HttpOptions",
    ];
    let mut actions = Vec::new();
    for attr in &verb_patterns {
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find(attr) {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + attr.len()..];
            if let Some(method_info) = extract_method_signature(rest) {
                actions.push(method_info);
            }
            search_start = actual_pos + 1;
        }
    }
    actions
}

/// Extract method name, params, and optional return type.
fn extract_method_signature(source: &str) -> Option<(String, String, Option<String>)> {
    let visibility_patterns = ["public ", "private ", "protected ", "internal "];
    for vis in &visibility_patterns {
        if let Some(pos) = source.find(vis) {
            let start = pos + vis.len();
            let rest = &source[start..];
            if let Some(paren_pos) = rest.find('(') {
                let signature = &rest[..paren_pos];
                let params = &rest[paren_pos + 1..];
                let close_paren = params.find(')')?;
                let params_str = params[..close_paren].to_string();
                let method_name = signature.split_whitespace().last()?.to_string();
                let return_type = if signature.contains(' ') {
                    let parts: Vec<&str> = signature.rsplitn(2, ' ').collect();
                    if parts.len() == 2 {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                return Some((method_name, params_str, return_type));
            }
        }
    }
    None
}

/// Extract entity names from DbSet<T> properties.
fn extract_dbset_entities(class_source: &str) -> Vec<String> {
    let mut entities = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("DbSet<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "DbSet<".len()..];
        if let Some(generic_end) = rest.find('>') {
            let entity_name = rest[..generic_end].trim().to_string();
            if !entities.contains(&entity_name) {
                entities.push(entity_name);
            }
        }
        search_start = actual_pos + 1;
    }
    entities
}

/// Extract source/destination type pairs from CreateMap<TSource, TDest>() calls.
fn extract_create_map_mappings(class_source: &str) -> Vec<(String, String)> {
    let mut mappings = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("CreateMap<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "CreateMap<".len()..];
        if let Some(generic_end) = rest.find('>') {
            let types = rest[..generic_end].trim().to_string();
            if let Some(comma_pos) = types.find(',') {
                let source = types[..comma_pos].trim().to_string();
                let dest = types[comma_pos + 1..].trim().to_string();
                mappings.push((source, dest));
            }
        }
        search_start = actual_pos + 1;
    }
    mappings
}

/// Extract class name from a .NET class declaration.
/// Follows the same pattern as the existing private extract_class_name()
/// functions in aspnet.rs, efcore.rs, etc.
pub fn extract_class_name_from_class(source: &str) -> Option<String> {
    let patterns = [
        "public class ",
        "internal class ",
        "private class ",
        "protected class ",
        "class ",
    ];
    for pattern in &patterns {
        if let Some(pos) = source.find(pattern) {
            let start = pos + pattern.len();
            let rest = &source[start..];
            let end = rest
                .find(|c: char| c == ':' || c == '<' || c.is_whitespace() || c == '{')
                .unwrap_or(rest.len());
            let name = rest[..end].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract hub method names from a SignalR Hub class.
fn extract_hub_methods(class_source: &str) -> Vec<String> {
    let mut methods = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("public ") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "public ".len()..];
        if let Some(paren_pos) = rest.find('(') {
            let signature = &rest[..paren_pos];
            let method_name = signature
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string();
            if !method_name.starts_with("On")
                && !method_name.starts_with("Dispose")
                && !method_name.is_empty()
            {
                methods.push(method_name);
            }
        }
        search_start = actual_pos + 1;
    }
    methods
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/semantic.rs"]
mod tests;
