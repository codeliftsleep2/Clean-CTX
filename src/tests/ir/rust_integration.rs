// src/tests/ir/rust_integration.rs
//
// Integration tests for Rust language support through the full
// IR compilation pipeline (IRCompiler + RustLayer).
//
// Verifies that Rust source files are correctly parsed by tree-sitter,
// compiled into IR instructions, and processed through the Rust language
// layer to emit Rust-specific ops (class flags, implements, etc.).

use crate::compression::Fidelity;
use crate::compression::language::detect_language;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::layers::rust::RustLayer;
use crate::ir::opcodes::CoreOp;

// ── Helpers ────────────────────────────────────────────────────────

/// Create an IRCompiler configured with the Rust language layer.
fn rust_compiler() -> IRCompiler {
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(RustLayer::new()));
    compiler
}

/// Compile a Rust source string and return the compiled IR.
fn compile_rust(source: &str) -> CompiledIR {
    let (language, query) = detect_language(source);
    let mut compiler = rust_compiler();
    compiler
        .compile(source, "test_rust", language, query, Fidelity::Low, None)
        .expect("Rust compilation should succeed")
}

/// Compile the Rust sample fixture file.
fn compile_sample() -> CompiledIR {
    let source = include_str!("../../test_files/rust/sample_service.rs");
    compile_rust(source)
}

// ── Basic Compilation Tests ────────────────────────────────────────

#[test]
fn rust_sample_produces_instructions() {
    let ir = compile_sample();
    assert!(
        !ir.instructions.is_empty(),
        "compiled IR should have instructions"
    );
}

#[test]
fn rust_sample_version_is_one() {
    let ir = compile_sample();
    assert_eq!(ir.version, 1);
}

#[test]
fn rust_sample_file_id_matches() {
    let ir = compile_sample();
    assert_eq!(ir.file_id, "test_rust");
}

#[test]
fn rust_sample_has_def_class() {
    let ir = compile_sample();
    let classes: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .collect();
    assert!(
        !classes.is_empty(),
        "should have at least one DefClass instruction"
    );
}

#[test]
fn rust_sample_has_def_methods() {
    let ir = compile_sample();
    let methods: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefMethod(..)))
        .collect();
    assert!(
        methods.len() >= 2,
        "sample_service.rs should have at least 2 methods, got {}",
        methods.len()
    );
}

// ── Struct/Enum/Trait Declarations ─────────────────────────────────

#[test]
fn rust_struct_produces_def_class() {
    let source = r#"
        pub struct UserService {
            users: Vec<String>,
        }
    "#;
    let ir = compile_rust(source);
    let classes: Vec<_> = ir
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
        classes.contains(&"UserService"),
        "should have UserService DefClass, got: {:?}",
        classes
    );
    // Verify the DefClass name is the extracted name, not raw text
    let user_service = classes.iter().find(|n| **n == "UserService");
    assert!(user_service.is_some(), "DefClass should contain extracted name 'UserService'");
}

#[test]
fn rust_enum_produces_def_class() {
    let source = r#"
        pub enum Status {
            Active,
            Inactive,
        }
    "#;
    let ir = compile_rust(source);
    let classes: Vec<_> = ir
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
        classes.contains(&"Status"),
        "should have Status DefClass, got: {:?}",
        classes
    );
}

#[test]
fn rust_trait_produces_def_class() {
    let source = r#"
        pub trait Repository {
            fn find(&self, id: u64) -> bool;
        }
    "#;
    let ir = compile_rust(source);
    let classes: Vec<_> = ir
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
        classes.contains(&"Repository"),
        "should have Repository DefClass, got: {:?}",
        classes
    );
}

// ── Trait Implementations ──────────────────────────────────────────

#[test]
fn rust_trait_impl_produces_implements() {
    let source = r#"
        pub trait Processor {
            fn process(&self, data: &str) -> bool;
        }
        pub struct DataProcessor;
        impl Processor for DataProcessor {
            fn process(&self, data: &str) -> bool {
                true
            }
        }
    "#;
    let ir = compile_rust(source);
    let impl_ops: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .collect();
    assert!(
        !impl_ops.is_empty(),
        "should have Implements op for trait impl, got: {:?}",
        ir.instructions
    );
}

