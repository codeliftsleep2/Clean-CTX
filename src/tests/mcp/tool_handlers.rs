// src/tests/mcp/tool_handlers.rs
//
// Tests for tool handler helper functions and handler robustness.
// Note: The handlers themselves call send_response (stdout), so we
// test the pure helper functions and verify handlers don't panic.
//
// Phase 6 IR-first audit: Added integration tests verifying response
// format compliance (content[0].text = LLM text, "ir" = hierarchical,
// "pretty" = fallback), cache invalidation, and angular/spring
// meta-layer abbreviation in compiled output.

use crate::compression::Fidelity;
use crate::mcp::tool_handlers::core::{
    contract_fields, contract_fields_focused, handle_compress_code_context,
};
use crate::mcp::tools::{dispatch_tools_call, parse_fidelity_arg, resolve_fidelity};
use serde_json::json;

// ── resolve_fidelity tests ──
// Signature: resolve_fidelity(explicit: Option<&str>, ext: Option<&str>, config: &CleanCtxConfig) -> Fidelity

#[test]
fn resolve_fidelity_explicit_low() {
    let result = resolve_fidelity(Some("low"), None, &crate::config::CleanCtxConfig::default());
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_explicit_medium() {
    let result = resolve_fidelity(
        Some("medium"),
        None,
        &crate::config::CleanCtxConfig::default(),
    );
    assert_eq!(result, Fidelity::Medium);
}

#[test]
fn resolve_fidelity_explicit_high() {
    let result = resolve_fidelity(
        Some("high"),
        None,
        &crate::config::CleanCtxConfig::default(),
    );
    assert_eq!(result, Fidelity::High);
}

#[test]
fn resolve_fidelity_none_uses_default() {
    let result = resolve_fidelity(None, None, &crate::config::CleanCtxConfig::default());
    // Default fidelity is "low" per config
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_invalid_string_falls_back_to_default() {
    let result = resolve_fidelity(
        Some("bogus"),
        None,
        &crate::config::CleanCtxConfig::default(),
    );
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_extension_override() {
    let mut config = crate::config::CleanCtxConfig::default();
    config
        .fidelity_overrides
        .insert("ts".to_string(), crate::compression::Fidelity::High);
    let result = resolve_fidelity(None, Some("ts"), &config);
    assert_eq!(result, Fidelity::High);
}

// ── parse_fidelity_arg tests ──

#[test]
fn parse_fidelity_arg_with_explicit_value() {
    let params = json!({ "arguments": { "fidelity": "high" } });
    let config = crate::config::CleanCtxConfig::default();
    let result = parse_fidelity_arg(&json!(1), &params, &config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Fidelity::High);
}

#[test]
fn parse_fidelity_arg_missing_defaults() {
    let params = json!({ "arguments": {} });
    let config = crate::config::CleanCtxConfig::default();
    let result = parse_fidelity_arg(&json!(1), &params, &config);
    assert!(result.is_ok());
}

#[test]
fn parse_fidelity_arg_invalid_returns_error() {
    let params = json!({ "arguments": { "fidelity": "turbo" } });
    let config = crate::config::CleanCtxConfig::default();
    let result = parse_fidelity_arg(&json!(1), &params, &config);
    assert!(result.is_err());
}

// ── Handler smoke tests (verify no panic) ──

#[test]
fn handle_context_stats_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    dispatch_tools_call(&id, "context_stats", &params, &state);
}

#[test]
fn handle_list_sessions_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    dispatch_tools_call(&id, "list_sessions", &params, &state);
}

#[test]
fn handle_context_history_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    dispatch_tools_call(&id, "context_history", &params, &state);
}

#[test]
fn handle_save_context_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic (returns error for nonexistent file, but doesn't panic)
    dispatch_tools_call(&id, "save_context", &params, &state);
}

#[test]
fn handle_restore_context_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic
    dispatch_tools_call(&id, "restore_context", &params, &state);
}

#[test]
fn handle_purge_old_deltas_smoke() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "days": 30 } });
    // Should not panic
    dispatch_tools_call(&id, "purge_old_deltas", &params, &state);
}

// ── IR-first integration tests ──

#[test]
fn handle_compress_code_context_with_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    // Use nonexistent file — should fall back to text pipeline gracefully
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic
    dispatch_tools_call(&id, "compress_code_context", &params, &state);
}

#[test]
fn handle_delta_code_context_no_baseline() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic — stores baseline IR, returns "no baseline" message
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
}

// ── Cache breakpoint injection regression tests ─────────────────────
// These tests guard against regressions in the cache breakpoint wiring
// added during the smart-cache FAANG audit.

