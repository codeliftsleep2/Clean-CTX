// src/tests/ir/layers/mod.rs
//
// Tests for the Layered Encoding module (Phase F).
// Verifies LanguageLayer, MetaLayer, and PatternRecognizer traits.

mod java;

use crate::compression::Fidelity;
use crate::ir::layers::LayerContext;
use crate::ir::layers::LanguageLayer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::layers::csharp::CSharpLayer;
// P0-4: ir::layers::MetaLayer and angular/spring/dotnet modules removed.
// Meta-layers now use the canonical trait in src/layers/meta/.
// LanguageLayer tests remain (TypeScript, C#).
use crate::ir::opcodes::CoreOp;

// ── TypeScript Layer Tests ────────────────────────────

#[test]
fn ts_layer_has_correct_name() {
    let layer = TypeScriptLayer::new();
    assert_eq!(layer.name(), "typescript");
}

#[test]
fn ts_layer_extracts_extends_from_class() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("class Foo extends Bar {}", Fidelity::Low);
    ctx.current_class = Some("C1".into());
    ctx.current_class_name = Some("Foo".into());

    let ops = layer.process_capture("class.root", "class Foo extends Bar {}", &mut ctx);

    let has_extend = ops.iter().any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "Bar"));
    assert!(has_extend, "TypeScript layer should emit EXT op for extends: {:?}", ops);
}

#[test]
fn ts_layer_extracts_implements_from_class() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("class Foo implements Bar, Baz {}", Fidelity::Low);
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "class Foo implements Bar, Baz {}", &mut ctx);

    let implement_count = ops.iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .count();
    assert_eq!(implement_count, 2, "Should emit 2 IMPL ops for 2 interfaces: {:?}", ops);
}

#[test]
fn ts_layer_extracts_export_flag() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("export class Foo {}", Fidelity::Low);
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "export class Foo {}", &mut ctx);

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(has_export, "TypeScript layer should emit EXPORT flag: {:?}", ops);
}

#[test]
fn ts_layer_extracts_async_flag() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("async doWork()", Fidelity::Low);
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "async doWork()", &mut ctx);

    let has_async = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"ASYNC".to_string()))
    });
    assert!(has_async, "TypeScript layer should emit ASYNC flag: {:?}", ops);
}

#[test]
fn ts_layer_does_nothing_for_unknown_capture() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("", Fidelity::Low);

    let ops = layer.process_capture("unknown.capture", "some text", &mut ctx);
    assert!(ops.is_empty(), "Unknown captures should produce no ops");
}

#[test]
fn ts_layer_extracts_static_flag() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("static doWork()", Fidelity::Low);
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "static doWork()", &mut ctx);

    let has_static = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"STATIC".to_string()))
    });
    assert!(has_static, "TypeScript layer should emit STATIC flag: {:?}", ops);
}

// ── C# Layer Tests ────────────────────────────────────

#[test]
fn cs_layer_has_correct_name() {
    let layer = CSharpLayer::new();
    assert_eq!(layer.name(), "csharp");
}

#[test]
fn cs_layer_extracts_inheritance_with_colon() {
    let mut layer = CSharpLayer::new();
    let mut ctx = LayerContext::new("public class MyClass : BaseClass", Fidelity::Low);
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public class MyClass : BaseClass", &mut ctx);

    let has_extend = ops.iter().any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "BaseClass"));
    assert!(has_extend, "C# layer should emit EXT op for inheritance: {:?}", ops);
}

#[test]
fn cs_layer_extracts_interfaces_from_colon() {
    let mut layer = CSharpLayer::new();
    let mut ctx = LayerContext::new("class MyClass : BaseClass, IInterface1, IInterface2", Fidelity::Low);
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "class MyClass : BaseClass, IInterface1, IInterface2", &mut ctx);

    // First item after : is the base class (Extends), rest are interfaces (Implements)
    let has_extend = ops.iter().any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "BaseClass"));
    assert!(has_extend, "C# should emit EXT for base class: {:?}", ops);

    let implement_count = ops.iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .count();
    assert_eq!(implement_count, 2, "C# should emit 2 IMPL ops for interfaces: {:?}", ops);
}

#[test]
fn cs_layer_extracts_public_flag() {
    let mut layer = CSharpLayer::new();
    let mut ctx = LayerContext::new("public class Foo {}", Fidelity::Low);
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public class Foo {}", &mut ctx);

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(has_export, "C# layer should emit EXPORT for public class: {:?}", ops);
}

#[test]
fn cs_layer_extracts_abstract_flag() {
    let mut layer = CSharpLayer::new();
    let mut ctx = LayerContext::new("public abstract class Foo {}", Fidelity::Low);
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public abstract class Foo {}", &mut ctx);

    let has_abstract = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"ABSTRACT".to_string()))
    });
    assert!(has_abstract, "C# layer should emit ABSTRACT flag: {:?}", ops);
}

// ── Layer Context Tests ───────────────────────────────

#[test]
fn layer_context_initializes_correctly() {
    let ctx = LayerContext::new("source code", Fidelity::High);
    assert_eq!(ctx.fidelity, Fidelity::High);
    assert_eq!(ctx.source, "source code");
    assert!(ctx.current_class.is_none());
    assert!(ctx.current_method.is_none());
    assert!(ctx.symbol_table.is_empty());
}

#[test]
fn layer_context_accepts_updates() {
    let mut ctx = LayerContext::new("source", Fidelity::Low);
    ctx.current_class = Some("C1".into());
    ctx.current_class_name = Some("MyClass".into());
    ctx.current_method = Some("M1".into());

    assert_eq!(ctx.current_class.as_deref(), Some("C1"));
    assert_eq!(ctx.current_class_name.as_deref(), Some("MyClass"));
    assert_eq!(ctx.current_method.as_deref(), Some("M1"));
}

// ── LanguageLayer Trait Tests ─────────────────────────

#[test]
fn layer_finalize_default_returns_empty() {
    let mut layer = TypeScriptLayer::new();
    let mut ctx = LayerContext::new("", Fidelity::Low);
    let ops = LanguageLayer::finalize(&mut layer, &mut ctx);
    assert!(ops.is_empty(), "Default finalize should return empty vec");
}

// ── Meta-Layer Integration Tests (P0-4) ────────────────
// Meta-layer tests now use the canonical LayerRegistry instead of the
// removed ir::layers::angular/spring/dotnet modules. See:
//   - src/tests/angular_meta/ (pre-existing Angular tests)
//   - src/tests/spring_meta/  (pre-existing Spring tests)
//   - src/tests/dotnet_meta/  (pre-existing .NET tests)
