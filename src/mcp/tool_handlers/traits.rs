// src/mcp/tool_handlers/traits.rs
//
// Minimal trait definitions for tool handler abstractions.

use serde_json::Value;
use crate::mcp::McpState;

/// Boxed handler function type.
pub type BoxedHandlerFn = Box<dyn Fn(&Value, &Value, &McpState) + Send + Sync>;
