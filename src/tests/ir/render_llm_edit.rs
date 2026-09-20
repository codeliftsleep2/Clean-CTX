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

// ── Edit Mode & High-Fidelity Rendering Tests (Phase 4) ───────────

/// Edit Mode: verbatim method body should be rendered after the
/// method signature line (byte-exact for replace_in_file).
#[test]
fn test_edit_mode_renders_verbatim_body() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("doWork");
    method.body = Some("{\n  let x = 1;\n  println!(\"{}\", x);\n}".to_string());
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Edit);
    // Method declaration present
    assert!(result.contains("M doWork"));
    // Verbatim body present
    assert!(result.contains("{\n  let x = 1;\n  println!(\"{}\", x);\n}"));
    // Structurally, the body should come after the method line.
    let method_pos = result.find("M doWork").unwrap();
    let body_pos = result.find("let x = 1").unwrap();
    assert!(
        method_pos < body_pos,
        "body should render after the method signature"
    );
}

/// Edit Mode: a method with no body should not emit anything extra.
#[test]
fn test_edit_mode_no_body_no_panic() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    class.methods.push(make_method("noBody"));
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Edit);
    assert!(result.contains("M noBody"));
}

/// High Fidelity: control-flow metadata should render as inline markers.
#[test]
fn test_high_fidelity_renders_control_flow() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("process");
    method.control_flow = vec![
        vec!["if".to_string(), "x > 0".to_string()],
        vec!["loop".to_string(), "items".to_string()],
    ];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::High);
    assert!(
        result.contains("cf:if:x > 0,loop:items"),
        "High fidelity should render control-flow markers: {}",
        result
    );
}

/// High Fidelity: control-flow metadata not rendered at Low fidelity.
#[test]
fn test_low_fidelity_no_control_flow() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("process");
    method.control_flow = vec![vec!["if".to_string(), "x > 0".to_string()]];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(
        !result.contains("cf:"),
        "Low fidelity should not render control-flow markers"
    );
}

/// High Fidelity: data-flow metadata should render as inline markers (Gap 1).
#[test]
fn test_high_fidelity_renders_data_flow() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("process");
    method.data_flow = vec![
        vec!["reads".to_string(), "config".to_string()],
        vec!["writes".to_string(), "users".to_string()],
    ];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::High);
    assert!(
        result.contains("df:reads:config,writes:users"),
        "High fidelity should render data-flow markers: {}",
        result
    );
}

/// High Fidelity: side-effect annotation should render (Gap 1).
#[test]
fn test_high_fidelity_renders_side_effect() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("save");
    method.side_effect = vec![SideEffectKind::Mutation];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::High);
    assert!(
        result.contains(" se:mutation"),
        "High fidelity should render the side-effect annotation: {}",
        result
    );
}

/// High Fidelity: execution-context annotation should render (Gap 1).
#[test]
fn test_high_fidelity_renders_execution_context() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("poll");
    method.execution_context = vec!["async".to_string()];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::High);
    assert!(
        result.contains(" ec:async"),
        "High fidelity should render the execution-context annotation: {}",
        result
    );
}

/// Low fidelity: data-flow / side-effect / execution-context must not render.
#[test]
fn test_low_fidelity_no_execution_metadata() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("process");
    method.data_flow = vec![vec!["reads".to_string(), "config".to_string()]];
    method.side_effect = vec![SideEffectKind::Io];
    method.execution_context = vec!["sync".to_string()];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(
        !result.contains("df:"),
        "Low fidelity should not render data-flow markers"
    );
    assert!(
        !result.contains(" se:"),
        "Low fidelity should not render side-effect annotations"
    );
    assert!(
        !result.contains(" ec:"),
        "Low fidelity should not render execution-context annotations"
    );
}

/// Edit fidelity: execution metadata stays compact (only bodies are verbatim).
#[test]
fn test_edit_fidelity_no_execution_metadata() {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    let mut method = make_method("doWork");
    method.body = Some("{\n  let x = 1;\n}".to_string());
    method.data_flow = vec![vec!["reads".to_string(), "config".to_string()]];
    method.side_effect = vec![SideEffectKind::Io];
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Edit);
    // The body must still be byte-exact and present.
    assert!(result.contains("{\n  let x = 1;\n}"));
    // Runtime metadata is NOT emitted at Edit — it would pollute the
    // byte-exact body region with non-source markers.
    assert!(!result.contains("df:"));
    assert!(!result.contains(" se:"));
}

#[test]
fn test_synthetic_class_is_rendered() {
    let mut hir = empty_hir();
    let mut class = make_class("__synthetic_C1");
    class.synthetic = true;
    class.fields.push(make_field("orphan", Some("$n")));
    hir.classes.push(class);

    // Synthetic classes are still rendered (they show as regular classes)
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── __synthetic_C1 ──"));
    assert!(result.contains("F orphan:$n"));
}

// ── Focused Edit Rendering Tests (Symbol Targeting) ───────────────

