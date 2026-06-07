// src/diff/differ.rs
//
// AST-level diff comparator. Takes two `CapturedStructure`s and emits a
// sequence of `DiffAction`s describing what changed.

use std::collections::{BTreeMap, BTreeSet};

use super::action::{DiffAction, DiffKind, DiffTarget};
use super::keys::{
    field_key, group_by_key, group_strings_by_key, method_key, summarize_class,
};
use super::snapshot::{CapturedClass, CapturedStructure};

/// Compute the AST-level diff between two snapshots.
pub fn diff_snapshots(
    baseline: &CapturedStructure,
    current: &CapturedStructure,
) -> Vec<DiffAction> {
    let mut actions: Vec<DiffAction> = Vec::new();

    // ---- Imports ---------------------------------------------------------
    let base_imports: BTreeMap<&str, ()> = baseline
        .imports
        .iter()
        .map(|s| (s.as_str(), ()))
        .collect();
    let cur_imports: BTreeMap<&str, ()> = current
        .imports
        .iter()
        .map(|s| (s.as_str(), ()))
        .collect();
    for imp in &current.imports {
        if !base_imports.contains_key(imp.as_str()) {
            actions.push(DiffAction {
                kind: DiffKind::Added,
                target: DiffTarget::Import,
                label: "import".to_string(),
                detail: imp.clone(),
                previous_detail: String::new(),
            });
        }
    }
    for imp in &baseline.imports {
        if !cur_imports.contains_key(imp.as_str()) {
            actions.push(DiffAction {
                kind: DiffKind::Removed,
                target: DiffTarget::Import,
                label: "import".to_string(),
                detail: imp.clone(),
                previous_detail: String::new(),
            });
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
                actions.push(DiffAction {
                    kind: DiffKind::Removed,
                    target: DiffTarget::Class,
                    label: format!("class {}", base_cls.name),
                    detail: summarize_class(base_cls),
                    previous_detail: String::new(),
                });
            }
            Some(cur_cls) => {
                let child_actions = diff_class(base_cls, cur_cls);
                // A class is "unchanged" only if no child element was
                // added/removed/modified. Per-method `=` markers don't
                // count as real changes.
                let has_real_changes = child_actions
                    .iter()
                    .any(|a| a.kind != DiffKind::Unchanged);
                if has_real_changes {
                    actions.push(DiffAction {
                        kind: DiffKind::Modified,
                        target: DiffTarget::Class,
                        label: format!("class {}", base_cls.name),
                        detail: String::new(),
                        previous_detail: String::new(),
                    });
                    actions.extend(child_actions);
                } else {
                    actions.push(DiffAction {
                        kind: DiffKind::Unchanged,
                        target: DiffTarget::Class,
                        label: format!("class {}", base_cls.name),
                        detail: String::new(),
                        previous_detail: String::new(),
                    });
                }
            }
        }
    }
    for cur_cls in &current.classes {
        if !seen.contains(&cur_cls.name) {
            actions.push(DiffAction {
                kind: DiffKind::Added,
                target: DiffTarget::Class,
                label: format!("class {}", cur_cls.name),
                detail: summarize_class(cur_cls),
                previous_detail: String::new(),
            });
        }
    }

    // ---- Orphan fields ---------------------------------------------------
    let base_orphans: BTreeMap<&str, ()> = baseline
        .orphan_fields
        .iter()
        .map(|s| (s.as_str(), ()))
        .collect();
    let cur_orphans: BTreeMap<&str, ()> = current
        .orphan_fields
        .iter()
        .map(|s| (s.as_str(), ()))
        .collect();
    for f in &current.orphan_fields {
        if !base_orphans.contains_key(f.as_str()) {
            actions.push(DiffAction {
                kind: DiffKind::Added,
                target: DiffTarget::Field,
                label: "field".to_string(),
                detail: f.clone(),
                previous_detail: String::new(),
            });
        }
    }
    for f in &baseline.orphan_fields {
        if !cur_orphans.contains_key(f.as_str()) {
            actions.push(DiffAction {
                kind: DiffKind::Removed,
                target: DiffTarget::Field,
                label: "field".to_string(),
                detail: f.clone(),
                previous_detail: String::new(),
            });
        }
    }

    actions
}

