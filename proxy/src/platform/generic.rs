// proxy/src/platform/generic.rs
//
// Generic/fallback API adapter.
//
// Handles unknown or custom API formats with best-effort detection.
// Uses heuristics to find tool results in any message format.

use serde_json::Value;
use super::PlatformAdapter;

/// Generic/fallback adapter for unknown platforms.
pub struct GenericAdapter;

impl PlatformAdapter for GenericAdapter {
    fn extract_tool_result(&self, block: &Value) -> Option<(String, String)> {
        if !self.is_tool_result(block) {
            return None;
        }

        // Try multiple content field locations
        let content = if let Some(text) = block["content"].as_str() {
            text.to_string()
        } else if let Some(text) = block["output"].as_str() {
            text.to_string()
        } else if let Some(text) = block["result"].as_str() {
            text.to_string()
        } else if let Some(arr) = block["content"].as_array() {
            arr.iter()
                .filter_map(|item| item["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            return None;
        };

        // Try multiple name field locations
        let name = block["name"].as_str()
            .or_else(|| block["tool_call_id"].as_str())
            .or_else(|| block["tool_use_id"].as_str())
            .or_else(|| block["function_name"].as_str())
            .unwrap_or("unknown")
            .to_string();

        Some((name, content))
    }

    fn is_tool_result(&self, block: &Value) -> bool {
        // Heuristic: if it has content that looks like tool output
        // and isn't a user message
        let has_content = block["content"].is_string()
            || block["output"].is_string()
            || block["result"].is_string();

        let not_user = block["role"].as_str() != Some("user")
            && block["role"].as_str() != Some("system");

        let has_tool_indicator = block["role"].as_str() == Some("tool")
            || block["type"].as_str() == Some("tool_result")
            || block["type"].as_str() == Some("function_response")
            || block["type"].as_str() == Some("tool");

        has_content && (not_user || has_tool_indicator)
    }

    fn intercept_path(&self) -> &str {
        // Default — should be overridden by config
        "/v1/messages"
    }

    fn platform_headers(&self) -> Vec<(String, String)> {
        vec![]
    }

    fn is_platform_model(&self, _model: &str) -> bool {
        // Generic accepts all models as fallback
        true
    }

    fn platform_name(&self) -> &str {
        "generic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generic_is_tool_result_role_tool() {
        let block = json!({"role": "tool", "content": "output"});
        assert!(GenericAdapter.is_tool_result(&block));
    }

    #[test]
    fn test_generic_is_tool_result_type_tool_result() {
        let block = json!({"type": "tool_result", "content": "output"});
        assert!(GenericAdapter.is_tool_result(&block));
    }

    #[test]
    fn test_generic_is_not_user_message() {
        let block = json!({"role": "user", "content": "hello"});
        assert!(!GenericAdapter.is_tool_result(&block));
    }

    #[test]
    fn test_generic_extract_tool_result_content_string() {
        let block = json!({"role": "tool", "content": "build output"});
        let result = GenericAdapter.extract_tool_result(&block);
        assert!(result.is_some());
        let (_, content) = result.unwrap();
        assert_eq!(content, "build output");
    }

    #[test]
    fn test_generic_extract_tool_result_output_field() {
        let block = json!({"output": "function output"});
        let result = GenericAdapter.extract_tool_result(&block);
        assert!(result.is_some());
        let (_, content) = result.unwrap();
        assert_eq!(content, "function output");
    }

    #[test]
    fn test_generic_extract_tool_result_name_field() {
        let block = json!({"name": "my_function", "content": "output"});
        let result = GenericAdapter.extract_tool_result(&block);
        assert!(result.is_some());
        let (name, _) = result.unwrap();
        assert_eq!(name, "my_function");
    }

    #[test]
    fn test_generic_intercept_path() {
        assert_eq!(GenericAdapter.intercept_path(), "/v1/messages");
    }

    #[test]
    fn test_generic_platform_name() {
        assert_eq!(GenericAdapter.platform_name(), "generic");
    }

    #[test]
    fn test_generic_accepts_all_models() {
        assert!(GenericAdapter.is_platform_model("claude-sonnet-4-20250514"));
        assert!(GenericAdapter.is_platform_model("gpt-4o"));
        assert!(GenericAdapter.is_platform_model("any-model-here"));
    }
}