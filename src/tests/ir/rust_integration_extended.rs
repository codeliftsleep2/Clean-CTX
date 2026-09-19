use super::*;

// ── Multiple Types ─────────────────────────────────────────────────

#[test]
fn rust_multiple_structs_all_captured() {
    let source = r#"
        pub struct Alpha { x: i32 }
        pub struct Beta { y: String }
        pub struct Gamma { z: bool }
    "#;
    let ir = compile_rust(source);
    let classes: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        classes.contains(&"Alpha"),
        "should have Alpha: {:?}",
        classes
    );
    assert!(classes.contains(&"Beta"), "should have Beta: {:?}", classes);
    assert!(
        classes.contains(&"Gamma"),
        "should have Gamma: {:?}",
        classes
    );
}

#[test]
fn rust_mixed_struct_enum_trait_all_captured() {
    let source = r#"
        pub struct MyStruct { x: i32 }
        pub enum MyEnum { A, B }
        pub trait MyTrait { fn method(&self); }
    "#;
    let ir = compile_rust(source);
    let classes: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        classes.contains(&"MyStruct"),
        "should have MyStruct: {:?}",
        classes
    );
    assert!(
        classes.contains(&"MyEnum"),
        "should have MyEnum: {:?}",
        classes
    );
    assert!(
        classes.contains(&"MyTrait"),
        "should have MyTrait: {:?}",
        classes
    );
}

// ── Generics ───────────────────────────────────────────────────────

#[test]
fn rust_generic_struct_captured() {
    let source = r#"
        pub struct Repository<T> {
            items: Vec<T>,
        }
    "#;
    let ir = compile_rust(source);
    let classes: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        classes.iter().any(|c| c.contains("Repository")),
        "should have Repository: {:?}",
        classes
    );
}

// ── Sample File Specific Assertions ────────────────────────────────

#[test]
fn rust_sample_has_struct_and_trait_and_impl() {
    let ir = compile_sample();
    let class_names: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();

    // sample_service.rs defines: User, Role, UserService, DataProcessor
    // (structs/enums/traits inside the impl blocks may or may not be separate)
    assert!(
        class_names.iter().any(|n| n.contains("UserService")),
        "should have UserService, got: {:?}",
        class_names
    );
    assert!(
        class_names.iter().any(|n| n.contains("DataProcessor")),
        "should have DataProcessor, got: {:?}",
        class_names
    );
}

#[test]
fn rust_sample_trait_impl_produces_implements() {
    let ir = compile_sample();
    let impl_ops: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .collect();
    assert!(
        !impl_ops.is_empty(),
        "sample has `impl Repository for UserService` — should produce Implements ops"
    );
}

#[test]
fn rust_sample_async_methods_have_flags() {
    let ir = compile_sample();
    let has_async = ir
        .instructions
        .iter()
        .any(|op| matches!(op, CoreOp::MethodModifiers(_, modifiers) if modifiers.contains(&DeclarationModifier::Async)));
    assert!(
        has_async,
        "sample has async get_user — should produce ASYNC modifier"
    );
}

// ── Phase C Regression Tests (P5) ─────────────────────────────────
//
// These tests verify that a standalone impl block (no preceding struct/
// enum/trait) still produces a DefClass in the IR, matching the text
// pipeline's behavior. Previously the IR pipeline silently dropped
// methods inside orphaned impl blocks.

/// Regression: a standalone inherent impl with no preceding struct
/// should produce a DefClass for the self-type.
#[test]
fn rust_standalone_inherent_impl_produces_def_class() {
    let source = r#"
        impl ForeignType {
            fn helper_one(&self) -> i32 { 42 }
            pub fn helper_two(&self) -> String { "ok".to_string() }
        }
    "#;
    let (language, query) = detect_language(source);
    let mut compiler = rust_compiler();
    let ir = compiler
        .compile(
            source,
            "test_standalone",
            language,
            query,
            Fidelity::Low,
            None,
        )
        .expect("compilation should succeed");

    let class_names: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        class_names.contains(&"ForeignType"),
        "standalone impl should produce DefClass for ForeignType, got: {:?}",
        class_names
    );

    // Methods inside the impl should also be present
    let method_count = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefMethod(..)))
        .count();
    assert!(
        method_count >= 2,
        "standalone impl should have at least 2 methods, got {}",
        method_count
    );
}

/// Regression: a standalone trait impl with no preceding struct
/// should produce both a DefClass and Implements ops.
#[test]
fn rust_standalone_trait_impl_produces_def_class_and_implements() {
    let source = r#"
        impl Processor for MyProcessor {
            fn process(&self) -> bool { true }
        }
    "#;
    let (language, query) = detect_language(source);
    let mut compiler = rust_compiler();
    let ir = compiler
        .compile(
            source,
            "test_standalone_trait",
            language,
            query,
            Fidelity::Low,
            None,
        )
        .expect("compilation should succeed");

    let class_names: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        class_names.contains(&"MyProcessor"),
        "standalone trait impl should produce DefClass for MyProcessor, got: {:?}",
        class_names
    );

    let impl_ops: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .collect();
    assert!(
        !impl_ops.is_empty(),
        "standalone trait impl should produce Implements ops"
    );
}

