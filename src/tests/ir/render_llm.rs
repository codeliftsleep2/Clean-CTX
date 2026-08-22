// src/tests/ir/render_llm.rs
//
// Tests for the LLM-optimized hierarchical IR renderer.
// Covers: all class types (TS, Rust, Java, Angular, Spring),
// all fidelity levels, edge cases, overloaded methods, patterns.

use crate::compression::Fidelity;
use crate::ir::{ClassNode, FieldNode, HierarchicalIR, MethodNode, PatternEntry};
use crate::ir::{render_hierarchical_for_llm, render_hierarchical_for_llm_focused};
use std::collections::HashSet;

// ── Helpers ──

fn empty_hir() -> HierarchicalIR {
    HierarchicalIR {
        classes: vec![],
        imports: vec![],
        type_aliases: vec![],
    }
}

fn make_class(name: &str) -> ClassNode {
    ClassNode {
        id: "C1".to_string(),
        name: name.to_string(),
        methods: vec![],
        fields: vec![],
        class_flags: None,
        extends: None,
        implements: vec![],
        injects: vec![],
        patterns: vec![],
        synthetic: false,
    }
}

fn make_method(name: &str) -> MethodNode {
    MethodNode {
        id: "M1".to_string(),
        name: name.to_string(),
        params: vec![],
        return_type: None,
        flags: None,
        patterns: vec![],
        body: None,
        control_flow: vec![],
        data_flow: vec![],
        side_effect: None,
        execution_context: None,
    }
}

fn make_field(name: &str, field_type: Option<&str>) -> FieldNode {
    FieldNode {
        id: "F1".to_string(),
        name: name.to_string(),
        field_type: field_type.map(|s| s.to_string()),
    }
}

fn make_pattern(name: &str, args: Vec<&str>) -> PatternEntry {
    PatternEntry {
        name: name.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn make_import(alias: &str, module: &str, named: &str) -> Vec<String> {
    vec![alias.to_string(), module.to_string(), named.to_string()]
}

fn make_type_alias(alias: &str, original: &str) -> Vec<String> {
    vec![alias.to_string(), original.to_string()]
}

// ── Tests ──

#[test]
fn test_empty_hir() {
    let hir = empty_hir();
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Should contain schema header but nothing else
    assert!(result.starts_with("// SCHEMA v2"));
    assert!(!result.contains("// ──"));
    assert!(!result.contains("$ "));
    assert!(!result.contains("T "));
}

#[test]
fn test_schema_header_present() {
    let mut hir = empty_hir();
    hir.classes.push(make_class("TestClass"));
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// SCHEMA v2"));
    assert!(result.contains("@=meta"));
    assert!(result.contains("X=extends"));
    assert!(result.contains("I=implements"));
    assert!(result.contains("F=field"));
    assert!(result.contains("M=method"));
    assert!(result.contains("$=import"));
    assert!(result.contains("→=scope"));
    assert!(result.contains("fl:=flags"));
}

#[test]
fn test_single_class_no_fields_no_methods() {
    let mut hir = empty_hir();
    hir.classes.push(make_class("User"));
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── User ──"));
    // Should have no F or M lines
    assert!(!result.contains("\nF "));
    assert!(!result.contains("\nM "));
}

#[test]
fn test_class_with_fields_low_fidelity() {
    let mut hir = empty_hir();
    let mut class = make_class("User");
    class.fields.push(make_field("id", Some("$n")));
    class.fields.push(make_field("name", Some("$s")));
    class.fields.push(make_field("email", Some("$s")));
    hir.classes.push(class);

    // Low fidelity: space-separated on one line
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("F id:$n name:$s email:$s"));
    // Should have exactly one F line
    let f_count = result.matches("\nF ").count();
    assert_eq!(f_count, 1, "Low fidelity should have one F line");
}

#[test]
fn test_class_with_fields_medium_fidelity() {
    let mut hir = empty_hir();
    let mut class = make_class("User");
    class.fields.push(make_field("id", Some("$n")));
    class.fields.push(make_field("name", Some("$s")));
    hir.classes.push(class);

    // Medium fidelity: one per line
    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    assert!(result.contains("F id:$n\n"));
    assert!(result.contains("F name:$s\n"));
    let f_count = result.matches("\nF ").count();
    assert_eq!(f_count, 2, "Medium fidelity should have two F lines");
}

#[test]
fn test_class_with_fields_high_fidelity() {
    let mut hir = empty_hir();
    let mut class = make_class("User");
    class.fields.push(make_field("id", Some("$n")));
    class.fields.push(make_field("name", Some("$s")));
    hir.classes.push(class);

    // High fidelity: one per line (same as Medium)
    let result = render_hierarchical_for_llm(&hir, Fidelity::High);
    assert!(result.contains("F id:$n\n"));
    assert!(result.contains("F name:$s\n"));
    let f_count = result.matches("\nF ").count();
    assert_eq!(f_count, 2, "High fidelity should have two F lines");
}

#[test]
fn test_fields_without_type() {
    let mut hir = empty_hir();
    let mut class = make_class("Test");
    class.fields.push(make_field("count", None));
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("F count"));
}

