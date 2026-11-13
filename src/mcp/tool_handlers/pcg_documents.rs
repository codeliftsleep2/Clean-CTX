//! Author-declared Markdown catalog: automatic reconciliation, no metadata inference.
use crate::{mcp::McpState, pcg::documents::Request, protocol};
use serde_json::{Value, json};

fn component_reference_schema() -> Value {
    json!({
        "type":"array","minItems":1,
        "description":"Required only for acquire: one or multiple exact returned component references. Input order and duplicates receive explicit outcomes; all required qualifications accompany each selection. Historical references must be explicitly marked historical; current pins never substitute a new revision.",
        "items":{
            "type":"object","additionalProperties":false,
            "required":["id","revision","ordinal","role","path","sourceSha256","start","end","contentSha256","selector"],
            "properties":{
                "id":{"type":"string","description":"Exact knowledge identity."},
                "revision":{"type":"string","description":"Exact retained knowledge revision."},
                "ordinal":{"type":"integer","minimum":0,"description":"Exact source-owned component ordinal."},
                "role":{"type":"string","enum":crate::pcg::documents::knowledge::ROLES},
                "label":{"type":["string","null"],"description":"Exact source-owned label, null or absent if unlabeled."},
                "path":{"type":"string","description":"Exact returned associated source path."},
                "sourceSha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},
                "start":{"type":"integer","minimum":0},
                "end":{"type":"integer","minimum":0},
                "contentSha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},
                "selector":{"type":"string","description":"Exact source-owned selector; never a guessed heading."},
                "historical":{"type":"boolean","description":"Default false requires current revision. Explicit true resolves the pinned retained revision as history, not current authority."}
            }
        }
    })
}

pub(crate) fn knowledge_descriptor() -> Value {
    json!({"name":"pcg_knowledge_v1",
    "description":"Reference-first architectural knowledge: search/get/related/context return graph structure and exact source references, never component text. Explicit acquire resolves one or multiple selected component references with mandatory qualifications. Explicit source retrieves a full original artifact. Rust/SQLite compute graph and acquisition; agents interpret knowledge. No token budgets. Status is source-attributed, not verification or approval.",
    "inputSchema":{
        "type":"object","additionalProperties":false,"required":["workspaceRoot","mode"],
        "description":"search accepts optional discovery predicates and forbids id; get, related, context and source require id. acquire requires a nonempty references array and forbids id and all query selectors. Graph completeness does not mean content acquisition. Rust enforces mode restrictions, source revision pinning and configured structural bounds; unknown fields/invalid combinations are refused.",
        "properties":{
            "workspaceRoot":{
                "type":"string","minLength":1,
                "description":"Required actual workspace filesystem root, matching the configured document profile."
            },
            "mode":{
                "type":"string","enum":["search","get","related","context","source","acquire"],
                "description":"Required operation: search discovers references; get returns unit/component descriptors; related returns typed edges; context returns the complete declared graph; acquire explicitly retrieves selected component bytes; source explicitly retrieves a full artifact."
            },
            "id":{
                "type":"string","minLength":1,
                "description":"Unit identity, required for get, related, context and source; forbidden for search and acquire. Use returned references."
            },
            "text":{
                "type":"string","minLength":1,
                "description":"Optional search-only SQLite FTS5 expression over declared knowledge components and explicit identity/title metadata; no whole-document inference."
            },
            "track":{
                "type":"string","minLength":1,
                "description":"Optional exact source-declared track filter; search only."
            },
            "kind":{
                "type":"string","enum":crate::pcg::documents::knowledge::KINDS,
                "description":"Optional exact source-declared kind filter; search only."
            },
            "status":{
                "type":"string","enum":crate::pcg::documents::knowledge::STATUSES,
                "description":"Optional exact source-declared lifecycle status filter; search only. Status is not executed verification."
            },
            "role":{
                "type":"string","enum":crate::pcg::documents::knowledge::ROLES,
                "description":"Optional exact component role predicate for search or get only. Required qualifications accompany a matching get selection."
            },
            "label":{
                "type":"string","minLength":1,
                "description":"Optional exact source-owned component label for search or get only; requires role."
            },
            "edgeKind":{
                "type":"string","enum":crate::pcg::documents::knowledge::RELATIONS,
                "description":"Optional exact typed relationship filter; related only."
            },
            "direction":{
                "type":"string","enum":["incoming","outgoing","both"],
                "description":"Optional related-only relationship direction relative to id; omitted means both."
            },
            "path":{
                "type":"string","minLength":1,
                "description":"Optional source-only path selecting an associated original artifact; use a returned source reference. Omitted selects the unit's source."
            },
            "sourceSha256":{
                "type":"string","pattern":"^[0-9a-f]{64}$",
                "description":"Optional source-only exact revision pin; a hash mismatch causes explicit refusal. Use the returned source reference."
            },
            "revision":{
                "type":"string","minLength":1,
                "description":"Optional source-only exact knowledge revision pin. Required with historical true; never silently substituted."
            },
            "historical":{
                "type":"boolean",
                "description":"source only: explicit retained artifact retrieval rather than current authority, requires revision. Default false."
            },
            "maxNodes":{
                "type":"integer","minimum":1,
                "description":"Optional context-only structural node bound, no greater than the configured profile limit."
            },
            "maxEdges":{
                "type":"integer","minimum":1,
                "description":"Optional context-only structural relationship bound, no greater than the configured profile limit."
            },
            "references":component_reference_schema()
        }
    }})
}

