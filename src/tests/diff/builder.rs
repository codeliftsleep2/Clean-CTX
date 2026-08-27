use super::*;

#[test]
fn build_snapshot_parses_a_simple_class() {
    let src = r#"
        export class Foo {
            public greet(name: string): string { return "hi " + name; }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot");
    assert!(!snap.classes.is_empty(), "expected at least one class");
    let foo = &snap.classes[0];
    assert_eq!(foo.name, "Foo");
}

#[test]
fn build_snapshot_handles_empty_source() {
    let snap = build_snapshot("", Fidelity::Low).expect("build_snapshot");
    assert!(snap.classes.is_empty());
    assert!(snap.imports.is_empty());
}

// ── Base-initializer sensitivity (body fingerprint) ──────────────────

/// The fingerprint covers everything after the method's OWN parameter
/// list. A base-initializer clause is behavior — it runs before the body —
/// so an initializer-only edit must change the fingerprint even when the
/// body text is identical. The LAST-group locator keyed the fingerprint
/// off the initializer's own parens, hiding initializer-only edits.
#[test]
fn method_body_fingerprint_is_initializer_sensitive() {
    let body = "{\n    Initialize(prefix);\n}";
    let a = format!("public Greeter(string prefix) : base(prefix)\n{body}");
    let b = format!("public Greeter(string prefix) : base(prefix + 1)\n{body}");
    let c = "public Greeter(string prefix) : base(prefix)\n{\n    Initialize(prefix);\n    Validate();\n}";

    let fa = extract_method_body(&a).expect("fingerprint");
    let fb = extract_method_body(&b).expect("fingerprint");
    let fc = extract_method_body(c).expect("fingerprint");

    assert_ne!(fa, fb, "initializer-only edit must change the fingerprint");
    assert_ne!(fa, fc, "body-only edit must change the fingerprint");
}

#[test]
fn build_snapshot_falls_back_to_other_language() {
    let src = r#"
        namespace MyApp {
            public class Greeter {
                public string Greet(string name) { return "hi " + name; }
            }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot");
    assert!(!snap.classes.is_empty() || !snap.imports.is_empty());
}

// ── Phase D: Rust diff regression tests ──────────────────────────

#[test]
fn build_snapshot_parses_rust_struct() {
    let src = r#"
        pub struct UserService {
            users: Vec<String>,
            cache: HashMap<u64, String>,
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust struct");
    assert!(!snap.classes.is_empty(), "should detect UserService class");
    assert_eq!(snap.classes[0].name, "UserService");
}

#[test]
fn build_snapshot_parses_rust_enum() {
    let src = r#"
        pub enum Status {
            Active,
            Inactive,
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust enum");
    assert!(!snap.classes.is_empty(), "should detect Status enum");
    assert_eq!(snap.classes[0].name, "Status");
}

#[test]
fn build_snapshot_parses_rust_trait() {
    let src = r#"
        pub trait Repository {
            fn find(&self, id: u64) -> bool;
            fn save(&self, item: String);
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust trait");
    assert!(!snap.classes.is_empty(), "should detect Repository trait");
    assert_eq!(snap.classes[0].name, "Repository");
}

#[test]
fn build_snapshot_parses_rust_impl_block() {
    let src = r#"
        impl UserService {
            fn new() -> Self { UserService { users: Vec::new() } }
            pub fn get_user(&self, id: u64) -> Option<String> { None }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust impl");
    assert!(!snap.classes.is_empty(), "should detect impl class");
}

#[test]
fn build_snapshot_parses_rust_trait_impl() {
    let src = r#"
        impl Repository for UserService {
            fn find(&self, id: u64) -> bool { false }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust trait impl");
    assert!(!snap.classes.is_empty(), "should detect trait impl class");
}

#[test]
fn build_snapshot_parses_rust_full_file() {
    // Verify that a complete Rust file with struct, impl, and use
    // declarations is correctly parsed.
    let src = r#"
        use std::collections::HashMap;

        pub struct UserService {
            users: Vec<String>,
        }

        impl UserService {
            pub fn new() -> Self {
                UserService { users: Vec::new() }
            }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust file");
    assert!(!snap.classes.is_empty(), "should detect Rust classes");
    let names: Vec<&str> = snap.classes.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"UserService"),
        "should contain UserService, got: {:?}",
        names
    );
}

// ── Non-CBM Tool Audit 2026-08-25, finding #1 ────────────────────────
//
// The diff snapshot builder routes `struct.root`/`enum.root` through
// `extract_rust_struct_name` for ALL languages. That helper only strips
// Rust visibility prefixes (`pub `, `pub(crate) `, `pub(super) `), so a
// C# declaration like `public enum PriorityLevel` produced the label
// `public`, which `diff_commits` then rendered as `~ class public`.
// Non-Rust class-like declarations must go through the shared
// `extract_class_name`; Rust behavior must stay exactly as-is.

#[test]
fn build_snapshot_csharp_internal_static_class_gets_identifier() {
    let src = r#"
        namespace MyApp.Tests.Support
        {
            internal static class TestDataFactory
            {
                internal static void Warmup() { }
            }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for C# internal class");
    assert!(!snap.classes.is_empty(), "should detect TestDataFactory");
    assert_eq!(
        snap.classes[0].name, "TestDataFactory",
        "access modifier must never become the class label"
    );
}

#[test]
fn build_snapshot_csharp_enum_gets_identifier_not_visibility_token() {
    let src = r#"
        namespace MyApp.Core
        {
            public enum PriorityLevel
            {
                Low,
                High
            }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for C# enum");
    assert!(!snap.classes.is_empty(), "should detect PriorityLevel enum");
    assert_eq!(
        snap.classes[0].name, "PriorityLevel",
        "enum label must be the identifier, not a visibility modifier"
    );
}

#[test]
fn build_snapshot_rust_enum_behavior_unchanged() {
    // Guards the language split: Rust enums keep their existing path.
    let src = "pub enum Status {\n    Active,\n}\n";
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot for Rust enum");
    assert_eq!(snap.classes[0].name, "Status");
}

// ── Nested-type ownership (diff_commits mislabeling) ──────────────────
//
// Every language query captures class/struct/enum/record declarations at
// ANY nesting depth (the queries are unanchored), so a declaration made
// INSIDE another declaration's body opens its own `CapturedClass` entry.
// Because method attachment used `classes.last_mut()`, every enclosing-
// declaration method captured AFTER a nested type was attributed to that
// type — `diff_commits` rendered `~ class OrderStatus` for methods that
// plainly belong to `OrderService`.
//
// Contract: a changed-member group must be owned by the declaration whose
// source span CONTAINS the member. A nested declaration that closes before
// the member starts must never own it. Nested types keep their own entries
// with correct labels (finding #1 guarantees the identifier, this fixes
// the ownership).

#[test]
fn build_snapshot_nested_enum_does_not_steal_enclosing_class_ownership() {
    let src = r#"
        namespace MyApp.Services
        {
            public class OrderService
            {
                public enum OrderStatus
                {
                    Pending,
                    Shipped
                }

                public async Task ShipAsync(int id)
                {
                    await Task.CompletedTask;
                }

                public void Cancel(int id)
                {
                }
            }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot");
    let names: Vec<&str> = snap.classes.iter().map(|c| c.name.as_str()).collect();
    let service: Vec<_> = snap
        .classes
        .iter()
        .filter(|c| c.name == "OrderService")
        .collect();
    assert_eq!(
        service.len(),
        1,
        "exactly one OrderService entry expected, got: {names:?}"
    );
    let sigs: Vec<&str> = service[0].methods.iter().map(|m| m.sig.as_str()).collect();
    assert!(
        sigs.iter().any(|s| s.contains("ShipAsync")),
        "ShipAsync must be owned by OrderService, got: {sigs:?}"
    );
    assert!(
        sigs.iter().any(|s| s.contains("Cancel")),
        "Cancel (declared after the nested enum closed) must be owned by \
         OrderService, got: {sigs:?}"
    );
    let stolen: Vec<&str> = snap
        .classes
        .iter()
        .filter(|c| c.name == "OrderStatus")
        .flat_map(|c| c.methods.iter().map(|m| m.sig.as_str()))
        .collect();
    assert!(
        stolen.is_empty(),
        "nested enum must never own the enclosing class's methods, got: {stolen:?}"
    );
}

#[test]
fn build_snapshot_nested_class_ownership_follows_span_containment() {
    // Same mechanism with a nested CLASS instead of an enum: the inner
    // method belongs to Inner, the outer method (declared after Inner's
    // body closes) belongs to Outer.
    let src = r#"
        namespace MyApp
        {
            public class Outer
            {
                public class Inner
                {
                    public void InnerMethod() { }
                }

                public void OuterMethod() { }
            }
        }
    "#;
    let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot");
    let owner_of = |needle: &str| -> Option<&str> {
        for c in &snap.classes {
            if c.methods.iter().any(|m| m.sig.contains(needle)) {
                return Some(&c.name);
            }
        }
        None
    };
    assert_eq!(
        owner_of("InnerMethod"),
        Some("Inner"),
        "inner method must stay with Inner, got: {:?}",
        snap.classes
            .iter()
            .map(|c| (&c.name, &c.methods))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        owner_of("OuterMethod"),
        Some("Outer"),
        "outer method declared after Inner closed must belong to Outer, got: {:?}",
        snap.classes
            .iter()
            .map(|c| (&c.name, &c.methods))
            .collect::<Vec<_>>()
    );
}