#[test]
fn test_method_with_params_and_flags() {
    let mut hir = empty_hir();
    let mut class = make_class("UserService");
    let mut method = make_method("getUser");
    method
        .params
        .push(vec!["P1".into(), "$n".into(), "id".into()]);
    method.return_type = Some("$s".into());
    method.flags = Some(vec!["ASYNC".into(), "RET".into()]);
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    assert!(result.contains("M getUser"));
    assert!(result.contains("p:id:$n"));
    assert!(result.contains("→ $s"));
    assert!(result.contains("fl:ASYNC,RET"));
}

#[test]
fn test_method_no_params_no_flags() {
    let mut hir = empty_hir();
    let mut class = make_class("Test");
    class.methods.push(make_method("doSomething"));
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("M doSomething\n"));
}

#[test]
fn test_method_low_fidelity_hides_params() {
    let mut hir = empty_hir();
    let mut class = make_class("Test");
    let mut method = make_method("getData");
    method
        .params
        .push(vec!["P1".into(), "$n".into(), "id".into()]);
    method.return_type = Some("$s".into());
    class.methods.push(method);
    hir.classes.push(class);

    // Low fidelity: should NOT show params, but SHOULD show return type and flags
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("M getData"));
    // Check that return type is still shown
    assert!(result.contains("→ $s"));
    // Check that params are NOT shown in low fidelity
    assert!(!result.contains("p:"));
}

#[test]
fn test_overloaded_methods_disambiguation() {
    let mut hir = empty_hir();
    let mut class = make_class("Calculator");

    // find() with 1 param
    let mut m1 = make_method("find");
    m1.params.push(vec!["P1".into(), "$n".into(), "id".into()]);

    // find() with 3 params
    let mut m2 = make_method("find");
    m2.params
        .push(vec!["P1".into(), "$n".into(), "name".into()]);
    m2.params.push(vec!["P2".into(), "$n".into(), "age".into()]);
    m2.params
        .push(vec!["P3".into(), "$s".into(), "role".into()]);

    // Non-overloaded method
    let m3 = make_method("clear");

    class.methods.push(m1);
    class.methods.push(m2);
    class.methods.push(m3);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    assert!(result.contains("M find(+1)"));
    assert!(result.contains("M find(+3)"));
    assert!(result.contains("M clear"));
    // The non-overloaded method should NOT have +0
    assert!(!result.contains("clear(+0)"));
}

