// proxy/src/filters.rs
//
// Main filter engine: applies compiled filter rules to tool output text.
// Pipeline: replace → match_output → strip/keep_lines → group_by → head/tail → max_lines → on_empty

use crate::filter_rules::{CompiledFilter, CompiledGroupBy};

/// Result of applying a filter.
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub content: String,
    pub original_lines: usize,
    pub filtered_lines: usize,
    pub original_tokens: usize,
    pub filtered_tokens: usize,
    pub program: String,
    pub reduction_pct: f32,
    pub truncated: bool,
    #[allow(dead_code)]
    pub collapsed: bool,
}

/// Apply a compiled filter rule to the given content.
///
/// Returns the filtered content and statistics.
pub fn apply_filter(filter: &CompiledFilter, content: &str, failed: bool) -> FilterResult {
    let original_lines = content.lines().count();
    let original_tokens = estimate_tokens(content);

    let mut current = content.to_string();
    let mut truncated = false;
    let mut collapsed = false;

    // Step 0: Strip ANSI codes if configured
    if filter.strip_ansi {
        let ansi_re = regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
        current = ansi_re.replace_all(&current, "").to_string();
    }

    // Step 1: Line-by-line replace
    for rule in &filter.replace {
        current = apply_replace(&rule.pattern, &rule.replacement, &current);
    }

    // Step 2: match_output — collapse to summary if pattern matches
    if !failed {
        for mco in &filter.match_output {
            if mco.pattern.is_match(&current) {
                let suppressed = mco
                    .unless
                    .as_ref()
                    .map(|u| u.is_match(&current))
                    .unwrap_or(false);

                if !suppressed {
                    current = mco.message.clone();
                    collapsed = true;
                    break;
                }
            }
        }
    }

    // Step 3: strip_lines and keep_lines (only if not collapsed)
    if !collapsed {
        let has_keep_rules = !filter.keep_lines.is_empty();
        let has_strip_rules = !filter.strip_lines.is_empty();

        if has_keep_rules || has_strip_rules {
            let lines: Vec<&str> = current.lines().collect();
            let mut kept: Vec<String> = Vec::with_capacity(lines.len());

            for line in &lines {
                if has_keep_rules {
                    let matches_keep = filter.keep_lines.iter().any(|re| re.is_match(line));
                    if matches_keep {
                        kept.push((*line).to_string());
                    }
                    continue;
                }

                let matches_strip = filter.strip_lines.iter().any(|re| re.is_match(line));
                if !matches_strip {
                    kept.push((*line).to_string());
                }
            }

            current = kept.join("\n");
        }
    }

    // Step 4: group_by (only if not collapsed)
    if !collapsed {
        if let Some(ref gb) = filter.group_by {
            current = apply_group_by(gb, &current);
        }
    }

    // Step 5: head/tail (only if not collapsed)
    if !collapsed {
        if let Some(n) = filter.head_lines {
            let lines: Vec<&str> = current.lines().collect();
            if lines.len() > n {
                current = lines[..n].join("\n");
                truncated = true;
            }
        }
        if let Some(n) = filter.tail_lines {
            let lines: Vec<&str> = current.lines().collect();
            if lines.len() > n {
                current = lines[lines.len() - n..].join("\n");
                truncated = true;
            }
        }
    }

    // Step 6: max_lines (hard cap)
    if !collapsed {
        if let Some(max) = filter.max_lines {
            let lines: Vec<&str> = current.lines().collect();
            if lines.len() > max {
                current = lines[..max].join("\n");
                truncated = true;
            }
        }
    }

    // Step 7: on_empty
    if current.trim().is_empty() {
        if let Some(ref fallback) = filter.on_empty {
            current = fallback.clone();
        }
    }

    let filtered_lines = current.lines().count();
    let filtered_tokens = estimate_tokens(&current);
    let reduction_pct = if original_tokens > 0 {
        ((original_tokens as f32 - filtered_tokens as f32) / original_tokens as f32) * 100.0
    } else {
        0.0
    };

    FilterResult {
        content: current,
        original_lines,
        filtered_lines,
        original_tokens,
        filtered_tokens,
        program: filter.name.clone(),
        reduction_pct,
        truncated,
        collapsed,
    }
}

