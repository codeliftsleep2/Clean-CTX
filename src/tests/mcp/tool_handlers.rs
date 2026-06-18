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

use crate::mcp::tools::{parse_fidelity_arg, resolve_fidelity};
use crate::compression::Fidelity;
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
    let result = resolve_fidelity(Some("medium"), None, &crate::config::CleanCtxConfig::default());
    assert_eq!(result, Fidelity::Medium);
}

#[test]
fn resolve_fidelity_explicit_high() {
    let result = resolve_fidelity(Some("high"), None, &crate::config::CleanCtxConfig::default());
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
    let result = resolve_fidelity(Some("bogus"), None, &crate::config::CleanCtxConfig::default());
    assert_eq!(result, Fidelity::Low);
}

#[test]
fn resolve_fidelity_extension_override() {
    let mut config = crate::config::CleanCtxConfig::default();
    config.fidelity_overrides.insert("ts".to_string(), "high".to_string());
    let result = resolve_fidelity(None, Some("ts"), &config);
    assert_eq!(result, Fidelity::High);
}

// ── parse_fidelity_arg tests ──

#[test]
fn parse_fidelity_arg_with_explicit_value() {
    let params = json!({ "arguments": { "fidelity": "high" } });
    let result = parse_fidelity_arg(&json!(1), &params);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Fidelity::High);
}

#[test]
fn parse_fidelity_arg_missing_defaults() {
    let params = json!({ "arguments": {} });
    let result = parse_fidelity_arg(&json!(1), &params);
    assert!(result.is_ok());
}

#[test]
fn parse_fidelity_arg_invalid_returns_error() {
    let params = json!({ "arguments": { "fidelity": "turbo" } });
    let result = parse_fidelity_arg(&json!(1), &params);
    assert!(result.is_err());
}

// ── Handler smoke tests (verify no panic) ──

#[test]
fn handle_context_stats_smoke() {
    use crate::mcp::tool_handlers::handle_context_stats;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    handle_context_stats(&id, &params, &mut state);
}

#[test]
fn handle_list_sessions_smoke() {
    use crate::mcp::tool_handlers::handle_list_sessions;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    handle_list_sessions(&id, &params, &mut state);
}

#[test]
fn handle_context_history_smoke() {
    use crate::mcp::tool_handlers::handle_context_history;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": {} });
    // Should not panic
    handle_context_history(&id, &params, &mut state);
}

#[test]
fn handle_save_context_smoke() {
    use crate::mcp::tool_handlers::handle_save_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic (returns error for nonexistent file, but doesn't panic)
    handle_save_context(&id, &params, &mut state);
}

#[test]
fn handle_restore_context_smoke() {
    use crate::mcp::tool_handlers::handle_restore_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent.ts" } });
    // Should not panic
    handle_restore_context(&id, &params, &mut state);
}

#[test]
fn handle_purge_old_deltas_smoke() {
    use crate::mcp::tool_handlers::handle_purge_old_deltas;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "days": 30 } });
    // Should not panic
    handle_purge_old_deltas(&id, &params, &mut state);
}

// ── IR-first integration tests ──

#[test]
fn handle_compress_code_context_with_fallback() {
    use crate::mcp::tool_handlers::handle_compress_code_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    // Use nonexistent file — should fall back to text pipeline gracefully
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic
    handle_compress_code_context(&id, &params, &mut state, "named");
}

#[test]
fn handle_delta_code_context_no_baseline() {
    use crate::mcp::tool_handlers::handle_delta_code_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic — stores baseline IR, returns "no baseline" message
    handle_delta_code_context(&id, &params, &mut state);
}

#[test]
fn handle_delta_text_context_no_baseline() {
    use crate::mcp::tool_handlers::handle_delta_text_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "filePath": "/nonexistent/file.ts", "fidelity": "low" } });
    // Should not panic — stores text delta baseline, returns full output
    handle_delta_text_context(&id, &params, &mut state);
}

