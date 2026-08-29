use super::*;
use crate::config::CleanCtxConfig;

// P3-3: Initialize handler registry before running tests
#[test]
fn setup_registry() {
    crate::mcp::tools::setup_handler_registry_for_tests();
}

#[test]
fn resolve_fidelity_prefers_explicit_arg() {
    let config = CleanCtxConfig::default();
    assert_eq!(
        resolve_fidelity(Some("high"), Some("ts"), &config),
        Fidelity::High
    );
}

#[test]
fn resolve_fidelity_uses_extension_override() {
    let mut config = CleanCtxConfig::default();
    config
        .fidelity_overrides
        .insert("ts".to_string(), crate::compression::Fidelity::High);
    assert_eq!(resolve_fidelity(None, Some("ts"), &config), Fidelity::High);
}

#[test]
fn resolve_fidelity_falls_back_to_default() {
    let config = CleanCtxConfig {
        default_fidelity: crate::compression::Fidelity::Medium,
        ..Default::default()
    };
    assert_eq!(
        resolve_fidelity(None, Some("cs"), &config),
        Fidelity::Medium
    );
}

#[test]
fn resolve_fidelity_hard_fallback_to_low() {
    let config = CleanCtxConfig::default();
    assert_eq!(resolve_fidelity(None, None, &config), Fidelity::Low);
}

#[test]
fn parse_fidelity_arg_rejects_typo() {
    let params = serde_json::json!({
        "arguments": { "fidelity": "hihg" }
    });
    let config = CleanCtxConfig::default();
    let result = parse_fidelity_arg(&Value::Null, &params, &config);
    assert!(result.is_err());
}

// ── Gap 4 fix: workspaceRoot schema presence (Phase 4) ─────────────

/// Every compression tool that resolves paths must advertise `workspaceRoot`
/// in its schema so clients (and the LLM) can pass a multi-repo root.
#[test]
fn schema_includes_workspace_root_on_path_resolving_tools() {
    let tools = tool_list();
    let tools_by_name: std::collections::HashMap<&str, &serde_json::Value> = tools
        .iter()
        .map(|t| (t["name"].as_str().unwrap_or(""), t))
        .collect();

    // Tools that accept absolute file paths should expose workspaceRoot.
    for name in [
        "compress_code_context",
        "diff_code_context",
        "delta_code_context",
        "restore_context",
        "provide_code_context",
    ] {
        let tool = tools_by_name
            .get(name)
            .unwrap_or_else(|| panic!("missing tool {} in tool_list", name));
        let has_root = tool["inputSchema"]["properties"]["workspaceRoot"].is_object();
        assert!(
            has_root,
            "tool '{}' schema is missing workspaceRoot (Gap 4)",
            name
        );
    }
}

/// Symbol targeting: `provide_code_context` schema advertises the optional
/// `focusMethods` array parameter for targeted Edit-fidelity rendering.
#[test]
fn schema_provide_code_context_includes_focus_methods() {
    let tools = tool_list();
    let provide = tools
        .iter()
        .find(|t| t["name"] == "provide_code_context")
        .unwrap_or_else(|| panic!("missing provide_code_context in tool_list"));

    let focus_methods = &provide["inputSchema"]["properties"]["focusMethods"];
    assert!(
        focus_methods.is_object(),
        "provide_code_context schema is missing focusMethods"
    );
    assert_eq!(
        focus_methods["type"], "array",
        "focusMethods should be an array"
    );
    assert_eq!(
        focus_methods["items"]["type"], "string",
        "focusMethods items should be strings"
    );
}

/// Gap 4 fix: fidelity enums include edit/verbatim where applicable.
#[test]
fn schema_fidelity_enums_include_edit_and_verbatim() {
    let tools = tool_list();
    for tool in &tools {
        let name = tool["name"].as_str().unwrap_or("");
        let Some(fid) = tool["inputSchema"]["properties"]["fidelity"].as_object() else {
            continue; // tool has no fidelity arg
        };
        let Some(enum_vals) = fid.get("enum").and_then(|e| e.as_array()) else {
            continue; // no enum constraint — skip (not all tools need it)
        };
        let vals: Vec<&str> = enum_vals.iter().filter_map(|v| v.as_str()).collect();
        assert!(
            vals.contains(&"edit") && vals.contains(&"verbatim"),
            "tool '{}' fidelity enum must include 'edit' and 'verbatim', got: {:?}",
            name,
            vals
        );
    }
}

// ---------- F-21: diff_code_context cache-hit fast path ----------