#[test]
fn test_overloaded_methods_params_shown_in_low_fidelity() {
    let mut hir = empty_hir();
    let mut class = make_class("Service");

    let mut m1 = make_method("find");
    m1.params.push(vec!["P1".into(), "$n".into(), "id".into()]);

    let mut m2 = make_method("find");
    m2.params
        .push(vec!["P1".into(), "$n".into(), "name".into()]);
    m2.params.push(vec!["P2".into(), "$n".into(), "age".into()]);

    class.methods.push(m1);
    class.methods.push(m2);
    hir.classes.push(class);

    // Even in Low fidelity, overloaded methods show params for disambiguation
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("M find(+1)"));
    assert!(result.contains("M find(+2)"));
    // Overloaded methods get +N shown even in low fidelity
    assert!(result.contains("p:id:$n") || result.contains("p:name:$n"));
}

#[test]
fn test_extends_and_implements() {
    let mut hir = empty_hir();
    let mut class = make_class("AdminService");
    class.extends = Some("BaseService".into());
    class.implements = vec!["AuthInterface".into(), "LogInterface".into()];
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("X BaseService\n"));
    assert!(result.contains("I AuthInterface LogInterface\n"));
}

#[test]
fn test_class_flags() {
    let mut hir = empty_hir();
    let mut class = make_class("AbstractRepo");
    class.class_flags = Some(vec!["ABSTRACT".into(), "EXPORT".into()]);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("cl: ABSTRACT EXPORT\n"));
}

#[test]
fn test_class_level_patterns() {
    let mut hir = empty_hir();
    let mut class = make_class("EmptyClass");
    class.patterns.push(make_pattern("EMPTY_CTOR", vec!["C1"]));
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("P EMPTY_CTOR C1\n"));
}

#[test]
fn test_method_level_patterns() {
    let mut hir = empty_hir();
    let mut class = make_class("Service");
    let mut method = make_method("create");
    method
        .patterns
        .push(make_pattern("CTOR", vec!["C1", "M1", "repo"]));
    class.methods.push(method);
    hir.classes.push(class);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    // Pattern should appear before the method declaration
    assert!(result.contains("P CTOR C1 M1 repo\n"));
    assert!(result.contains("M create\n"));
    // Method declaration should come AFTER the pattern
    let pat_pos = result.find("P CTOR").unwrap();
    let method_pos = result.find("M create").unwrap();
    assert!(pat_pos < method_pos, "Pattern should appear before method");
}

#[test]
fn test_imports() {
    let mut hir = empty_hir();
    hir.imports.push(make_import("IM1", "./module", "Foo"));
    hir.imports
        .push(make_import("IM2", "std::collections", "HashMap"));

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("$ IM1 ./module [Foo]\n"));
    assert!(result.contains("$ IM2 std::collections [HashMap]\n"));
}

#[test]
fn test_import_wildcard() {
    let mut hir = empty_hir();
    hir.imports.push(make_import("IM1", "react", "*"));

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("$ IM1 react\n"));
    assert!(!result.contains("[*]"));
}

#[test]
fn test_type_aliases() {
    let mut hir = empty_hir();
    hir.type_aliases.push(make_type_alias("UserId", "$n"));
    hir.type_aliases.push(make_type_alias("UserMap", "HashMap"));

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("T UserId = $n\n"));
    assert!(result.contains("T UserMap = HashMap\n"));
}

