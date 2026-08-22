// proxy/src/filter_rules.rs
//
// TOML filter rule schema, parsing, and inline test runner.
// Defines the data model for per-program output filter rules.

use regex::Regex;
use serde::Deserialize;

/// Top-level TOML structure for a filter file.
#[derive(Debug, Clone, Deserialize)]
pub struct FilterFile {
    /// Map of filter name → filter rule definition.
    pub filters: std::collections::HashMap<String, FilterRuleDef>,

    /// Inline conformance tests keyed by filter name.
    #[serde(default)]
    pub tests: std::collections::HashMap<String, Vec<FilterTestDef>>,
}

/// Raw TOML filter rule (before compilation).
#[derive(Debug, Clone, Deserialize)]
pub struct FilterRuleDef {
    /// Human-readable description.
    pub description: Option<String>,

    /// Regex pattern to match against the command string.
    pub match_command: String,

    /// Priority for tie-breaking when multiple filters match.
    #[serde(default)]
    pub priority: i32,

    /// Whether to strip ANSI codes (redundant if proxy already does it).
    #[serde(default = "default_true")]
    pub strip_ansi: bool,

    /// Whether to filter stderr content.
    #[serde(default)]
    pub filter_stderr: bool,

    /// Whether to pass through complete JSON unchanged.
    #[serde(default = "default_true")]
    pub reduce_json: bool,

    /// Line-by-line regex substitution rules.
    #[serde(default)]
    pub replace: Vec<ReplaceRuleDef>,

    /// Collapse to summary if output matches pattern.
    #[serde(default)]
    pub match_output: Vec<MatchOutputDef>,

    /// Strip lines matching any of these patterns.
    #[serde(default)]
    pub strip_lines_matching: Vec<String>,

    /// Keep ONLY lines matching at least one of these patterns.
    #[serde(default)]
    pub keep_lines_matching: Vec<String>,

    /// Group lines by regex key and cap per group.
    pub group_by: Option<GroupByDef>,

    /// Keep only the first N lines.
    pub head_lines: Option<usize>,

    /// Keep only the last N lines.
    pub tail_lines: Option<usize>,

    /// Hard cap on total lines.
    pub max_lines: Option<usize>,

    /// Fallback message if all output was stripped.
    pub on_empty: Option<String>,

    /// User-facing config key (e.g., "cargo", "npm").
    pub user_config_key: Option<String>,
}

fn default_true() -> bool {
    true
}

/// A line-by-line regex substitution rule (raw TOML).
#[derive(Debug, Clone, Deserialize)]
pub struct ReplaceRuleDef {
    pub pattern: String,
    pub replacement: String,
}

/// A collapse rule (raw TOML).
#[derive(Debug, Clone, Deserialize)]
pub struct MatchOutputDef {
    pub pattern: String,
    pub message: String,
    /// If this pattern also matches, suppress the collapse.
    pub unless: Option<String>,
}

/// Group-by configuration (raw TOML).
#[derive(Debug, Clone, Deserialize)]
pub struct GroupByDef {
    pub key: String,
    #[serde(default = "default_max_per_group")]
    pub max_per_group: usize,
    #[serde(default = "default_max_groups")]
    pub max_groups: usize,
    #[serde(default = "default_omit_label")]
    pub omit_label: String,
}

fn default_max_per_group() -> usize {
    3
}
fn default_max_groups() -> usize {
    10
}
fn default_omit_label() -> String {
    "... {n} more in {key}".to_string()
}

/// Inline test definition (raw TOML).
#[derive(Debug, Clone, Deserialize)]
pub struct FilterTestDef {
    pub name: String,
    pub input: Option<String>,
    pub expected: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    #[serde(default)]
    pub failed: bool,
    pub min_saved_percent: Option<u32>,
    #[serde(default)]
    pub draft: bool,
}

