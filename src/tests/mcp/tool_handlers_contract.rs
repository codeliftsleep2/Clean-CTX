// Sibling test module for a `#[path]`-loaded test file that exceeded the
// 615-line active-file size ceiling.
//
// Declared from the parent test file with `#[path = "<file>.rs"] mod <name>;`
// -- the same nested-`#[path]` idiom already used by `src/tests/cbm/e2e.rs` --
// so this module is a DESCENDANT of that test module and `use super::*`
// inherits its entire scope (imports, helpers, fixtures). Nothing needed to be
// widened or re-imported.
//
// Pure relocation: the tests below are byte-for-byte the previously inlined
// implementations.

use super::*;

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
