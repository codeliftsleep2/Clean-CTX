// src/dotnet_meta/signalr.rs
//
// SignalR extraction — Hubs, methods, clients, groups, streaming.
//
// Detects:
// - `Hub<T>` classes (with optional client interface)
// - Hub methods with `[HubMethodName]` attribute
// - Strongly-typed client interfaces (`IClientProxy`)
// - Group management (`Groups.AddToGroupAsync`, `Groups.RemoveFromGroupAsync`)
// - User targeting (`Clients.User(userId)`)
// - Streaming endpoints (`ChannelReader<T>`, `IAsyncEnumerable<T>`)
// - Connection lifecycle (`OnConnectedAsync`, `OnDisconnectedAsync`)

use super::markers::{
    build_connection_line, build_group_line, build_hub_line, build_hub_method_line,
    build_stream_line, build_user_line,
};
use crate::compression::Fidelity;
use crate::dotnet_meta::MetaBlock;

/// Extract SignalR markers from a single class capture.
///
/// Returns `None` when the class is not a SignalR Hub.
pub fn extract_signalr(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    let mut lines = Vec::new();

    // Detect Hub class
    let is_hub = class_source.contains(": Hub<") || class_source.contains(": Hub");

    if !is_hub {
        return None;
    }

    // Extract class name
    let class_name = extract_class_name(class_source)?;

    // Extract client interface (if Hub<T> where T is the client interface)
    let client_interface = extract_client_interface(class_source);

    // Emit Hub marker
    lines.push(build_hub_line(&class_name, client_interface.as_deref()));

    // Extract hub methods
    if fidelity != Fidelity::Low {
        lines.extend(extract_hub_methods(class_source, fidelity));
    }

    // Extract connection lifecycle events
    if fidelity == Fidelity::High {
        lines.extend(extract_connection_events(class_source));
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

/// Extract client interface from `Hub<T>` generic parameter.
fn extract_client_interface(source: &str) -> Option<String> {
    // Match `: Hub<IClientInterface>` or `: Hub<IClientInterface>`
    if let Some(pos) = source.find(": Hub<") {
        let start = pos + ": Hub<".len();
        let rest = &source[start..];

        if let Some(generic_end) = rest.find('>') {
            let interface = rest[..generic_end].trim().to_string();
            // Skip if it's just "Hub" (non-generic)
            if interface != "Hub" && !interface.is_empty() {
                return Some(interface);
            }
        }
    }

    None
}

/// Extract hub methods with client invocations.
#[allow(unused_variables)]
fn extract_hub_methods(class_source: &str, fidelity: Fidelity) -> Vec<String> {
    let mut methods = Vec::new();

    // Look for public methods (hub methods)
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("public ") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "public ".len()..];

        // Find method signature
        if let Some(paren_pos) = rest.find('(') {
            let signature = &rest[..paren_pos];
            let params_str = &rest[paren_pos + 1..];

            // Find closing paren
            if let Some(close_paren) = params_str.find(')') {
                let params = params_str[..close_paren].to_string();

                // Extract method name (last word before '(')
                let method_name = signature
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .to_string();

                // Skip if it's a lifecycle method (handled separately)
                if method_name == "OnConnectedAsync" || method_name == "OnDisconnectedAsync" {
                    search_start = actual_pos + 1;
                    continue;
                }

                // Extract return type
                let return_type = if signature.contains(' ') {
                    let parts: Vec<&str> = signature.trim().rsplitn(2, ' ').collect();
                    if parts.len() == 2 {
                        Some(parts[0].trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Detect client invocation target
                let target = detect_client_target(class_source, &method_name);

                // Check if streaming
                let is_streaming = return_type
                    .as_ref()
                    .map(|rt| rt.contains("ChannelReader") || rt.contains("IAsyncEnumerable"))
                    .unwrap_or(false);

                if is_streaming {
                    let stream_type = return_type.unwrap_or_default();
                    methods.push(build_stream_line(&method_name, &stream_type));
                } else if let Some(client_target) = target {
                    methods.push(build_hub_method_line(&method_name, &params, &client_target));
                }

                // Extract group management
                methods.extend(extract_group_operations(rest, &method_name));

                // Extract user targeting
                methods.extend(extract_user_operations(rest, &method_name));
            }
        }

        search_start = actual_pos + 1;
    }

    methods
}

/// Detect client invocation target (e.g., `Clients.All`, `Clients.Caller`, `Clients.User(id)`).
#[allow(unused_variables)]
fn detect_client_target(method_body: &str, method_name: &str) -> Option<String> {
    // Look for `Clients.` patterns
    if let Some(pos) = method_body.find("Clients.") {
        let rest = &method_body[pos + "Clients.".len()..];

        // Extract until ';' or newline
        let end = rest
            .find(|c: char| [';', '\n', '\r'].contains(&c))
            .unwrap_or(rest.len());
        let target = rest[..end].trim().to_string();

        if !target.is_empty() {
            return Some(target);
        }
    }

    None
}

/// Extract group management operations within a method.
#[allow(unused_variables)]
fn extract_group_operations(method_body: &str, method_name: &str) -> Vec<String> {
    let mut groups = Vec::new();

    // Look for `Groups.AddToGroupAsync` or `Groups.RemoveFromGroupAsync`
    if method_body.contains("Groups.AddToGroupAsync")
        || method_body.contains("Groups.RemoveFromGroupAsync")
    {
        // Extract group name (simplified)
        if let Some(pos) = method_body.find("Groups.") {
            let rest = &method_body[pos + "Groups.".len()..];
            if let Some(semicolon) = rest.find(';') {
                let operation = rest[..semicolon].trim().to_string();
                if let Some(group_name) = extract_group_name(&operation) {
                    groups.push(build_group_line(&group_name));
                }
            }
        }
    }

    groups
}

/// Extract group name from a Groups operation.
fn extract_group_name(operation: &str) -> Option<String> {
    // Pattern: AddToGroupAsync(connectionId, "groupName")
    if let Some(pos) = operation.find("\"") {
        let rest = &operation[pos + 1..];
        if let Some(quote_end) = rest.find('"') {
            return Some(rest[..quote_end].to_string());
        }
    }

    None
}

/// Extract user targeting operations within a method.
#[allow(unused_variables)]
fn extract_user_operations(method_body: &str, method_name: &str) -> Vec<String> {
    let mut users = Vec::new();

    // Look for `Clients.User(userId)`
    if method_body.contains("Clients.User(") {
        if let Some(pos) = method_body.find("Clients.User(") {
            let rest = &method_body[pos + "Clients.User(".len()..];
            if let Some(close_paren) = rest.find(')') {
                let user_id = rest[..close_paren].trim().to_string();
                users.push(build_user_line(&user_id));
            }
        }
    }

    users
}

/// Extract connection lifecycle events.
fn extract_connection_events(class_source: &str) -> Vec<String> {
    let mut events = Vec::new();

    if class_source.contains("OnConnectedAsync") {
        events.push(build_connection_line("OnConnectedAsync"));
    }

    if class_source.contains("OnDisconnectedAsync") {
        events.push(build_connection_line("OnDisconnectedAsync"));
    }

    events
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/signalr.rs"]
mod tests;
