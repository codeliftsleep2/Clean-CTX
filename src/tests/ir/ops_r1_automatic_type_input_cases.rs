//! Effective empty, reset, inheritance and malformed-input acquisition controls.
use super::support::{fixture, resolver, write};
use crate::ir::typescript_resolution::shared_acquisition::operation;
use serde_json::{Value, json};

fn configured(
    types: Option<Value>,
    base_types: Option<Value>,
    installed: bool,
) -> tempfile::TempDir {
    let root = fixture();
    let path = root.path();
    std::fs::remove_file(path.join("a.ts")).unwrap();
    std::fs::remove_file(path.join("b.ts")).unwrap();
    write(path, "one/a.ts", "class A { peer?: Z; }");
    write(path, "two/b.ts", "class Z {}");
    write(
        path,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    let mut config = json!({"compilerOptions":{"module":"esnext","moduleResolution":"bundler",
        "moduleDetection":"legacy","noLib":true}, "include":["**/*.ts"]});
    if let Some(value) = types {
        config["compilerOptions"]["types"] = value;
    }
    if let Some(value) = base_types {
        write(
            path,
            "base.json",
            serde_json::to_vec(&json!({"compilerOptions":{"types":value}})).unwrap(),
        );
        config["extends"] = json!("./base.json");
    }
    write(path, "tsconfig.json", serde_json::to_vec(&config).unwrap());
    if installed {
        write(
            path,
            "node_modules/@types/local/index.d.ts",
            "declare class AmbientLocal {}",
        );
    }
    root
}

#[test]
fn ops_r1_types_empty_absent_null_and_inherited_empty_share_actual_program() {
    let cases = [
        ("absent, no packages", None, None, false),
        ("explicit empty", Some(json!([])), None, false),
        (
            "explicit empty ignores visible package",
            Some(json!([])),
            None,
            true,
        ),
        ("null reset, no packages", Some(Value::Null), None, false),
        ("inherited empty", None, Some(json!([])), false),
        (
            "inherited empty ignores visible package",
            None,
            Some(json!([])),
            true,
        ),
        (
            "child empty overrides nonempty base",
            Some(json!([])),
            Some(json!(["missing"])),
            true,
        ),
        (
            "child null clears empty base, no packages",
            Some(Value::Null),
            Some(json!([])),
            false,
        ),
        (
            "inherited null reset, no packages",
            None,
            Some(Value::Null),
            false,
        ),
    ];
    for (label, types, base_types, installed) in cases {
        let root = configured(types, base_types, installed);
        let path = root.path();
        let op = operation();
        let mut first = resolver(path, "one/a.ts", &op);
        let mut second = resolver(path, "two/b.ts", &op);
        assert_eq!(
            first.program_units(&path.join("one/a.ts")).unwrap().len(),
            2,
            "{label}"
        );
        assert_eq!(
            second.program_units(&path.join("two/b.ts")).unwrap().len(),
            2,
            "{label}"
        );
        assert_eq!(
            op.counts(),
            [1, 1, 2],
            "{label}: empty effective input must share acquisition"
        );
        assert!(first.host.current() && second.host.current(), "{label}");
    }
}

#[test]
fn ops_r1_types_null_and_absent_preserve_nonempty_default_discovery() {
    let cases = [
        ("absent discovers visible package", None, None),
        (
            "null reset discovers visible package",
            Some(Value::Null),
            None,
        ),
        (
            "child null resets inherited empty",
            Some(Value::Null),
            Some(json!([])),
        ),
        (
            "child null resets inherited nonempty",
            Some(Value::Null),
            Some(json!(["missing"])),
        ),
        ("inherited null uses default", None, Some(Value::Null)),
        ("explicit named package", Some(json!(["local"])), None),
    ];
    for (label, types, base_types) in cases {
        let root = configured(types, base_types, true);
        let path = root.path();
        let op = operation();
        let mut first = resolver(path, "one/a.ts", &op);
        let mut second = resolver(path, "two/b.ts", &op);
        assert_eq!(
            first.program_units(&path.join("one/a.ts")).unwrap().len(),
            3,
            "{label}"
        );
        assert_eq!(
            second.program_units(&path.join("two/b.ts")).unwrap().len(),
            3,
            "{label}"
        );
        assert_eq!(
            op.counts(),
            [2, 2, 6],
            "{label}: nonempty defaults must not be called empty"
        );
        assert!(first.host.current() && second.host.current(), "{label}");
    }
}

#[test]
fn ops_r1_types_blank_malformed_and_empty_entries_never_grant_shared_proof() {
    let cases = [
        json!(""),
        json!("   "),
        json!("[]"),
        json!({}),
        json!(false),
        json!(0),
        json!([null]),
        json!([""]),
        json!(["   "]),
        json!([[]]),
        json!([{}]),
    ];
    for types in cases {
        let label = types.to_string();
        let root = configured(Some(types), None, false);
        let path = root.path();
        let op = operation();
        let mut first = resolver(path, "one/a.ts", &op);
        let mut second = resolver(path, "two/b.ts", &op);
        assert!(
            first.program_units(&path.join("one/a.ts")).is_err(),
            "{label}"
        );
        assert!(
            second.program_units(&path.join("two/b.ts")).is_err(),
            "{label}"
        );
        assert_eq!(
            op.counts()[1],
            0,
            "{label}: no successful acquisition proof"
        );
        assert_eq!(
            op.work_counts()[1],
            0,
            "{label}: no successful shared qualification"
        );
    }
}
