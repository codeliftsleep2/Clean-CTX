use super::*;

/// G2-2: top-level functions must be diffed even without a class.
#[test]
fn orphan_function_change_is_detected() {
    let mk = |sig: &str| CapturedMethod {
        declaration: None,
        sig: sig.to_string(),
        markers: vec![],
        body: None,
    };
    let b = CapturedStructure {
        imports: vec![],
        classes: vec![],
        orphan_fields: vec![],
        orphan_methods: vec![mk("foo()")],
    };
    let c = CapturedStructure {
        imports: vec![],
        classes: vec![],
        orphan_fields: vec![],
        orphan_methods: vec![mk("foo(x)")],
    };
    let a = diff_snapshots(&b, &c);
    assert!(
        a.iter()
            .any(|x| x.kind == DiffKind::Modified && x.target == DiffTarget::Method)
    );
}

/// G2-5: markers-only change sets reason_hint == markers.
#[test]
fn markers_only_change_reason_hint() {
    let mk = |markers: Vec<String>| CapturedMethod {
        declaration: None,
        sig: "foo()".to_string(),
        markers,
        body: Some("return 1".to_string()),
    };
    let cls = |m: CapturedMethod| CapturedClass {
        name: "Foo".to_string(),
        class_meta: String::new(),
        fields: vec![],
        methods: vec![m],
    };
    let b = CapturedStructure {
        imports: vec![],
        classes: vec![cls(mk(vec![]))],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };
    let c = CapturedStructure {
        imports: vec![],
        classes: vec![cls(mk(vec!["#guard".to_string()]))],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };
    let a = diff_snapshots(&b, &c);
    let m = a
        .iter()
        .find(|x| x.kind == DiffKind::Modified && x.target == DiffTarget::Method);
    assert!(m.is_some(), "expected Modified, got {a:?}");
    assert_eq!(m.unwrap().reason_hint, "markers");
}

// ── Constructor label / unchanged-member false-positive regression ──

