// src/diff/differ.rs
//
// AST-level diff comparator. Takes two `CapturedStructure`s and emits a
// sequence of `DiffAction`s describing what changed.

use std::collections::{BTreeMap, BTreeSet};

use super::action::{DiffAction, DiffKind, DiffTarget};
use super::keys::{field_key, group_by_key, group_strings_by_key, method_key, summarize_class};
use super::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

/// Compute the AST-level diff between two snapshots.
pub fn diff_snapshots(
    baseline: &CapturedStructure,
    current: &CapturedStructure,
) -> Vec<DiffAction> {
    let mut actions: Vec<DiffAction> = Vec::new();

    // ---- Imports ---------------------------------------------------------
    let base_imports: BTreeMap<&str, ()> =
        baseline.imports.iter().map(|s| (s.as_str(), ())).collect();
    let cur_imports: BTreeMap<&str, ()> =
        current.imports.iter().map(|s| (s.as_str(), ())).collect();
    for imp in &current.imports {
        if !base_imports.contains_key(imp.as_str()) {
            actions.push(new_action(
                DiffKind::Added,
                DiffTarget::Import,
                "import",
                imp.clone(),
                String::new(),
            ));
        }
    }
    for imp in &baseline.imports {
        if !cur_imports.contains_key(imp.as_str()) {
            actions.push(new_action(
                DiffKind::Removed,
                DiffTarget::Import,
                "import",
                imp.clone(),
                String::new(),
            ));
        }
    }

    // ---- Classes ---------------------------------------------------------
    let cur_by_name: BTreeMap<&str, &CapturedClass> = current
        .classes
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut seen: BTreeSet<String> = BTreeSet::new();
    for base_cls in &baseline.classes {
        seen.insert(base_cls.name.clone());
        match cur_by_name.get(base_cls.name.as_str()) {
            None => {
                actions.push(new_action(
                    DiffKind::Removed,
                    DiffTarget::Class,
                    &format!("class {}", base_cls.name),
                    summarize_class(base_cls),
                    String::new(),
                ));
            }
            Some(cur_cls) => {
                let child_actions = diff_class(base_cls, cur_cls);
                // A class is "unchanged" only if no child element was
                // added/removed/modified AND the class-level metadata
                // (base class / interface list) is identical. F-04 diff
                // audit: previously a change to `class Foo : BaseA` →
                // `class Foo : BaseB` reported the class as unchanged.
                let class_meta_changed = base_cls.class_meta != cur_cls.class_meta;
                let has_child_changes = child_actions.iter().any(|a| a.kind != DiffKind::Unchanged);
                if class_meta_changed && !has_child_changes {
                    actions.push(new_action(
                        DiffKind::Modified,
                        DiffTarget::Class,
                        &format!("class {}", base_cls.name),
                        cur_cls.class_meta.clone(),
                        base_cls.class_meta.clone(),
                    ));
                } else if has_child_changes {
                    actions.push(new_action(
                        DiffKind::Modified,
                        DiffTarget::Class,
                        &format!("class {}", base_cls.name),
                        String::new(),
                        String::new(),
                    ));
                    actions.extend(child_actions);
                } else {
                    actions.push(new_action(
                        DiffKind::Unchanged,
                        DiffTarget::Class,
                        &format!("class {}", base_cls.name),
                        String::new(),
                        String::new(),
                    ));
                }
            }
        }
    }
    for cur_cls in &current.classes {
        if !seen.contains(&cur_cls.name) {
            actions.push(new_action(
                DiffKind::Added,
                DiffTarget::Class,
                &format!("class {}", cur_cls.name),
                summarize_class(cur_cls),
                String::new(),
            ));
        }
    }

    // ---- Orphan fields ---------------------------------------------------
    diff_orphan_fields(
        &baseline.orphan_fields,
        &current.orphan_fields,
        &mut actions,
    );

    // ---- Orphan methods (top-level functions) ----------------------------
    // G2-2 diff audit: files with only top-level functions (TS `function`,
    // C# top-level statements) previously produced zero methods and any
    // change to them was a false negative for `diff_commits`. Diff them
    // with the same method-grouping logic used inside classes.
    diff_methods(
        &baseline.orphan_methods,
        &current.orphan_methods,
        &mut actions,
    );

    actions
}

/// Construct a `DiffAction` with an empty reason hint.
fn new_action(
    kind: DiffKind,
    target: DiffTarget,
    label: &str,
    detail: String,
    previous: String,
) -> DiffAction {
    DiffAction {
        kind,
        target,
        label: label.to_string(),
        detail,
        previous_detail: previous,
        reason_hint: String::new(),
    }
}

/// Diff two collections of orphan/top-level field strings as Added/Removed.
fn diff_orphan_fields(baseline: &[String], current: &[String], actions: &mut Vec<DiffAction>) {
    let base: BTreeMap<&str, ()> = baseline.iter().map(|s| (s.as_str(), ())).collect();
    let cur: BTreeMap<&str, ()> = current.iter().map(|s| (s.as_str(), ())).collect();
    for f in current {
        if !base.contains_key(f.as_str()) {
            actions.push(new_action(
                DiffKind::Added,
                DiffTarget::Field,
                "field",
                f.clone(),
                String::new(),
            ));
        }
    }
    for f in baseline {
        if !cur.contains_key(f.as_str()) {
            actions.push(new_action(
                DiffKind::Removed,
                DiffTarget::Field,
                "field",
                f.clone(),
                String::new(),
            ));
        }
    }
}

