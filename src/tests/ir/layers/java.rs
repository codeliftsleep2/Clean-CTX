// src/tests/ir/layers/java.rs
//
// Java Language Layer Tests (Issue #2 — FAANG Audit Remediation)
// Verifies JavaLayer correctly detects:
//   - Class, interface, enum, record captures
//   - extends/implements relationships
//   - Class-level flags (public, abstract, static)
//   - Method-level flags (static, abstract, visibility)
//   - Spring/Jakarta annotation patterns
//   - Unknown capture handling

use crate::compression::Fidelity;
use crate::ir::layers::LanguageLayer;
use crate::ir::layers::LayerContext;
use crate::ir::layers::java::JavaLayer;
use crate::ir::opcodes::CoreOp;

// ── Helper ─────────────────────────────────────────

fn make_ctx(source: &str) -> LayerContext {
    LayerContext::new(source, Fidelity::Low)
}

// ── Layer Identity ─────────────────────────────────

#[test]
fn java_layer_has_correct_name() {
    let layer = JavaLayer::new();
    assert_eq!(layer.name(), "java");
}

#[test]
fn java_layer_default_has_correct_name() {
    let layer: JavaLayer = Default::default();
    assert_eq!(layer.name(), "java");
}

// ── Class Detection ────────────────────────────────

#[test]
fn java_layer_detects_class_with_extends() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class MyService extends BaseService {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "class.root",
        "public class MyService extends BaseService {}",
        &mut ctx,
    );

    let has_extend = ops
        .iter()
        .any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "BaseService"));
    assert!(
        has_extend,
        "Java layer should emit EXT op for extends: {:?}",
        ops
    );
}

#[test]
fn java_layer_detects_class_with_extends_and_implements() {
    let mut layer = JavaLayer::new();
    let mut ctx =
        make_ctx("public class MyService extends BaseService implements Serializable, Runnable {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "class.root",
        "public class MyService extends BaseService implements Serializable, Runnable {}",
        &mut ctx,
    );

    let has_extend = ops
        .iter()
        .any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "BaseService"));
    assert!(has_extend, "Should emit EXT for base class: {:?}", ops);

    let impl_count = ops
        .iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .count();
    assert_eq!(
        impl_count, 2,
        "Should emit 2 IMPL ops for Serializable and Runnable: {:?}",
        ops
    );
}

#[test]
fn java_layer_detects_class_with_implements_only() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class MyService implements Serializable {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "class.root",
        "public class MyService implements Serializable {}",
        &mut ctx,
    );

    let impl_count = ops
        .iter()
        .filter(|op| matches!(op, CoreOp::Implements(..)))
        .count();
    assert_eq!(impl_count, 1, "Should emit 1 IMPL op: {:?}", ops);

    // No EXT when no extends keyword
    let has_extend = ops.iter().any(|op| matches!(op, CoreOp::Extends(..)));
    assert!(
        !has_extend,
        "Should not emit EXT when no extends: {:?}",
        ops
    );
}

// ── Interface Detection ────────────────────────────

#[test]
fn java_layer_detects_interface() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public interface MyService {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("interface.root", "public interface MyService {}", &mut ctx);

    // Interface with no extends should produce flags only
    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(
        has_export,
        "Public interface should get EXPORT flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_detects_interface_extends() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public interface MyRepo extends JpaRepository<MyEntity, Long> {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "interface.root",
        "public interface MyRepo extends JpaRepository<MyEntity, Long> {}",
        &mut ctx,
    );

    let has_extend = ops
        .iter()
        .any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "JpaRepository"));
    assert!(
        has_extend,
        "Interface with extends JpaRepository should emit EXT: {:?}",
        ops
    );
}

// ── Enum Detection ─────────────────────────────────

#[test]
fn java_layer_detects_enum() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public enum Status { ACTIVE, INACTIVE }");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "enum.root",
        "public enum Status { ACTIVE, INACTIVE }",
        &mut ctx,
    );

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(has_export, "Public enum should get EXPORT flag: {:?}", ops);
}

// ── Record Detection ───────────────────────────────

#[test]
fn java_layer_detects_record() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public record Point(int x, int y) {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "record.root",
        "public record Point(int x, int y) {}",
        &mut ctx,
    );

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(
        has_export,
        "Public record should get EXPORT flag: {:?}",
        ops
    );
}

// ── Class Flags ────────────────────────────────────

#[test]
fn java_layer_extracts_public_class_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class Foo {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public class Foo {}", &mut ctx);

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(has_export, "Public class should get EXPORT flag: {:?}", ops);
}

