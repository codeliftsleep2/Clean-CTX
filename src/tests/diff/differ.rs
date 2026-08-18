use super::*;
use crate::diff::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

fn make_class(name: &str, methods: &[&str], fields: &[&str]) -> CapturedClass {
    CapturedClass {
        name: name.to_string(),
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
fn detects_added_class() {
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
    let has_added_bar = actions.iter().any(|a| {
        a.kind == DiffKind::Added && a.target == DiffTarget::Class && a.label == "class Bar"
    });
    assert!(has_added_bar, "expected `+ class Bar` action, got {:?}", actions);
}

#[test]
fn detects_removed_class() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![
            make_class("Foo", &["foo()"], &[]),
            make_class("Bar", &["bar()"], &[]),
        ],
        orphan_fields: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["foo()"], &[])],
        orphan_fields: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let has_removed = actions.iter().any(|a| {
        a.kind == DiffKind::Removed && a.target == DiffTarget::Class && a.label == "class Bar"
    });
    assert!(has_removed, "expected `- class Bar` action");
}

#[test]
fn detects_modified_method() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["process(id:string):boolean"], &[])],
        orphan_fields: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["process(id:number):boolean"], &[])],
        orphan_fields: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let modified = actions
        .iter()
        .find(|a| a.kind == DiffKind::Modified && a.target == DiffTarget::Method);
    assert!(modified.is_some(), "expected a `~` method action");
    let m = modified.unwrap();
    assert!(m.detail.contains("number"));
    assert!(m.previous_detail.contains("string"));
}

/// Regression: body-only changes (logic fixes with unchanged signatures)
/// must be reported as Modified, not Unchanged.
#[test]
fn body_only_change_is_detected() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            &[("process(id):void", Some("return id + 1;"))],
        )],
        orphan_fields: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            // Same signature — logic fix only.
            &[("process(id):void", Some("return id + 2;"))],
        )],
        orphan_fields: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let modified = actions
        .iter()
        .find(|a| a.kind == DiffKind::Modified && a.target == DiffTarget::Method);
    assert!(
        modified.is_some(),
        "body-only change must produce a Modified action, got {:?}",
        actions
    );
    // The class itself must be Modified too — not `= class Foo (unchanged)`.
    let class_modified = actions
        .iter()
        .any(|a| a.kind == DiffKind::Modified && a.target == DiffTarget::Class);
    assert!(
        class_modified,
        "class must be marked Modified when a method body changes, got {:?}",
        actions
    );
}

/// Identical signatures AND bodies → Unchanged (no false positive).
#[test]
fn identical_method_with_same_body_is_unchanged() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            &[("process(id):void", Some("return id + 1;"))],
        )],
        orphan_fields: vec![],
    };
    let current = baseline.clone();
    let actions = diff_snapshots(&baseline, &current);
    let unchanged = actions
        .iter()
        .any(|a| a.kind == DiffKind::Unchanged && a.target == DiffTarget::Class);
    assert!(unchanged, "identical snapshots should emit Unchanged");
}

/// When both bodies are None (abstract methods / test fixtures), equality
/// still holds — `None == None` so this pair is Unchanged.
#[test]
fn abstract_methods_without_bodies_unchanged() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            &[("abstract doWork(): void;", None)],
        )],
        orphan_fields: vec![],
    };
    let current = baseline.clone();
    let actions = diff_snapshots(&baseline, &current);
    let unchanged = actions
        .iter()
        .any(|a| a.kind == DiffKind::Unchanged && a.target == DiffTarget::Class);
    assert!(unchanged, "abstract methods with None bodies are unchanged");
}

#[test]
fn detects_added_removed_imports() {
    let baseline = CapturedStructure {
        imports: vec!["OldService".to_string()],
        classes: vec![],
        orphan_fields: vec![],
    };
    let current = CapturedStructure {
        imports: vec!["NewService".to_string()],
        classes: vec![],
        orphan_fields: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let added = actions.iter().any(|a| {
        a.kind == DiffKind::Added
            && a.target == DiffTarget::Import
            && a.detail == "NewService"
    });
    let removed = actions.iter().any(|a| {
        a.kind == DiffKind::Removed
            && a.target == DiffTarget::Import
            && a.detail == "OldService"
    });
    assert!(added && removed);
}

#[test]
fn unchanged_class_emit_equals_marker() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["foo()"], &[])],
        orphan_fields: vec![],
    };
    let current = baseline.clone();
    let actions = diff_snapshots(&baseline, &current);
    let unchanged = actions
        .iter()
        .any(|a| a.kind == DiffKind::Unchanged && a.target == DiffTarget::Class);
    assert!(unchanged, "expected an `=` class action for unchanged snapshot");
}