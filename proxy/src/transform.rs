// proxy/src/transform.rs
//
// Request body transforms: tool dropping, ANSI stripping, Bash git trim,
// model override, secret scrubbing, and tool output filtering.

use regex::Regex;
use serde_json::Value;
use tracing::debug;

use crate::filters::{apply_filter, build_filtered_marker, json_guard};
use crate::filter_registry::FilterRegistry;
use crate::filter_stats::FilterStats;
use crate::platform::PlatformAdapter;
use crate::scrub;

/// Statistics tracked across transform operations.
#[derive(Debug, Clone)]
pub struct TransformStats {
    /// Number of tools dropped.
    pub tools_dropped: u64,

    /// Number of ANSI escape sequences stripped.
    pub ansi_sequences_stripped: u64,

    /// Total bytes of ANSI sequences removed.
    #[allow(dead_code)]
    pub ansi_bytes_stripped: u64,

    /// Bash tool trimmed (git section).
    pub bash_git_trims: u64,

    /// Model name overrides applied.
    pub model_overrides: u64,

    /// Number of secrets scrubbed.
    pub secrets_scrubbed: u64,

    /// Number of tool output filter applications.
    pub tool_filters_applied: u64,

    /// Per-program filter savings (only populated when TOOL_FILTERS is enabled).
    pub filter_stats: FilterStats,

    /// Sliding window stats.
    pub sliding_window: SlidingWindowStats,
}

/// Sliding context window statistics.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SlidingWindowStats {
    /// Number of tool results aged this request.
    pub items_aged: u64,
    /// Bytes removed by aging this request.
    pub bytes_removed: u64,
    /// Cumulative bytes removed over the session.
    pub cumulative_bytes_removed: u64,
}

impl SlidingWindowStats {
    /// Record an aging event.
    pub fn record(&mut self, bytes: usize) {
        self.items_aged += 1;
        self.bytes_removed += bytes as u64;
        self.cumulative_bytes_removed += bytes as u64;
    }

    /// Reset per-request counters (items_aged, bytes_removed) for the next request.
    #[allow(dead_code)]
    pub fn reset_request(&mut self) {
        self.items_aged = 0;
        self.bytes_removed = 0;
    }
}

impl Default for TransformStats {
    fn default() -> Self {
        Self {
            tools_dropped: 0,
            ansi_sequences_stripped: 0,
            ansi_bytes_stripped: 0,
            bash_git_trims: 0,
            model_overrides: 0,
            secrets_scrubbed: 0,
            tool_filters_applied: 0,
            filter_stats: FilterStats::new(),
            sliding_window: SlidingWindowStats::default(),
        }
    }
}

/// Lazy-initialized ANSI escape regex.
fn ansi_regex() -> &'static Regex {
    static ANSI_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    ANSI_RE.get_or_init(|| {
        Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").expect("Invalid ANSI regex")
    })
}

/// Lazy-initialized regex for matching Claude model names in text.
fn model_regex() -> &'static Regex {
    static MODEL_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    MODEL_RE.get_or_init(|| {
        Regex::new(r"claude-[a-z]+-\d[\d.-]+[a-z\d]").expect("Invalid model regex")
    })
}

/// Drop tools from body.tools[] that match the provided exclusion set.
pub fn drop_tools(body: &mut Value, drop_set: &std::collections::HashSet<String>, stats: &mut TransformStats) -> usize {
    if drop_set.is_empty() {
        return 0;
    }

    let before_count = body["tools"].as_array().map(|a| a.len()).unwrap_or(0);
    if before_count == 0 {
        return 0;
    }

    if let Some(tools) = body["tools"].as_array_mut() {
        tools.retain(|tool| {
            let name = tool["name"].as_str().unwrap_or("");
            let keep = !drop_set.contains(name);
            if !keep {
                debug!("[transform] Dropped tool: {}", name);
            }
            keep
        });

        let after_count = tools.len();
        let dropped = before_count - after_count;
        stats.tools_dropped += dropped as u64;
        dropped
    } else {
        0
    }
}