/// Existing behavior: impl that follows a struct definition should NOT
/// create a duplicate DefClass (the struct already created one).
#[test]
fn rust_impl_after_struct_does_not_duplicate_def_class() {
    let source = r#"
        pub struct MyStruct { x: i32 }
        impl MyStruct {
            pub fn new() -> Self { MyStruct { x: 0 } }
        }
    "#;
    let ir = compile_rust(source);
    let class_count = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .count();
    // Should have exactly 1 DefClass (from the struct, not the impl)
    assert_eq!(
        class_count, 1,
        "struct + impl should produce exactly 1 DefClass, got {}",
        class_count
    );
}

// ── Phase B Regression Tests (P3 + P4) ─────────────────────────────
//
// These tests verify that extract_cfg and extract_generic_params are
// ACTUALLY CALLED during IR compilation. They guard against regressions
// where these functions are defined but not wired into the pipeline.

/// Regression: extract_cfg must be called during compilation.
/// A struct with #[cfg(feature = "...")] should produce CFG(...) in ClassFlags.
#[test]
fn rust_cfg_struct_produces_cfg_flag() {
    let source = r#"
        #[cfg(feature = "unstable")]
        pub struct UnstableFeature {
            field: i32,
        }
    "#;
    let ir = compile_rust(source);
    let has_cfg_flag = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if {
            flags.iter().any(|f| f.starts_with("CFG("))
        })
    });
    assert!(
        has_cfg_flag,
        "struct with #[cfg(...)] should produce CFG(...) class flag, got instructions:\n{:?}",
        ir.instructions
    );
}

/// Regression: extract_cfg still returns None for uncfg'd structs.
#[test]
fn rust_plain_struct_no_cfg_flag() {
    let source = r#"
        pub struct Simple {
            x: i32,
        }
    "#;
    let ir = compile_rust(source);
    let has_cfg_flag = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if {
            flags.iter().any(|f| f.starts_with("CFG("))
        })
    });
    assert!(
        !has_cfg_flag,
        "struct without #[cfg(...)] should NOT produce CFG flag"
    );
}

/// Regression: extract_generic_params must be called at Medium fidelity.
/// A generic struct should produce GP<T> in ClassFlags.
#[test]
fn rust_generic_struct_medium_produces_gp_flag() {
    let source = r#"
        pub struct Repository<T> {
            items: Vec<T>,
        }
    "#;
    let (language, query) = detect_language(source);
    let mut compiler = crate::ir::compiler::IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::rust::RustLayer::new()));
    let ir = compiler
        .compile(source, "test_gp", language, query, Fidelity::Medium, None)
        .expect("compilation should succeed");

    let has_gp_flag = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if {
            flags.iter().any(|f| f.contains("GP<") || f == "GP<T>")
        })
    });
    assert!(
        has_gp_flag,
        "generic struct at Medium fidelity should produce GP<T> flag, got instructions:\n{:?}",
        ir.instructions
    );
}

/// Regression: at Low fidelity, generic params should NOT produce GP flag.
#[test]
fn rust_generic_struct_low_no_gp_flag() {
    let source = r#"
        pub struct Repository<T> {
            items: Vec<T>,
        }
    "#;
    let ir = compile_rust(source); // compile_rust uses Low fidelity
    let has_gp_flag = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if {
            flags.iter().any(|f| f.contains("GP<"))
        })
    });
    assert!(
        !has_gp_flag,
        "generic struct at Low fidelity should NOT produce GP flag"
    );
}

/// Regression: extract_generic_params AND extract_cfg both called
/// simultaneously for a cfg'd generic struct at Medium fidelity.
#[test]
fn rust_cfg_generic_struct_medium_has_both_flags() {
    let source = r#"
        #[cfg(feature = "nightly")]
        pub struct NightlyRepo<T> {
            data: T,
        }
    "#;
    let (language, query) = detect_language(source);
    let mut compiler = crate::ir::compiler::IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::rust::RustLayer::new()));
    let ir = compiler
        .compile(source, "test_both", language, query, Fidelity::Medium, None)
        .expect("compilation should succeed");

    let has_cfg = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if {
            flags.iter().any(|f| f.starts_with("CFG("))
        })
    });
    assert!(has_cfg, "should have CFG flag at Medium");

    let has_gp = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if {
            flags.iter().any(|f| f.contains("GP<"))
        })
    });
    assert!(has_gp, "should have GP flag at Medium");
}
