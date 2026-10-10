//! Structured C# declaration identity and Class base-list lowering.

use super::{CapturedNode, CoreOp, PassContext};
use std::collections::HashMap;

const CSHARP_TYPE_ROOTS: [&str; 5] = [
    "class.root",
    "interface.root",
    "struct.root",
    "enum.root",
    "record.root",
];

pub(super) fn apply_csharp_declaration_names(captures: &mut [CapturedNode]) {
    let names: Vec<(String, String, usize, usize)> = captures
        .iter()
        .filter_map(|capture| {
            csharp_root_for_name_capture(&capture.name).map(|root| {
                (
                    root.to_string(),
                    capture.raw_text.clone(),
                    capture.start_byte,
                    capture.end_byte,
                )
            })
        })
        .collect();

    for root in captures
        .iter_mut()
        .filter(|capture| CSHARP_TYPE_ROOTS.contains(&capture.name.as_str()))
    {
        if let Some((_, name, _, _)) = names
            .iter()
            .filter(|(kind, _, start, end)| {
                kind == &root.name && root.start_byte <= *start && *end <= root.end_byte
            })
            .min_by_key(|(_, _, start, _)| *start)
        {
            root.text = name.clone();
        }
    }
}

fn csharp_root_for_name_capture(capture: &str) -> Option<&'static str> {
    match capture {
        "csharp.class_name" => Some("class.root"),
        "csharp.interface_name" => Some("interface.root"),
        "csharp.struct_name" => Some("struct.root"),
        "csharp.enum_name" => Some("enum.root"),
        "csharp.record_name" => Some("record.root"),
        _ => None,
    }
}

pub(super) fn emit_csharp_class_base_type(
    state: &mut PassContext,
    base_type: &CapturedNode,
    captures: &[CapturedNode],
    class_owners: &HashMap<(usize, usize), String>,
) {
    let Some(owner) = nearest_csharp_type_root(captures, base_type) else {
        return;
    };
    if owner.name != "class.root" {
        return;
    }
    let Some(owner_id) = class_owners.get(&(owner.start_byte, owner.end_byte)) else {
        return;
    };
    let written_type = base_type.raw_text.trim();
    if written_type.is_empty() {
        return;
    }
    let first = captures
        .iter()
        .filter(|candidate| candidate.name == "csharp.base_type")
        .filter(|candidate| {
            nearest_csharp_type_root(captures, candidate).is_some_and(|candidate_owner| {
                candidate_owner.start_byte == owner.start_byte
                    && candidate_owner.end_byte == owner.end_byte
            })
        })
        .min_by_key(|candidate| candidate.start_byte)
        .is_some_and(|candidate| {
            candidate.start_byte == base_type.start_byte && candidate.end_byte == base_type.end_byte
        });

    if first {
        state.instructions.push(CoreOp::BaseTypeRef(
            owner_id.clone(),
            written_type.to_string(),
        ));
        dispatch_synthetic_csharp_base(state, "csharp.class_base_type", written_type);
    } else {
        let target = state
            .layer_context
            .symbol_table
            .alias_for(written_type)
            .map(str::to_string)
            .unwrap_or_else(|| written_type.to_string());
        state
            .instructions
            .push(CoreOp::Implements(owner_id.clone(), target));
        dispatch_synthetic_csharp_base(state, "csharp.class_interface_type", written_type);
    }
}

fn nearest_csharp_type_root<'a>(
    captures: &'a [CapturedNode],
    capture: &CapturedNode,
) -> Option<&'a CapturedNode> {
    captures
        .iter()
        .filter(|candidate| {
            CSHARP_TYPE_ROOTS.contains(&candidate.name.as_str())
                && candidate.start_byte <= capture.start_byte
                && capture.end_byte <= candidate.end_byte
        })
        .min_by_key(|candidate| candidate.end_byte - candidate.start_byte)
}

fn dispatch_synthetic_csharp_base(state: &mut PassContext, capture_name: &str, written: &str) {
    for layer in state.language_layers.iter_mut() {
        state.instructions.extend(layer.process_capture(
            capture_name,
            written,
            &mut state.layer_context,
        ));
    }
}