/// Strip ANSI escape codes from all text fields in the request body.
///
/// Uses the platform adapter to detect tool result blocks across different
/// API formats (Anthropic's `type: "tool_result"`, OpenAI's `role: "tool"`, etc.).
pub fn strip_ansi(body: &mut Value, stats: &mut TransformStats, adapter: &dyn PlatformAdapter) -> usize {
    let re = ansi_regex();
    let mut total_sequences: usize = 0;

    if let Some(messages) = body["messages"].as_array_mut() {
        for msg in messages.iter_mut() {
            if let Some(content) = msg["content"].as_array_mut() {
                for block in content.iter_mut() {
                    if let Some(text) = block["text"].as_str() {
                        let seq_count = re.find_iter(text).count();
                        if seq_count > 0 {
                            let cleaned = re.replace_all(text, "").to_string();
                            total_sequences += seq_count;
                            block["text"] = Value::String(cleaned);
                        }
                    }

                    if adapter.is_tool_result(block) {
                        if let Some(text) = block["content"].as_str() {
                            let seq_count = re.find_iter(text).count();
                            if seq_count > 0 {
                                let cleaned = re.replace_all(text, "").to_string();
                                total_sequences += seq_count;
                                block["content"] = Value::String(cleaned);
                            }
                        }
                        if let Some(content2) = block["content"].as_array_mut() {
                            for inner in content2.iter_mut() {
                                if let Some(text) = inner["text"].as_str() {
                                    let seq_count = re.find_iter(text).count();
                                    if seq_count > 0 {
                                        let cleaned = re.replace_all(text, "").to_string();
                                        total_sequences += seq_count;
                                        inner["text"] = Value::String(cleaned);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    stats.ansi_sequences_stripped += total_sequences as u64;
    total_sequences
}

/// Truncate the Bash tool description at the "Committing changes" git section.
pub fn trim_bash_git(body: &mut Value, stats: &mut TransformStats) -> bool {
    let trim_marker = "Committing changes";

    if let Some(tools) = body["tools"].as_array_mut() {
        for tool in tools.iter_mut() {
            let name = tool["name"].as_str().unwrap_or("");
            if name == "Bash" {
                if let Some(desc) = tool["description"].as_str() {
                    if let Some(pos) = desc.find(trim_marker) {
                        let trimmed = format!(
                            "{}…\n\n[Git commit/PR sections removed by proxy — set TRIM_BASH_GIT=0 to restore]",
                            &desc[..pos]
                        );
                        tool["description"] = Value::String(trimmed);
                        stats.bash_git_trims += 1;
                        debug!("[transform] Bash git section trimmed");
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Override the model name in the request body.
pub fn override_model(body: &mut Value, model: &str, stats: &mut TransformStats) -> bool {
    let mut changed = false;

    if let Some(current) = body["model"].as_str() {
        if current != model {
            body["model"] = Value::String(model.to_string());
            changed = true;
        }
    }

    let re = model_regex();
    if let Some(system) = body["system"].as_array_mut() {
        for block in system.iter_mut() {
            if let Some(text) = block["text"].as_str() {
                if text.contains("Claude") || text.contains("claude") {
                    let rewritten = re.replace_all(text, model);
                    if rewritten != text {
                        block["text"] = Value::String(rewritten.to_string());
                        changed = true;
                    }
                }
            }
        }
    }

    if changed {
        stats.model_overrides += 1;
        debug!("[transform] Model overridden to: {model}");
    }

    changed
}

/// Scrub secrets from all text fields in the request body.
///
/// Uses the platform adapter to detect tool result blocks across different
/// API formats (Anthropic's `type: "tool_result"`, OpenAI's `role: "tool"`, etc.).
pub fn scrub_secrets(body: &mut Value, stats: &mut TransformStats, adapter: &dyn PlatformAdapter) -> u64 {
    let mut total_hits: u64 = 0;

    if let Some(messages) = body["messages"].as_array_mut() {
        for msg in messages.iter_mut() {
            if let Some(content) = msg["content"].as_array_mut() {
                for block in content.iter_mut() {
                    if let Some(text) = block["text"].as_str() {
                        let result = scrub::scrub_secrets(text);
                        if !result.hits.is_empty() {
                            total_hits += result.hits.len() as u64;
                            block["text"] = Value::String(result.content);
                        }
                    }

                    if adapter.is_tool_result(block) {
                        if let Some(text) = block["content"].as_str() {
                            let result = scrub::scrub_secrets(text);
                            if !result.hits.is_empty() {
                                total_hits += result.hits.len() as u64;
                                block["content"] = Value::String(result.content);
                            }
                        }
                        if let Some(content2) = block["content"].as_array_mut() {
                            for inner in content2.iter_mut() {
                                if let Some(text) = inner["text"].as_str() {
                                    let result = scrub::scrub_secrets(text);
                                    if !result.hits.is_empty() {
                                        total_hits += result.hits.len() as u64;
                                        inner["text"] = Value::String(result.content);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    stats.secrets_scrubbed += total_hits;
    total_hits
}

/// Age tool result blocks based on the sliding context window policy.
///
/// This is the core of R-41 Tier 1: deterministic, rule-based age truncation.
///
/// Rules:
/// 1. System prompt is never aged.
/// 2. Messages within `force_preserve_floor` turns of the end are never aged.
/// 3. If a path string from a candidate aged item appears anywhere later in
///    the array, the item is preserved (path cross-reference check).
/// 4. Assistant messages (`role: "assistant"`) are never aged.
/// 5. Aged tool results are replaced with a stub, not deleted.
/// 6. The stub contains the original tool name and approximate position.
pub fn age_tool_results(
    body: &mut Value,
    stats: &mut TransformStats,
    max_age_turns: usize,
    force_preserve_floor: usize,
    adapter: &dyn PlatformAdapter,
) -> usize {
    let messages = match body["messages"].as_array_mut() {
        Some(m) => m,
        None => return 0,
    };

    if messages.is_empty() {
        return 0;
    }

    let total_messages = messages.len();
    let mut aged_count: usize = 0;
    let mut bytes_removed: usize = 0;

    // Collect all path-like strings from the last `floor` messages
    // for cross-reference checking.
    let recent_paths: Vec<String> = {
        let start = total_messages.saturating_sub(force_preserve_floor);
        let mut paths = Vec::new();
        for i in start..total_messages {
            if let Some(msg) = messages.get(i) {
                extract_path_strings(msg, &mut paths);
            }
        }
        paths
    };

    // Process messages from oldest to newest (but preserve the last N)
    // Use index-based iteration to avoid borrow conflicts
    let mut i = 0;
    while i < total_messages {
        // Skip messages within the force-preserve floor (most recent turns)
        if i >= total_messages.saturating_sub(force_preserve_floor) {
            i += 1;
            continue;
        }

        // Check role without borrowing the whole message
        let role = messages[i]["role"].as_str().map(|s| s.to_string());

        // Skip assistant messages (they contain reasoning, never age them)
        if role.as_deref() == Some("assistant") {
            i += 1;
            continue;
        }

        // Skip system messages (they set context, never age them)
        if role.as_deref() == Some("system") {
            i += 1;
            continue;
        }

        // Now we need mutable access to modify tool result blocks
        // Get the content array length first
        let content_len = messages[i]["content"].as_array().map(|a| a.len()).unwrap_or(0);
        if content_len == 0 {
            i += 1;
            continue;
        }

        // Process each content block
        let mut j = 0;
        while j < content_len {
            // Check if this is a tool result (immutable borrow)
            let is_tool = {
                let block = &messages[i]["content"][j];
                adapter.is_tool_result(block)
            };

            if !is_tool {
                j += 1;
                continue;
            }

            // Get tool name and original text (immutable borrow)
            let (tool_name, original_text) = {
                let block = &messages[i]["content"][j];
                let name = block["tool_use_id"]
                    .as_str()
                    .or_else(|| block["name"].as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let text = if let Some(t) = block["content"].as_str() {
                    t.to_string()
                } else if let Some(arr) = block["content"].as_array() {
                    arr.iter()
                        .filter_map(|item| item["text"].as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    String::new()
                };
                (name, text)
            };

            if original_text.is_empty() {
                j += 1;
                continue;
            }

            // Path cross-reference check
            let mut content_paths = Vec::new();
            extract_path_strings_from_text(&original_text, &mut content_paths);
            let has_cross_ref = content_paths.iter().any(|p| {
                recent_paths.iter().any(|rp| rp.contains(p))
            });

            if has_cross_ref {
                debug!("[sliding_window] Preserved aged item {} (cross-reference)", tool_name);
                j += 1;
                continue;
            }

            // Check age: skip if within max_age_turns of the end
            let age = total_messages.saturating_sub(i);
            if age <= max_age_turns {
                j += 1;
                continue;
            }

            // Safe to age this tool result — replace with a stub
            let original_bytes = original_text.len();
            let stub = format!(
                "[aged: {} output, {} tokens, turn {}]",
                tool_name,
                estimate_tokens(&original_text),
                i
            );
            let stub_len = stub.len();

            // Write the stub back (mutable access)
            if let Some(content_arr) = messages[i]["content"].as_array_mut() {
                if let Some(block_mut) = content_arr.get_mut(j) {
                    if block_mut["content"].is_string() {
                        block_mut["content"] = Value::String(stub.clone());
                    } else if block_mut["content"].is_array() {
                        block_mut["content"] = Value::String(stub);
                    }
                }
            }

            aged_count += 1;
            bytes_removed += original_bytes.saturating_sub(stub_len);

            debug!(
                "[sliding_window] Aged tool result '{}' at turn {} (age={})",
                tool_name, i, age
            );

            j += 1;
        }

        i += 1;
    }

    if aged_count > 0 {
        stats.sliding_window.record(bytes_removed);
        debug!(
            "[sliding_window] Aged {} items, removed {} bytes",
            aged_count, bytes_removed
        );
    }

    aged_count
}

/// Rough token estimate from byte length (4 chars ≈ 1 token for code).
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4 + 1
}

/// Extract path-like strings from a message value.
fn extract_path_strings(msg: &Value, paths: &mut Vec<String>) {
    // Check content array for text blocks
    if let Some(content) = msg["content"].as_array() {
        for block in content {
            if let Some(text) = block["text"].as_str() {
                extract_path_strings_from_text(text, paths);
            }
            if let Some(text) = block["content"].as_str() {
                extract_path_strings_from_text(text, paths);
            }
            if let Some(arr) = block["content"].as_array() {
                for inner in arr {
                    if let Some(text) = inner["text"].as_str() {
                        extract_path_strings_from_text(text, paths);
                    }
                }
            }
        }
    }
    // Check direct text fields
    if let Some(text) = msg["text"].as_str() {
        extract_path_strings_from_text(text, paths);
    }
}

/// Extract path-like strings (containing / or \) from text.
fn extract_path_strings_from_text(text: &str, paths: &mut Vec<String>) {
    for word in text.split_whitespace() {
        // Simple heuristic: looks like a file path
        if word.contains('/') || word.contains('\\') {
            let trimmed = word.trim_matches(|c: char| c == '"' || c == '\'' || c == '`' || c == ',' || c == ')' || c == ']' || c == '}');
            if !trimmed.is_empty() && (trimmed.contains('.') || trimmed.contains('/') || trimmed.contains('\\')) {
                paths.push(trimmed.to_string());
            }
        }
    }
}

/// Apply tool output filters to all tool result blocks in the request body.
///
/// Uses the platform adapter to detect tool results across different API formats
/// (Anthropic's `type: "tool_result"`, OpenAI's `role: "tool"`, etc.).
/// We detect commands by examining the first non-empty line of the tool result
/// content and trying all registered filters against it.
pub fn apply_tool_filters(
    body: &mut Value,
    registry: &FilterRegistry,
    stats: &mut TransformStats,
    adapter: &dyn PlatformAdapter,
) -> usize {
    let mut total_applied: usize = 0;

    if let Some(messages) = body["messages"].as_array_mut() {
        for msg in messages.iter_mut() {
            if let Some(content) = msg["content"].as_array_mut() {
                for block in content.iter_mut() {
                    if !adapter.is_tool_result(block) {
                        continue;
                    }

                    let original_text = if let Some(text) = block["content"].as_str() {
                        text.to_string()
                    } else if let Some(arr) = block["content"].as_array() {
                        arr.iter()
                            .filter_map(|item| item["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        continue;
                    };

                    if original_text.trim().is_empty() {
                        continue;
                    }

                    let first_line = original_text
                        .lines()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or("");

                    let filter = registry.select_for_command(first_line)
                        .or_else(|| registry.select_for_command(&original_text));

                    if let Some(filter) = filter {
                        let failed = original_text.contains("exit code")
                            || original_text.contains("Exit code")
                            || original_text.contains("error:");

                        let result = apply_filter(filter, &original_text, failed);

                        let (filtered_content, _) = json_guard(
                            &original_text,
                            &result.content,
                            result.truncated,
                            filter.reduce_json,
                        );

                        let marker = build_filtered_marker(&result);
                        let final_content = format!("{}\n{}", filtered_content, marker);

                        if block["content"].is_string() {
                            block["content"] = Value::String(final_content);
                        } else if let Some(arr) = block["content"].as_array_mut() {
                            arr.clear();
                            arr.push(serde_json::json!({
                                "type": "text",
                                "text": final_content
                            }));
                        }

                        stats.filter_stats.record_application(
                            &result.program,
                            result.original_tokens,
                            result.filtered_tokens,
                            result.original_lines,
                            result.filtered_lines,
                        );
                        stats.tool_filters_applied += 1;
                        total_applied += 1;
                    }
                }
            }
        }
    }

    total_applied
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    #[test]
    fn test_drop_tools() {
        let mut body = json!({
            "model": "claude-sonnet-4-20250514",
            "tools": [
                {"name": "Bash", "description": "Run commands"},
                {"name": "Read", "description": "Read files"},
                {"name": "NotebookEdit", "description": "Edit notebooks"},
                {"name": "CronCreate", "description": "Create cron jobs"}
            ]
        });

        let mut drop_set = HashSet::new();
        drop_set.insert("NotebookEdit".to_string());
        drop_set.insert("CronCreate".to_string());

        let mut stats = TransformStats::default();
        let dropped = drop_tools(&mut body, &drop_set, &mut stats);

        assert_eq!(dropped, 2);
        assert_eq!(stats.tools_dropped, 2);
        let tools = body["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["name"], "Bash");
        assert_eq!(tools[1]["name"], "Read");
    }

    #[test]
    fn test_strip_ansi() {
        let esc = "\x1B";
        let mut body = json!({
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": format!("Hello {}[32mworld{}[0m!", esc, esc)},
                    {"type": "tool_result", "content": format!("Line 1{}[K\nLine 2", esc)}
                ]
            }]
        });

        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        strip_ansi(&mut body, &mut stats, &adapter);

        let text = body["messages"][0]["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "Hello world!");
        let result = body["messages"][0]["content"][1]["content"].as_str().unwrap();
        assert_eq!(result, "Line 1\nLine 2");
        assert!(stats.ansi_sequences_stripped >= 3);
    }

    #[test]
    fn test_trim_bash_git() {
        let mut body = json!({
            "tools": [
                {"name": "Bash", "description": "Run shell commands.\n\nCommitting changes\n\nThis section about git."},
                {"name": "Read", "description": "Read files."}
            ]
        });

        let mut stats = TransformStats::default();
        let trimmed = trim_bash_git(&mut body, &mut stats);

        assert!(trimmed);
        assert_eq!(stats.bash_git_trims, 1);
        let desc = body["tools"][0]["description"].as_str().unwrap();
        assert!(desc.contains("Run shell commands."));
        assert!(desc.contains("[Git commit/PR sections removed"));
        assert!(!desc.contains("Committing changes"));
    }

    #[test]
    fn test_override_model() {
        let mut body = json!({
            "model": "claude-sonnet-4-20250514",
            "system": [
                {"type": "text", "text": "You are claude-sonnet-4-20250514, a helpful assistant."}
            ]
        });

        let mut stats = TransformStats::default();
        let changed = override_model(&mut body, "claude-opus-4-6", &mut stats);

        assert!(changed);
        assert_eq!(body["model"], "claude-opus-4-6");
        let sys_text = body["system"][0]["text"].as_str().unwrap();
        assert!(sys_text.contains("claude-opus-4-6"));
        assert!(!sys_text.contains("claude-sonnet-4-20250514"));
    }

    #[test]
    fn test_scrub_secrets_in_body() {
        let mut body = json!({
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "tool_result", "content": "AWS key: AKIAIOSFODNN7EXAMPLE"}
                ]
            }]
        });

        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let hits = scrub_secrets(&mut body, &mut stats, &adapter);

        assert!(hits > 0);
        assert!(stats.secrets_scrubbed > 0);
        let content = body["messages"][0]["content"][0]["content"].as_str().unwrap();
        assert!(content.contains("[REDACTED]"));
        assert!(!content.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_scrub_secrets_empty_body() {
        let mut body = json!({
            "messages": []
        });

        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let hits = scrub_secrets(&mut body, &mut stats, &adapter);

        assert_eq!(hits, 0);
        assert_eq!(stats.secrets_scrubbed, 0);
    }

    #[test]
    fn test_scrub_secrets_no_secrets() {
        let mut body = json!({
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "tool_result", "content": "Hello world, no secrets here."}
                ]
            }]
        });

        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let hits = scrub_secrets(&mut body, &mut stats, &adapter);

        assert_eq!(hits, 0);
        let content = body["messages"][0]["content"][0]["content"].as_str().unwrap();
        assert_eq!(content, "Hello world, no secrets here.");
    }

    // ── Sliding Window Tests ──────────────────────────────────────

    #[test]
    fn test_age_tool_results_empty_messages() {
        let mut body = json!({ "messages": [] });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let aged = age_tool_results(&mut body, &mut stats, 20, 15, &adapter);
        assert_eq!(aged, 0);
        assert_eq!(stats.sliding_window.items_aged, 0);
    }

    #[test]
    fn test_age_tool_results_no_tool_results() {
        let mut body = json!({
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "Hello"}]},
                {"role": "assistant", "content": [{"type": "text", "text": "Hi!"}]}
            ]
        });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let aged = age_tool_results(&mut body, &mut stats, 5, 0, &adapter);
        assert_eq!(aged, 0);
    }

    #[test]
    fn test_age_tool_results_ages_old_tool_results() {
        // Create a body with 10 messages where the first has an old tool result
        let mut messages = Vec::new();
        // Message 0: user with tool result (should be aged — age=10, max_age=5)
        messages.push(serde_json::json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": "cargo_build",
                "content": "Compiling foo v1.0.0\n  Finished dev profile\n"
            }]
        }));
        // Messages 1-8: filler assistant/user exchanges
        for i in 1..9 {
            messages.push(serde_json::json!({
                "role": if i % 2 == 1 { "assistant" } else { "user" },
                "content": [{"type": "text", "text": format!("Message {}", i)}]
            }));
        }
        // Message 9: user with tool result (within max_age — should NOT be aged)
        messages.push(serde_json::json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": "read_file",
                "content": "src/main.rs\nfn main() {}\n"
            }]
        }));

        let mut body = serde_json::json!({ "messages": messages });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;

        // max_age_turns=5, force_preserve_floor=0
        let aged = age_tool_results(&mut body, &mut stats, 5, 0, &adapter);

        // Only message 0 should be aged (it's age=10, max_age=5)
        assert_eq!(aged, 1, "one old tool result should be aged");
        assert_eq!(stats.sliding_window.items_aged, 1);

        // Verify message 0 was stubbed
        let content = body["messages"][0]["content"][0]["content"].as_str().unwrap();
        assert!(content.contains("[aged:"), "aged content should have stub");
        assert!(content.contains("turn 0"), "stub should contain turn number");

        // Verify message 9 was NOT aged
        let content9 = body["messages"][9]["content"][0]["content"].as_str().unwrap();
        assert!(content9.contains("src/main.rs"), "recent content should be preserved");
    }

    #[test]
    fn test_age_tool_results_preserves_assistant_messages() {
        // With max_age_turns=0, any message outside the floor is eligible for aging.
        // We test that assistant messages are NEVER aged.
        // Message at index 0: assistant → should be preserved
        // Message at index 1: user tool result with age=2, max_age=0 → eligible for aging
        let mut body = json!({
            "messages": [
                {"role": "assistant", "content": [{"type": "text", "text": "Let me check that file for you."}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "bash", "content": "line1\nline2\nline3\n"}]}
            ]
        });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let aged = age_tool_results(&mut body, &mut stats, 0, 0, &adapter);
        // User tool result is at position 1, age=2, max_age=0 → age > max_age → should be aged
        assert_eq!(aged, 1);

        // Assistant message should still be intact
        let text = body["messages"][0]["content"][0]["text"].as_str().unwrap();
        assert_eq!(text, "Let me check that file for you.");
    }

    #[test]
    fn test_age_tool_results_force_preserve_floor() {
        let mut messages: Vec<serde_json::Value> = (0..20).map(|i| {
            serde_json::json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": format!("tool_{}", i),
                    "content": format!("output_{}", i)
                }]
            })
        }).collect();
        messages.push(serde_json::json!({
            "role": "user", "content": [{"type": "text", "text": "latest"}]
        }));

        let mut body = serde_json::json!({ "messages": messages });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;

        // max_age_turns=5, force_preserve_floor=5
        // Messages with age > 5 AND outside the last 5 should be aged
        // Total = 21 messages. Last 5 preserved = indices 16-20.
        // Among indices 0-15: those with age > 5 → indices 0-15 all have age >= 6.
        let aged = age_tool_results(&mut body, &mut stats, 5, 5, &adapter);

        // 16 messages outside floor (0-15), all with age > 5 → all should be aged
        assert_eq!(aged, 16);

        // Last 5 messages preserved
        for i in 16..=20 {
            let msg = &body["messages"][i];
            let role = msg["role"].as_str().unwrap();
            assert_eq!(role, "user");
        }
    }

    #[test]
    fn test_age_tool_results_path_cross_reference() {
        // Old tool result contains a path that also appears in recent message
        let messages = vec![
            serde_json::json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "read_file",
                    "content": "src/main.rs\nfn main() {}\n"
                }]
            }),
            serde_json::json!({
                "role": "user",
                "content": [{"type": "text", "text": "Check src/main.rs again"}]
            }),
        ];

        let mut body = serde_json::json!({ "messages": messages });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;

        let aged = age_tool_results(&mut body, &mut stats, 5, 0, &adapter);

        // The old tool result references src/main.rs which appears in recent messages
        // → should NOT be aged
        assert_eq!(aged, 0, "cross-referenced content should be preserved");
    }

    #[test]
    fn test_age_tool_results_system_preserved() {
        // With max_age_turns=0, any message outside the floor is eligible for aging.
        // We test that system messages are NEVER aged.
        let mut body = json!({
            "messages": [
                {"role": "system", "content": [{"type": "text", "text": "You are a helpful assistant."}]},
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "bash", "content": "output"}]}
            ]
        });
        let mut stats = TransformStats::default();
        let adapter = crate::platform::anthropic::AnthropicAdapter;
        let aged = age_tool_results(&mut body, &mut stats, 0, 0, &adapter);
        // System is preserved, user tool result (position 1, age=2) should be aged
        assert_eq!(aged, 1);
        // System message should still be intact
        let sys_text = body["messages"][0]["content"][0]["text"].as_str().unwrap();
        assert_eq!(sys_text, "You are a helpful assistant.");
    }

    #[test]
    fn test_extract_path_strings_from_text() {
        let mut paths = Vec::new();
        extract_path_strings_from_text("Checked src/main.rs, src/lib.rs", &mut paths);
        assert!(paths.contains(&"src/main.rs".to_string()));
        assert!(paths.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn test_estimate_tokens() {
        let tokens = estimate_tokens("hello world");
        assert!(tokens > 0);
        let tokens2 = estimate_tokens("a");
        assert_eq!(tokens2, 1);
    }

    #[test]
    fn test_sliding_window_stats() {
        let mut stats = SlidingWindowStats::default();
        assert_eq!(stats.items_aged, 0);
        stats.record(1000);
        assert_eq!(stats.items_aged, 1);
        assert_eq!(stats.bytes_removed, 1000);
        assert_eq!(stats.cumulative_bytes_removed, 1000);
        stats.record(500);
        assert_eq!(stats.items_aged, 2);
        assert_eq!(stats.bytes_removed, 1500);

        // Reset per-request counters
        stats.reset_request();
        assert_eq!(stats.items_aged, 0);
        assert_eq!(stats.bytes_removed, 0);
        // Cumulative is persistent across resets
        assert_eq!(stats.cumulative_bytes_removed, 1500);
    }
}