#[test]
fn inject_baseline_breakpoint_helper_injects_hint() {
    use crate::mcp::tool_helpers::inject_baseline_breakpoint;
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "compressed output" }] }
    });

    inject_baseline_breakpoint(&mut response, &state, "compressed output");

    assert!(
        response.get("_meta").is_none(),
        "_meta should NOT be at response root"
    );
    let hints = &response["result"]["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0]["region"], "baseline");
    assert_eq!(breakpoints[0]["ttl"], "1h");
    assert!(
        breakpoints[0]["breaker"]
            .as_str()
            .unwrap()
            .starts_with("bl_")
    );
    assert_eq!(state.cache_metrics_lock().misses, 1);
}

#[test]
fn inject_tail_breakpoint_helper_injects_hint() {
    use crate::mcp::tool_helpers::inject_tail_breakpoint;
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "delta output" }] }
    });

    inject_tail_breakpoint(&mut response, &state);

    assert!(
        response.get("_meta").is_none(),
        "_meta should NOT be at response root"
    );
    let hints = &response["result"]["_meta"]["cache_hints"];
    let breakpoints = hints["breakpoints"].as_array().unwrap();
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0]["region"], "tail");
    assert_eq!(breakpoints[0]["ttl"], "5m");
    assert_eq!(breakpoints[0]["breaker"], "rolling");
    assert_eq!(
        state.cache_metrics_lock().breakpoints.get("tail").unwrap(),
        "ephemeral"
    );
}

#[test]
fn inject_baseline_breakpoint_skips_when_cache_disabled() {
    use crate::mcp::tool_helpers::inject_baseline_breakpoint;
    let mut config = crate::tests::test_config();
    config.cache.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "compressed output" }] }
    });

    inject_baseline_breakpoint(&mut response, &state, "compressed output");

    assert!(response["result"].get("_meta").is_none());
    assert_eq!(state.cache_metrics_lock().misses, 0);
}

#[test]
fn inject_tail_breakpoint_skips_when_cache_disabled() {
    use crate::mcp::tool_helpers::inject_tail_breakpoint;
    let mut config = crate::tests::test_config();
    config.cache.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "content": [{ "type": "text", "text": "delta output" }] }
    });

    inject_tail_breakpoint(&mut response, &state);

    assert!(response["result"].get("_meta").is_none());
    assert_eq!(state.cache_metrics_lock().misses, 0);
}

// REGRESSION: The cached-IR fast path in `delta_code_context` must not panic.
#[test]
fn delta_code_context_cached_ir_path_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
}

#[test]
fn handle_apply_delta_no_baseline() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = json!(1);
    let params = json!({ "arguments": { "delta": { "file": "α1", "from": 1, "to": 2, "ops": { "+": [], "-": [], "~": [] } } } });
    // Should not panic — returns "UnknownFile" error
    dispatch_tools_call(&id, "apply_delta", &params, &state);
}

// ── render_llm integration tests ──

