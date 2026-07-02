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

use super::markers::{build_action_line, build_api_controller_line, build_auth_line, build_controller_line};
use crate::dotnet_meta::MetaBlock;
use crate::compression::Fidelity;

/// Extract ASP.NET Core markers from a single class capture.
///
/// Returns `None` when the class is not an ASP.NET Core construct.
pub fn extract_aspnet(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
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
    if fidelity != Fidelity::Low {
        lines.extend(extract_actions(class_source));
    }

    // Extract models/DTOs
    if fidelity == Fidelity::High {
        lines.extend(extract_models(class_source));
    }

    if lines.is_empty() {
        None
    } else {
        Some(MetaBlock { lines })
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
fn extract_actions(class_source: &str) -> Vec<String> {
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
        if let Some(pos) = class_source.find(attr) {
            let rest = &class_source[pos + attr.len()..];
            if let Some(method_info) = extract_method_signature(rest) {
                actions.push(build_action_line(
                    verb,
                    &method_info.0,
                    &method_info.1,
                    method_info.2.as_deref(),
                ));
            }
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