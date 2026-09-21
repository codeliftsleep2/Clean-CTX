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

#[test]
fn test_spring_boot_class_with_meta() {
    let mut hir = empty_hir();
    let mut class = make_class("UserController");

    // Spring meta annotations stored as TypeAlias ops
    // In real IR pipeline, these would be emitted by spring.rs meta layer
    // For this test, we show them in the type_aliases section

    class.extends = Some("BaseController".into());
    class
        .fields
        .push(make_field("userService", Some("UserService")));

    let mut m1 = make_method("getAll");
    m1.control_summaries = vec![vec![ControlSummary::Return]];
    class.methods.push(m1);

    let mut m2 = make_method("find");
    m2.params.push(vec!["P1".into(), "$n".into(), "id".into()]);
    m2.control_summaries = vec![vec![ControlSummary::Return]];
    class.methods.push(m2);

    let mut m3 = make_method("find");
    m3.params
        .push(vec!["P1".into(), "$n".into(), "name".into()]);
    m3.params.push(vec!["P2".into(), "$n".into(), "age".into()]);
    m3.params
        .push(vec!["P3".into(), "$s".into(), "role".into()]);
    m3.control_summaries = vec![vec![ControlSummary::Return, ControlSummary::Branch]];
    class.methods.push(m3);

    hir.classes.push(class);

    // Spring meta type aliases
    hir.type_aliases
        .push(make_type_alias("@rest", "UserController"));
    hir.type_aliases
        .push(make_type_alias("@map", "GET /users POST /users"));
    hir.imports.push(make_import(
        "IM1",
        "org.springframework.web.bind.annotation",
        "*",
    ));

    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    assert!(result.contains("// ── UserController ──"));
    assert!(result.contains("X BaseController"));
    assert!(result.contains("F userService:UserService"));
    assert!(result.contains("M getAll"));
    assert!(result.contains("M find(+1)"));
    assert!(result.contains("M find(+3)"));
    assert!(result.contains("ctl:RET,IF"));
    assert!(result.contains("T @rest = UserController"));
    assert!(result.contains("T @map = GET /users POST /users"));
    assert!(result.contains("$ IM1 org.springframework.web.bind.annotation"));
}

#[test]
fn test_triple_overloaded_methods() {
    let mut hir = empty_hir();
    let mut class = make_class("Overloader");

    for i in 0..3 {
        let mut m = make_method("process");
        for j in 0..=i {
            m.params.push(vec![
                format!("P{}", j + 1),
                "$n".into(),
                format!("arg{}", j + 1),
            ]);
        }
        class.methods.push(m);
    }

    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("M process(+1)"));
    assert!(result.contains("M process(+2)"));
    assert!(result.contains("M process(+3)"));
}

#[test]
fn test_no_name_collision_with_unique_methods() {
    let mut hir = empty_hir();
    let mut class = make_class("Unique");

    class.methods.push(make_method("init"));
    class.methods.push(make_method("start"));
    class.methods.push(make_method("stop"));

    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // None should have +N
    assert!(result.contains("M init\n"));
    assert!(result.contains("M start\n"));
    assert!(result.contains("M stop\n"));
    assert!(!result.contains("init(+0)"));
    assert!(!result.contains("start(+0)"));
    assert!(!result.contains("stop(+0)"));
}

#[test]
fn test_multiple_classes_with_imports_and_type_aliases() {
    let mut hir = empty_hir();

    let mut c1 = make_class("Alpha");
    c1.fields.push(make_field("a", Some("$n")));
    hir.classes.push(c1);

    let mut c2 = make_class("Beta");
    c2.fields.push(make_field("b", Some("$s")));
    hir.classes.push(c2);

    hir.imports.push(make_import("IM1", "lib", "A, B"));
    hir.type_aliases.push(make_type_alias("TypeA", "$n"));

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);

    // Order should be: classes first, then imports, then type aliases
    let alpha_pos = result.find("// ── Alpha ──").unwrap();
    let beta_pos = result.find("// ── Beta ──").unwrap();
    let import_pos = result.find("$ IM1").unwrap();
    let alias_pos = result.find("T TypeA").unwrap();

    assert!(alpha_pos < beta_pos, "Alpha should appear before Beta");
    assert!(
        beta_pos < import_pos,
        "Classes should appear before imports"
    );
    assert!(
        import_pos < alias_pos,
        "Imports should appear before type aliases"
    );
}

#[test]
fn test_empty_class_name_still_renders() {
    let mut hir = empty_hir();
    hir.classes.push(make_class(""));
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ──  ──"));
}

#[test]
fn test_many_params_formatted_correctly() {
    let mut hir = empty_hir();
    let mut class = make_class("BigMethod");
    let mut method = make_method("processData");
    method
        .params
        .push(vec!["P1".into(), "$n".into(), "a".into()]);
    method
        .params
        .push(vec!["P2".into(), "$s".into(), "b".into()]);
    method
        .params
        .push(vec!["P3".into(), "$b".into(), "c".into()]);
    method
        .params
        .push(vec!["P4".into(), "$s[]".into(), "d".into()]);
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    assert!(result.contains("p:a:$n b:$s c:$b d:$s[]"));
}

#[test]
fn test_renderer_no_panic_on_large_hir() {
    // Stress test: lots of classes, methods, fields
    let mut hir = empty_hir();
    for i in 0..50 {
        let mut class = make_class(&format!("Class{}", i));
        for j in 0..10 {
            let mut method = make_method(&format!("method{}", j));
            method
                .params
                .push(vec!["P1".into(), "$n".into(), "x".into()]);
            class.methods.push(method);
        }
        for j in 0..5 {
            class
                .fields
                .push(make_field(&format!("field{}", j), Some("$n")));
        }
        hir.classes.push(class);
    }

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── Class0 ──"));
    assert!(result.contains("// ── Class49 ──"));
    assert_eq!(result.matches("// ── ").count(), 50);
}

#[test]
fn test_injects_are_explicit_in_control_full() {
    let mut hir = empty_hir();
    let mut class = make_class("InjectedService");
    class.injects.push(vec!["Dep1".into(), "Dep2".into()]);
    hir.classes.push(class);

    let result = crate::ir::render_control_full("alpha", "fixture.ts", 1, Fidelity::Low, &hir, &[]);
    assert!(result.contains("\"injection_occurrences\""));
    assert!(result.contains("\"Dep1\""));
    assert!(result.contains("\"Dep2\""));
}