#[test]
fn handle_apply_delta_no_baseline() {
    use crate::mcp::tool_handlers::handle_apply_delta;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = json!(1);
    let params = json!({ "arguments": { "delta": { "file": "α1", "from": 1, "to": 2, "ops": { "+": [], "-": [], "~": [] } } } });
    // Should not panic — returns "UnknownFile" error
    handle_apply_delta(&id, &params, &mut state);
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
    };
    class.methods.push(m1);
    class.methods.push(m2);
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![vec!["IM1".into(), "org.springframework.web".into(), "*".into()]],
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
            FieldNode { id: "F1".into(), name: "x".into(), field_type: Some("$n".into()) },
            FieldNode { id: "F2".into(), name: "y".into(), field_type: Some("$n".into()) },
            FieldNode { id: "F3".into(), name: "label".into(), field_type: Some("$s".into()) },
        ],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR { classes: vec![class], imports: vec![], type_aliases: vec![] };
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
            FieldNode { id: "F1".into(), name: "x".into(), field_type: Some("$n".into()) },
            FieldNode { id: "F2".into(), name: "y".into(), field_type: Some("$n".into()) },
        ],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR { classes: vec![class], imports: vec![], type_aliases: vec![] };
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
    let hir = HierarchicalIR { classes: vec![class], imports: vec![], type_aliases: vec![] };
    // Should not panic — injects are structural (pattern-level), not rendered
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── Service ──"));
}

// ── LLM text cache integration tests ──

#[test]
fn mcp_state_llm_text_cache_insert_and_read() {
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    // Insert into cache
    state.llm_text_cache.insert("α1".to_string(), "// SCHEMA v2\n// ── Foo ──\n".to_string());
    // Read from cache
    let cached = state.llm_text_cache.get("α1");
    assert!(cached.is_some());
    assert!(cached.unwrap().contains("SCHEMA v2"));
    assert!(cached.unwrap().contains("Foo"));
}

#[test]
fn mcp_state_llm_text_cache_miss_returns_none() {
    let config = crate::config::CleanCtxConfig::default();
    let state = crate::mcp::McpState::new(config);
    assert!(!state.llm_text_cache.contains_key("nonexistent"));
}

#[test]
fn mcp_state_llm_text_cache_clear_on_new() {
    let config = crate::config::CleanCtxConfig::default();
    let state = crate::mcp::McpState::new(config);
    // Fresh state should have empty cache
    assert!(state.llm_text_cache.is_empty());
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

#[test]
fn micro_opcode_apply_expand_roundtrip_with_new_markers() {
    use crate::compression::micro_opcodes::{apply_micro_opcodes, expand_micro_opcodes};
    let original = "Foo{field1};⊕guard check() ⊕loop iterate() ⊕⇒result ⊕!err";
    let compressed = apply_micro_opcodes(original, Fidelity::Low);
    let expanded = expand_micro_opcodes(&compressed);
    assert_eq!(expanded, original, "Round-trip must preserve original content");
    // Verify compression replaces markers
    assert!(compressed.contains("§I"), "⊕guard should be compressed to §I");
    assert!(compressed.contains("§L"), "⊕loop should be compressed to §L");
    assert!(compressed.contains("§E"), "⊕⇒ should be compressed to §E");
    assert!(compressed.contains("§C"), "{{ and }} should be compressed to §C");
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
    let abs_root = if cfg!(windows) { "D:\\myproject" } else { "/home/user/myproject" };
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
    use crate::mcp::tool_handlers::handle_compress_code_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    handle_compress_code_context(&id, &params, &mut state, "named");
}

#[test]
fn handle_delta_code_context_accepts_relative_path() {
    use crate::mcp::tool_handlers::handle_delta_code_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    handle_delta_code_context(&id, &params, &mut state);
}

#[test]
fn handle_delta_text_context_accepts_relative_path() {
    use crate::mcp::tool_handlers::handle_delta_text_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    handle_delta_text_context(&id, &params, &mut state);
}

#[test]
fn handle_diff_code_context_accepts_relative_path() {
    use crate::mcp::tool_handlers::handle_diff_code_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    handle_diff_code_context(&id, &params, &mut state);
}

#[test]
fn handle_restore_context_accepts_relative_path() {
    use crate::mcp::tool_handlers::handle_restore_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "fidelity": "low" } });
    handle_restore_context(&id, &params, &mut state);
}

#[test]
fn handle_provide_code_context_accepts_relative_path() {
    use crate::mcp::tool_handlers::handle_provide_code_context;
    let config = crate::config::CleanCtxConfig::default();
    let mut state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": "src/lib.rs", "intent": "overview" } });
    handle_provide_code_context(&id, &params, &mut state);
}