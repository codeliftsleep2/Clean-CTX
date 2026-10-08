//! Standard MCP effect annotations for the public Clean-CTX tool catalog.
//!
//! These are client hints, not authorization. Trusted-root validation,
//! structural ownership, persistence transactions, and handler checks remain
//! the enforcement boundaries. Internal caches and session projections do not
//! make an otherwise source-read/query tool externally mutating.

use serde_json::{Value, json};

#[derive(Clone, Copy)]
struct ToolEffects {
    read_only: bool,
    destructive: bool,
    idempotent: bool,
}

const READ_ONLY: ToolEffects = ToolEffects {
    read_only: true,
    destructive: false,
    idempotent: true,
};

const MUTATING: ToolEffects = ToolEffects {
    read_only: false,
    destructive: true,
    idempotent: false,
};

fn effects(name: &str) -> ToolEffects {
    match name {
        "cargo_check" => ToolEffects {
            read_only: false,
            destructive: false,
            idempotent: false,
        },
        // Source, graph, history, and diagnostic reads. Internal cache,
        // baseline, or session projection updates are not external mutations.
        "compress_code_context"
        | "diff_code_context"
        | "delta_code_context"
        | "provide_code_context"
        | "restore_context"
        | "context_history"
        | "list_sessions"
        | "inspect_legacy_fallbacks"
        | "replay_history"
        | "context_stats"
        | "diff_commits"
        | "workspace_query"
        | "graph_search"
        | "graph_query"
        | "graph_trace"
        | "get_architecture"
        | "get_cbm_status"
        | "list_projects" => READ_ONLY,

        // Repeating the same explicit save has the same durable semantic
        // effect and does not delete source or historical state.
        "save_context" => ToolEffects {
            read_only: false,
            destructive: false,
            idempotent: true,
        },

        // Repeating these operations has no additional intended effect, but
        // they delete semantic state/history or replace the external graph
        // index and therefore retain the conservative destructive hint.
        "delete_context" | "purge_old_deltas" | "index_repository" => ToolEffects {
            read_only: false,
            destructive: true,
            idempotent: true,
        },

        // Exact edits and delta acknowledgements consume current authority;
        // blind retries are unsafe. The generic proxy is also conservative
        // because it can dispatch index_repository as well as read operations.
        "apply_delta" | "apply_edit" | "cbm_proxy" => MUTATING,

        _ => panic!("public MCP tool `{name}` has no effect classification"),
    }
}

pub(super) fn inject(mut tools: Vec<Value>) -> Vec<Value> {
    for tool in &mut tools {
        let name = tool["name"]
            .as_str()
            .expect("public MCP tool definition must have a name");
        let effects = effects(name);
        let open_world = name == "cargo_check";
        tool.as_object_mut()
            .expect("public MCP tool definition must be an object")
            .insert(
                "annotations".into(),
                json!({
                    "readOnlyHint": effects.read_only,
                    "destructiveHint": effects.destructive,
                    "idempotentHint": effects.idempotent,
                    "openWorldHint": open_world,
                }),
            );
    }
    tools
}
