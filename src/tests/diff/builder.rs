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