/// Compiled filter rule (regex compiled, ready to apply).
#[derive(Debug, Clone)]
pub struct CompiledFilter {
    pub name: String,
    pub description: String,
    pub match_command: Regex,
    pub priority: i32,
    pub strip_ansi: bool,
    #[allow(dead_code)]
    pub filter_stderr: bool,
    pub reduce_json: bool,
    pub replace: Vec<CompiledReplace>,
    pub match_output: Vec<CompiledMatchOutput>,
    pub strip_lines: Vec<Regex>,
    pub keep_lines: Vec<Regex>,
    pub group_by: Option<CompiledGroupBy>,
    pub head_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub on_empty: Option<String>,
    #[allow(dead_code)]
    pub user_config_key: Option<String>,
}

/// Compiled line-by-line substitution rule.
#[derive(Debug, Clone)]
pub struct CompiledReplace {
    pub pattern: Regex,
    pub replacement: String,
}

/// Compiled collapse rule.
#[derive(Debug, Clone)]
pub struct CompiledMatchOutput {
    pub pattern: Regex,
    pub message: String,
    pub unless: Option<Regex>,
}

/// Compiled group-by configuration.
#[derive(Debug, Clone)]
pub struct CompiledGroupBy {
    pub key: Regex,
    pub max_per_group: usize,
    pub max_groups: usize,
    pub omit_label: String,
}

/// Inline conformance test (compiled).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompiledFilterTest {
    pub name: String,
    pub input: Option<String>,
    pub expected: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub failed: bool,
    pub min_saved_percent: Option<u32>,
    pub draft: bool,
}

/// Error during filter compilation.
#[derive(Debug, Clone)]
pub struct FilterCompileError {
    pub filter_name: String,
    pub field: String,
    pub source: String,
}

impl std::fmt::Display for FilterCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to compile filter '{}', field '{}': {}",
            self.filter_name, self.field, self.source
        )
    }
}

/// Compile a raw FilterFile into CompiledFilters.
pub fn compile_filter_file(
    file: &FilterFile,
) -> Result<Vec<(CompiledFilter, Vec<CompiledFilterTest>)>, Vec<FilterCompileError>> {
    let mut errors = Vec::new();
    let mut results = Vec::new();

    for (name, def) in &file.filters {
        match compile_filter(name, def) {
            Ok(mut compiled) => {
                // Sort strip_lines and keep_lines for stable matching
                compiled.strip_lines.sort_by_key(|a| a.as_str().len());
                compiled
                    .keep_lines
                    .sort_by_key(|b| std::cmp::Reverse(b.as_str().len()));

                let tests = file
                    .tests
                    .get(name)
                    .map(|ts| ts.iter().map(compile_test).collect())
                    .unwrap_or_default();

                results.push((compiled, tests));
            }
            Err(errs) => errors.extend(errs),
        }
    }

    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}

/// Compile a single filter rule definition.
fn compile_filter(
    name: &str,
    def: &FilterRuleDef,
) -> Result<CompiledFilter, Vec<FilterCompileError>> {
    let mut errors = Vec::new();

    let match_command = compile_regex(name, "match_command", &def.match_command, &mut errors);

    let replace: Vec<CompiledReplace> = def
        .replace
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            compile_regex_opt(
                name,
                &format!("replace[{i}].pattern"),
                &r.pattern,
                &mut errors,
            )
            .map(|pattern| CompiledReplace {
                pattern,
                replacement: r.replacement.clone(),
            })
        })
        .collect();

    let match_output: Vec<CompiledMatchOutput> = def
        .match_output
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            compile_regex_opt(
                name,
                &format!("match_output[{i}].pattern"),
                &m.pattern,
                &mut errors,
            )
            .map(|pattern| CompiledMatchOutput {
                pattern,
                message: m.message.clone(),
                unless: m.unless.as_ref().and_then(|u| {
                    compile_regex_opt(name, &format!("match_output[{i}].unless"), u, &mut errors)
                }),
            })
        })
        .collect();

    let strip_lines: Vec<Regex> = def
        .strip_lines_matching
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            compile_regex_opt(name, &format!("strip_lines_matching[{i}]"), p, &mut errors)
        })
        .collect();

    let keep_lines: Vec<Regex> = def
        .keep_lines_matching
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            compile_regex_opt(name, &format!("keep_lines_matching[{i}]"), p, &mut errors)
        })
        .collect();

    let group_by = def.group_by.as_ref().and_then(|gb| {
        compile_regex_opt(name, "group_by.key", &gb.key, &mut errors).map(|key| CompiledGroupBy {
            key,
            max_per_group: gb.max_per_group,
            max_groups: gb.max_groups,
            omit_label: gb.omit_label.clone(),
        })
    });

    if errors.is_empty() {
        let match_command = match_command.unwrap();
        Ok(CompiledFilter {
            name: name.to_string(),
            description: def.description.clone().unwrap_or_default(),
            match_command,
            priority: def.priority,
            strip_ansi: def.strip_ansi,
            filter_stderr: def.filter_stderr,
            reduce_json: def.reduce_json,
            replace,
            match_output,
            strip_lines,
            keep_lines,
            group_by,
            head_lines: def.head_lines,
            tail_lines: def.tail_lines,
            max_lines: def.max_lines,
            on_empty: def.on_empty.clone(),
            user_config_key: def.user_config_key.clone(),
        })
    } else {
        Err(errors)
    }
}