fn diff_class(baseline: &CapturedClass, current: &CapturedClass) -> Vec<DiffAction> {
    let mut actions: Vec<DiffAction> = Vec::new();

    // Methods: key by method name (part before `(`).
    let base_methods = group_by_key(&baseline.methods, |m| method_key(&m.sig));
    let cur_methods = group_by_key(&current.methods, |m| method_key(&m.sig));

    let mut emitted_keys: BTreeSet<String> = BTreeSet::new();
    for (key, base_group) in &base_methods {
        emitted_keys.insert(key.clone());
        match cur_methods.get(key) {
            None => {
                for m in base_group {
                    actions.push(DiffAction {
                        kind: DiffKind::Removed,
                        target: DiffTarget::Method,
                        label: format!("method {}", key),
                        detail: m.sig.clone(),
                        previous_detail: String::new(),
                    });
                }
            }
            Some(cur_group) => {
                let n = base_group.len().max(cur_group.len());
                for i in 0..n {
                    match (base_group.get(i), cur_group.get(i)) {
                        (Some(b), Some(c)) => {
                            if b.sig == c.sig && b.markers == c.markers {
                                actions.push(DiffAction {
                                    kind: DiffKind::Unchanged,
                                    target: DiffTarget::Method,
                                    label: format!("method {}", key),
                                    detail: c.sig.clone(),
                                    previous_detail: String::new(),
                                });
                            } else {
                                actions.push(DiffAction {
                                    kind: DiffKind::Modified,
                                    target: DiffTarget::Method,
                                    label: format!("method {}", key),
                                    detail: c.sig.clone(),
                                    previous_detail: b.sig.clone(),
                                });
                            }
                        }
                        (Some(b), None) => actions.push(DiffAction {
                            kind: DiffKind::Removed,
                            target: DiffTarget::Method,
                            label: format!("method {}", key),
                            detail: b.sig.clone(),
                            previous_detail: String::new(),
                        }),
                        (None, Some(c)) => actions.push(DiffAction {
                            kind: DiffKind::Added,
                            target: DiffTarget::Method,
                            label: format!("method {}", key),
                            detail: c.sig.clone(),
                            previous_detail: String::new(),
                        }),
                        _ => {}
                    }
                }
            }
        }
    }
    for (key, cur_group) in &cur_methods {
        if !emitted_keys.contains(key) {
            for m in cur_group {
                actions.push(DiffAction {
                    kind: DiffKind::Added,
                    target: DiffTarget::Method,
                    label: format!("method {}", key),
                    detail: m.sig.clone(),
                    previous_detail: String::new(),
                });
            }
        }
    }

    // Fields: key by name (part before `:`).
    let base_fields = group_strings_by_key(&baseline.fields, field_key);
    let cur_fields = group_strings_by_key(&current.fields, field_key);

    for (key, base_group) in &base_fields {
        match cur_fields.get(key) {
            None => {
                for f in base_group {
                    actions.push(DiffAction {
                        kind: DiffKind::Removed,
                        target: DiffTarget::Field,
                        label: format!("field {}", key),
                        detail: f.clone(),
                        previous_detail: String::new(),
                    });
                }
            }
            Some(cur_group) => {
                let n = base_group.len().max(cur_group.len());
                for i in 0..n {
                    match (base_group.get(i), cur_group.get(i)) {
                        (Some(b), Some(c)) => {
                            if b == c {
                                actions.push(DiffAction {
                                    kind: DiffKind::Unchanged,
                                    target: DiffTarget::Field,
                                    label: format!("field {}", key),
                                    detail: c.clone(),
                                    previous_detail: String::new(),
                                });
                            } else {
                                actions.push(DiffAction {
                                    kind: DiffKind::Modified,
                                    target: DiffTarget::Field,
                                    label: format!("field {}", key),
                                    detail: c.clone(),
                                    previous_detail: b.clone(),
                                });
                            }
                        }
                        (Some(b), None) => actions.push(DiffAction {
                            kind: DiffKind::Removed,
                            target: DiffTarget::Field,
                            label: format!("field {}", key),
                            detail: b.clone(),
                            previous_detail: String::new(),
                        }),
                        (None, Some(c)) => actions.push(DiffAction {
                            kind: DiffKind::Added,
                            target: DiffTarget::Field,
                            label: format!("field {}", key),
                            detail: c.clone(),
                            previous_detail: String::new(),
                        }),
                        _ => {}
                    }
                }
            }
        }
    }
    for (key, cur_group) in &cur_fields {
        if !base_fields.contains_key(key) {
            for f in cur_group {
                actions.push(DiffAction {
                    kind: DiffKind::Added,
                    target: DiffTarget::Field,
                    label: format!("field {}", key),
                    detail: f.clone(),
                    previous_detail: String::new(),
                });
            }
        }
    }

    actions
}

#[cfg(test)]
#[path = "../tests/diff/differ.rs"]
mod tests;
