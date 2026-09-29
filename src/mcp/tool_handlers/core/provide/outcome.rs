//! Typed boundary between one context evaluation and MCP response delivery.

use crate::error::{CleanCtxError, to_jsonrpc_error};
use serde_json::{Value, json};

pub(super) type ProvideResult = Result<ProvidedContext, ProvideFailure>;

pub(in crate::mcp::tool_handlers::core) struct ProvidedContext {
    pub(super) text: String,
    pub(super) meta: Value,
}

impl ProvidedContext {
    pub(in crate::mcp::tool_handlers::core) fn new(text: String, meta: Value) -> Self {
        Self { text, meta }
    }

    pub(super) fn legacy_response(&self, id: &Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": self.text.clone() }],
                "_meta": self.meta.clone(),
            }
        })
    }
}

pub(super) struct ProvideFailure {
    pub(super) code: i64,
    pub(super) message: String,
    pub(super) data: Option<Value>,
}

impl ProvideFailure {
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

    pub(super) fn projection(error: &crate::ir::hierarchical::HierarchicalProjectionError) -> Self {
        let mapped = CleanCtxError::Ir(format!("hierarchical projection failed: {error}"));
        let response = to_jsonrpc_error(&Value::Null, &mapped);
        Self::new(
            response["error"]["code"].as_i64().unwrap_or(-32603),
            response["error"]["message"]
                .as_str()
                .unwrap_or("hierarchical projection failed")
                .to_string(),
            Some(json!({
                "retryable": mapped.is_retryable(),
                "projection_code": error.code(),
            })),
        )
    }

    pub(super) fn response(&self, id: &Value) -> Value {
        crate::mcp::tool_helpers::jsonrpc_error(
            id.clone(),
            self.code,
            self.message.clone(),
            self.data.clone(),
        )
    }
}
