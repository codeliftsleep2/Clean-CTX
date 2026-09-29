// src/mcp/tools.rs
//
// Public tool catalog plus compatibility re-exports for call dispatch.

use crate::cbm;
use serde_json::Value;

#[cfg(test)]
pub use super::tool_dispatch::setup_handler_registry_for_tests;
pub(crate) use super::tool_dispatch::{
    dispatch_tools_call, parse_fidelity_arg, parse_tokenizer_arg,
};
#[cfg(test)]
pub(crate) use super::tool_dispatch::{inline_tool_names, resolve_fidelity};

#[cfg(test)]
pub(crate) use super::tool_helpers::diff_code_context_handler;

/// Compute the list of supported languages based on enabled Cargo features.
/// This surfaces to clients which file extensions the binary can actually
/// process, avoiding "unsupported extension" errors for unbuilt grammars.
fn supported_languages() -> Vec<&'static str> {
    let mut langs = Vec::new();
    if cfg!(feature = "typescript") {
        langs.push("typescript");
    }
    if cfg!(feature = "csharp") {
        langs.push("csharp");
    }
    if cfg!(feature = "rust") {
        langs.push("rust");
    }
    if cfg!(feature = "java") {
        langs.push("java");
    }
    langs
}

/// Attach parser capability metadata only where enabled languages affect
/// whether the tool can process its requested source input.
fn inject_supported_languages(mut tools: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let supported = supported_languages();
    for tool in &mut tools {
        let source_processing = tool
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| {
                matches!(
                    name,
                    "compress_code_context"
                        | "diff_code_context"
                        | "delta_code_context"
                        | "provide_code_context"
                        | "apply_edit"
                        | "diff_commits"
                        | "workspace_query"
                )
            });
        if source_processing && let Some(obj) = tool.as_object_mut() {
            obj.insert(
                "supportedLanguages".to_string(),
                serde_json::json!(supported),
            );
        }
    }
    tools
}

/// Build the nested discovery contract separately so the broad
/// `workspace_query` definition remains a shallow macro expansion.
fn workspace_query_discovery_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "description": "Optional. Present ONLY when discovery deviated from its expected path (healthy CBM, completed across every configured root, ready projects, no candidates); absent when nothing noteworthy happened. Every field is omitted while it holds its expected value, and no field restates another.",
        "properties": {
            "provider": {
                "type": "string",
                "enum": ["filesystem", "cbm_and_filesystem", "none"],
                "description": "Present only when a provider other than CBM supplied this query's coverage."
            },
            "status": {
                "type": "string",
                "enum": ["partial", "unavailable"],
                "description": "Present only when discovery was not complete: 'partial' = it ran but did not cover every configured root; 'unavailable' = nothing could run, so the answer rests only on what the index already held."
            },
            "fallback_reason": {
                "type": "string",
                "enum": ["cbm_unavailable", "cbm_discovery_failed", "cbm_partial_failure", "cbm_scope_unavailable", "filesystem_unavailable"],
                "description": "Present only when filesystem fallback was engaged for at least one root; its presence IS that fact."
            },
            "discovered": { "type": "integer", "description": "Present only when > 0 - candidate file paths discovered by this query." },
            "compiled": { "type": "integer", "description": "Present only when > 0 - unique, previously-unindexed candidate files compiled into the WorkspaceIndex by this query." },
            "projects": {
                "type": "array",
                "description": "Present only when a configured project's coverage was exceptional; healthy searched/ready projects are omitted and do not consume the diagnostic bound.",
                "items": {
                    "type": "object",
                    "properties": {
                        "project": { "type": "string" },
                        "status": { "type": "string", "enum": ["searched", "search_failed", "skipped"] },
                        "readiness": { "type": "string", "enum": ["still_indexing", "failed"], "description": "Only for an exceptional 'searched' entry: the project was not ready, so its contribution may be incomplete." },
                        "reason": { "type": "string", "enum": ["cbm_unavailable", "additional_root_not_registered"], "description": "Only when it adds a distinction the status does not already carry." }
                    },
                    "required": ["project", "status"]
                }
            },
            "projects_truncated": { "type": "integer", "description": "Present only when > 0 - exceptional project entries dropped by the diagnostic bound." }
        }
    })
}

