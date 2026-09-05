// src/tests/layers/meta/builtin.rs
//
// Tests for BuiltinMetaLayer semantic edge emission (Phase 2).
// Verifies that generic `Extends` relations are emitted from the
// `extends` keyword, and that C# `:` syntax is correctly deferred.

use crate::compression::Fidelity;
use crate::layers::meta::MetaLayer;
use crate::layers::meta::builtin::BuiltinMetaLayer;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};

fn class_capture(name: &str, text: &str) -> (String, String) {
    (name.to_string(), text.to_string())
}

fn find_edges_by_relation(
    edges: &[SemanticEdge],
    relation: SemanticRelation,
) -> Vec<&SemanticEdge> {
    edges.iter().filter(|e| e.relation == relation).collect()
}

// ── Extends from `extends` keyword (Java + TypeScript) ──────────────────

#[test]
fn builtin_emits_extends_for_java_extends() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture(
        "class.root",
        "public class OrderRepository extends BaseRepository {}",
    )];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let extends_edges = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert_eq!(extends_edges.len(), 1);
    assert_eq!(extends_edges[0].subject.name, "OrderRepository");
    assert_eq!(extends_edges[0].object.name, "BaseRepository");
    assert_eq!(extends_edges[0].layer, "builtin");
}

#[test]
fn builtin_emits_extends_for_typescript_extends() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture(
        "class.root",
        "export class UserComponent extends BaseComponent {}",
    )];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let extends_edges = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert_eq!(extends_edges.len(), 1);
    assert_eq!(extends_edges[0].subject.name, "UserComponent");
    assert_eq!(extends_edges[0].object.name, "BaseComponent");
}

#[test]
fn builtin_emits_extends_with_decorator() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture(
        "class.root",
        "@Component({ selector: 'app-user' })\nexport class UserComponent extends BaseComponent {}",
    )];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let extends_edges = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert_eq!(extends_edges.len(), 1);
    assert_eq!(extends_edges[0].object.name, "BaseComponent");
}

// ── C# `:` syntax — DEFERRED ────────────────────────────────────────────

#[test]
fn builtin_defers_extends_for_csharp_base_only() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture(
        "class.root",
        "public class ApplicationDbContext : DbContext {}",
    )];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let extends_edges = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert!(
        extends_edges.is_empty(),
        "C# `:` syntax must NOT emit Extends"
    );
}

#[test]
fn builtin_defers_extends_for_csharp_interface_only() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture("class.root", "public class Foo : IFoo {}")];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let extends_edges = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert!(
        extends_edges.is_empty(),
        "C# `:` syntax must NOT emit Extends for interface-only"
    );
}

#[test]
fn builtin_defers_implements_for_java_implements() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture(
        "class.root",
        "public class OrderRepository implements IRepository {}",
    )];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let implements_edges = find_edges_by_relation(&edges, SemanticRelation::Implements);
    assert!(
        implements_edges.is_empty(),
        "`implements` must NOT emit (can't distinguish Java from TS)"
    );
}

#[test]
fn builtin_no_extends_without_keyword() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture("class.root", "public class PlainClass {}")];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let extends_edges = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert!(extends_edges.is_empty());
}

#[test]
fn builtin_self_defines_preserved_with_extends() {
    let layer = BuiltinMetaLayer::new();
    let captures = [class_capture(
        "class.root",
        "export class UserComponent extends BaseComponent {}",
    )];
    let edges = layer.extract_semantic_edges_paired("", &captures, Fidelity::High, None);

    let defines_edges = find_edges_by_relation(&edges, SemanticRelation::Defines);
    assert_eq!(defines_edges.len(), 1);
    assert_eq!(defines_edges[0].subject.name, "UserComponent");
    assert_eq!(defines_edges[0].object.name, "UserComponent");
}