#[test]
fn test_full_typescript_class() {
    let mut hir = empty_hir();
    let mut class = make_class("UserListComponent");

    // Meta annotations (stored as type_aliases since Angular doesn't have multi-turn Φ in compile yet)
    // These would normally come from the Angular meta-layer as TypeAlias ops

    class.extends = Some("BaseListComponent".into());
    class.implements = vec!["OnInit".into(), "OnDestroy".into()];

    class.fields.push(make_field("users", Some("$s[]")));
    class.fields.push(make_field("selectedUser", Some("$n")));

    let mut m1 = make_method("ngOnInit");
    m1.flags = Some(vec!["IF".into()]);
    class.methods.push(m1);

    let mut m2 = make_method("trackById");
    m2.params
        .push(vec!["P1".into(), "$n".into(), "index".into()]);
    m2.params
        .push(vec!["P2".into(), "$s".into(), "user".into()]);
    m2.flags = Some(vec!["RET".into()]);
    class.methods.push(m2);

    hir.classes.push(class);

    // Add imports
    hir.imports
        .push(make_import("IM1", "./core", "OnInit, OnDestroy"));

    let result = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    assert!(result.contains("// ── UserListComponent ──"));
    assert!(result.contains("X BaseListComponent"));
    assert!(result.contains("I OnInit OnDestroy"));
    assert!(result.contains("F users:$s[]"));
    assert!(result.contains("F selectedUser:$n"));
    assert!(result.contains("M ngOnInit"));
    assert!(result.contains("M trackById"));
    assert!(result.contains("p:index:$n user:$s"));
    assert!(result.contains("fl:IF"));
    assert!(result.contains("fl:RET"));
    assert!(result.contains("$ IM1 ./core [OnInit, OnDestroy]"));
}

#[test]
fn test_full_rust_class() {
    let mut hir = empty_hir();
    let mut class = make_class("User");

    class.fields.push(make_field("id", Some("$n")));
    class.fields.push(make_field("name", Some("$s")));
    class.fields.push(make_field("email", Some("$s")));

    class.patterns.push(make_pattern("EMPTY_CTOR", vec!["C1"]));

    hir.classes.push(class);

    // Second class: UserService
    let mut svc = make_class("UserService");
    svc.extends = Some("Repository<User>".into());
    svc.fields.push(make_field("users", Some("HashMap")));
    svc.fields.push(make_field("cache", Some("RwLock")));

    let mut m1 = make_method("new");
    m1.flags = Some(vec!["CTOR".into()]);
    svc.methods.push(m1);

    let mut m2 = make_method("get_user");
    m2.params.push(vec!["P1".into(), "$n".into(), "id".into()]);
    m2.flags = Some(vec!["ASYNC".into()]);
    svc.methods.push(m2);

    hir.classes.push(svc);

    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── User ──"));
    assert!(result.contains("// ── UserService ──"));
    assert!(result.contains("P EMPTY_CTOR"));
    assert!(result.contains("X Repository<User>"));
    assert!(result.contains("F id:$n name:$s email:$s"));
    assert!(result.contains("M new"));
    assert!(result.contains("M get_user"));
}

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
    m1.flags = Some(vec!["RET".into()]);
    class.methods.push(m1);

    let mut m2 = make_method("find");
    m2.params.push(vec!["P1".into(), "$n".into(), "id".into()]);
    m2.flags = Some(vec!["RET".into()]);
    class.methods.push(m2);

    let mut m3 = make_method("find");
    m3.params
        .push(vec!["P1".into(), "$n".into(), "name".into()]);
    m3.params.push(vec!["P2".into(), "$n".into(), "age".into()]);
    m3.params
        .push(vec!["P3".into(), "$s".into(), "role".into()]);
    m3.flags = Some(vec!["RET".into(), "IF".into()]);
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
    assert!(result.contains("fl:RET,IF"));
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
fn test_injects_field_not_rendered_as_separate_line() {
    // Injects are part of the class node but not directly rendered as a
    // marker line in the current renderer (they flow through IR pipeline).
    // This test verifies they don't cause panics.
    let mut hir = empty_hir();
    let mut class = make_class("InjectedService");
    class.injects.push("Dep1".into());
    class.injects.push("Dep2".into());
    hir.classes.push(class);

    // Should render without error, injects are structural (pattern-level)
    // not directly rendered as standalone markers
    let result = render_hierarchical_for_llm(&hir, Fidelity::Low);
    assert!(result.contains("// ── InjectedService ──"));
}

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
    method.side_effect = Some("mutation".to_string());
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
    method.execution_context = Some("async".to_string());
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
    method.side_effect = Some("io".to_string());
    method.execution_context = Some("sync".to_string());
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
    method.side_effect = Some("io".to_string());
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