#[test]
fn render_hierarchical_for_llm_typescript_class() {
    use crate::ir::*;
    let mut class = ClassNode {
        id: "C1".into(),
        name: "UserListComponent".into(),
        methods: vec![],
        fields: vec![FieldNode {
            id: "F1".into(),
            name: "users".into(),
            field_type: Some("$s[]".into()),
        }],
        class_flags: None,
        extends: Some("BaseListComponent".into()),
        implements: vec!["OnInit".into()],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    class.methods.push(MethodNode {
        id: "M1".into(),
        name: "ngOnInit".into(),
        params: vec![],
        return_type: None,
        flags: Some(vec!["IF".into()]),
        patterns: vec![],
        body: None,
        body_start: None,
        body_end: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: None,
        execution_context: None,
    });
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![vec!["IM1".into(), "./core".into(), "OnInit".into()]],
        type_aliases: vec![],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Phase 6 IR-first format: SCHEMA v2 header with structural markers
    assert!(result.contains("SCHEMA v2"));
    assert!(result.contains("// ── UserListComponent ──"));
    assert!(result.contains("X BaseListComponent"));
    assert!(result.contains("I OnInit"));
    assert!(result.contains("F users:$s[]"));
    assert!(result.contains("M ngOnInit"));
    assert!(result.contains("fl:IF"));
    assert!(result.contains("$ IM1 ./core [OnInit]"));
}

#[test]
fn render_hierarchical_for_llm_spring_boot_class() {
    use crate::ir::*;
    let mut class = ClassNode {
        id: "C1".into(),
        name: "UserController".into(),
        methods: vec![],
        fields: vec![FieldNode {
            id: "F1".into(),
            name: "userService".into(),
            field_type: Some("UserService".into()),
        }],
        class_flags: None,
        extends: Some("BaseController".into()),
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    // overloaded find methods
    let m1 = MethodNode {
        id: "M1".into(),
        name: "find".into(),
        params: vec![vec!["P1".into(), "$n".into(), "id".into()]],
        return_type: None,
        flags: Some(vec!["RET".into()]),
        patterns: vec![],
        body: None,
        body_start: None,
        body_end: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: None,
        execution_context: None,
    };
    let m2 = MethodNode {
        id: "M2".into(),
        name: "find".into(),
        params: vec![
            vec!["P1".into(), "$n".into(), "name".into()],
            vec!["P2".into(), "$n".into(), "age".into()],
        ],
        return_type: None,
        flags: Some(vec!["RET".into(), "IF".into()]),
        patterns: vec![],
        body: None,
        body_start: None,
        body_end: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: None,
        execution_context: None,
    };
    class.methods.push(m1);
    class.methods.push(m2);
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![vec![
            "IM1".into(),
            "org.springframework.web".into(),
            "*".into(),
        ]],
        type_aliases: vec![
            vec!["@rest".into(), "UserController".into()],
            vec!["@map".into(), "GET /users".into()],
        ],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    // Abbreviated meta-layer ops (Phase 2-4)
    assert!(result.contains("@rest"));
    assert!(result.contains("@map"));
    // Overloaded method disambiguation (Fix B)
    assert!(result.contains("M find(+1)"));
    assert!(result.contains("M find(+2)"));
    // Params shown in Medium fidelity
    assert!(result.contains("p:id:$n"));
    assert!(result.contains("p:name:$n age:$n"));
}

#[test]
fn render_hierarchical_for_llm_angular_class() {
    use crate::ir::*;
    let class = ClassNode {
        id: "C1".into(),
        name: "AppComponent".into(),
        methods: vec![],
        fields: vec![],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![],
        type_aliases: vec![
            vec!["@cmp".into(), "AppComponent".into()],
            vec!["@sel".into(), "app-root".into()],
        ],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Abbreviated Angular meta-layer ops
    assert!(result.contains("@cmp"));
    assert!(result.contains("@sel"));
}

#[test]
fn render_hierarchical_for_llm_empty_hir_produces_header() {
    use crate::ir::*;
    let hir = HierarchicalIR {
        classes: vec![],
        imports: vec![],
        type_aliases: vec![],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Always has schema header even with empty HIR
    assert!(result.starts_with("// SCHEMA v2"));
}

#[test]
fn render_hierarchical_for_llm_fidelity_low_compact_fields() {
    use crate::ir::*;
    let class = ClassNode {
        id: "C1".into(),
        name: "Data".into(),
        methods: vec![],
        fields: vec![
            FieldNode {
                id: "F1".into(),
                name: "x".into(),
                field_type: Some("$n".into()),
            },
            FieldNode {
                id: "F2".into(),
                name: "y".into(),
                field_type: Some("$n".into()),
            },
            FieldNode {
                id: "F3".into(),
                name: "label".into(),
                field_type: Some("$s".into()),
            },
        ],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![],
        type_aliases: vec![],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Low fidelity: space-separated fields on one line
    assert!(result.contains("F x:$n y:$n label:$s"));
    assert_eq!(result.matches("\nF ").count(), 1);
}

#[test]
fn render_hierarchical_for_llm_fidelity_medium_one_field_per_line() {
    use crate::ir::*;
    let class = ClassNode {
        id: "C1".into(),
        name: "Data".into(),
        methods: vec![],
        fields: vec![
            FieldNode {
                id: "F1".into(),
                name: "x".into(),
                field_type: Some("$n".into()),
            },
            FieldNode {
                id: "F2".into(),
                name: "y".into(),
                field_type: Some("$n".into()),
            },
        ],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![],
        type_aliases: vec![],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    // Medium fidelity: one field per line
    assert!(result.contains("F x:$n\n"));
    assert!(result.contains("F y:$n\n"));
    assert_eq!(result.matches("\nF ").count(), 2);
}

#[test]
fn render_hierarchical_for_llm_injects_do_not_panic() {
    use crate::ir::*;
    let class = ClassNode {
        id: "C1".into(),
        name: "Service".into(),
        methods: vec![],
        fields: vec![],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec!["DepA".into(), "DepB".into()],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![],
        type_aliases: vec![],
    };
    // Should not panic — injects are structural (pattern-level), not rendered
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── Service ──"));
}

// ── LLM text cache integration tests ──

#[test]
fn mcp_state_llm_text_cache_insert_and_read() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    // Insert into cache
    state
        .llm_text_cache_lock()
        .insert("α1".to_string(), "// SCHEMA v2\n// ── Foo ──\n".to_string());
    // Read from cache
    let cache_guard = state.llm_text_cache_lock();
    let cached = cache_guard.get("α1");
    assert!(cached.is_some());
    assert!(cached.unwrap().contains("SCHEMA v2"));
    assert!(cached.unwrap().contains("Foo"));
}

#[test]
fn mcp_state_llm_text_cache_miss_returns_none() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    assert!(!state.llm_text_cache_lock().contains_key("nonexistent"));
}

#[test]
fn mcp_state_llm_text_cache_clear_on_new() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    // Fresh state should have empty cache
    assert!(state.llm_text_cache_lock().is_empty());
}

// ── Micro-opcode expanded table verification (Phase 8) ──

#[test]
fn micro_opcode_table_includes_new_markers() {
    use crate::compression::micro_opcodes::micro_opcode_table;
    let table = micro_opcode_table();
    // Must have all 6 entries (6 replace patterns, some share opcodes like §C)
    assert_eq!(table.len(), 6);
    // Verify each new marker exists
    let patterns: Vec<&str> = table.iter().map(|(_, p, _)| *p).collect();
    assert!(patterns.contains(&"⊕guard"), "Should have ⊕guard pattern");
    assert!(patterns.contains(&"⊕loop"), "Should have ⊕loop pattern");
    assert!(patterns.contains(&"⊕⇒"), "Should have ⊕⇒ pattern");
    // Verify each new replacement
    let replacements: Vec<&str> = table.iter().map(|(_, _, r)| *r).collect();
    assert!(replacements.contains(&"§I"), "Should have §I replacement");
    assert!(replacements.contains(&"§L"), "Should have §L replacement");
    assert!(replacements.contains(&"§E"), "Should have §E replacement");
}

// ── Regression: relative path resolution ─────────────────────────
// FAANG audit follow-up: ensure all handlers resolve relative paths
// via resolve_file_path() instead of using bare PathBuf::from().
// This prevents "file not found" errors when clients pass relative
// paths (e.g., "src/cbm/client.rs") to the MCP pipeline.

#[test]
fn resolve_file_path_absolute_is_passthrough() {
    use crate::mcp::tool_helpers::resolve_file_path;
    let abs = if cfg!(windows) {
        "C:\\projects\\foo\\bar.rs".to_string()
    } else {
        "/projects/foo/bar.rs".to_string()
    };
    let result = resolve_file_path(&abs, None);
    assert_eq!(result, abs);
}

#[test]
fn resolve_file_path_relative_joins_cwd() {
    use crate::mcp::tool_helpers::resolve_file_path;
    let cwd = std::env::current_dir().unwrap_or_default();
    let result = resolve_file_path("src/lib.rs", None);
    let expected = cwd.join("src/lib.rs").to_string_lossy().into_owned();
    assert_eq!(result, expected);
}

#[test]
fn resolve_file_path_with_workspace_root() {
    use crate::mcp::tool_helpers::resolve_file_path;
    let abs_root = if cfg!(windows) {
        "D:\\myproject"
    } else {
        "/home/user/myproject"
    };
    let result = resolve_file_path("src/main.ts", Some(abs_root));
    let expected = if cfg!(windows) {
        "D:\\myproject\\src\\main.ts"
    } else {
        "/home/user/myproject/src/main.ts"
    };
    assert_eq!(result.replace('\\', "/"), expected.replace('\\', "/"));
}

#[test]
fn handle_compress_code_context_accepts_relative_path() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    handle_compress_code_context(&id, &params, &state);
}

// ── M-8 regression: compress_workspace honors workspaceRoot ─────────
// The schema advertises `workspaceRoot`; the dispatch handler must pass
// it through to `resolve_file_path_checked` (not pin to CWD). This smoke
// test exercises the dispatch path with a workspaceRoot arg to ensure
// the handler reads it without panicking.
#[test]
fn handle_compress_workspace_accepts_workspace_root() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "directoryPath": "src",
            "workspaceRoot": ".",
            "fidelity": "low"
        }
    });
    // Should not panic — the handler reads workspaceRoot and resolves
    // directoryPath against it (M-8 regression).
    dispatch_tools_call(&id, "compress_workspace", &params, &state);
}

