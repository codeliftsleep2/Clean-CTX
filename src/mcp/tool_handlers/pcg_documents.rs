//! Author-declared Markdown catalog: automatic reconciliation, no metadata inference.
use crate::{mcp::McpState, pcg::documents::Request, protocol};
use serde_json::{Value, json};

pub(crate) fn knowledge_descriptor() -> Value {
    json!({"name":"pcg_knowledge_v1",
        "description":"Normal architectural knowledge workflow: discover independently bounded source-owned units, acquire exact components/rationale/qualifications, resolve typed relationships and complete declared authority contexts, or explicitly retrieve original source evidence. Rust/SQLite resolve boundaries and graph composition. No token budgets. Status remains source-attributed, not executed verification or authenticated approval.",
        "inputSchema":{"type":"object","additionalProperties":false,"required":["workspaceRoot","mode"],
            "properties":{"workspaceRoot":{"type":"string","minLength":1},"mode":{"enum":["search","get","related","context","source"]},
                "id":{"type":"string","minLength":1},"text":{"type":"string","minLength":1,"description":"SQLite FTS5 over declared knowledge components and explicit identity/title metadata; no whole-document inference."},
                "track":{"type":"string","minLength":1},"kind":{"enum":crate::pcg::documents::knowledge::KINDS},
                "status":{"enum":crate::pcg::documents::knowledge::STATUSES},"role":{"enum":crate::pcg::documents::knowledge::ROLES},"label":{"type":"string","minLength":1},
                "edgeKind":{"enum":crate::pcg::documents::knowledge::RELATIONS},"direction":{"enum":["incoming","outgoing","both"]},"path":{"type":"string","minLength":1},
                "sourceSha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},
                "maxNodes":{"type":"integer","minimum":1},"maxEdges":{"type":"integer","minimum":1}},
            "allOf":[
                {"if":{"anyOf":[{"required":["text"]},{"required":["track"]},{"required":["kind"]},{"required":["status"]}]},"then":{"properties":{"mode":{"const":"search"}}}},
                {"if":{"anyOf":[{"required":["role"]},{"required":["label"]}]},"then":{"properties":{"mode":{"enum":["search","get"]}}}},
                {"if":{"required":["label"]},"then":{"required":["role"]}},
                {"if":{"anyOf":[{"required":["edgeKind"]},{"required":["direction"]}]},"then":{"properties":{"mode":{"const":"related"}}}},
                {"if":{"anyOf":[{"required":["path"]},{"required":["sourceSha256"]}]},"then":{"properties":{"mode":{"const":"source"}}}},
                {"if":{"anyOf":[{"required":["maxNodes"]},{"required":["maxEdges"]}]},"then":{"properties":{"mode":{"const":"context"}}}}
            ],
            "oneOf":[{"properties":{"mode":{"const":"search"}},"not":{"required":["id"]}},
                {"properties":{"mode":{"enum":["get","related","context","source"]}},"required":["id"]}]}})
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
