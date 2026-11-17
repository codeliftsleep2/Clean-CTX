//! Agent MaxHeadRoom: spelling-only portability for frozen subject.ts identities.
use serde_json::Value;

pub(crate) fn first_difference(left: &Value, right: &Value, path: String) -> String {
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            for key in a.keys().chain(b.keys()) {
                if a.get(key) != b.get(key) {
                    return first_difference(&left[key], &right[key], format!("{path}.{key}"));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                if a != b {
                    return first_difference(a, b, format!("{path}[{index}]"));
                }
            }
        }
        _ => {}
    }
    path
}

pub(crate) fn normalize(value: &mut Value, root: &str) {
    match value {
        Value::String(text) => {
            *text = text
                .replace(&root.replace('\\', "\\\\"), "<ROOT>")
                .replace(root, "<ROOT>");
            // Debug-escaped Windows extended-path prefix, already root-bound.
            *text = text.replace(r"\\\\?\\<ROOT>", "<ROOT>");
            for name in ["subject.ts", "tsconfig.json", "jsconfig.json"] {
                *text = text
                    .replace(&format!(r"<ROOT>\\{name}"), &format!("<ROOT>/{name}"))
                    .replace(&format!(r"<ROOT>\{name}"), &format!("<ROOT>/{name}"));
            }
            // Two Debug representations of the fixture's one exact timestamp.
            // Other timestamps remain observable.
            *text = text
                .replace(
                    "SystemTime { intervals: 134117966450000000 }",
                    "<2026-01-02T03:04:05Z>",
                )
                .replace(
                    "SystemTime { tv_sec: 1767323045, tv_nsec: 0 }",
                    "<2026-01-02T03:04:05Z>",
                );
        }
        Value::Array(items) => items.iter_mut().for_each(|item| normalize(item, root)),
        Value::Object(items) => items.values_mut().for_each(|item| normalize(item, root)),
        _ => {}
    }
}

#[test]
fn af002_frozen_identity_portability() {
    use serde_json::json;
    let mut windows = json!([
        r"<ROOT>\subject.ts",
        r#"target: "\\\\?\\<ROOT>\\subject.ts""#,
        "SystemTime { intervals: 134117966450000000 }",
    ]);
    let mut unix = json!([
        "<ROOT>/subject.ts",
        r#"target: "<ROOT>/subject.ts""#,
        "SystemTime { tv_sec: 1767323045, tv_nsec: 0 }",
    ]);
    normalize(&mut windows, "/temporary-root");
    normalize(&mut unix, "/temporary-root");
    assert_eq!(windows, unix);
    let mut unrelated = json!([
        r"elsewhere\subject.ts",
        r"<ROOT>\different.ts",
        "SystemTime { tv_sec: 1767323046, tv_nsec: 0 }",
        1767323045,
        [1, 2, 3],
    ]);
    let original = unrelated.clone();
    normalize(&mut unrelated, "/temporary-root");
    assert_eq!(unrelated, original);
    assert_eq!(
        first_difference(&json!({"a": [1, 2]}), &json!({"a": [1, 3]}), "$".into()),
        "$.a[1]"
    );
}
