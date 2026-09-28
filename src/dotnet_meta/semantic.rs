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
    #[cfg(test)]
    super::class_analysis_metrics::record_analysis();

    let mut edges = crate::dotnet_meta::aspnet::analyze_aspnet(raw_class, fidelity)
        .map(|analysis| analysis.semantic_edges)
        .unwrap_or_default();
    edges.extend(
        crate::dotnet_meta::efcore::analyze_efcore(raw_class, fidelity)
            .map(|analysis| analysis.semantic_edges)
            .unwrap_or_default(),
    );
    edges.extend(extract_non_aspnet_efcore_semantic_edges(
        raw_class, class_name, fidelity,
    ));
    edges
}

pub(crate) fn extract_non_aspnet_efcore_semantic_edges(
    raw_class: &str,
    class_name: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();

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
                call_evidence: None,
            });
            edges.push(SemanticEdge {
                relation: SemanticRelation::MapsTo,
                subject: profile.clone(),
                object: EntityRef::new("dotnet", "Entity", dest),
                layer: "dotnet",
                call_evidence: None,
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
                call_evidence: None,
            });
        }
    }

    edges
}
// ── Private Helpers (mirroring existing .NET extractor patterns) ──────────

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