pub(crate) fn handle_knowledge(id: &Value, params: &Value, state: &McpState) {
    let result = (|| {
        let value = params
            .get("arguments")
            .ok_or(crate::pcg::Error("invalid_request"))?;
        if value
            .as_object()
            .is_none_or(|fields| fields.values().any(Value::is_null))
        {
            return Err(crate::pcg::Error("invalid_request"));
        }
        let request: crate::pcg::documents::knowledge::Request =
            serde_json::from_value(value.clone())
                .map_err(|_| crate::pcg::Error("invalid_request"))?;
        state
            .pcg
            .documents
            .as_ref()
            .ok_or(crate::pcg::Error("document_disabled"))?
            .knowledge(&request)
    })();
    match result {
        Ok(payload) => protocol::send_response(
            &json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":payload.to_string()}],"isError":false}}),
        ),
        Err(error) => protocol::send_response(
            &json!({"jsonrpc":"2.0","id":id,"error":{"code":if ["invalid_request","invalid_id","scope_refused","request_limits"].contains(&error.0) {-32602}else{-32603},"message":format!("pcg/{}",error.0)}}),
        ),
    }
}

pub(crate) fn descriptor() -> Value {
    json!({"name":"pcg_documents_v1",
        "description":"Discover author-declared architectural documents. Automatically validates/reconciles configured Markdown roots into SQLite before search/get/history/related/context. Exact IDs, source-declared tracks and SQLite FTS5; metadata/status/verification remain source-attributed, never authenticated approval. Does not rewrite source or import a corpus manifest.",
        "inputSchema":{"type":"object","additionalProperties":false,
            "required":["workspaceRoot","mode","tokenizer"],
            "properties":{"workspaceRoot":{"type":"string","minLength":1},
                "mode":{"enum":["search","get","history","related","context"]},
                "responseFormat":{"enum":["inline_v1","source_refs_v1"],"description":"Default inline_v1 preserves version 1 bytes. Explicit source_refs_v1 is context-only and returns version 2 with exact shared sources and per-node references."},
                "id":{"type":"string","minLength":1},"track":{"type":"string","minLength":1},
                "text":{"type":"string","minLength":1,"description":"SQLite FTS5 expression, bound as data"},
                "tokenizer":{"enum":["cl100k","o200k"]},"tokenBudget":{"type":"integer","minimum":0,"description":"Deprecated compatibility hint; document results are not token-budget limited."},"maxNodes":{"type":"integer","minimum":1},"maxEdges":{"type":"integer","minimum":1}},
            "allOf":[{"if":{"required":["responseFormat"],"properties":{"responseFormat":{"const":"source_refs_v1"}}},"then":{"properties":{"mode":{"const":"context"}}}}],
            "oneOf":[{"properties":{"mode":{"const":"search"}},"not":{"required":["id"]}},
                {"properties":{"mode":{"enum":["get","history","related","context"]}},"required":["id"],
                    "not":{"anyOf":[{"required":["track"]},{"required":["text"]}]}}]}})
}

pub(crate) fn handle(id: &Value, params: &Value, state: &McpState) {
    let result = (|| {
        let value = params
            .get("arguments")
            .ok_or(crate::pcg::Error("invalid_request"))?;
        for name in [
            "id",
            "track",
            "text",
            "maxNodes",
            "maxEdges",
            "responseFormat",
        ] {
            if value.get(name).is_some_and(Value::is_null) {
                return Err(crate::pcg::Error("invalid_request"));
            }
        }
        let request: Request = serde_json::from_value(value.clone())
            .map_err(|_| crate::pcg::Error("invalid_request"))?;
        state
            .pcg
            .documents
            .as_ref()
            .ok_or(crate::pcg::Error("document_disabled"))?
            .query(&request)
    })();
    match result {
        Ok(response) => {
            protocol::send_pcg_result(id, &response);
        }
        Err(error) => {
            let code = match error.0 {
                "invalid_request"
                | "invalid_id"
                | "scope_refused"
                | "request_limits"
                | "unsupported_tokenizer" => -32602,
                _ => -32603,
            };
            protocol::send_response(&json!({"jsonrpc":"2.0","id":id,"error":{"code":code,
                "message":format!("pcg/{}",error.0)}}));
        }
    }
}

#[cfg(test)]
#[path = "../../tests/mcp/pcg_documents.rs"]
mod tests;
