// proxy/src/platform/openai.rs
//
// OpenAI API adapter.
//
// Handles OpenAI's message format:
// - Tool results are `role: "tool"` messages in the messages array
// - Tool names are on `tool_call_id` field
// - No cache_control breakpoints (OpenAI doesn't support them)
// - No special headers needed

use super::PlatformAdapter;
use serde_json::Value;

/// OpenAI API adapter.
pub struct OpenAIAdapter;

impl PlatformAdapter for OpenAIAdapter {
    fn extract_tool_result(&self, block: &Value) -> Option<(String, String)> {
        if !self.is_tool_result(block) {
            return None;
        }

        let content = block["content"].as_str().unwrap_or("").to_string();

        let name = block["tool_call_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Some((name, content))
    }

    fn is_tool_result(&self, block: &Value) -> bool {
        block["role"].as_str() == Some("tool")
    }

    fn intercept_path(&self) -> &str {
        "/v1/chat/completions"
    }

    fn platform_headers(&self) -> Vec<(String, String)> {
        vec![] // OpenAI doesn't need special headers
    }

    fn is_platform_model(&self, model: &str) -> bool {
        model.starts_with("gpt-")
            || model.starts_with("o1")
            || model.starts_with("o3")
            || model.starts_with("o4")
            || model.starts_with("chatgpt-")
    }

    fn platform_name(&self) -> &str {
        "openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_openai_is_tool_result() {
        let block = json!({"role": "tool", "content": "output", "tool_call_id": "call_123"});
        assert!(OpenAIAdapter.is_tool_result(&block));

        let block = json!({"role": "user", "content": "hello"});
        assert!(!OpenAIAdapter.is_tool_result(&block));
    }

    #[test]
    fn test_openai_extract_tool_result() {
        let block = json!({
            "role": "tool",
            "content": "cargo build output",
            "tool_call_id": "call_123"
        });
        let result = OpenAIAdapter.extract_tool_result(&block);
        assert!(result.is_some());
        let (name, content) = result.unwrap();
        assert_eq!(name, "call_123");
        assert_eq!(content, "cargo build output");
    }

    #[test]
    fn test_openai_intercept_path() {
        assert_eq!(OpenAIAdapter.intercept_path(), "/v1/chat/completions");
    }

    #[test]
    fn test_openai_platform_name() {
        assert_eq!(OpenAIAdapter.platform_name(), "openai");
    }

    #[test]
    fn test_openai_is_platform_model() {
        assert!(OpenAIAdapter.is_platform_model("gpt-4o"));
        assert!(OpenAIAdapter.is_platform_model("o1-preview"));
        assert!(OpenAIAdapter.is_platform_model("o3-mini"));
        assert!(!OpenAIAdapter.is_platform_model("claude-sonnet-4-20250514"));
    }
}
