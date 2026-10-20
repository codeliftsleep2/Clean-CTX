//! Parser-owned method attributes associated with their direct class member.

use super::{AttributeInput, MethodInput};
use crate::compression::capture_pipeline::CapEntry;
use crate::ir::csharp_resolution::CSharpSourceSpan;

pub(super) fn direct_methods(
    captures: &[CapEntry],
    owner: &CapEntry,
    records: &[(String, String)],
) -> Vec<MethodInput> {
    let mut roots = captures
        .iter()
        .filter(|capture| capture.name == "method.root" && contains(owner, capture))
        .filter(|capture| {
            !captures.iter().any(|nested| {
                nested.name == "class.root"
                    && nested.start_byte > owner.start_byte
                    && nested.end_byte < owner.end_byte
                    && contains(nested, capture)
            })
        })
        .collect::<Vec<_>>();
    roots.sort_by_key(|capture| capture.start_byte);
    roots
        .into_iter()
        .zip(records)
        .map(|(method, (_, name))| MethodInput {
            name: name.clone(),
            attributes: direct_method_attributes(captures, method),
        })
        .collect()
}

fn direct_method_attributes(captures: &[CapEntry], method: &CapEntry) -> Vec<AttributeInput> {
    captures
        .iter()
        .filter(|capture| capture.name == "csharp.method_attribute" && contains(method, capture))
        .filter_map(|capture| {
            let raw = capture.raw_text.trim();
            let written = raw.split_once('(').map_or(raw, |(name, _)| name).trim();
            (!written.is_empty()).then(|| AttributeInput {
                written: written.to_string(),
                raw: raw.to_string(),
                span: CSharpSourceSpan {
                    start_byte: capture.start_byte,
                    end_byte: capture.end_byte,
                },
            })
        })
        .collect()
}

fn contains(owner: &CapEntry, nested: &CapEntry) -> bool {
    owner.start_byte <= nested.start_byte && nested.end_byte <= owner.end_byte
}