/// Line-by-line regex substitution.
fn apply_replace(pattern: &regex::Regex, replacement: &str, content: &str) -> String {
    content
        .lines()
        .map(|line| pattern.replace_all(line, replacement).to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Group lines by regex key, cap per group and total groups.
fn apply_group_by(gb: &CompiledGroupBy, content: &str) -> String {
    use std::collections::HashMap;

    let lines: Vec<&str> = content.lines().collect();
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    let mut group_order: Vec<String> = Vec::new();

    for line in &lines {
        if let Some(caps) = gb.key.captures(line) {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            if !groups.contains_key(&key) {
                group_order.push(key.clone());
                groups.insert(key.clone(), Vec::new());
            }
            if let Some(bucket) = groups.get_mut(&key) {
                if bucket.len() < gb.max_per_group {
                    bucket.push((*line).to_string());
                }
            }
        }
    }

    let mut result: Vec<String> = Vec::new();
    let mut groups_used = 0;

    for key in &group_order {
        if groups_used >= gb.max_groups {
            break;
        }
        if let Some(bucket) = groups.get(key) {
            for line in bucket {
                result.push(line.clone());
            }
            groups_used += 1;
        }
    }

    // Add omission markers
    if group_order.len() > gb.max_groups {
        let omitted = group_order.len() - gb.max_groups;
        let marker = gb.omit_label.replace("{n}", &omitted.to_string());
        let marker = marker.replace("{key}", "files");
        result.push(marker);
    }

    result.join("\n")
}

/// Rough token estimate (words * 1.3, minimum 1 per non-empty content).
pub fn estimate_tokens(s: &str) -> usize {
    if s.trim().is_empty() {
        return 0;
    }
    let word_count = s.split_whitespace().count();
    ((word_count as f64 * 1.3) as usize).max(1)
}

/// Check if content is complete JSON.
pub fn is_complete_json(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let starts = trimmed.starts_with('{') || trimmed.starts_with('[');
    let ends = trimmed.ends_with('}') || trimmed.ends_with(']');
    if !starts || !ends {
        return false;
    }
    serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
}

/// JSON guard: if truncated and content is valid JSON, pass through or omit.
pub fn json_guard(
    original: &str,
    filtered: &str,
    truncated: bool,
    reduce_json: bool,
) -> (String, bool) {
    if !truncated || reduce_json {
        return (filtered.to_string(), truncated);
    }

    if is_complete_json(original) {
        const MAX_JSON_PASSTHROUGH: usize = 50_000;
        if original.len() <= MAX_JSON_PASSTHROUGH {
            return (original.to_string(), false);
        } else {
            return (
                format!("[JSON document omitted: {} bytes]", original.len()),
                true,
            );
        }
    }

    (filtered.to_string(), truncated)
}

/// Build the §FILTERED marker line.
pub fn build_filtered_marker(result: &FilterResult) -> String {
    let pct_str = format!("{:.1}", result.reduction_pct);
    format!(
        "§FILTERED {}: {} → {} lines ({}% ↓)",
        result.program, result.original_lines, result.filtered_lines, pct_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter_rules::CompiledMatchOutput;

    fn make_simple_filter() -> CompiledFilter {
        use regex::Regex;
        CompiledFilter {
            name: "test".to_string(),
            description: "test filter".to_string(),
            match_command: Regex::new("^test").unwrap(),
            priority: 0,
            strip_ansi: false,
            filter_stderr: false,
            reduce_json: false,
            replace: vec![],
            match_output: vec![],
            strip_lines: vec![Regex::new("^\\s*$").unwrap()],
            keep_lines: vec![],
            group_by: None,
            head_lines: None,
            tail_lines: None,
            max_lines: Some(50),
            on_empty: Some("test: ok".to_string()),
            user_config_key: None,
        }
    }

    #[test]
    fn test_apply_filter_strips_blank_lines() {
        let filter = make_simple_filter();
        let input = "line1\n\nline2\n\nline3\n";
        let result = apply_filter(&filter, input, false);
        assert_eq!(result.content, "line1\nline2\nline3");
    }

    #[test]
    fn test_apply_filter_max_lines() {
        let mut filter = make_simple_filter();
        filter.max_lines = Some(2);
        let input = "line1\nline2\nline3\nline4\nline5";
        let result = apply_filter(&filter, input, false);
        assert_eq!(result.content, "line1\nline2");
        assert!(result.truncated);
    }

    #[test]
    fn test_apply_filter_on_empty() {
        let filter = make_simple_filter();
        let input = "\n\n  \n";
        let result = apply_filter(&filter, input, false);
        assert_eq!(result.content, "test: ok");
    }

    #[test]
    fn test_apply_filter_match_output_collapse() {
        use regex::Regex;
        let mut filter = make_simple_filter();
        filter.strip_lines = vec![];
        filter.match_output = vec![CompiledMatchOutput {
            pattern: Regex::new("(?m)Finished").unwrap(),
            message: "cargo: ok".to_string(),
            unless: None,
        }];
        let input = "   Compiling app v0.1.0\n    Finished test in 2.34s\ntest result: ok";
        let result = apply_filter(&filter, input, false);
        assert_eq!(result.content, "cargo: ok");
        assert!(result.collapsed);
    }

    #[test]
    fn test_apply_filter_match_output_suppressed_on_failure() {
        use regex::Regex;
        let mut filter = make_simple_filter();
        filter.strip_lines = vec![];
        filter.match_output = vec![CompiledMatchOutput {
            pattern: Regex::new("(?m)Finished").unwrap(),
            message: "cargo: ok".to_string(),
            unless: None,
        }];
        let input = "    Finished test in 1.45s\ntest result: ok. 3 passed";
        let result = apply_filter(&filter, input, true);
        assert!(!result.collapsed);
        assert!(result.content.contains("Finished"));
    }

    #[test]
    fn test_apply_filter_match_output_unless() {
        use regex::Regex;
        let mut filter = make_simple_filter();
        filter.strip_lines = vec![];
        filter.match_output = vec![CompiledMatchOutput {
            pattern: Regex::new("(?m)Finished").unwrap(),
            message: "cargo: ok".to_string(),
            unless: Some(Regex::new("(?i)warning").unwrap()),
        }];
        let input = "warning: unused variable\n    Finished test";
        let result = apply_filter(&filter, input, false);
        assert!(!result.collapsed);
    }

    #[test]
    fn test_apply_replace() {
        use regex::Regex;
        let re = Regex::new("foo").unwrap();
        let input = "foo bar\nbaz foo";
        let result = apply_replace(&re, "REPLACED", input);
        assert_eq!(result, "REPLACED bar\nbaz REPLACED");
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hello world"), 2);
    }

    #[test]
    fn test_is_complete_json() {
        assert!(is_complete_json(r#"{"key": "value"}"#));
        assert!(is_complete_json(r#"[1, 2, 3]"#));
        assert!(!is_complete_json("not json"));
        assert!(!is_complete_json(r#"{"key": "#));
        assert!(!is_complete_json(""));
    }

    #[test]
    fn test_json_guard_passes_through_valid_json() {
        let json = r#"{"key": "value"}"#;
        let (result, truncated) = json_guard(json, "filtered", true, false);
        assert_eq!(result, json);
        assert!(!truncated);
    }

    #[test]
    fn test_json_guard_not_truncated() {
        let json = r#"{"key": "value"}"#;
        let (result, truncated) = json_guard(json, "filtered", false, false);
        assert_eq!(result, "filtered");
        assert!(!truncated);
    }

    #[test]
    fn test_json_guard_reduce_json_opt_out() {
        let json = r#"{"key": "value"}"#;
        let (result, truncated) = json_guard(json, "filtered", true, true);
        assert_eq!(result, "filtered");
        assert!(truncated);
    }

    #[test]
    fn test_build_filtered_marker() {
        let result = FilterResult {
            content: "ok".to_string(),
            original_lines: 100,
            filtered_lines: 5,
            original_tokens: 500,
            filtered_tokens: 25,
            program: "cargo".to_string(),
            reduction_pct: 95.0,
            truncated: false,
            collapsed: false,
        };
        let marker = build_filtered_marker(&result);
        assert!(marker.contains("§FILTERED cargo"));
        assert!(marker.contains("100 → 5 lines"));
        assert!(marker.contains("95.0%"));
    }
}
