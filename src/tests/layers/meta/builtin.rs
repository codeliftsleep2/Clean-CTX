// src/tests/layers/meta/builtin.rs
//
// Tests for BuiltinMetaLayer semantic edge emission (Phase 2).
// Verifies that generic `Extends` relations are emitted from the
// `extends` keyword, and that C# `:` syntax is correctly deferred.

use crate::compression::Fidelity;
use crate::layers::meta::MetaLayer;
use crate::layers::meta::builtin::BuiltinMetaLayer;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

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
// ── Phase 30-A: Java `implements` semantic projection ─────────────────────

/// Build a Java-attributed source containing `decl`. Java language
/// attribution comes from the `package` declaration (see `is_java_source`).
fn java_source(decl: &str) -> String {
    format!("package com.example;\n\n{}\n", decl)
}

#[test]
fn builtin_emits_implements_for_java_class() {
    let layer = BuiltinMetaLayer::new();
    let cap = "public class Foo implements Bar {}";
    let source = java_source(cap);
    let captures = [class_capture("class.root", cap)];
    let edges = layer.extract_semantic_edges_paired(&source, &captures, Fidelity::High, None);

    let ifaces = find_edges_by_relation(&edges, SemanticRelation::Implements);
    assert_eq!(ifaces.len(), 1);
    assert_eq!(ifaces[0].subject, EntityRef::new("builtin", "Class", "Foo"));
    assert_eq!(
        ifaces[0].object,
        EntityRef::new("builtin", "Interface", "Bar")
    );
    assert_eq!(ifaces[0].layer, "builtin");
}

#[test]
fn builtin_emits_implements_for_multiple_interfaces() {
    let layer = BuiltinMetaLayer::new();
    let cap = "public class Foo implements A, B, C {}";
    let source = java_source(cap);
    let captures = [class_capture("class.root", cap)];
    let edges = layer.extract_semantic_edges_paired(&source, &captures, Fidelity::High, None);

    let ifaces = find_edges_by_relation(&edges, SemanticRelation::Implements);
    assert_eq!(ifaces.len(), 3);
    assert!(
        ifaces
            .iter()
            .all(|e| e.subject == EntityRef::new("builtin", "Class", "Foo"))
    );
    let names: Vec<&str> = ifaces.iter().map(|e| e.object.name.as_str()).collect();
    assert!(names.contains(&"A") && names.contains(&"B") && names.contains(&"C"));
}

#[test]
fn builtin_java_extends_and_implements_coexist() {
    let layer = BuiltinMetaLayer::new();
    let cap = "public class Foo extends Base implements A, B {}";
    let source = java_source(cap);
    let captures = [class_capture("class.root", cap)];
    let edges = layer.extract_semantic_edges_paired(&source, &captures, Fidelity::High, None);

    let ext = find_edges_by_relation(&edges, SemanticRelation::Extends);
    assert_eq!(ext.len(), 1, "exactly one Extends edge");
    assert_eq!(ext[0].subject, EntityRef::new("builtin", "Class", "Foo"));
    assert_eq!(ext[0].object, EntityRef::new("builtin", "Class", "Base"));

    let ifaces = find_edges_by_relation(&edges, SemanticRelation::Implements);
    assert_eq!(ifaces.len(), 2, "implements A and B are both projected");
}

#[test]
fn builtin_implements_never_creates_binds() {
    let layer = BuiltinMetaLayer::new();
    // Spring-annotated Java class: the builtin layer projects the language
    // fact, but must NEVER derive a DI registration from it.
    let cap = "@Repository\npublic class Foo implements Bar {}";
    let source = java_source(cap);
    let captures = [class_capture("class.root", cap)];
    let edges = layer.extract_semantic_edges_paired(&source, &captures, Fidelity::High, None);

    assert_eq!(
        find_edges_by_relation(&edges, SemanticRelation::Implements).len(),
        1
    );
    assert!(
        find_edges_by_relation(&edges, SemanticRelation::Binds).is_empty(),
        "Implements must never fabricate a Binds edge"
    );
}

#[test]
fn builtin_implements_generic_uses_bare_interface_name() {
    let layer = BuiltinMetaLayer::new();
    // The authoritative shared extractor strips generic arguments at the
    // first `<` — this phase preserves that exact behavior (no new erasure
    // rules, no normalization). The consumer-side `Token/Repository<User>`
    // mismatch remains a separate fail-closed boundary.
    let cap = "public class RepositoryImpl implements Repository<User> {}";
    let source = java_source(cap);
    let captures = [class_capture("class.root", cap)];
    let edges = layer.extract_semantic_edges_paired(&source, &captures, Fidelity::High, None);

    let ifaces = find_edges_by_relation(&edges, SemanticRelation::Implements);
    assert_eq!(ifaces.len(), 1);
    assert_eq!(
        ifaces[0].object,
        EntityRef::new("builtin", "Interface", "Repository")
    );
    assert_ne!(
        ifaces[0].object,
        EntityRef::new("builtin", "Interface", "Repository<User>")
    );
}

#[test]
fn builtin_implements_qualified_interface_preserved() {
    let layer = BuiltinMetaLayer::new();
    let cap = "public class Foo implements com.example.Bar {}";
    let source = java_source(cap);
    let captures = [class_capture("class.root", cap)];
    let edges = layer.extract_semantic_edges_paired(&source, &captures, Fidelity::High, None);

    let ifaces = find_edges_by_relation(&edges, SemanticRelation::Implements);
    assert_eq!(ifaces.len(), 1);
    assert_eq!(
        ifaces[0].object,
        EntityRef::new("builtin", "Interface", "com.example.Bar")
    );
}

#[test]
fn builtin_defers_implements_for_non_java_source() {
    let layer = BuiltinMetaLayer::new();
    // TypeScript-styled source: no `package` and no Java `import a.b.C;`
    // marker → NOT attributed as Java → the `implements` shape is deferred
    // (the language-agnostic deferral boundary, now pinned concretely).
    let ts_source = "import { Injectable } from '@angular/core';\n\n@Injectable()\nexport class Foo implements Bar {}\n";
    let captures = [class_capture(
        "class.root",
        "export class Foo implements Bar {}",
    )];
    let edges = layer.extract_semantic_edges_paired(ts_source, &captures, Fidelity::High, None);

    assert!(
        find_edges_by_relation(&edges, SemanticRelation::Implements).is_empty(),
        "non-Java-attributed sources must not emit Implements"
    );
}
