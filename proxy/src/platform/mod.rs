// proxy/src/platform/mod.rs
//
// Platform adapter abstraction: makes the proxy work with any AI API
// (Anthropic, OpenAI, Google, etc.) by abstracting message format differences.
//
// Each platform implements `PlatformAdapter` to handle:
// - Tool result extraction (how tool output appears in messages)
// - Endpoint interception (which URL path to intercept)
// - Platform-specific headers (e.g., Anthropic's cache headers)
// - Model detection (which models belong to this platform)

pub mod anthropic;
pub mod openai;
pub mod generic;

use serde_json::Value;

/// A platform adapter that abstracts API-specific message format differences.
pub trait PlatformAdapter: Send + Sync {
    /// Extract tool result text from a message block.
    /// Returns (tool_name, tool_output_text) or None if not a tool result.
    #[allow(dead_code)]
    fn extract_tool_result(&self, block: &Value) -> Option<(String, String)>;

    /// Check if a message block is a tool result.
    fn is_tool_result(&self, block: &Value) -> bool;

    /// Get the endpoint path to intercept (e.g., "/v1/messages", "/v1/chat/completions").
    #[allow(dead_code)]
    fn intercept_path(&self) -> &str;

    /// Get platform-specific headers to inject into requests.
    #[allow(dead_code)]
    fn platform_headers(&self) -> Vec<(String, String)>;

    /// Check if a model name matches this platform.
    fn is_platform_model(&self, model: &str) -> bool;

    /// Get the platform name for logging.
    fn platform_name(&self) -> &str;
}

/// Auto-detect the platform from the request body.
pub fn detect_platform(body: &Value) -> Box<dyn PlatformAdapter> {
    let model = body["model"].as_str().unwrap_or("");

    if anthropic::AnthropicAdapter.is_platform_model(model) {
        Box::new(anthropic::AnthropicAdapter)
    } else if openai::OpenAIAdapter.is_platform_model(model) {
        Box::new(openai::OpenAIAdapter)
    } else {
        Box::new(generic::GenericAdapter)
    }
}

/// Get a platform adapter by name.
pub fn get_platform(name: &str) -> Box<dyn PlatformAdapter> {
    match name.to_lowercase().as_str() {
        "anthropic" => Box::new(anthropic::AnthropicAdapter),
        "openai" => Box::new(openai::OpenAIAdapter),
        _ => Box::new(generic::GenericAdapter),
    }
}