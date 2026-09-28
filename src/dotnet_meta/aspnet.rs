// src/dotnet_meta/aspnet.rs
//
// ASP.NET Core extraction — Controllers, Minimal APIs, actions, models, auth.
//
// Detects:
// - `[ApiController]` / `[Controller]` classes
// - `[Route("...")]` on class or method
// - HTTP verb attributes: `[HttpGet]`, `[HttpPost]`, `[HttpPut]`, `[HttpDelete]`, `[HttpPatch]`
// - Action methods with parameters and return types
// - `[Authorize]` with optional policy/roles
// - `[FromBody]`, `[FromRoute]`, `[FromQuery]` parameter bindings
// - Input/output model DTOs

use super::markers::{
    build_action_line, build_api_controller_line, build_auth_line, build_controller_line,
};
use crate::compression::Fidelity;
use crate::dotnet_meta::MetaBlock;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) struct AspNetAnalysis {
    pub block: MetaBlock,
    pub semantic_edges: Vec<SemanticEdge>,
}

/// Extract ASP.NET Core markers from a single class capture.
///
/// Returns `None` when the class is not an ASP.NET Core construct.
pub fn extract_aspnet(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    analyze_aspnet(class_source, fidelity).map(|analysis| analysis.block)
}

pub(crate) fn analyze_aspnet(
    class_source: &str,
    fidelity: Fidelity,
) -> Option<AspNetAnalysis> {
    #[cfg(test)]
    super::class_analysis_metrics::record_analysis();

    let mut lines = Vec::new();

    // Detect controller class
    let is_controller = class_source.contains("[ApiController]")
        || class_source.contains("[Controller]")
        || class_source.contains(": ControllerBase")
        || class_source.contains(": Controller");

    if !is_controller {
        return None;
    }

    // Extract class name
    let class_name = extract_class_name(class_source)?;

    // Detect ApiController attribute
    if class_source.contains("[ApiController]") {
        lines.push(build_api_controller_line(&class_name));
    }

    // Detect controller with route
    let route = extract_route(class_source);
    if route.is_some() || is_controller {
        lines.push(build_controller_line(&class_name, route.as_deref()));
    }

    // Detect authorization
    if class_source.contains("[Authorize]") || class_source.contains("[Authorize(") {
        let policy = extract_authorize_policy(class_source);
        lines.push(build_auth_line(policy.as_deref()));
    }

    // Extract actions (methods with HTTP verb attributes)
    let actions = (fidelity != Fidelity::Low).then(|| extract_action_facts(class_source));
    if let Some(actions) = &actions {
        lines.extend(actions.iter().filter(|action| action.marker).map(|action| {
            build_action_line(
                action.verb,
                &action.name,
                &action.params,
                action.return_type.as_deref(),
            )
        }));
    }

    // Extract models/DTOs
    if fidelity == Fidelity::High {
        lines.extend(extract_models(class_source));
    }

    if lines.is_empty() {
        None
    } else {
        let controller = EntityRef::new("dotnet", "Controller", &class_name);
        let mut semantic_edges = Vec::new();
        if let Some(route) = extract_semantic_route(class_source) {
            semantic_edges.push(SemanticEdge {
                relation: SemanticRelation::HasRoute,
                subject: controller.clone(),
                object: EntityRef::new("dotnet", "Route", &route),
                layer: "dotnet",
                call_evidence: None,
            });
        }
        if let Some(actions) = actions {
            semantic_edges.extend(actions.into_iter().map(|action| SemanticEdge {
                relation: SemanticRelation::ControllerAction,
                subject: controller.clone(),
                object: EntityRef::new("dotnet", "Action", &action.name),
                layer: "dotnet",
                call_evidence: None,
            }));
        }
        Some(AspNetAnalysis {
            block: MetaBlock { lines },
            semantic_edges,
        })
    }
}

/// Extract the class name from a class declaration.
fn extract_class_name(source: &str) -> Option<String> {
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

/// Extract route template from [Route("...")] attribute.
fn extract_route(source: &str) -> Option<String> {
    if let Some(pos) = source.find("[Route(") {
        let start = pos + "[Route(".len();
        let rest = &source[start..];
        if let Some(quote_end) = rest.find('"') {
            let route = rest[..quote_end].to_string();
            return Some(route);
        }
    }
    None
}

fn extract_semantic_route(source: &str) -> Option<String> {
    let pos = source.find("[Route(")?;
    let rest = &source[pos + "[Route(".len()..];
    let quote_start = rest.find('"')? + 1;
    let quote_end = rest[quote_start..].find('"')?;
    Some(rest[quote_start..quote_start + quote_end].to_string())
}

/// Extract authorization policy from [Authorize(Policy = "...")] or [Authorize(Roles = "...")].
fn extract_authorize_policy(source: &str) -> Option<String> {
    if let Some(pos) = source.find("[Authorize(") {
        let start = pos + "[Authorize(".len();
        let rest = &source[start..];
        if let Some(policy_pos) = rest.find("Policy = \"") {
            let policy_start = policy_pos + "Policy = \"".len();
            if let Some(quote_end) = rest[policy_start..].find('"') {
                return Some(rest[policy_start..policy_start + quote_end].to_string());
            }
        }
        if let Some(roles_pos) = rest.find("Roles = \"") {
            let roles_start = roles_pos + "Roles = \"".len();
            if let Some(quote_end) = rest[roles_start..].find('"') {
                return Some(rest[roles_start..roles_start + quote_end].to_string());
            }
        }
    }
    None
}

/// Extract action methods with HTTP verb attributes.
struct ActionFact {
    verb: &'static str,
    marker: bool,
    name: String,
    params: String,
    return_type: Option<String>,
}

fn extract_action_facts(class_source: &str) -> Vec<ActionFact> {
    let mut actions = Vec::new();

    let verb_patterns = [
        ("[HttpGet", "GET"),
        ("[HttpPost", "POST"),
        ("[HttpPut", "PUT"),
        ("[HttpDelete", "DELETE"),
        ("[HttpPatch", "PATCH"),
        ("[HttpHead", "HEAD"),
        ("[HttpOptions", "OPTIONS"),
    ];

    for (attr, verb) in &verb_patterns {
        let mut search_start = 0;
        let mut marker = true;
        while let Some(pos) = class_source[search_start..].find(attr) {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + attr.len()..];
            if let Some((name, params, return_type)) = extract_method_signature(rest) {
                actions.push(ActionFact {
                    verb,
                    marker,
                    name,
                    params,
                    return_type,
                });
            }
            marker = false;
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

/// Extract model/DTO references from action parameters.
fn extract_models(class_source: &str) -> Vec<String> {
    let mut models = Vec::new();

    if let Some(pos) = class_source.find("[FromBody]") {
        let rest = &class_source[pos + "[FromBody]".len()..];
        if let Some(paren_pos) = rest.find('(') {
            let param_type = &rest[..paren_pos];
            if let Some(type_name) = param_type.split_whitespace().last() {
                if !type_name.is_empty() && !is_primitive(type_name) {
                    models.push(super::markers::build_model_line(type_name));
                }
            }
        }
    }

    models
}

/// Check if a type is a primitive (skip these).
fn is_primitive(type_name: &str) -> bool {
    matches!(
        type_name,
        "int" | "long" | "string" | "bool" | "double" | "float" | "decimal" | "Guid" | "DateTime"
    )
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/aspnet.rs"]
mod tests;
