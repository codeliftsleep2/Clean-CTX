// proxy/src/transform.rs
//
// Request body transforms: tool dropping, ANSI stripping, Bash git trim,
// and model override.

use regex::Regex;
use serde_json::Value;
use tracing::debug;

/// Statistics tracked across transform operations.
#[derive(Debug, Clone, Default)]
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
        // Matches "claude-sonnet-4-20250514", "claude-opus-4-6", etc.
        Regex::new(r"claude-[a-z]+-\d[\d.-]+[a-z\d]").expect("Invalid model regex")
    })
}

/// Drop tools from body.tools[] that match the provided exclusion set.
///
/// Returns the number of tools dropped.
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
/// Scans messages[].content[].text and tool_result blocks (both string and array content).
/// Returns the number of ANSI sequences stripped.
pub fn strip_ansi(body: &mut Value, stats: &mut TransformStats) -> usize {
    let re = ansi_regex();
    let mut total_sequences: usize = 0;

    if let Some(messages) = body["messages"].as_array_mut() {
        for msg in messages.iter_mut() {
            if let Some(content) = msg["content"].as_array_mut() {
                for block in content.iter_mut() {
                    // Handle direct text blocks
                    if let Some(text) = block["text"].as_str() {
                        let seq_count = re.find_iter(text).count();
                        if seq_count > 0 {
                            let cleaned = re.replace_all(text, "").to_string();
                            total_sequences += seq_count;
                            block["text"] = Value::String(cleaned);
                        }
                    }

                    // Handle tool_result blocks — content can be a string or array
                    if block["type"].as_str() == Some("tool_result") {
                        // Handle string content
                        if let Some(text) = block["content"].as_str() {
                            let seq_count = re.find_iter(text).count();
                            if seq_count > 0 {
                                let cleaned = re.replace_all(text, "").to_string();
                                total_sequences += seq_count;
                                block["content"] = Value::String(cleaned);
                            }
                        }
                        // Handle array content
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
///
/// Claude Code's Bash tool description includes git-commit and PR-creation
/// subsections. If you don't use git through Claude Code, this saves ~1,800
/// tokens per turn.
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
///
/// Rewrites both the top-level `model` field and any model-name
/// references inside `system` blocks for consistency.
pub fn override_model(body: &mut Value, model: &str, stats: &mut TransformStats) -> bool {
    let mut changed = false;

    // Override top-level model
    if let Some(current) = body["model"].as_str() {
        if current != model {
            body["model"] = Value::String(model.to_string());
            changed = true;
        }
    }

    // Rewrite model references in system blocks (e.g., "You are Claude..." self-descriptions)
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
        // Use actual ESC byte (0x1B) directly
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
        strip_ansi(&mut body, &mut stats);

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
}