#[test]
fn handle_delta_code_context_accepts_relative_path() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
}

#[test]
fn handle_diff_code_context_accepts_relative_path() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    dispatch_tools_call(&id, "diff_code_context", &params, &state);
}

#[test]
fn handle_restore_context_accepts_relative_path() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    dispatch_tools_call(&id, "restore_context", &params, &state);
}

#[test]
fn handle_provide_code_context_accepts_relative_path() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "intent": "overview" } });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

// ── Edit Mode response contract field tests (Gap 5/3/6 fixes) ─────

/// Gap 5 fix: structural-only fidelities report `content_kind == "skeleton"`
/// and no byte-exact regions.
#[test]
fn contract_fields_low_is_skeleton() {
    let (kind, byte_exact) = contract_fields(Fidelity::Low);
    assert_eq!(kind, "skeleton");
    assert!(
        byte_exact.is_empty(),
        "Low must not claim byte-exact regions"
    );
}

#[test]
fn contract_fields_medium_is_skeleton() {
    let (kind, byte_exact) = contract_fields(Fidelity::Medium);
    assert_eq!(kind, "skeleton");
    assert!(byte_exact.is_empty());
}

#[test]
fn contract_fields_high_is_skeleton() {
    let (kind, byte_exact) = contract_fields(Fidelity::High);
    assert_eq!(kind, "skeleton");
    assert!(byte_exact.is_empty());
}

