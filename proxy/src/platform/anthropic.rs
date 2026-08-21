// proxy/src/platform/anthropic.rs
//
// Anthropic API adapter.
//
// Handles Anthropic's message format:
// - Tool results are `type: "tool_result"` blocks inside `content` arrays
// - Tool names are on preceding `tool_use` blocks (not on tool_result)
// - Uses `cache_control` breakpoints for prompt caching
// - Uses `anthropic-beta` header for extended cache TTL

use super::PlatformAdapter;
use serde_json::Value;

/// Anthropic API adapter.
pub struct AnthropicAdapter;

impl PlatformAdapter for AnthropicAdapter {
    fn extract_tool_result(&self, block: &Value) -> Option<(String, String)> {
        if !self.is_tool_result(block) {
            return None;
        }

        // Anthropic tool_result blocks have content as string or array
        let content = if let Some(text) = block["content"].as_str() {
            text.to_string()
        } else if let Some(arr) = block["content"].as_array() {
            arr.iter()
                .filter_map(|item| item["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            return None;
        };

        // Tool name is on the preceding tool_use block, not here
        // We use "unknown" as fallback — the filter system uses first-line matching
        let name = block["tool_use_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Some((name, content))
    }

    fn is_tool_result(&self, block: &Value) -> bool {
        block["type"].as_str() == Some("tool_result")
    }

    fn intercept_path(&self) -> &str {
        "/v1/messages"
    }

    fn platform_headers(&self) -> Vec<(String, String)> {
        vec![(
            "anthropic-beta".into(),
            "extended-cache-ttl-2025-04-11".into(),
        )]
    }

    fn is_platform_model(&self, model: &str) -> bool {
        model.contains("claude")
    }

    fn platform_name(&self) -> &str {
        "anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_anthropic_is_tool_result() {
        let block = json!({"type": "tool_result", "content": "output"});
        assert!(AnthropicAdapter.is_tool_result(&block));

        let block = json!({"type": "text", "text": "hello"});
        assert!(!AnthropicAdapter.is_tool_result(&block));
    }

    #[test]
    fn test_anthropic_extract_tool_result_string() {
        let block = json!({"type": "tool_result", "content": "cargo build output"});
        let result = AnthropicAdapter.extract_tool_result(&block);
        assert!(result.is_some());
        let (_name, content) = result.unwrap();
        assert_eq!(content, "cargo build output");
    }

    #[test]
    fn test_anthropic_extract_tool_result_array() {
        let block = json!({
            "type": "tool_result",
            "content": [
                {"type": "text", "text": "line 1"},
                {"type": "text", "text": "line 2"}
            ]
        });
        let result = AnthropicAdapter.extract_tool_result(&block);
        assert!(result.is_some());
        let (_, content) = result.unwrap();
        assert_eq!(content, "line 1\nline 2");
    }

    #[test]
    fn test_anthropic_intercept_path() {
        assert_eq!(AnthropicAdapter.intercept_path(), "/v1/messages");
    }

    #[test]
    fn test_anthropic_platform_name() {
        assert_eq!(AnthropicAdapter.platform_name(), "anthropic");
    }

    #[test]
    fn test_anthropic_is_platform_model() {
        assert!(AnthropicAdapter.is_platform_model("claude-sonnet-4-20250514"));
        assert!(AnthropicAdapter.is_platform_model("claude-opus-4-6"));
        assert!(!AnthropicAdapter.is_platform_model("gpt-4o"));
    }
}
