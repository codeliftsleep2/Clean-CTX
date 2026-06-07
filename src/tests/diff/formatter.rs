use super::*;
use crate::diff::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};
use crate::diff::differ::diff_snapshots;

fn make_class(name: &str, methods: &[&str], fields: &[&str]) -> CapturedClass {
    CapturedClass {
        name: name.to_string(),
        fields: fields.iter().map(|s| s.to_string()).collect(),
        methods: methods
            .iter()
            .map(|s| CapturedMethod {
                sig: s.to_string(),
                markers: vec![],
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
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![
            make_class("Foo", &["foo()"], &[]),
            make_class("Bar", &["bar()"], &[]),
        ],
        orphan_fields: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let rendered = format_diff(&actions, Fidelity::Low);
    assert!(rendered.contains("+ class Bar"));
    assert!(rendered.contains("= class Foo"));
}