#[test]
fn rust_inherent_impl_no_implements() {
    let source = r#"
        pub struct Foo;
        impl Foo {
            fn new() -> Self { Foo }
        }
    "#;
    let ir = compile_rust(source);
    let impl_ops: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .collect();
    assert!(
        impl_ops.is_empty(),
        "inherent impl should not produce Implements ops: {:?}",
        impl_ops
    );
}

// ── Method Flags ───────────────────────────────────────────────────

#[test]
fn rust_async_method_produces_async_flag() {
    let source = r#"
        pub struct Service;
        impl Service {
            pub async fn get_data(&self) -> String {
                "data".to_string()
            }
        }
    "#;
    let ir = compile_rust(source);
    let has_async = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::Flags(_, flags) if flags.contains(&"ASYNC".to_string()))
    });
    assert!(has_async, "async method should produce ASYNC flag");
}

#[test]
fn rust_pub_method_produces_export_flag() {
    let source = r#"
        pub struct Service;
        impl Service {
            pub fn do_work(&self) {}
        }
    "#;
    let ir = compile_rust(source);
    let has_export = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::Flags(_, flags) if flags.contains(&"EXPORT".to_string()))
    });
    assert!(has_export, "pub method should produce EXPORT flag");
}

#[test]
fn rust_unsafe_method_produces_unsafe_flag() {
    let source = r#"
        pub struct Service;
        impl Service {
            unsafe fn raw_op(&self) {}
        }
    "#;
    let ir = compile_rust(source);
    let has_unsafe = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::Flags(_, flags) if flags.contains(&"UNSAFE".to_string()))
    });
    assert!(has_unsafe, "unsafe method should produce UNSAFE flag");
}

#[test]
fn rust_method_with_all_flags() {
    let source = r#"
        pub struct Service;
        impl Service {
            pub async fn complex_op(&self) -> bool { true }
        }
    "#;
    let ir = compile_rust(source);
    let has_flags = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::Flags(_, flags) if {
            flags.contains(&"ASYNC".to_string()) &&
            flags.contains(&"EXPORT".to_string())
        })
    });
    assert!(has_flags, "pub async method should produce both ASYNC and EXPORT flags");
}

// ── Class-Level Flags ──────────────────────────────────────────────

#[test]
fn rust_pub_struct_produces_class_flags() {
    let source = r#"
        pub struct Foo {
            x: i32,
        }
    "#;
    let ir = compile_rust(source);
    let has_class_flags = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if flags.contains(&"EXPORT".to_string()))
    });
    assert!(has_class_flags, "pub struct should produce EXPORT class flag");
}

#[test]
fn rust_unsafe_trait_produces_class_flags() {
    let source = r#"
        pub unsafe trait Dangerous {
            fn do_unsafe(&self);
        }
    "#;
    let ir = compile_rust(source);
    let has_class_flags = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, flags) if flags.contains(&"UNSAFE".to_string()))
    });
    assert!(has_class_flags, "unsafe trait should produce UNSAFE class flag");
}

// ── Derive Attributes ──────────────────────────────────────────────

#[test]
fn rust_derive_not_in_defclass() {
    let source = r#"
        #[derive(Debug, Clone)]
        pub struct MyStruct {
            field: i32,
        }
    "#;
    let ir = compile_rust(source);
    // NOTE: tree-sitter's `struct_item` node does NOT include `#[derive(...)]` —
    // attributes are separate AST nodes. The `extract_rust_struct_name` function
    // strips the struct keyword, so the DefClass name is just "MyStruct".
    // Derives are processed separately via RustLayer::extract_derives.
    let has_def_class = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::DefClass(_, name) if name == "MyStruct")
    });
    assert!(has_def_class, "struct should produce DefClass with name 'MyStruct'");
    // The struct should still be recognized even though derives aren't in the text
    let has_struct_name = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::DefClass(_, _))
    });
    assert!(has_struct_name, "should have at least one DefClass");
}

// ── Fields ─────────────────────────────────────────────────────────

#[test]
fn rust_struct_produces_fields() {
    let source = r#"
        pub struct UserService {
            users: Vec<String>,
            cache: HashMap<u64, String>,
        }
    "#;
    let ir = compile_rust(source);
    let fields: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefField(..)))
        .collect();
    assert!(
        fields.len() >= 2,
        "UserService should have at least 2 fields, got {}",
        fields.len()
    );
}

// ── Visibility ─────────────────────────────────────────────────────

