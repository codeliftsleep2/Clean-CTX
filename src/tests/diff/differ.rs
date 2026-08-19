use super::*;
use crate::diff::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

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
fn detects_added_class() {
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
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["foo()"], &[])],
        orphan_fields: vec![],
orphan_methods: vec![],
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
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["process(id:number):boolean"], &[])],
        orphan_fields: vec![],
orphan_methods: vec![],
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
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class_with_bodies(
            "Foo",
            // Same signature — logic fix only.
            &[("process(id):void", Some("return id + 2;"))],
        )],
        orphan_fields: vec![],
orphan_methods: vec![],
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
orphan_methods: vec![],
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
orphan_methods: vec![],
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
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec!["NewService".to_string()],
        classes: vec![],
        orphan_fields: vec![],
orphan_methods: vec![],
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
orphan_methods: vec![],
    };
    let current = baseline.clone();
    let actions = diff_snapshots(&baseline, &current);
    let unchanged = actions
        .iter()
        .any(|a| a.kind == DiffKind::Unchanged && a.target == DiffTarget::Class);
    assert!(unchanged, "expected an `=` class action for unchanged snapshot");
}

/// Regression: a C# property-only change (adding a property to a class)
/// must be detected. Previously `CS_QUERY` only captured
/// `(field_declaration)`, so C# properties (`public string Name { get; set; }`)
/// were invisible to the diff — a critical false negative. F-01 diff audit.
#[test]
fn property_only_change_is_detected() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["foo()"], &[])],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["foo()"], &["Name:string"])],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let added_field = actions
        .iter()
        .any(|a| a.kind == DiffKind::Added && a.target == DiffTarget::Field);
    assert!(
        added_field,
        "property-only change must produce an Added field action, got {:?}",
        actions
    );
    // The class must NOT be reported unchanged.
    let class_unchanged = actions
        .iter()
        .any(|a| a.kind == DiffKind::Unchanged && a.target == DiffTarget::Class);
    assert!(
        !class_unchanged,
        "class with a new property must not be reported unchanged, got {:?}",
        actions
    );
}

/// Regression: C# return-type-first method signatures must produce the
/// correct method key. Previously `method_key` split on the first
/// whitespace, taking the return type as the key — producing doubled
/// tokens like `+ method bool bool Resolve(...)`. F-02 diff audit.
#[test]
fn csharp_return_type_first_method_key() {
    use crate::diff::keys::method_key;
    assert_eq!(method_key("bool Resolve(term,__)"), "Resolve");
    assert_eq!(
        method_key("GetTestOrgUnitValidatorData GetTestOrgUnitValidatorData()"),
        "GetTestOrgUnitValidatorData"
    );
    assert_eq!(method_key("void Delete(int id)"), "Delete");
    assert_eq!(method_key("Task<IActionResult> Create(...)"), "Create");
    // TS name-first still works.
    assert_eq!(method_key("getUser(id:string):Promise<User>"), "getUser");
}

/// G3-5: TS/Java name-first signatures with leading declarator keywords
/// (`export function`, `async function`) must key on the actual method
/// name, not the first token. Previously `method_key` took the FIRST
/// whitespace token, so every top-level `export function foo` grouped
/// under the key "export" — all top-level functions in a file merged
/// into one group and the rendered label was wrong.
#[test]
fn ts_export_function_method_key() {
    use crate::diff::keys::method_key;
    assert_eq!(method_key("export function formatName(name:string):string"), "formatName");
    assert_eq!(method_key("export async function loadData(id:number):Promise<void>"), "loadData");
    assert_eq!(method_key("async function fetchUser(id:number):Promise<User>"), "fetchUser");
    assert_eq!(method_key("function plain():void"), "plain");
    // Plain TS method (no declarator) still works.
    assert_eq!(method_key("getUser(id:string):Promise<User>"), "getUser");
}

/// Regression: two C# methods with the same return type must group
/// correctly (not be merged under the return-type key). F-02 diff audit.
#[test]
fn csharp_methods_with_same_return_type_group_correctly() {
    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["bool Foo()", "bool Bar()"], &[])],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![make_class("Foo", &["bool Foo()", "bool Bar()", "bool Baz()"], &[])],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    // Only Baz should be Added — Foo and Bar must be Unchanged.
    let added = actions
        .iter()
        .filter(|a| a.kind == DiffKind::Added && a.target == DiffTarget::Method)
        .collect::<Vec<_>>();
    assert_eq!(added.len(), 1, "only Baz should be added, got {:?}", actions);
    assert!(
        added[0].label.contains("Baz"),
        "added method should be Baz, got {:?}",
        added[0].label
    );
}

/// Regression: a change to the base class / interface list must be
/// detected even when the class name is unchanged. F-04 diff audit.
#[test]
fn class_meta_change_is_detected() {
    let mut base = make_class("Foo", &["foo()"], &[]);
    base.class_meta = ":BaseA".to_string();
    let mut cur = make_class("Foo", &["foo()"], &[]);
    cur.class_meta = ":BaseB".to_string();

    let baseline = CapturedStructure {
        imports: vec![],
        classes: vec![base],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let current = CapturedStructure {
        imports: vec![],
        classes: vec![cur],
        orphan_fields: vec![],
orphan_methods: vec![],
    };
    let actions = diff_snapshots(&baseline, &current);
    let class_modified = actions
        .iter()
        .any(|a| a.kind == DiffKind::Modified && a.target == DiffTarget::Class);
    assert!(
        class_modified,
        "base class change must produce a Modified class action, got {:?}",
        actions
    );
    // The class must NOT be reported unchanged.
    let class_unchanged = actions
        .iter()
        .any(|a| a.kind == DiffKind::Unchanged && a.target == DiffTarget::Class);
    assert!(
        !class_unchanged,
        "class with changed base must not be reported unchanged, got {:?}",
        actions
    );
}

/// G2-2: top-level functions must be diffed even without a class.
#[test]
fn orphan_function_change_is_detected() {
    let mk = |sig: &str| CapturedMethod { sig: sig.to_string(), markers: vec![], body: None };
    let b = CapturedStructure { imports: vec![], classes: vec![], orphan_fields: vec![], orphan_methods: vec![mk("foo()")] };
    let c = CapturedStructure { imports: vec![], classes: vec![], orphan_fields: vec![], orphan_methods: vec![mk("foo(x)")] };
    let a = diff_snapshots(&b, &c);
    assert!(a.iter().any(|x| x.kind == DiffKind::Modified && x.target == DiffTarget::Method));
}

/// G2-5: markers-only change sets reason_hint == markers.
#[test]
fn markers_only_change_reason_hint() {
    let mk = |markers: Vec<String>| CapturedMethod { sig: "foo()".to_string(), markers, body: Some("return 1".to_string()) };
    let cls = |m: CapturedMethod| CapturedClass { name: "Foo".to_string(), class_meta: String::new(), fields: vec![], methods: vec![m] };
    let b = CapturedStructure { imports: vec![], classes: vec![cls(mk(vec![]))], orphan_fields: vec![], orphan_methods: vec![] };
    let c = CapturedStructure { imports: vec![], classes: vec![cls(mk(vec!["#guard".to_string()]))], orphan_fields: vec![], orphan_methods: vec![] };
    let a = diff_snapshots(&b, &c);
    let m = a.iter().find(|x| x.kind == DiffKind::Modified && x.target == DiffTarget::Method);
    assert!(m.is_some(), "expected Modified, got {a:?}");
    assert_eq!(m.unwrap().reason_hint, "markers");
}