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
        modifiers: vec![],
        class_flags: vec![],
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
        modifiers: vec![],
        control_summaries: vec![vec![ControlSummary::Branch]],
        flags: vec![],
        patterns: vec![],
        body: None,
        body_start: None,
        body_end: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: Vec::new(),
        execution_context: Vec::new(),
    });
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![vec!["IM1".into(), "./core".into(), "OnInit".into()]],
        type_aliases: vec![],
        calls: vec![],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Phase 6 IR-first format: compact LLM schema with typed semantic families.
    assert!(result.contains("SCHEMA v4"));
    assert!(result.contains("// ── UserListComponent ──"));
    assert!(result.contains("X BaseListComponent"));
    assert!(result.contains("I OnInit"));
    assert!(result.contains("F users:$s[]"));
    assert!(result.contains("M ngOnInit"));
    assert!(result.contains("ctl:IF"));
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
        modifiers: vec![],
        class_flags: vec![],
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
        modifiers: vec![],
        control_summaries: vec![vec![ControlSummary::Return]],
        flags: vec![],
        patterns: vec![],
        body: None,
        body_start: None,
        body_end: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: Vec::new(),
        execution_context: Vec::new(),
    };
    let m2 = MethodNode {
        id: "M2".into(),
        name: "find".into(),
        params: vec![
            vec!["P1".into(), "$n".into(), "name".into()],
            vec!["P2".into(), "$n".into(), "age".into()],
        ],
        return_type: None,
        modifiers: vec![],
        control_summaries: vec![vec![ControlSummary::Return, ControlSummary::Branch]],
        flags: vec![],
        patterns: vec![],
        body: None,
        body_start: None,
        body_end: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: Vec::new(),
        execution_context: Vec::new(),
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
        calls: vec![],
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
        modifiers: vec![],
        class_flags: vec![],
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
        calls: vec![],
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
        calls: vec![],
    };
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Always has schema header even with empty HIR
    assert!(result.starts_with("// SCHEMA v4"));
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
        modifiers: vec![],
        class_flags: vec![],
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
        calls: vec![],
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
        modifiers: vec![],
        class_flags: vec![],
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
        calls: vec![],
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
        modifiers: vec![],
        class_flags: vec![],
        extends: None,
        implements: vec![],
        injects: vec![vec!["DepA".into(), "DepB".into()]],
        patterns: vec![],
        synthetic: false,
    };
    let hir = HierarchicalIR {
        classes: vec![class],
        imports: vec![],
        type_aliases: vec![],
        calls: vec![],
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
        .insert("α1".to_string(), "// SCHEMA v4\n// ── Foo ──\n".to_string());
    // Read from cache
    let cache_guard = state.llm_text_cache_lock();
    let cached = cache_guard.get("α1");
    assert!(cached.is_some());
    assert!(cached.unwrap().contains("SCHEMA v4"));
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
