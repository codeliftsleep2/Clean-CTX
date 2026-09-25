//! Owner-aware file-local call inspection over a fresh canonical IR candidate.
//!
//! This is deliberately separate from `WorkspaceIndex`: Model C identity is
//! correct for cross-file graph queries but does not retain a method's typed
//! owner or overload identity. `calls_in_file` stays at the file/canonical-IR
//! boundary and never invents resolved callee identity.

use super::{query_scope, send_scope_rejection};
use crate::compression::Fidelity;
use crate::ir::hierarchical::{HierarchicalCall, HierarchicalIR, MethodNode};
use crate::mcp::McpState;
use crate::protocol::send_response;
use serde_json::{Value, json};

#[derive(Debug)]
struct CallRequest {
    file_path: String,
    owner_kind: String,
    owner_name: String,
    method_name: String,
    parameters: Option<Vec<String>>,
    return_type: Option<String>,
}

pub(super) fn handle_calls_in_file(id: &Value, args: &Value, state: &McpState) {
    let request = match parse_request(args) {
        Ok(request) => request,
        Err(message) => return send_invalid_params(id, message),
    };
    let scope = match query_scope(state, args) {
        Ok(scope) => scope,
        Err(message) => return send_scope_rejection(id, message),
    };
    let resolved_path = match crate::mcp::tool_helpers::resolve_file_path_checked(
        &request.file_path,
        args["workspaceRoot"].as_str(),
        &state.config.additional_roots,
    ) {
        Ok(path) => path,
        Err(message) => return send_invalid_params(id, message),
    };
    let canonical_path = crate::dictionary::path::canonical_identity_key(&resolved_path);
    if scope
        .as_ref()
        .is_some_and(|scope| scope.has_narrowing() && !scope.admits(&canonical_path))
    {
        return send_answer(id, args, state, &request, &resolved_path, Vec::new());
    }

    // Candidate compilation reads current source and produces canonical facts
    // without installing an alias, IR baseline, semantic edges, or index state.
    let (compiled, _, _) = match crate::mcp::tool_helpers::compile_file_ir_candidate(
        &resolved_path,
        Fidelity::High,
        state,
    ) {
        Ok(candidate) => candidate,
        Err(error) => {
            send_response(&crate::error::to_jsonrpc_error(id, &error));
            return;
        }
    };
    let hierarchy = match crate::ir::hierarchical::try_ir_to_hierarchical(&compiled) {
        Ok(hierarchy) => hierarchy,
        Err(error) => {
            send_response(&crate::mcp::tool_handlers::core::projection_error_response(
                id, &error,
            ));
            return;
        }
    };
    let overloads = match matching_overloads(&hierarchy, &request) {
        Ok(overloads) => overloads,
        Err(message) => return send_invalid_params(id, message),
    };
    send_answer(id, args, state, &request, &resolved_path, overloads);
}