fn make_class_with_bodies(names: &[(&str, &str)]) -> HierarchicalIR {
    let mut hir = empty_hir();
    let mut class = make_class("MyService");
    for (name, body) in names {
        let mut method = make_method(name);
        method.body = Some(body.to_string());
        class.methods.push(method);
    }
    hir.classes.push(class);
    hir
}

/// Focused Edit: only the named method gets its full verbatim body.
#[test]
fn test_edit_focused_renders_only_named_method_bodies() {
    let hir = make_class_with_bodies(&[
        ("methodA", "{\n  let a = 1;\n}"),
        ("methodB", "{\n  let b = 2;\n}"),
        ("methodC", "{\n  let c = 3;\n}"),
    ]);
    let focus: HashSet<String> = ["methodB".to_string()].into_iter().collect();

    let result = render_hierarchical_for_llm_focused(&hir, Fidelity::Edit, Some(&focus));

    // All method signatures present
    assert!(result.contains("M methodA"));
    assert!(result.contains("M methodB"));
    assert!(result.contains("M methodC"));
    // Only methodB gets its body
    assert!(
        !result.contains("let a = 1"),
        "unfocused method body must not render"
    );
    assert!(
        result.contains("let b = 2"),
        "focused method body must render verbatim"
    );
    assert!(
        !result.contains("let c = 3"),
        "unfocused method body must not render"
    );
}

/// Focused rendering: `None` focus is identical to the unfocused function.
#[test]
fn test_edit_focused_none_renders_all_bodies() {
    let hir = make_class_with_bodies(&[
        ("methodA", "{\n  let a = 1;\n}"),
        ("methodB", "{\n  let b = 2;\n}"),
    ]);

    let focused = render_hierarchical_for_llm_focused(&hir, Fidelity::Edit, None);
    let unfocused = render_hierarchical_for_llm(&hir, Fidelity::Edit);

    assert_eq!(focused, unfocused);
    assert!(focused.contains("let a = 1"));
    assert!(focused.contains("let b = 2"));
}

/// Focused rendering: a focus name that doesn't exist degrades gracefully
/// to all-signatures (no bodies), no crash.
#[test]
fn test_edit_focused_no_match_renders_all_signatures() {
    let hir = make_class_with_bodies(&[
        ("methodA", "{\n  let a = 1;\n}"),
        ("methodB", "{\n  let b = 2;\n}"),
    ]);
    let focus = HashSet::from(["NoSuchMethod".to_string()]);

    let result = render_hierarchical_for_llm_focused(&hir, Fidelity::Edit, Some(&focus));

    assert!(result.contains("M methodA"));
    assert!(result.contains("M methodB"));
    assert!(!result.contains("let a = 1"));
    assert!(!result.contains("let b = 2"));
}

/// Focused rendering: multiple named methods all get bodies, only those.
#[test]
fn test_edit_focused_multiple_names() {
    let hir = make_class_with_bodies(&[
        ("methodA", "{\n  let a = 1;\n}"),
        ("methodB", "{\n  let b = 2;\n}"),
        ("methodC", "{\n  let c = 3;\n}"),
    ]);
    let focus = HashSet::from(["methodA".to_string(), "methodC".to_string()]);

    let result = render_hierarchical_for_llm_focused(&hir, Fidelity::Edit, Some(&focus));

    assert!(result.contains("let a = 1"));
    assert!(!result.contains("let b = 2"));
    assert!(result.contains("let c = 3"));
}

/// Focus has no effect at non-Edit fidelities.
#[test]
fn test_edit_focused_non_edit_fidelity_ignores_focus() {
    let hir = make_class_with_bodies(&[
        ("methodA", "{\n  let a = 1;\n}"),
        ("methodB", "{\n  let b = 2;\n}"),
    ]);
    let focus = HashSet::from(["methodB".to_string()]);

    // Low fidelity ignores focus entirely — bodies never render.
    let result_low = render_hierarchical_for_llm_focused(&hir, Fidelity::Low, Some(&focus));
    assert!(!result_low.contains("let a = 1"));
    assert!(!result_low.contains("let b = 2"));

    // High fidelity also ignores focus — bodies never render at High.
    let result_high = render_hierarchical_for_llm_focused(&hir, Fidelity::High, Some(&focus));
    assert!(!result_high.contains("let a = 1"));
    assert!(!result_high.contains("let b = 2"));
}

/// Focused rendering: empty set renders all signatures (no bodies).
#[test]
fn test_edit_focused_empty_set_renders_all_signatures() {
    let hir = make_class_with_bodies(&[
        ("methodA", "{\n  let a = 1;\n}"),
        ("methodB", "{\n  let b = 2;\n}"),
    ]);
    let focus = HashSet::new();

    let result = render_hierarchical_for_llm_focused(&hir, Fidelity::Edit, Some(&focus));

    assert!(result.contains("M methodA"));
    assert!(result.contains("M methodB"));
    assert!(!result.contains("let a = 1"));
    assert!(!result.contains("let b = 2"));
}
