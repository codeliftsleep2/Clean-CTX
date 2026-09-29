//! Reduced hierarchical wire envelope.
//!
//! Content-producing MCP handlers ship a reduced `result.ir` that omits the
//! workspace-query-served fact families, so the result no longer re-ships what
//! the file-context `content` already drops. See [`super::hierarchy_to_wire`]
//! for the full envelope.

use super::{HierarchicalIR, hierarchy_to_wire};
use crate::ir::compiler::CompiledIR;
use serde_json::Value;

/// Reduced hierarchical wire envelope: same shape as
/// [`super::hierarchy_to_wire`], but with the workspace-query-served fact
/// families stripped. Stripped: top-level calls (`ca`), owner injections (`ij`)
/// and owner patterns (`p`), and the per-method R-43a fact families
/// (`cs`/`pf`/`fl`/`pa`/`cf`/`df`/`se`/`ec`). Declarations, signatures,
/// modifiers, class flags, and bodies remain.
pub(crate) fn hierarchy_to_wire_reduced(ir: &CompiledIR, hir: &HierarchicalIR) -> Value {
    let mut wire = hierarchy_to_wire(ir, hir);
    if let Some(node) = wire.pointer_mut("/ir") {
        strip_dropped_families(node);
    }
    wire
}

/// Remove the workspace-query-served fact families from a hierarchical `ir`
/// value in place. See [`hierarchy_to_wire_reduced`].
fn strip_dropped_families(ir: &mut Value) {
    let Some(root) = ir.as_object_mut() else {
        return;
    };
    root.remove("ca");
    for family in ["c", "if"] {
        let Some(owners) = root.get_mut(family).and_then(Value::as_array_mut) else {
            continue;
        };
        for owner in owners {
            let Some(owner) = owner.as_object_mut() else {
                continue;
            };
            owner.remove("ij");
            owner.remove("p");
            let Some(methods) = owner.get_mut("m").and_then(Value::as_array_mut) else {
                continue;
            };
            for method in methods {
                let Some(method) = method.as_object_mut() else {
                    continue;
                };
                for dropped in ["cs", "pf", "fl", "pa", "cf", "df", "se", "ec"] {
                    method.remove(dropped);
                }
            }
        }
    }
}