fn parse_request(args: &Value) -> Result<CallRequest, String> {
    required_nested_str(args, None, "workspaceRoot")?;
    let file_path = required_nested_str(args, None, "filePath")?;
    let owner_kind = required_nested_str(args, Some("owner"), "kind")?;
    if !matches!(owner_kind.as_str(), "class" | "interface") {
        return Err("'owner.kind' must be 'class' or 'interface'".to_string());
    }
    let owner_name = required_nested_str(args, Some("owner"), "name")?;
    let method_name = required_nested_str(args, Some("method"), "name")?;
    let method = args
        .get("method")
        .and_then(Value::as_object)
        .ok_or_else(|| "Missing required object: 'method' for calls_in_file".to_string())?;
    let parameters = match method.get("parameters") {
        None => None,
        Some(value) => {
            let values = value
                .as_array()
                .ok_or_else(|| "'method.parameters' must be an array of strings".to_string())?;
            Some(
                values
                    .iter()
                    .map(|value| {
                        value.as_str().map(str::to_string).ok_or_else(|| {
                            "'method.parameters' must contain only strings".to_string()
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
        }
    };
    let return_type = match method.get("return_type") {
        None => None,
        Some(value) => Some(
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "'method.return_type' must be a non-empty string".to_string())?
                .to_string(),
        ),
    };
    Ok(CallRequest {
        file_path,
        owner_kind,
        owner_name,
        method_name,
        parameters,
        return_type,
    })
}

fn required_nested_str(args: &Value, object: Option<&str>, field: &str) -> Result<String, String> {
    let value = match object {
        Some(object) => args
            .get(object)
            .and_then(Value::as_object)
            .and_then(|value| value.get(field)),
        None => args.get(field),
    };
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            let name =
                object.map_or_else(|| field.to_string(), |object| format!("{object}.{field}"));
            format!("Missing required string: '{name}' for calls_in_file")
        })
}

fn matching_overloads(
    hierarchy: &HierarchicalIR,
    request: &CallRequest,
) -> Result<Vec<Value>, String> {
    let owners: Vec<&[MethodNode]> = if request.owner_kind == "class" {
        hierarchy
            .classes
            .iter()
            .filter(|owner| owner.name == request.owner_name)
            .map(|owner| owner.methods.as_slice())
            .collect()
    } else {
        hierarchy
            .interfaces
            .iter()
            .filter(|owner| owner.name == request.owner_name)
            .map(|owner| owner.methods.as_slice())
            .collect()
    };
    let methods = match owners.as_slice() {
        [] => return Ok(Vec::new()),
        [methods] => *methods,
        _ => {
            return Err(format!(
                "calls_in_file owner is ambiguous: {} '{}'",
                request.owner_kind, request.owner_name
            ));
        }
    };

    Ok(methods
        .iter()
        .filter(|method| method.name == request.method_name)
        .enumerate()
        .filter_map(|(overload_occurrence, method)| {
            let parameters = visible_parameters(method);
            if request
                .parameters
                .as_ref()
                .is_some_and(|selector| selector != &parameters)
                || request
                    .return_type
                    .as_ref()
                    .is_some_and(|selector| method.return_type.as_ref() != Some(selector))
            {
                return None;
            }
            let calls = calls_for_method(&hierarchy.calls, &method.id);
            Some(json!({
                "overload_occurrence": overload_occurrence,
                "parameters": parameters,
                "return_type": method.return_type,
                "calls": calls,
            }))
        })
        .collect())
}

fn visible_parameters(method: &MethodNode) -> Vec<String> {
    method
        .params
        .iter()
        .map(|parameter| match (parameter.get(1), parameter.get(2)) {
            (Some(kind), Some(written)) if kind == crate::ir::opcodes::TYPE_VOID => written.clone(),
            (Some(kind), _) => kind.clone(),
            _ => String::new(),
        })
        .collect()
}

fn calls_for_method(calls: &[HierarchicalCall], method_id: &str) -> Vec<Value> {
    calls
        .iter()
        .filter(|call| call.caller == method_id)
        .enumerate()
        .map(|(occurrence, call)| {
            json!({
                "occurrence": occurrence,
                "callee_written": call.callee,
                "explicit_argument_count": call.explicit_arg_count,
                "has_spread": call.has_spread,
            })
        })
        .collect()
}

fn send_answer(
    id: &Value,
    args: &Value,
    state: &McpState,
    request: &CallRequest,
    resolved_path: &str,
    overloads: Vec<Value>,
) {
    let count = overloads
        .iter()
        .filter_map(|overload| overload["calls"].as_array())
        .map(Vec::len)
        .sum::<usize>();
    let structured = json!({
        "file": resolved_path,
        "owner": { "kind": request.owner_kind, "name": request.owner_name },
        "method": request.method_name,
        "overload_count": overloads.len(),
        "overloads": overloads,
        "count": count,
    });
    let content = super::content::render(
        "calls_in_file",
        args,
        &structured,
        &state.config.additional_roots,
    );
    send_response(&json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": content }],
            "structuredContent": structured,
        }
    }));
}

fn send_invalid_params(id: &Value, message: String) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32602,
        message,
        None,
    ));
}
