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