/// Gap 3/Gap 5 fix: Edit reports verbatim method bodies as the byte-exact
/// region, matching the `byte_exact` promise in the SYSTEM_PROMPT.
#[test]
fn contract_fields_edit_reports_method_bodies() {
    let (kind, byte_exact) = contract_fields(Fidelity::Edit);
    assert_eq!(kind, "skeleton_with_verbatim_bodies");
    assert_eq!(byte_exact, vec!["method_bodies"]);
}

/// Gap 3/Gap 5 fix: Verbatim reports the entire document as byte-exact.
#[test]
fn contract_fields_verbatim_is_document() {
    let (kind, byte_exact) = contract_fields(Fidelity::Verbatim);
    assert_eq!(kind, "verbatim_document");
    assert_eq!(byte_exact, vec!["document"]);
}

// ── contract_fields_focused tests (Symbol Targeting) ──────────────

/// None focus at Edit → identical to unfocused (every body byte-exact).
#[test]
fn contract_fields_focused_none_edit_is_all_bodies() {
    let (kind, byte_exact) = contract_fields_focused(Fidelity::Edit, None);
    assert_eq!(kind, "skeleton_with_verbatim_bodies");
    assert_eq!(byte_exact, vec!["method_bodies"]);
}

/// Empty focus set at Edit → zero method bodies are byte-exact.
/// Must report `"skeleton"` with no byte-exact regions, otherwise the
/// LLM would attempt replace_in_file SEARCH on bodies that don't exist.
#[test]
fn contract_fields_focused_empty_set_edit_is_skeleton() {
    let focus = std::collections::HashSet::new();
    let (kind, byte_exact) = contract_fields_focused(Fidelity::Edit, Some(&focus));
    assert_eq!(kind, "skeleton");
    assert!(byte_exact.is_empty());
}

/// Non-empty focus set at Edit → only focused method bodies are byte-exact.
#[test]
fn contract_fields_focused_some_edit_is_focused_bodies() {
    let focus = std::collections::HashSet::from(["GetOrgUnitDic".to_string()]);
    let (kind, byte_exact) = contract_fields_focused(Fidelity::Edit, Some(&focus));
    assert_eq!(kind, "skeleton_with_focused_verbatim_bodies");
    assert_eq!(byte_exact, vec!["focused_method_bodies"]);
}

/// Focus is silently ignored at non-Edit fidelities — structural only.
#[test]
fn contract_fields_focused_non_edit_ignores_focus() {
    let focus = std::collections::HashSet::from(["doWork".to_string()]);
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let (kind, byte_exact) = contract_fields_focused(fidelity, Some(&focus));
        assert_eq!(kind, "skeleton");
        assert!(byte_exact.is_empty());
    }
}

/// Verbatim always reports the entire document as byte-exact.
#[test]
fn contract_fields_focused_verbatim_is_document() {
    let focus = std::collections::HashSet::from(["doWork".to_string()]);
    let (kind, byte_exact) = contract_fields_focused(Fidelity::Verbatim, Some(&focus));
    assert_eq!(kind, "verbatim_document");
    assert_eq!(byte_exact, vec!["document"]);
}

/// `contract_fields` is the None-focus specialization of `contract_fields_focused`.
#[test]
fn contract_fields_delegates_to_focused_none() {
    for fidelity in [
        Fidelity::Low,
        Fidelity::Medium,
        Fidelity::High,
        Fidelity::Edit,
        Fidelity::Verbatim,
    ] {
        assert_eq!(
            contract_fields(fidelity),
            contract_fields_focused(fidelity, None)
        );
    }
}

// ── Edit Mode response contract smoke tests (Phase 4) ──────────────

/// Gap 5/3 fix: `provide_code_context` with `intent="edit"` must not panic
/// and must produce a response carrying the self-reporting contract fields
/// (`content_kind`, `byte_exact`, `degradation`). Since handlers write to
/// stdout, we verify the handler path is exercised without panic.
#[test]
fn provide_code_context_edit_intent_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "intent": "edit" } });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