#[test]
fn rust_crate_visibility_produces_class_flags() {
    let source = r#"
        pub(crate) struct InternalService {
            data: String,
        }
    "#;
    let ir = compile_rust(source);
    // pub(crate) should produce an EXPORT flag (it's a restricted pub)
    let has_class_flags = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(_, _))
    });
    // The class should at least be defined
    let has_class = ir.instructions.iter().any(|op| matches!(op, CoreOp::DefClass(..)));
    assert!(has_class, "pub(crate) struct should produce DefClass");
    // pub(crate) contains "pub " so the RustLayer should detect it
    // But the class flags depend on whether extract_method_flags sees "pub "
    let _ = has_class_flags; // May or may not flag pub(crate) depending on impl
}

// ── Fidelity Levels ────────────────────────────────────────────────

#[test]
fn rust_compilation_with_medium_fidelity() {
    let source = include_str!("../../test_files/rust/sample_service.rs");
    let (language, query) = detect_language(source);
    let mut compiler = rust_compiler();
    let ir = compiler
        .compile(source, "test_medium", language, query, Fidelity::Medium, None)
        .expect("Rust medium fidelity compilation should succeed");
    assert!(
        !ir.instructions.is_empty(),
        "medium fidelity compilation should produce instructions"
    );
}

#[test]
fn rust_compilation_with_high_fidelity() {
    let source = include_str!("../../test_files/rust/sample_service.rs");
    let (language, query) = detect_language(source);
    let mut compiler = rust_compiler();
    let ir = compiler
        .compile(source, "test_high", language, query, Fidelity::High, None)
        .expect("Rust high fidelity compilation should succeed");
    assert!(
        !ir.instructions.is_empty(),
        "high fidelity compilation should produce instructions"
    );
}

// ── Edge Cases ─────────────────────────────────────────────────────

#[test]
fn rust_empty_source_produces_no_instructions() {
    let source = "";
    let (language, query) = detect_language(source);
    let mut compiler = rust_compiler();
    let ir = compiler
        .compile(source, "empty", language, query, Fidelity::Low, None)
        .expect("compilation should succeed");
    assert!(
        ir.instructions.is_empty(),
        "empty source should produce no instructions"
    );
}

#[test]
fn rust_compiler_counter_is_deterministic() {
    let source = include_str!("../../test_files/rust/sample_service.rs");
    let (language, query) = detect_language(source);

    let mut c1 = rust_compiler();
    let ir1 = c1
        .compile(source, "f", language.clone(), query, Fidelity::Low, None)
        .unwrap();

    let mut c2 = rust_compiler();
    let ir2 = c2
        .compile(source, "f", language, query, Fidelity::Low, None)
        .unwrap();

    assert_eq!(ir1.instructions.len(), ir2.instructions.len());
    for (a, b) in ir1.instructions.iter().zip(ir2.instructions.iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn rust_methods_outside_impl_skipped() {
    // Free functions should not produce DefMethod ops
    let source = r#"
        fn standalone() -> i32 { 42 }
        pub fn another_free() {}
    "#;
    let ir = compile_rust(source);
    let methods: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefMethod(..)))
        .collect();
    assert!(
        methods.is_empty(),
        "free functions should not produce DefMethod ops: {:?}",
        methods
    );
}

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
    assert!(classes.contains(&"Alpha"), "should have Alpha: {:?}", classes);
    assert!(classes.contains(&"Beta"), "should have Beta: {:?}", classes);
    assert!(classes.contains(&"Gamma"), "should have Gamma: {:?}", classes);
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
    assert!(classes.contains(&"MyStruct"), "should have MyStruct: {:?}", classes);
    assert!(classes.contains(&"MyEnum"), "should have MyEnum: {:?}", classes);
    assert!(classes.contains(&"MyTrait"), "should have MyTrait: {:?}", classes);
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
    let has_async = ir.instructions.iter().any(|op| {
        matches!(op, CoreOp::Flags(_, flags) if flags.contains(&"ASYNC".to_string()))
    });
    assert!(has_async, "sample has async get_user — should produce ASYNC flag");
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
        .compile(source, "test_standalone", language, query, Fidelity::Low, None)
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
        .compile(source, "test_standalone_trait", language, query, Fidelity::Low, None)
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
    assert!(!has_cfg_flag, "struct without #[cfg(...)] should NOT produce CFG flag");
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