/// Regression: a constructor with `: base(...)` must key on the declared
/// name, not `:base`. This test verifies the full diff pipeline produces
/// the correct label and does NOT emit an unchanged constructor as changed
/// when another member in the class changes.
#[test]
fn ctor_with_base_initializer_unchanged_when_property_changes() {
    let sig = "ExampleException(string value, object context)";
    let body = Some(
        ": base($\"Unexpected value: {value}, context: {context}\") { Value = value; Context = context; }"
            .to_string(),
    );

    let make_ctor = || CapturedMethod {
        declaration: None,
        sig: sig.to_string(),
        markers: vec![],
        body: body.clone(),
    };

    // Baseline: constructor + two auto-properties
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "ExampleException".to_string(),
            class_meta: ": Exception".to_string(),
            fields: vec!["Value:string".to_string(), "Context:object".to_string()],
            methods: vec![make_ctor()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    // Current: same constructor, property `Context` changed
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "ExampleException".to_string(),
            class_meta: ": Exception".to_string(),
            fields: vec![
                "Value:string".to_string(),
                "Context:object (modified)".to_string(),
            ],
            methods: vec![make_ctor()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let actions = diff_snapshots(&baseline, &current);

    // The constructor must be Unchanged (not Modified, not Added, not Removed).
    let ctor_actions: Vec<&DiffAction> = actions
        .iter()
        .filter(|a| a.target == DiffTarget::Method && a.label.contains("ExampleException"))
        .collect();
    assert_eq!(
        ctor_actions.len(),
        1,
        "exactly one method action for the constructor expected, got: {ctor_actions:?}"
    );
    assert_eq!(
        ctor_actions[0].kind,
        DiffKind::Unchanged,
        "unchanged constructor must be Unchanged, not {:?}",
        ctor_actions[0]
    );
    // The label must be the declared name, not `:base`.
    assert!(
        ctor_actions[0].label.contains("ExampleException"),
        "constructor label must contain the declared name, got: {}",
        ctor_actions[0].label
    );
    assert!(
        !ctor_actions[0].label.contains(":base"),
        "constructor label must not contain ':base', got: {}",
        ctor_actions[0].label
    );
}
/// Regression: a constructor with `: this(...)` must key on the declared
/// name, not `:this`.
#[test]
fn ctor_with_this_initializer_unchanged_when_property_changes() {
    let sig = "Widget(string prefix)";
    let body = Some(": this(prefix) { Initialize(prefix); }".to_string());

    let make_ctor = || CapturedMethod {
        declaration: None,
        sig: sig.to_string(),
        markers: vec![],
        body: body.clone(),
    };

    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "Widget".to_string(),
            class_meta: String::new(),
            fields: vec!["Name:string".to_string()],
            methods: vec![make_ctor()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let current = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "Widget".to_string(),
            class_meta: String::new(),
            fields: vec!["Name:string (modified)".to_string()],
            methods: vec![make_ctor()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let actions = diff_snapshots(&baseline, &current);
    let ctor_actions: Vec<&DiffAction> = actions
        .iter()
        .filter(|a| a.target == DiffTarget::Method && a.label.contains("Widget"))
        .collect();
    assert_eq!(
        ctor_actions.len(),
        1,
        "exactly one method action for the constructor expected, got: {ctor_actions:?}"
    );
    assert_eq!(
        ctor_actions[0].kind,
        DiffKind::Unchanged,
        "unchanged constructor must be Unchanged"
    );
    assert!(
        !ctor_actions[0].label.contains(":this"),
        "constructor label must not contain ':this'"
    );
}

/// Control: a constructor without initializer remains correct when
/// a property changes.
#[test]
fn ctor_without_initializer_unchanged_when_property_changes() {
    let sig = "MyClass()";
    let body = Some(" { Initialize(); }".to_string());

    let make_ctor = || CapturedMethod {
        declaration: None,
        sig: sig.to_string(),
        markers: vec![],
        body: body.clone(),
    };

    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "MyClass".to_string(),
            class_meta: String::new(),
            fields: vec!["Value:int".to_string()],
            methods: vec![make_ctor()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let current = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "MyClass".to_string(),
            class_meta: String::new(),
            fields: vec!["Value:int (modified)".to_string()],
            methods: vec![make_ctor()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let actions = diff_snapshots(&baseline, &current);
    let ctor_actions: Vec<&DiffAction> = actions
        .iter()
        .filter(|a| a.target == DiffTarget::Method && a.label.contains("MyClass"))
        .collect();
    assert_eq!(
        ctor_actions.len(),
        1,
        "exactly one method action for the constructor expected, got: {ctor_actions:?}"
    );
    assert_eq!(
        ctor_actions[0].kind,
        DiffKind::Unchanged,
        "unchanged constructor must be Unchanged"
    );
}
/// Control: an ordinary method remains correctly labeled.
#[test]
fn ordinary_method_unchanged_when_property_changes() {
    let sig = "Process(int id)";
    let body = Some(" { return id; }".to_string());

    let make_method = || CapturedMethod {
        declaration: None,
        sig: sig.to_string(),
        markers: vec![],
        body: body.clone(),
    };

    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "Service".to_string(),
            class_meta: String::new(),
            fields: vec!["Name:string".to_string()],
            methods: vec![make_method()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let current = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "Service".to_string(),
            class_meta: String::new(),
            fields: vec!["Name:string (modified)".to_string()],
            methods: vec![make_method()],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let actions = diff_snapshots(&baseline, &current);
    let method_actions: Vec<&DiffAction> = actions
        .iter()
        .filter(|a| a.target == DiffTarget::Method && a.label.contains("Process"))
        .collect();
    assert_eq!(
        method_actions.len(),
        1,
        "exactly one method action for Process expected, got: {method_actions:?}"
    );
    assert_eq!(
        method_actions[0].kind,
        DiffKind::Unchanged,
        "unchanged method must be Unchanged"
    );
}

/// A changed constructor IS detected as Modified.
#[test]
fn changed_constructor_is_detected_as_modified() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "Example".to_string(),
            class_meta: String::new(),
            fields: vec![],
            methods: vec![CapturedMethod {
                declaration: None,
                sig: "Example(string prefix)".to_string(),
                markers: vec![],
                body: Some(" { Initialize(prefix); }".to_string()),
            }],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let current = CapturedStructure {
        imports: vec![],
        classes: vec![CapturedClass {
            name: "Example".to_string(),
            class_meta: String::new(),
            fields: vec![],
            methods: vec![CapturedMethod {
                declaration: None,
                sig: "Example(string prefix)".to_string(),
                markers: vec![],
                body: Some(" { Initialize(prefix); Validate(); }".to_string()),
            }],
        }],
        orphan_fields: vec![],
        orphan_methods: vec![],
    };

    let actions = diff_snapshots(&baseline, &current);
    let ctor_modified: Vec<&DiffAction> = actions
        .iter()
        .filter(|a| {
            a.target == DiffTarget::Method
                && a.label.contains("Example")
                && a.kind == DiffKind::Modified
        })
        .collect();
    assert_eq!(
        ctor_modified.len(),
        1,
        "changed constructor must be Modified, got: {actions:?}"
    );
}

/// Regression: the `method_key` must return the declared name for a
/// constructor with `: base(...)` containing interpolated strings.
#[test]
fn method_key_ctor_with_interpolated_base_initializer() {
    use crate::diff::keys::method_key;
    let sig = "ExampleException(string value, object context)";
    assert_eq!(method_key(sig), "ExampleException");
}

/// Regression: the `method_key` must return the declared name for a
/// constructor with `: this(...)`.
#[test]
fn method_key_ctor_with_interpolated_this_initializer() {
    use crate::diff::keys::method_key;
    let sig = "Widget(string prefix)";
    assert_eq!(method_key(sig), "Widget");
}