/// Gap 5/3 fix: `provide_code_context` with explicit `fidelity="edit"` must
/// not panic (the edit-mode IR path with verbatim bodies).
#[test]
fn provide_code_context_explicit_edit_fidelity_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "edit" } });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

/// Gap 2 fix: `provide_code_context` with an invalid explicit fidelity must
/// not panic (the handler should return -32602, not crash).
#[test]
fn provide_code_context_invalid_fidelity_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "full" } });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

/// Gap 3 fix: `fidelity="verbatim"` must not panic and must bypass
/// compression entirely (raw source byte-exact). Exercises the new
/// Verbatim short-circuit in `handle_provide_code_context`.
#[test]
fn provide_code_context_verbatim_fidelity_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "verbatim" } });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

// ── PATHMAP footer scoping tests ───────────────────────────────────
// Verifies that format_dict_footer_for_aliases produces a request-scoped
// PATHMAP containing only the aliases needed by each individual response.

#[test]
fn pathmap_footer_contains_only_requested_alias() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let alias_a = state.get_or_create_alias("/workspace/file_a.ts".to_string());
    let alias_b = state.get_or_create_alias("/workspace/file_b.ts".to_string());

    // Footer for file A should only contain A's alias.
    let footer_a = state.format_dict_footer_for_aliases(&[&alias_a]);
    assert!(footer_a.contains("§PATHMAP"), "footer must have header");
    assert!(
        footer_a.contains(&alias_a),
        "footer for A must contain {alias_a}"
    );
    assert!(
        !footer_a.contains(&alias_b),
        "footer for A must NOT contain file_b alias {alias_b}"
    );

    // Footer for file B should only contain B's alias.
    let footer_b = state.format_dict_footer_for_aliases(&[&alias_b]);
    assert!(
        footer_b.contains(&alias_b),
        "footer for B must contain {alias_b}"
    );
    assert!(
        !footer_b.contains(&alias_a),
        "footer for B must NOT contain file_a alias {alias_a}"
    );
}

#[test]
fn pathmap_footer_does_not_grow_with_unrelated_files() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    // Simulate sequential requests.
    let alias_a = state.get_or_create_alias("/project/src/a.ts".to_string());
    let _alias_b = state.get_or_create_alias("/project/src/b.ts".to_string());
    let _alias_c = state.get_or_create_alias("/project/src/c.ts".to_string());
    let _alias_d = state.get_or_create_alias("/project/src/d.ts".to_string());

    // Footer for a later request for file A should be the same size as if A alone existed.
    let footer_a = state.format_dict_footer_for_aliases(&[&alias_a]);
    let line_count_a = footer_a.lines().filter(|l| l.contains('=')).count();
    assert_eq!(
        line_count_a, 1,
        "scoped footer for A must have exactly 1 alias line, got {line_count_a}: {footer_a:?}"
    );
}

#[test]
fn full_pathmap_footer_retains_all_aliases() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let alias_a = state.get_or_create_alias("/project/src/a.ts".to_string());
    let alias_b = state.get_or_create_alias("/project/src/b.ts".to_string());

    // The full format_footer must contain all aliases.
    let full = state.format_dict_footer();
    assert!(
        full.contains(&alias_a),
        "full footer must contain {alias_a}"
    );
    assert!(
        full.contains(&alias_b),
        "full footer must contain {alias_b}"
    );

    let line_count = full.lines().filter(|l| l.contains('=')).count();
    assert_eq!(
        line_count, 2,
        "full footer must have exactly 2 alias lines, got {line_count}"
    );
}

/// Verbatim short-circuit in `handle_compress_code_context` must not panic.
#[test]
fn compress_code_context_verbatim_fidelity_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "verbatim" } });
    dispatch_tools_call(&id, "compress_code_context", &params, &state);
}

/// Symbol targeting: `provide_code_context` with `focusMethods` at edit
/// fidelity must not panic (the new focused-render path).
#[test]
fn provide_code_context_focus_methods_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": "src/lib.rs",
            "fidelity": "edit",
            "focusMethods": ["test_method", "another_method"]
        }
    });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

/// Symbol targeting: `focusMethods` with an empty array must not panic
/// and should degrade gracefully.
#[test]
fn provide_code_context_focus_methods_empty_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params = serde_json::json!({
        "arguments": {
            "filePath": "src/lib.rs",
            "fidelity": "edit",
            "focusMethods": []
        }
    });
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

// ── Blast radius integration regression tests ──────────────────────
// These tests ensure blast radius is properly integrated into the
// compression pipeline and cannot be accidentally removed or broken.

#[test]
fn blast_radius_disabled_by_default_does_not_panic() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    // Should not panic when blast radius is disabled (default)
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