/// Diff two method collections (class methods or top-level functions) by
/// method name, emitting Unchanged/Added/Removed/Modified per method.
/// G2-2 audit: extracted from `diff_class` so top-level functions reuse
/// the same grouping logic.
fn diff_methods(base: &[CapturedMethod], cur: &[CapturedMethod], actions: &mut Vec<DiffAction>) {
    let bg = group_by_key(base, |m| method_key(&m.sig));
    let cg = group_by_key(cur, |m| method_key(&m.sig));
    let mut keys: BTreeSet<String> = bg.keys().cloned().collect();
    keys.extend(cg.keys().cloned());

    for key in keys {
        match (bg.get(&key), cg.get(&key)) {
            (None, Some(g)) => {
                for m in g {
                    actions.push(method_action(DiffKind::Added, &key, m, "", ""));
                }
            }
            (Some(g), None) => {
                for m in g {
                    actions.push(method_action(DiffKind::Removed, &key, m, "", ""));
                }
            }
            (Some(g1), Some(g2)) => {
                for i in 0..g1.len().max(g2.len()) {
                    match (g1.get(i), g2.get(i)) {
                        (Some(b), Some(c)) => {
                            if b.sig == c.sig && b.markers == c.markers && b.body == c.body {
                                actions.push(method_action(DiffKind::Unchanged, &key, c, "", ""));
                            } else {
                                let reason = if b.sig != c.sig {
                                    "sig"
                                } else if b.markers != c.markers && b.body == c.body {
                                    "markers"
                                } else {
                                    "body"
                                };
                                actions.push(method_action(
                                    DiffKind::Modified,
                                    &key,
                                    c,
                                    &b.sig,
                                    reason,
                                ));
                            }
                        }
                        (Some(b), None) => {
                            actions.push(method_action(DiffKind::Removed, &key, b, "", ""))
                        }
                        (None, Some(c)) => {
                            actions.push(method_action(DiffKind::Added, &key, c, "", ""))
                        }
                        _ => {}
                    }
                }
            }
            (None, None) => {} // union of keys — unreachable, but Rust requires the arm
        }
    }
}

fn method_action(
    kind: DiffKind,
    key: &str,
    m: &CapturedMethod,
    previous_sig: &str,
    reason: &str,
) -> DiffAction {
    DiffAction {
        kind,
        target: DiffTarget::Method,
        label: format!("method {}", key),
        detail: m.sig.clone(),
        previous_detail: previous_sig.to_string(),
        reason_hint: reason.to_string(),
    }
}

fn diff_class(baseline: &CapturedClass, current: &CapturedClass) -> Vec<DiffAction> {
    let mut actions: Vec<DiffAction> = Vec::new();

    // Methods: key by method name (part before `(`).
    diff_methods(&baseline.methods, &current.methods, &mut actions);

    // Fields: key by name (part before `:`).
    let base_fields = group_strings_by_key(&baseline.fields, field_key);
    let cur_fields = group_strings_by_key(&current.fields, field_key);

    for (key, base_group) in &base_fields {
        match cur_fields.get(key) {
            None => {
                for f in base_group {
                    actions.push(new_action(
                        DiffKind::Removed,
                        DiffTarget::Field,
                        &format!("field {}", key),
                        f.clone(),
                        String::new(),
                    ));
                }
            }
            Some(cur_group) => {
                let n = base_group.len().max(cur_group.len());
                for i in 0..n {
                    match (base_group.get(i), cur_group.get(i)) {
                        (Some(b), Some(c)) => {
                            if b == c {
                                actions.push(new_action(
                                    DiffKind::Unchanged,
                                    DiffTarget::Field,
                                    &format!("field {}", key),
                                    c.clone(),
                                    String::new(),
                                ));
                            } else {
                                actions.push(new_action(
                                    DiffKind::Modified,
                                    DiffTarget::Field,
                                    &format!("field {}", key),
                                    c.clone(),
                                    b.clone(),
                                ));
                            }
                        }
                        (Some(b), None) => actions.push(new_action(
                            DiffKind::Removed,
                            DiffTarget::Field,
                            &format!("field {}", key),
                            b.clone(),
                            String::new(),
                        )),
                        (None, Some(c)) => actions.push(new_action(
                            DiffKind::Added,
                            DiffTarget::Field,
                            &format!("field {}", key),
                            c.clone(),
                            String::new(),
                        )),
                        _ => {}
                    }
                }
            }
        }
    }
    for (key, cur_group) in &cur_fields {
        if !base_fields.contains_key(key) {
            for f in cur_group {
                actions.push(new_action(
                    DiffKind::Added,
                    DiffTarget::Field,
                    &format!("field {}", key),
                    f.clone(),
                    String::new(),
                ));
            }
        }
    }

    actions
}

#[cfg(test)]
#[path = "../tests/diff/differ.rs"]
mod tests;
