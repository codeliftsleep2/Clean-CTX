// src/diff/formatter.rs
//
// Render a `Vec<DiffAction>` into the canonical compact change-set format
// used by the diff tool, and provide a one-line summary helper.

use crate::compression::Fidelity;

use super::action::{DiffAction, DiffKind, DiffTarget};

/// Format a sequence of diff actions into the canonical compact change-set.
/// The output is grouped by class and only emits `=` for unchanged classes
/// at the top level (one line per untouched class) so the reader can see
/// scope at a glance.
pub fn format_diff(actions: &[DiffAction], fidelity: Fidelity) -> String {
    let mut out = String::new();
    let mut current_class: Option<String> = None;

    for action in actions {
        // Break the "method X / field Y" indentation under the right class.
        match action.target {
            DiffTarget::Method | DiffTarget::Field => {
                if let Some(cls) = &current_class {
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    let _ = cls; // keep current_class alive
                }
            }
            _ => {}
        }

        if action.target == DiffTarget::Class {
            // Blank line between classes for Medium/High fidelity.
            if fidelity != Fidelity::Low && !out.is_empty() {
                out.push('\n');
            }
            if matches!(action.kind, DiffKind::Unchanged) {
                out.push_str(&format!("= {} (unchanged)\n", action.label));
            } else {
                out.push_str(
                    &format!(
                        "{} {} {}\n",
                        action.kind.symbol(),
                        action.label,
                        action.detail,
                    )
                    .trim_end(),
                );
                out.push('\n');
            }
            current_class = Some(action.label.clone());
        } else {
            let indent = if fidelity == Fidelity::Low { "" } else { "  " };
            match action.kind {
                DiffKind::Modified => {
                    out.push_str(&format!(
                        "{}{} {} ~ {}\n",
                        indent,
                        action.kind.symbol(),
                        action.label,
                        action.detail
                    ));
                    if !action.previous_detail.is_empty() {
                        out.push_str(&format!(
                            "{}    was: {}\n",
                            indent, action.previous_detail
                        ));
                    }
                }
                DiffKind::Unchanged => {
                    out.push_str(&format!(
                        "{}{} {}\n",
                        indent,
                        action.kind.symbol(),
                        action.detail
                    ));
                }
                _ => {
                    out.push_str(&format!(
                        "{}{} {} {}\n",
                        indent,
                        action.kind.symbol(),
                        action.label,
                        action.detail
                    ));
                }
            }
        }
    }
    out
}

/// Render a single one-line summary of the diff, suitable for headers.
pub fn diff_summary(actions: &[DiffAction]) -> (usize, usize, usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    let mut modified = 0;
    let mut unchanged = 0;
    for a in actions {
        match a.kind {
            DiffKind::Added => added += 1,
            DiffKind::Removed => removed += 1,
            DiffKind::Modified => modified += 1,
            DiffKind::Unchanged => unchanged += 1,
        }
    }
    (added, removed, modified, unchanged)
}

#[cfg(test)]
mod tests {
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
}
