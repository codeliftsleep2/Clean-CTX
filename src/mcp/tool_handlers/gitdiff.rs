// src/mcp/tool_handlers/gitdiff.rs
//
// Handler for the `diff_commits` MCP tool (multi-file git-ref diff).
//
// D2 migration: moved verbatim from the inline dispatch arm in
// `src/mcp/tools.rs` into the canonical registry handler path.
// Parameter validation, error envelopes, response content, `_meta`,
// and cache-breakpoint injection are preserved exactly.

use crate::mcp::McpState;
use crate::mcp::cache_hints::{compute_workspace_breaker, inject_cache_breakpoints};
use crate::protocol::send_response;
use serde_json::Value;

use super::super::tool_helpers::resolve_file_path_checked;
use super::super::tools::parse_fidelity_arg;

pub(crate) fn handle_diff_commits(id: &Value, params: &Value, state: &McpState) {
    // Resolve the workspace root (XPIA mitigation). Defaults to CWD.
    // Pass the caller-supplied workspaceRoot through so the boundary
    // check honors it (multi-repo support) instead of pinning to CWD.
    let root_arg = crate::mcp::tool_helpers::arg_str(params, "workspaceRoot");
    let root = match resolve_file_path_checked(
        root_arg.unwrap_or("."),
        root_arg,
        &state.config.additional_roots,
    ) {
        Ok(p) => p,
        Err(msg) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                msg,
                None,
            ));
            return;
        }
    };

    // fromRef is required and strictly validated (flag-injection guard).
    let from_ref = match params["arguments"]["fromRef"].as_str() {
        Some(r) => r,
        None => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32602,
                "missing required argument: fromRef",
                None,
            ));
            return;
        }
    };
    if let Err(e) = crate::gitdiff::validate_ref(from_ref) {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32602,
            e.to_string(),
            None,
        ));
        return;
    }

    // toRef is optional; if present, validate it too.
    let to_ref = params["arguments"]["toRef"].as_str();
    if let Some(t) = to_ref
        && let Err(e) = crate::gitdiff::validate_ref(t)
    {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32602,
            e.to_string(),
            None,
        ));
        return;
    }

    // Fail-closed: the root must be a git repository.
    if !crate::gitdiff::is_git_repo(&root) {
        send_response(&crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            -32603,
            format!("not a git repository: {root}"),
            None,
        ));
        return;
    }

    let fidelity = match parse_fidelity_arg(id, params, &state.config) {
        Ok(f) => f,
        Err(()) => return,
    };

    // Resource limits from config: cap changed-file count and per-file size.
    let max_files = Some(state.config.resource_limits.max_workspace_files);
    let max_file_size = Some(state.config.resource_limits.max_file_size_bytes);

    match crate::gitdiff::gitdiff_workspace(
        &root,
        from_ref,
        to_ref,
        fidelity,
        max_files,
        max_file_size,
    ) {
        Ok(summary) => {
            let mut response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": summary.manifest }],
                    "_meta": {
                        "fileCount": summary.file_count,
                        "counts": {
                            "added": summary.counts.0,
                            "deleted": summary.counts.1,
                            "modified": summary.counts.2,
                            "renamed": summary.counts.3,
                        },
                        "skipped": summary.skipped,
                    }
                }
            });
            if state.config.cache.enabled {
                let ttl = state.config.cache.baseline_ttl.clone();
                let breaker = compute_workspace_breaker(std::slice::from_ref(&summary.manifest));
                let tok_box =
                    crate::tokenizer::create_tokenizer(crate::tokenizer::resolve_tokenizer_kind(
                        None,
                        Some(&state.config.tokenizer.to_string()),
                    ))
                    .ok();
                let tok_ref: Option<&dyn crate::tokenizer::Tokenizer> = tok_box.as_deref();
                if let Some(result_obj) = response.get_mut("result") {
                    inject_cache_breakpoints(
                        result_obj, state, "baseline", &ttl, &breaker, tok_ref,
                    );
                }
            }
            send_response(&response);
        }
        Err(e) => {
            send_response(&crate::mcp::tool_helpers::jsonrpc_error(
                id.clone(),
                -32603,
                e.to_string(),
                None,
            ));
        }
    }
}
