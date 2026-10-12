//! C# diff comparison evidence from the same parser/query walk as the snapshot.
//! Exact owner spans and query-match IDs associate facts; display text is never
//! reparsed and global compression/IR queries are not changed.

use super::snapshot::DeclarationEvidence;
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
            (
                owner,
                DeclarationEvidence {
                    return_type,
                    attributes: attrs.into_iter().map(|(_, text)| text).collect(),
                },
            )
        })
        .collect()
}
