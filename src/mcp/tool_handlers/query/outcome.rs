//! Typed internal boundary between workspace-query evaluation and MCP delivery.

use crate::mcp::McpState;
use serde_json::{Value, json};

pub(super) type QueryResult = Result<QueryAnswer, QueryFailure>;

pub(super) struct QueryAnswer {
    pub(super) query_type: &'static str,
    pub(super) structured: Value,
}

impl QueryAnswer {
    pub(super) fn new(query_type: &'static str, structured: Value) -> Self {
        Self {
            query_type,
            structured,
        }
    }
}

pub(super) struct QueryFailure {
    pub(super) code: i64,
    pub(super) message: String,
    pub(super) data: Option<Value>,
}

impl QueryFailure {
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self::new(-32602, message, None)
    }

    pub(super) fn internal(message: impl Into<String>) -> Self {
        Self::new(-32603, message, None)
    }

    pub(super) fn new(code: i64, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }

    pub(super) fn from_clean_ctx(error: &crate::error::CleanCtxError) -> Self {
        Self::new(
            error.status_code(),
            error.to_string(),
            Some(json!({ "retryable": error.is_retryable() })),
        )
    }

    pub(super) fn from_projection(
        error: &crate::ir::hierarchical::HierarchicalProjectionError,
    ) -> Self {
        let mapped =
            crate::error::CleanCtxError::Ir(format!("hierarchical projection failed: {error}"));
        Self::new(
            mapped.status_code(),
            mapped.to_string(),
            Some(json!({
                "retryable": mapped.is_retryable(),
                "projection_code": error.code(),
            })),
        )
    }

    fn response(&self, id: &Value) -> Value {
        crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            self.code,
            self.message.clone(),
            self.data.clone(),
        )
    }
}

pub(super) fn send_single(id: &Value, args: &Value, state: &McpState, result: QueryResult) {
    let response = match result {
        Ok(answer) => {
            let content = super::content::render(
                answer.query_type,
                args,
                &answer.structured,
                &state.config.additional_roots,
            );
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": content }],
                    "structuredContent": answer.structured,
                }
            })
        }
        Err(error) => error.response(id),
    };
    crate::protocol::send_response(&response);
}