/// F-21: calling `diff_code_context` on an unchanged file should
/// return a "No changes" message without re-parsing the source.
#[test]
fn diff_code_context_unchanged_file_skips_reparse() {
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("sample.ts");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "export class Foo {{ bar(): void {{}} }}").unwrap();
    }

    let mut cache = crate::cache::LocalStateCache::new();

    // Read source for the diff handler (A-08: source_cache integration)
    let source1 = std::fs::read_to_string(&path).unwrap();

    // First call: stores baseline.
    let result1 = diff_code_context_handler(path.clone(), &source1, &mut cache, Fidelity::Low)
        .expect("first diff call should succeed");
    assert!(
        result1.contains("No baseline snapshot"),
        "first call should store baseline, got: {}",
        result1
    );

    // Second call (unchanged file): should short-circuit.
    let source2 = std::fs::read_to_string(&path).unwrap();
    let result2 = diff_code_context_handler(path.clone(), &source2, &mut cache, Fidelity::Low)
        .expect("second diff call should succeed");
    assert!(
        result2.contains("No changes"),
        "second call on unchanged file should say 'No changes', got: {}",
        result2
    );
}

/// F-21: calling `diff_code_context` after modifying the file should
/// produce a real diff (not the "No changes" fast path).
#[test]
fn diff_code_context_changed_file_produces_diff() {
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("change.ts");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "export class Alpha {{ run(): void {{}} }}").unwrap();
    }

    let mut cache = crate::cache::LocalStateCache::new();

    let source_before = std::fs::read_to_string(&path).unwrap();

    // First call: stores baseline.
    let _ = diff_code_context_handler(path.clone(), &source_before, &mut cache, Fidelity::Low)
        .expect("first diff call should succeed");

    // Modify the file.
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "export class Alpha {{ run(): void {{}} }}\nexport class Beta {{ go(): string {{ return ''; }} }}"
        )
        .unwrap();
    }

    let source_after = std::fs::read_to_string(&path).unwrap();

    // Second call (changed file): should produce a real diff.
    let result = diff_code_context_handler(path, &source_after, &mut cache, Fidelity::Low)
        .expect("diff call on changed file should succeed");
    assert!(
        result.contains("AST Diff") && !result.contains("No changes"),
        "changed file should produce a real diff, not a no-change message, got: {}",
        result
    );
}

// ── P3-21: Tool name cross-verification test ────────────────────────

/// P3-21: Verify that `tool_list()` and the handler registry contain
/// matching tool names. A tool added to one but not the other would
/// either not appear in the list or not be dispatchable.
#[test]
fn p3_21_tool_names_match_tool_list_and_registry() {
    use crate::mcp::tool_handlers::registry::create_default_registry;

    // Get tool names from tool_list()
    let tool_list_names: std::collections::HashSet<String> = tool_list()
        .into_iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str().map(str::to_string)))
        .collect();

    // Get tool names from the registry
    let registry_names: std::collections::HashSet<String> = create_default_registry()
        .tool_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Get inline tool names (tools dispatched directly in tools.rs)
    let inline_names: std::collections::HashSet<String> = {
        let mut set = std::collections::HashSet::new();
        // Inline tools from dispatch_tools_call() in tools.rs
        set.insert("graph_search".to_string());
        set.insert("graph_query".to_string());
        set.insert("graph_trace".to_string());
        set.insert("get_architecture".to_string());
        set.insert("get_cbm_status".to_string());
        set.insert("cbm_proxy".to_string());
        set.insert("list_projects".to_string());
        set
    };

    // Verify: every tool in tool_list is either inline or in registry (or both)
    for name in &tool_list_names {
        let in_inline = inline_names.contains(name);
        let in_registry = registry_names.contains(name);
        assert!(
            in_inline || in_registry,
            "P3-21: Tool '{}' in tool_list() is neither inline nor in registry!",
            name
        );
    }

    // Verify: no tool is in both inline and registry (double-fire hazard)
    let intersection: Vec<_> = inline_names.intersection(&registry_names).collect();
    assert!(
        intersection.is_empty(),
        "P3-21: Tools in both inline and registry (double-fire hazard): {:?}",
        intersection
    );

    // Verify: union of inline + registry equals tool_list
    let union: std::collections::HashSet<String> =
        inline_names.union(&registry_names).cloned().collect();
    assert_eq!(
        tool_list_names,
        union,
        "P3-21: tool_list() names don't match inline + registry union.\n\
         In tool_list but not in union: {:?}\n\
         In union but not in tool_list: {:?}",
        tool_list_names.difference(&union).collect::<Vec<_>>(),
        union.difference(&tool_list_names).collect::<Vec<_>>()
    );
}
