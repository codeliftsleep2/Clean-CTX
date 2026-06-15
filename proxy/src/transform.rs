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
}