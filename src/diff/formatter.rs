// src/diff/formatter.rs
//
// Render a `Vec<DiffAction>` into the canonical compact change-set format
// used by the diff tool, and provide a one-line summary helper.

use std::fmt::Write;

use crate::compression::Fidelity;

use super::action::{DiffAction, DiffKind, DiffTarget};

/// True when two methods share the same signature and markers but differ
/// in body content — i.e. a body-only change (logic fix, no API change).
fn is_body_only_change(before: &str, after: &str) -> bool {
    before == after
}

/// Format a sequence of diff actions into the canonical compact change-set.
/// The output is grouped by class and only emits `=` for unchanged classes
/// at the top level (one line per untouched class) so the reader can see
/// scope at a glance.
pub fn format_diff(actions: &[DiffAction], fidelity: Fidelity) -> String {
    let mut out = String::new();
    let mut current_class: Option<String> = None;

    for action in actions {
        // Break the "method X / field Y" indentation under the right class.
        if matches!(action.target, DiffTarget::Method | DiffTarget::Field)
            && current_class.is_some()
            && !out.ends_with('\n')
        {
            out.push('\n');
        }

        if action.target == DiffTarget::Class {
            // Blank line between classes for Medium/High fidelity.
            if fidelity != Fidelity::Low && !out.is_empty() {
                out.push('\n');
            }
            if matches!(action.kind, DiffKind::Unchanged) {
                let _ = writeln!(out, "= {} (unchanged)", action.label);
            } else {
                let _ = write!(out, "{} {} {}", action.kind.symbol(), action.label, action.detail);
                out.push('\n');
            }
            current_class = Some(action.label.clone());
        } else {
            let indent = if fidelity == Fidelity::Low { "" } else { "  " };
            match action.kind {
                DiffKind::Modified => {
                    // Body-only change? The previous_detail and detail carry
                    // signatures which are identical in that case. Emit a
                    // clear "(body changed)" marker so the reader knows the
                    // method's logic changed even though its signature didn't.
                    if action.target == DiffTarget::Method
                        && is_body_only_change(&action.previous_detail, &action.detail)
                    {
                        let _ = writeln!(out, "{}{} {} (body changed)", indent, action.kind.symbol(), action.label);
                    } else {
                        let _ = writeln!(out, "{}{} {} ~ {}", indent, action.kind.symbol(), action.label, action.detail);
                        if !action.previous_detail.is_empty() {
                            let _ = writeln!(out, "{}    was: {}", indent, action.previous_detail);
                        }
                    }
                }
                DiffKind::Unchanged => {
                    let _ = writeln!(out, "{}{} {}", indent, action.kind.symbol(), action.detail);
                }
                _ => {
                    let _ = writeln!(out, "{}{} {} {}", indent, action.kind.symbol(), action.label, action.detail);
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
#[path = "../tests/diff/formatter.rs"]
mod tests;
