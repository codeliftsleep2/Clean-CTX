// src/mcp/tool_handlers/traits.rs
//
// Minimal trait definitions for tool handler abstractions.

use crate::mcp::McpState;
use serde_json::Value;

/// Boxed handler function type.
pub type BoxedHandlerFn = Box<dyn Fn(&Value, &Value, &McpState) + Send + Sync>;