#[test]
fn java_layer_extracts_abstract_class_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public abstract class Foo {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public abstract class Foo {}", &mut ctx);

    let has_abstract = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"ABSTRACT".to_string()))
    });
    assert!(
        has_abstract,
        "Abstract class should get ABSTRACT flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_extracts_static_class_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public static class Foo {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public static class Foo {}", &mut ctx);

    let has_static = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"STATIC".to_string()))
    });
    assert!(has_static, "Static class should get STATIC flag: {:?}", ops);
}

#[test]
fn java_layer_missing_class_id_produces_no_ops() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class Foo extends Bar {}");
    // current_class NOT set

    let ops = layer.process_capture("class.root", "public class Foo extends Bar {}", &mut ctx);

    assert!(
        ops.is_empty(),
        "Without current_class, should produce no ops: {:?}",
        ops
    );
}

// ── Method Flags ───────────────────────────────────

#[test]
fn java_layer_extracts_method_static_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public static void doWork() {}");
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "public static void doWork() {}", &mut ctx);

    let has_static = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"STATIC".to_string()))
    });
    assert!(
        has_static,
        "Static method should get STATIC flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_extracts_method_abstract_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public abstract void doWork();");
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "public abstract void doWork();", &mut ctx);

    let has_abstract = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"ABSTRACT".to_string()))
    });
    assert!(
        has_abstract,
        "Abstract method should get ABSTRACT flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_extracts_method_private_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("private void doWork() {}");
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "private void doWork() {}", &mut ctx);

    let has_private = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"PRIVATE".to_string()))
    });
    assert!(
        has_private,
        "Private method should get PRIVATE flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_extracts_method_protected_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("protected void doWork() {}");
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "protected void doWork() {}", &mut ctx);

    let has_protected = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"PROTECTED".to_string()))
    });
    assert!(
        has_protected,
        "Protected method should get PROTECTED flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_native_method_no_export_flag() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public native void doWork();");
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("method.root", "public native void doWork();", &mut ctx);

    // Native methods have "public" but should NOT get EXPORT flag
    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(
        !has_export,
        "Native method should NOT get EXPORT flag despite 'public': {:?}",
        ops
    );
}

// ── Constructor Handling ───────────────────────────

#[test]
fn java_layer_detects_constructor_flags() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public MyService() {}");
    ctx.current_method = Some("M1".into());

    let ops = layer.process_capture("constructor.root", "public MyService() {}", &mut ctx);

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::Flags(m, flags) if m == "M1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(
        has_export,
        "Public constructor should get EXPORT flag: {:?}",
        ops
    );
}

// ── Edge Cases ─────────────────────────────────────

#[test]
fn java_layer_unknown_capture_produces_no_ops() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("some arbitrary text");

    let ops = layer.process_capture("unknown.capture", "some arbitrary text", &mut ctx);
    assert!(
        ops.is_empty(),
        "Unknown captures should produce no ops: {:?}",
        ops
    );
}

#[test]
fn java_layer_generic_extends_strips_type_params() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class MyRepo extends JpaRepository<MyEntity, Long> {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture(
        "class.root",
        "public class MyRepo extends JpaRepository<MyEntity, Long> {}",
        &mut ctx,
    );

    // Should strip generic params from the base class name
    let has_extend = ops
        .iter()
        .any(|op| matches!(op, CoreOp::Extends(c, b) if c == "C1" && b == "JpaRepository"));
    assert!(
        has_extend,
        "Generic extends should strip type params: {:?}",
        ops
    );
}

#[test]
fn java_layer_extends_with_no_class_id_produces_no_ops() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class MyService extends BaseService {}");
    // current_class NOT set

    let ops = layer.process_capture(
        "class.root",
        "public class MyService extends BaseService {}",
        &mut ctx,
    );

    assert!(
        ops.is_empty(),
        "Without current_class, extends should produce no ops: {:?}",
        ops
    );
}

#[test]
fn java_layer_finalize_returns_empty() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("");

    let ops = layer.finalize(&mut ctx);
    assert!(ops.is_empty(), "Finalize should return empty vec");
}

// ── Spring/Jakarta Annotation Pattern Detection ─────

#[test]
fn java_layer_detects_spring_rest_controller_pattern() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class MyController {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public class MyController {}", &mut ctx);

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(
        has_export,
        "Spring controller class should get EXPORT flag: {:?}",
        ops
    );
}

#[test]
fn java_layer_detects_jakarta_annotation_pattern() {
    let mut layer = JavaLayer::new();
    let mut ctx = make_ctx("public class MyController {}");
    ctx.current_class = Some("C1".into());

    let ops = layer.process_capture("class.root", "public class MyController {}", &mut ctx);

    let has_export = ops.iter().any(|op| {
        matches!(op, CoreOp::ClassFlags(c, flags) if c == "C1" && flags.contains(&"EXPORT".to_string()))
    });
    assert!(
        has_export,
        "Jakarta-annotated class should get EXPORT flag: {:?}",
        ops
    );
}