fn compile_test(def: &FilterTestDef) -> CompiledFilterTest {
    CompiledFilterTest {
        name: def.name.clone(),
        input: def.input.clone(),
        expected: def.expected.clone(),
        stdout: def.stdout.clone(),
        stderr: def.stderr.clone(),
        failed: def.failed,
        min_saved_percent: def.min_saved_percent,
        draft: def.draft,
    }
}

fn compile_regex(
    filter_name: &str,
    field: &str,
    pattern: &str,
    errors: &mut Vec<FilterCompileError>,
) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(re) => Some(re),
        Err(e) => {
            errors.push(FilterCompileError {
                filter_name: filter_name.to_string(),
                field: field.to_string(),
                source: e.to_string(),
            });
            None
        }
    }
}

fn compile_regex_opt(
    filter_name: &str,
    field: &str,
    pattern: &str,
    errors: &mut Vec<FilterCompileError>,
) -> Option<Regex> {
    compile_regex(filter_name, field, pattern, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_valid_toml() {
        let toml_str = r#"
[filters.cargo]
description = "Compact cargo output"
match_command = "^cargo\\s+(build|test|check)\\b"
strip_ansi = true
max_lines = 100
on_empty = "cargo: ok"
strip_lines_matching = ["^\\s*$", "^\\s*Compiling "]
keep_lines_matching = ["^error", "^warning"]

[[tests.cargo]]
name = "success collapses"
input = "Finished test\ntest result: ok"
expected = "cargo: ok"
"#;
        let file: FilterFile = toml::from_str(toml_str).unwrap();
        let result = compile_filter_file(&file);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
        let compiled = result.unwrap();
        assert_eq!(compiled.len(), 1);
        assert_eq!(compiled[0].0.name, "cargo");
        assert_eq!(compiled[0].1.len(), 1);
    }

    #[test]
    fn test_compile_invalid_regex() {
        let toml_str = r#"
[filters.bad]
match_command = "[invalid"
"#;
        let file: FilterFile = toml::from_str(toml_str).unwrap();
        let result = compile_filter_file(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].field.contains("match_command"));
    }

    #[test]
    fn test_compile_replace_rules() {
        let toml_str = r#"
[filters.test]
match_command = "^test"
replace = [
    { pattern = "foo", replacement = "bar" },
    { pattern = "\\d+", replacement = "N" }
]
"#;
        let file: FilterFile = toml::from_str(toml_str).unwrap();
        let result = compile_filter_file(&file).unwrap();
        assert_eq!(result[0].0.replace.len(), 2);
        assert_eq!(result[0].0.replace[0].replacement, "bar");
    }

    #[test]
    fn test_compile_match_output() {
        let toml_str = r#"
[filters.test]
match_command = "^test"
match_output = [
    { pattern = "ok", message = "passed", unless = "fail" }
]
"#;
        let file: FilterFile = toml::from_str(toml_str).unwrap();
        let result = compile_filter_file(&file).unwrap();
        assert_eq!(result[0].0.match_output.len(), 1);
        assert!(result[0].0.match_output[0].unless.is_some());
    }
}