#[test]
fn blast_radius_enabled_does_not_panic_without_cbm() {
    let mut config = crate::tests::test_config();
    config.intelligence.blast_radius_enabled = true;
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    // Should not panic when blast radius is enabled but CBM is unavailable
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
}

#[test]
fn blast_radius_delta_mode_does_not_panic() {
    let mut config = crate::tests::test_config();
    config.intelligence.blast_radius_enabled = true;
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params =
        serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    // Should not panic in delta mode with blast radius enabled
    dispatch_tools_call(&id, "delta_code_context", &params, &state);
}

// Verifies post-compression economic invariant: candidate_tokens <= raw_tokens

// for all fidelity levels. See token_economics.rs for the two-stage gate design.
// ── Token-economics regression tests (2026-08-31) ───────────────────

// ── Token-economics regression tests (2026-08-31) ───────────────────
// Verifies post-compression economic invariant: candidate_tokens <= raw_tokens
// for all fidelity levels. See token_economics.rs for the two-stage gate.

use crate::mcp::tool_handlers::core::maybe_economics_fallback;

fn count_tokens(text: &str) -> usize {
    let kind = crate::tokenizer::TokenizerKind::default();
    let tok = crate::tokenizer::create_tokenizer(kind).unwrap();
    tok.count_tokens(text)
}

// ── maybe_economics_fallback unit tests ────────────────────────────

#[test]
fn economics_candidate_worse_than_raw_triggers_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    let result = maybe_economics_fallback(
        &id,
        source,
        100,
        200,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
    );
    assert!(result, "should fall back when candidate > raw");
}

#[test]
fn economics_candidate_cheaper_than_raw_does_not_trigger_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    let result = maybe_economics_fallback(
        &id,
        source,
        200,
        100,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
    );
    assert!(!result, "should NOT fall back when candidate < raw");
}

#[test]
fn economics_candidate_equal_to_raw_does_not_trigger_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    let result = maybe_economics_fallback(
        &id,
        source,
        100,
        100,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
    );
    assert!(!result, "should NOT fall back when candidate == raw");
}

#[test]
fn economics_fallback_works_for_all_fidelity_levels() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    for fidelity in &[
        crate::compression::Fidelity::Low,
        crate::compression::Fidelity::Medium,
        crate::compression::Fidelity::High,
        crate::compression::Fidelity::Edit,
    ] {
        let result = maybe_economics_fallback(
            &id,
            source,
            100,
            200,
            &state,
            "/test/file.ts",
            false,
            *fidelity,
            "test",
        );
        assert!(result, "{fidelity:?}: must fall back when candidate > raw");
        let result = maybe_economics_fallback(
            &id,
            source,
            200,
            100,
            &state,
            "/test/file.ts",
            false,
            *fidelity,
            "test",
        );
        assert!(
            !result,
            "{fidelity:?}: must NOT fall back when candidate < raw"
        );
    }
}

// ── Integration tests with real files ──────────────────────────────

fn create_multi_method_fixture(dir: &tempfile::TempDir, name: &str, mc: usize) -> String {
    let path = dir.path().join(name);
    let mut s = String::new();
    s.push_str("use std::collections::HashMap;\nuse std::sync::Arc;\n");
    s.push_str("use std::sync::Mutex;\nuse std::time::Instant;\n");
    s.push_str("use std::path::PathBuf;\nuse std::fs::File;\n");
    s.push_str("use std::io::{self, BufRead, BufReader, Write};\n");
    s.push_str("use std::fmt::Debug;\n\n");
    s.push_str("pub struct Service {\n    name: String,\n    count: u64,\n    active: bool,\n    data: HashMap<String, Vec<u8>>,\n}\n\n");
    s.push_str("impl Service {\n");
    s.push_str("    pub fn new(name: String) -> Self {\n");
    s.push_str("        Self { name, count: 0, active: true, data: HashMap::new() }\n    }\n\n");
    for i in 0..mc {
        s.push_str(&format!("    pub fn method_{i}(&self) -> &str {{\n"));
        s.push_str("        &self.name\n    }\n\n");
    }
    s.push_str("    pub fn process(&mut self) -> io::Result<()> {\n");
    s.push_str("        self.count += 1;\n");
    s.push_str("        if !self.active { return Ok(()); }\n");
    s.push_str("        let _ = self.data.insert(\"key\".into(), vec![1, 2, 3]);\n");
    s.push_str("        Ok(())\n    }\n}\n");
    std::fs::write(&path, &s).unwrap();
    path.to_string_lossy().into_owned()
}

fn resp_kind(resp: &serde_json::Value) -> String {
    resp["result"]["_meta"]["content_kind"]
        .as_str()
        .unwrap_or("missing")
        .to_string()
}

