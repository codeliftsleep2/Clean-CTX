//! C# diff comparison evidence from the same parser/query walk as the snapshot.
//! Exact owner spans and query-match IDs associate facts; display text is never
//! reparsed and global compression/IR queries are not changed.

use super::snapshot::DeclarationEvidence;
use crate::compaction::method::find_method_params;
use crate::compaction::modifiers::{MODIFIERS_LOW, strip_csharp_attributes};
use crate::compaction::signature::split_head_parts;
use crate::compression::capture_pipeline::CapturedNode;
use std::collections::HashMap;

pub(super) const QUERY: &str = r#"
    (method_declaration returns: (_) @diff.signature.return) @diff.signature.owner
    (method_declaration (attribute_list) @diff.signature.attribute) @diff.signature.owner
"#;

pub(super) fn collect(captures: &[CapturedNode]) -> HashMap<(usize, usize), DeclarationEvidence> {
    let owners: HashMap<_, _> = captures
        .iter()
        .filter(|capture| capture.name == "diff.signature.owner")
        .map(|capture| (capture.match_index, (capture.start_byte, capture.end_byte)))
        .collect();
    let owner_sources: HashMap<_, _> = captures
        .iter()
        .filter(|capture| capture.name == "diff.signature.owner")
        .map(|capture| {
            (
                (capture.start_byte, capture.end_byte),
                capture.raw_text.as_str(),
            )
        })
        .collect();
    let mut returns = HashMap::new();
    let mut attributes: HashMap<_, Vec<(usize, String)>> = HashMap::new();
    for capture in captures {
        let Some(owner) = owners.get(&capture.match_index) else {
            continue;
        };
        match capture.name.as_str() {
            "diff.signature.return" => {
                returns.insert(*owner, capture.raw_text.clone());
            }
            "diff.signature.attribute" => {
                attributes
                    .entry(*owner)
                    .or_default()
                    .push((capture.start_byte, capture.raw_text.clone()));
            }
            _ => {}
        }
    }
    returns
        .into_iter()
        .map(|(owner, return_type)| {
            let mut attrs = attributes.remove(&owner).unwrap_or_default();
            attrs.sort_by_key(|(start, _)| *start);
            attrs.dedup_by_key(|(start, _)| *start);
            let modifiers = owner_sources
                .get(&owner)
                .map(|source| extract_modifiers(source, &return_type))
                .unwrap_or_default();
            (
                owner,
                DeclarationEvidence {
                    return_type,
                    attributes: attrs.into_iter().map(|(_, text)| text).collect(),
                    modifiers,
                },
            )
        })
        .collect()
}

/// Extract written C# method modifiers from a declaration already owned by
/// structured C# evidence. The structured return capture establishes the
/// return-type boundary; this adapter only retains recognized leading tokens
/// before it and never reparses display output or infers declaration identity.
fn extract_modifiers(source: &str, return_type: &str) -> Vec<String> {
    let declaration = strip_csharp_attributes(source);
    let Some((open, _)) = find_method_params(declaration) else {
        return Vec::new();
    };
    let Some(parts) = split_head_parts(declaration, open) else {
        return Vec::new();
    };
    let prefix = parts.prefix.trim_end();
    let Some(written_modifiers) = prefix.strip_suffix(return_type.trim()) else {
        return Vec::new();
    };
    written_modifiers
        .split_whitespace()
        .filter(|token| {
            MODIFIERS_LOW
                .iter()
                .any(|modifier| modifier.trim() == *token)
                || matches!(*token, "internal" | "readonly" | "unsafe" | "partial")
        })
        .map(str::to_string)
        .collect()
}