/// Build cycle output fields separately so `workspace_query` remains below
/// `serde_json::json!`'s default macro-recursion limit.
fn workspace_query_cycle_schema() -> Value {
    serde_json::json!({
        "type": "array",
        "description": "One deterministic ordered dependency-cycle witness (has_cycle).",
        "items": { "type": "object" }
    })
}

pub(crate) fn tool_list() -> Vec<serde_json::Value> {
    let tools = vec![
        serde_json::json!({
            "name": "compress_code_context",
            "description": "High-speed local AST compilation, hash-caching, and variable mapping tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts, .cs, .rs, or .java file." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low' (max compression, ~85% reduction), 'medium' (balanced, preserves fields/async/markers, ~70-80%), 'high' (minimal compression, preserves most semantic depth, ~50-60%), 'edit' (structural skeleton + verbatim method bodies for safe apply_edit operations), 'verbatim' (full raw source, zero compression). Default: 'low'." },
                    "encoding": { "type": "string", "description": "IR encoding format: 'named' (standard tuple with opcode strings), 'positional' (stripped opcode ~30% savings), or 'tagged' (positional with opcode preserved). Default: 'named'." },
                    "tokenizer": { "type": "string", "description": "Tokenizer backend for token counting: 'o200k' (GPT-4o, default), 'cl100k' (GPT-4), 'claude' (Anthropic), 'llama3' (Meta). Overrides config default." },
                    "workspaceRoot": { "type": "string", "description": "Strongly recommended. Explicit workspace root for reliable path resolution; defaults to CWD for backward compatibility." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "diff_code_context",
            "description": "AST-level diff compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts, .cs, or .rs file." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: 'low'." },
                    "workspaceRoot": { "type": "string", "description": "Strongly recommended. Explicit workspace root for reliable path resolution; defaults to CWD for backward compatibility." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "delta_code_context",
            "description": "IR-level delta compression using versioned positional sequence edits.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: config default." },
                    "workspaceRoot": { "type": "string", "description": "Strongly recommended. Explicit workspace root for reliable path resolution; defaults to CWD for backward compatibility." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "apply_delta",
            "description": "Applies an IR delta. With durable persistence enabled, corrected positional deltas must match a server-generated pending semantic transition.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "delta": { "type": "object" },
                    "currentVersion": { "type": "integer" }
                },
                "required": ["delta", "currentVersion"]
            }
        }),
        serde_json::json!({
            "name": "provide_code_context",
            "description": "Provides complete current model-facing context for one file, or an ordered failure-isolated batch of up to 8 files. Batch requests share workspaceRoot and tokenizer while each item chooses intent, fidelity, and focusMethods. Structured delta transport is available separately through delta_code_context.",
            "inputSchema": {
                "type": "object",
                "properties": super::tool_schemas::provide_code_context_properties(),
                "oneOf": super::tool_schemas::provide_code_context_request_variants()
            },
            "outputSchema": {
                "type": "object",
                "properties": {
                    "batch": { "const": true },
                    "results": super::tool_schemas::provide_code_context_batch_results()
                },
                "required": ["batch", "results"]
            }
        }),
        serde_json::json!({
            "name": "restore_context",
            "description": "Restores a file's persisted canonical IR, delta history, fidelity, source hash, and complete semantic-edge state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "workspaceRoot": { "type": "string", "description": "Strongly recommended. Explicit workspace root for reliable path resolution; defaults to CWD for backward compatibility." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "apply_edit",
            "description": "Applies an atomic batch of structural edits to a previously tracked and owned source file; it never creates arbitrary files. Supported operations are `replace_body`, `delete`, `insert_after`, and `insert_before`. Edits resolve known structural units rather than generic text coordinates, validate expected unit text where required, preserve byte-exact source behavior, and return operation-specific absolute byte spans. Current source bytes must match the live and durable owned source identity; stale or externally diverged source fails structurally. Successful edits use the staged durable transaction before publishing live semantic state. Request edit/verbatim fidelity and a full method body when compact representation is insufficient for safe editing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to a previously-seen .ts, .cs, .rs, or .java file." },
                    "operations": {
                        "type": "array",
                        "description": "Structural operations applied atomically (all-or-nothing). Each: {\"type\":\"replace_body\",\"target\":\"Class.method\",\"expectedOldText\":\"{...}\",\"newText\":\"{...}\"} | {\"type\":\"delete\",\"target\":..., \"expectedOldText\":...} | {\"type\":\"insert_after\",\"anchor\":\"Class.method\",\"unitText\":...} | {\"type\":\"insert_before\",...}. expectedOldText must byte-match the text this session last delivered for that unit.",
                        "items": super::tool_schemas::apply_edit_operations()
                    },
                    "verify": { "type": "boolean", "description": "Optional. When true, echoes each replacement's new verbatim text back as a receipt. Default false." },
                    "workspaceRoot": { "type": "string", "description": "Strongly recommended. Explicit workspace root for reliable path resolution; defaults to CWD for backward compatibility." }
                },
                "required": ["filePath", "operations"]
            },
            "outputSchema": {
                "type": "object",
                "properties": {
                    "operations": {
                        "type": "array",
                        "description": "Per-operation outcomes in request order, measured against the NEW file. All operations apply atomically; this list equals the requested batch on success.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "kind": {
                                    "type": "string",
                                    "enum": ["replace_body", "insert_after", "insert_before", "delete"],
                                    "description": "Applied operation type."
                                },
                                "target": {
                                    "type": "string",
                                    "description": "Targeted unit name (replace_body/delete) or anchor unit (insert_after/insert_before)."
                                },
                                "startByte": {
                                    "type": "integer",
                                    "minimum": 0,
                                    "description": "Absolute start byte affected in the NEW file (insertions: insertion point)."
                                },
                                "endByte": {
                                    "type": "integer",
                                    "minimum": 0,
                                    "description": "Absolute end byte affected in the NEW file (insertions: same as startByte)."
                                },
                                "byteDelta": {
                                    "type": "integer",
                                    "description": "Signed size change contributed by this operation (new minus old; negative for deletes)."
                                },
                                "newText": {
                                    "type": "string",
                                    "description": "Verbatim new text receipt. Present only when verify=true and the operation carries new text (replace_body, insert_after, insert_before); never for delete."
                                }
                            },
                            "required": ["kind", "target", "startByte", "endByte", "byteDelta"]
                        }
                    }
                },
                "required": ["operations"]
            }
        }),
        serde_json::json!({
            "name": "context_history",
            "description": "Read compression history and savings for tracked files without creating, restoring, or mutating context ownership.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" }
                }
            }
        }),
        serde_json::json!({
            "name": "save_context",
            "description": "Explicitly save current in-memory context to the persistence DB.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "delete_context",
            "description": "Delete one file's persisted and session semantic context without modifying the source file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "list_sessions",
            "description": "Read committed persisted contexts from the DB without flushing pending lifecycle work — per-file rows with fidelity, token counts, delta count and last-update time.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "inspect_legacy_fallbacks",
            "description": "Read quarantined legacy fallback artifacts and report why they are incomplete; never recover, import, delete, rewrite, or mutate semantic/session/durable state.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        serde_json::json!({
            "name": "replay_history",
            "description": "Replay deltas from the DB for a file up to a specific edit sequence.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "targetSequence": { "type": "integer" },
                    "fidelity": { "type": "string" }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "purge_old_deltas",
            "description": "Purge old delta history from the persistence DB.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "days": { "type": "integer" },
                    "filePath": { "type": "string" }
                }
            }
        }),
        serde_json::json!({
            "name": "context_stats",
            "description": "Read the Clean-CTX dashboard without flushing persistence or mutating lifecycle state: token savings, compression stats, and session metrics.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string" },
                    "format": { "type": "string", "enum": ["text", "json"] }
                }
            }
        }),
        serde_json::json!({
            "name": "diff_commits",
            "description": "Diff an entire workspace between two git refs; emits per-file AST-level change-sets in one call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workspaceRoot": { "type": "string", "description": "Strongly recommended. Explicit workspace root resolved against the trusted root; defaults to CWD for backward compatibility." },
                    "fromRef": { "type": "string", "description": "Required. e.g. HEAD~1, main, abc123, v1.0. Strictly validated." },
                    "toRef": { "type": "string", "description": "Optional. Defaults to working tree (uncommitted changes)." },
                    "fidelity": { "type": "string", "enum": ["low", "medium", "high", "edit", "verbatim"], "description": "Compression fidelity: 'low', 'medium', 'high', 'edit', 'verbatim'. Default: config default." }
                },
                "required": ["fromRef"]
            }
        }),
        serde_json::json!({
            "name": "workspace_query",
            "description": "Run one workspace query or a heterogeneous batch of up to 32 queries with ordered, per-item outcomes. A batch shares workspaceRoot and withinPath, deduplicates equivalent hydration, and isolates item failures. Name-bearing cross-file operations use WorkspaceIndex plus registered hydration; has_cycle remains index-only, and calls_in_file remains a fresh unpublished canonical-file query.",
            "inputSchema": {
                "type": "object",
                "properties": super::tool_schemas::workspace_query_properties(),
                "oneOf": super::tool_schemas::workspace_query_request_variants()
            },
            "outputSchema": {
                "type": "object",
                "properties": {
                    "entities": {
                        "type": "array",
                        "description": "Matching entities (find_entities, entities_in_file).",
                        "items": { "type": "object" }
                    },
                    "edges": {
                        "type": "array",
                        "description": "Semantic edges (forward_edges, reverse_edges).",
                        "items": { "type": "object" }
                    },
                    "dependencies": {
                        "type": "array",
                        "description": "Transitive dependency entities (transitive_dependencies).",
                        "items": { "type": "object" }
                    },
                    "count": {
                        "type": "integer",
                        "description": "Number of result items."
                    },
                    "has_cycle": {
                        "type": "boolean",
                        "description": "Cycle detection result (has_cycle)."
                    },
                    "cycle": workspace_query_cycle_schema(),
                    "coverage": { "type": "object", "description": "Index-evidence coverage limits for has_cycle." },
                    "identity_model": { "type": "string", "description": "Semantic identity model used by has_cycle." },
                    "identity_ambiguous": { "type": "boolean", "description": "Whether a has_cycle witness identity has multiple admitted physical occurrences." },
                    "identity_ambiguities": { "type": "array", "description": "Ambiguous witness identities and their admitted occurrence files.", "items": { "type": "object" } },
                    "depth_used": {
                        "type": "integer",
                        "description": "Actual traversal depth used (transitive_dependencies)."
                    },
                    "resolved_identity": { "type": "object", "description": "Exact semantic identity used by forward_edges, reverse_edges, or transitive_dependencies." },
                    "file": { "type": "string", "description": "Resolved source file for calls_in_file." },
                    "owner": { "type": "object", "description": "Resolved typed owner for calls_in_file." },
                    "method": { "type": "string", "description": "Caller method name for calls_in_file." },
                    "overloads": { "type": "array", "description": "Matching overload declarations with ordered call occurrences.", "items": { "type": "object" } },
                    "overload_count": { "type": "integer", "description": "Number of matching overload declarations." },
                    "discovery": workspace_query_discovery_schema(),
                    "batch": { "type": "boolean", "const": true, "description": "Present only for a batch response." },
                    "results": super::tool_schemas::workspace_query_batch_results()
                }
            }
        }),
    ]
    .into_iter()
    .chain(cbm::cbm_tool_list())
    .collect();
    super::tool_annotations::inject(inject_supported_languages(tools))
}

#[cfg(test)]
#[path = "../tests/mcp/tools.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/mcp/tool_contracts.rs"]
mod tool_contracts_tests;
