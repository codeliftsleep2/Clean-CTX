use super::*;
use crate::diff::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};
use crate::diff::differ::diff_snapshots;

fn make_class(name: &str, methods: &[&str], fields: &[&str]) -> CapturedClass {
    CapturedClass {
        name: name.to_string(),
        class_meta: String::new(),
        fields: fields.iter().map(|s| s.to_string()).collect(),
        methods: methods
            .iter()
            .map(|s| CapturedMethod {
                sig: s.to_string(),
                markers: vec![],
                body: None,
            })
            .collect(),
    }
}

/// Build a class with methods that carry explicit body fingerprints.
fn make_class_with_bodies(
    name: &str,
    methods: &[(&str, Option<&str>)],
) -> CapturedClass {
    CapturedClass {
        name: name.to_string(),
        class_meta: String::new(),
        fields: vec![],
        methods: methods
            .iter()
            .map(|(sig, body)| CapturedMethod {
                sig: sig.to_string(),
                markers: vec![],
                body: body.map(|b| b.to_string()),
            })
            .collect(),
    }
}

#[test]
fn format_diff_renders_markers() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["foo()"], &[])],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![
            make_class("Foo", &["foo()"], &[]),
            make_class("Bar", &["bar()"], &[]),
        ],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let rendered = format_diff(&actions, Fidelity::Low);
    assert!(rendered.contains("+ class Bar"));
    assert!(rendered.contains("= class Foo"));
}

/// Regression: a body-only change (same signature, different body) must
/// render a visible change marker, not an unchanged marker.
#[test]
fn format_diff_renders_body_only_change_marker() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            &[("process(id):void", Some("return id + 1;"))],
        )],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            &[("process(id):void", Some("return id + 2;"))],
        )],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let rendered = format_diff(&actions, Fidelity::Low);
    assert!(
        rendered.contains("~ method process (body changed)"),
        "body-only change should render a visible marker, got:\n{}",
        rendered
    );
    // The class must NOT be reported unchanged.
    assert!(
        !rendered.contains("= class Foo"),
        "class with a body-changed method must not be reported unchanged, got:\n{}",
        rendered
    );
}