fn resp_text(resp: &serde_json::Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

#[test]
fn economics_small_file_edit_uses_raw_passthrough() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("tiny.rs");
    std::fs::write(&path, "fn small() { 42 }").unwrap();
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": path.to_string_lossy().into_owned(), "fidelity": "edit" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
    let resp = crate::protocol::captured_responses()
        .pop()
        .expect("handler must send response");
    let kind = resp_kind(&resp);
    assert_eq!(
        kind, "raw_passthrough",
        "small Edit -> raw_passthrough, got: {kind}"
    );
}

#[test]
fn economics_edit_multi_method_candidate_vs_raw() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path_str = create_multi_method_fixture(&dir, "svc.rs", 35);
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    assert!(
        (800..=2000).contains(&raw_tokens),
        "fixture: ~800-2000 raw tokens, got: {raw_tokens}"
    );
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": path_str, "fidelity": "edit" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
    let resp = crate::protocol::captured_responses()
        .pop()
        .expect("handler must send response");
    let kind = resp_kind(&resp);
    let text = resp_text(&resp);
    let comp_tokens = count_tokens(&text);
    if kind == "raw_passthrough" {
        assert_eq!(text, source, "raw_passthrough must return verbatim source");
    } else {
        assert!(
            comp_tokens <= raw_tokens,
            "invariant at Edit: raw={raw_tokens}, candidate={comp_tokens}"
        );
    }
}

#[test]
fn economics_structural_fidelities_obey_invariant() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path_str = create_multi_method_fixture(&dir, "s.rs", 15);
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    for &fidelity in &["low", "medium", "high"] {
        let params = serde_json::json!({ "arguments": { "filePath": path_str.clone(), "fidelity": fidelity } });
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(&id, "provide_code_context", &params, &state);
        let resp = crate::protocol::captured_responses()
            .pop()
            .expect("handler must send resp");
        let kind = resp_kind(&resp);
        let text = resp_text(&resp);
        let comp_tokens = count_tokens(&text);
        if kind == "raw_passthrough" {
            assert_eq!(text, source, "raw_passthrough at {fidelity}");
        } else {
            assert!(
                comp_tokens <= raw_tokens,
                "invariant at {fidelity}: raw={raw_tokens}, candidate={comp_tokens}"
            );
        }
    }
}

#[test]
fn economics_positive_compression_still_selected() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("large.rs");
    let mut source = String::new();
    source.push_str("use std::collections::HashMap;\nuse std::sync::Arc;\n\n");
    source.push_str("pub struct LargeService {\n    data: HashMap<String, Vec<u8>>,\n    name: String,\n    count: u64,\n}\n\n");
    source.push_str("impl LargeService {\n");
    for i in 0..80 {
        source.push_str(&format!(
            "    pub fn method_{i}(&self, key: &str) -> Option<&Vec<u8>> {{\n"
        ));
        source.push_str("        self.data.get(key)\n    }\n\n");
    }
    source.push_str("}\n");
    std::fs::write(&path, &source).unwrap();
    let path_str = path.to_string_lossy().into_owned();
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    assert!(
        raw_tokens > 1500,
        "fixture: >1500 raw tokens, got: {raw_tokens}"
    );
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    for &fidelity in &["low", "medium", "high"] {
        let params = serde_json::json!({ "arguments": { "filePath": path_str.clone(), "fidelity": fidelity } });
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(&id, "provide_code_context", &params, &state);
        let resp = crate::protocol::captured_responses()
            .pop()
            .expect("handler must send resp");
        let kind = resp_kind(&resp);
        let text = resp_text(&resp);
        let comp_tokens = count_tokens(&text);
        assert_ne!(
            kind, "raw_passthrough",
            "{fidelity}: large file must compress"
        );
        assert!(
            comp_tokens <= raw_tokens,
            "invariant at {fidelity}: raw={raw_tokens}, candidate={comp_tokens}"
        );
        assert!(
            raw_tokens > comp_tokens,
            "{fidelity}: saves tokens, raw={raw_tokens}, candidate={comp_tokens}"
        );
    }
}

#[test]
fn economics_intent_edit_obeys_invariant() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path_str = create_multi_method_fixture(&dir, "intent.rs", 20);
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": path_str, "intent": "edit" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
    let resp = crate::protocol::captured_responses()
        .pop()
        .expect("handler must send resp");
    let kind = resp_kind(&resp);
    let text = resp_text(&resp);
    let comp_tokens = count_tokens(&text);
    if kind == "raw_passthrough" {
        assert_eq!(text, source, "raw_passthrough must return verbatim source");
    } else {
        assert!(
            comp_tokens <= raw_tokens,
            "invariant for intent=edit: raw={raw_tokens}, candidate={comp_tokens}"
        );
    }
}
