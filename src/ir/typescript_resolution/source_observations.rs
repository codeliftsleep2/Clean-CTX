//! Opt-in written shapes, never Angular API, runtime or canonical registration proof.
//! No public tool, index, global owner or persistence path consumes these records.
use super::{Host, Witness};
#[path = "direct_identity.rs"]
pub mod direct_identity;
#[cfg(feature = "angular")]
pub use crate::angular_meta::written_claim_composition as claim_composition;
#[path = "observation_bounded.rs"]
pub(crate) mod bounded;
#[path = "request_budget.rs"]
pub mod request_budget;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Half-open UTF-8 byte range in one exact source snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    fn of(node: tree_sitter::Node<'_>) -> Self {
        Self {
            start: node.start_byte(),
            end: node.end_byte(),
        }
    }
}
/// Classification of syntax already acquired. Neither variant resolves a target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WrittenExpression {
    ReferenceText {
        text: String,
        span: Span,
    },
    Unsupported {
        text: String,
        syntax_kind: String,
        span: Span,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingObservation {
    pub object: Span,
    pub path_literal: Span,
    pub route_path: String,
    pub resolve_pair: Span,
    pub mapping_pair: Span,
    pub key_span: Span,
    pub key: String,
    pub expression: WrittenExpression,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OmissionReason {
    DuplicatePath,
    DuplicateResolve,
    DuplicateMappingKey,
    UnsupportedContainer,
    UnsupportedPath,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Omission {
    pub span: Span,
    pub reason: OmissionReason,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    UnreadableOrUnauthorized,
    /// The pinned frontend cannot certify structure, including grammar gaps.
    /// This refusal does not prove the source is invalid TypeScript.
    MalformedSource,
    StaleSource,
}

/// Unforgeable candidate: identity, source records and consumed proof are private.
pub struct Candidate {
    source: PathBuf,
    source_hash: String,
    observations: Vec<MappingObservation>,
    omissions: Vec<Omission>,
    witnesses: Vec<Witness>,
    source_bytes: usize,
    scan_stop: Option<request_budget::Stop>,
}
/// A validated owned snapshot, not an indefinitely current canonical authority.
pub struct Admitted {
    candidate: Candidate,
}
impl Candidate {
    /// Disk-backed source only. No config, imports, ambient types or libraries.
    pub fn capture(roots: &[PathBuf], file: &Path) -> Result<Self, Refusal> {
        let mut host = Host::new(roots).map_err(|_| Refusal::UnreadableOrUnauthorized)?;
        let source = host
            .normalized(file)
            .ok_or(Refusal::UnreadableOrUnauthorized)?;
        let bytes = host
            .read(&source)
            .ok_or(Refusal::UnreadableOrUnauthorized)?;
        let language = if matches!(
            source.extension().and_then(|x| x.to_str()),
            Some("tsx" | "jsx")
        ) {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            crate::compression::language::safe_typescript_language()
                .ok_or(Refusal::MalformedSource)?
        };
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language)
            .map_err(|_| Refusal::MalformedSource)?;
        #[cfg(test)]
        super::e1_metrics::parse(&source, bytes.len());
        let (tree, _) =
            super::syntax_frontend::parse(&mut parser, &bytes).ok_or(Refusal::MalformedSource)?;
        let (observations, omissions) = extract(tree.root_node(), &bytes);
        Ok(Self {
            source,
            source_hash: crate::cache::content_hash(bytes.as_bytes()),
            observations,
            omissions,
            witnesses: host.witnesses.into_vec(),
            source_bytes: bytes.len(),
            scan_stop: None,
        })
    }
    /// Independent final boundary. Required witnesses are only this capture's inputs.
    pub fn admit(self) -> Result<Admitted, Refusal> {
        if !Witness::all_current(&self.witnesses) {
            return Err(Refusal::StaleSource);
        }
        Ok(Admitted { candidate: self })
    }
}
impl Admitted {
    pub fn source(&self) -> &Path {
        &self.candidate.source
    }
    pub fn source_hash(&self) -> &str {
        &self.candidate.source_hash
    }
    pub fn observations(&self) -> &[MappingObservation] {
        &self.candidate.observations
    }
    pub fn omissions(&self) -> &[Omission] {
        &self.candidate.omissions
    }
    /// Revalidation is explicit; storage age or earlier admission is not currentness.
    pub fn is_current(&self) -> bool {
        Witness::all_current(&self.candidate.witnesses)
    }
    /// Retained proof serialization size, not an allocator/heap-size claim.
    pub fn serialized_witness_bytes(&self) -> usize {
        serde_json::to_vec(&self.candidate.witnesses)
            .expect("serializable witnesses")
            .len()
    }
    /// Owned record text bytes; parsed source/tree/program are not retained.
    pub fn retained_record_text_bytes(&self) -> usize {
        self.candidate
            .observations
            .iter()
            .map(|o| {
                o.key.len()
                    + o.route_path.len()
                    + match &o.expression {
                        WrittenExpression::ReferenceText { text, .. } => text.len(),
                        WrittenExpression::Unsupported {
                            text, syntax_kind, ..
                        } => text.len() + syntax_kind.len(),
                    }
            })
            .sum()
    }
}

fn key(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    let raw = node.utf8_text(source.as_bytes()).ok()?;
    match node.kind() {
        "property_identifier" | "identifier" if !raw.contains('\\') => Some(raw.into()),
        "string" => super::string_value::decode(raw),
        _ => None,
    }
}
fn pairs<'a>(
    node: tree_sitter::Node<'a>,
    source: &str,
) -> Option<BTreeMap<String, Vec<tree_sitter::Node<'a>>>> {
    let mut groups = BTreeMap::<String, Vec<_>>::new();
    let mut cursor = node.walk();
    for pair in node
        .named_children(&mut cursor)
        .filter(|n| n.kind() != "comment")
    {
        if pair.kind() != "pair" {
            return None;
        }
        let name = key(pair.child_by_field_name("key")?, source)?;
        groups.entry(name).or_default().push(pair);
    }
    Some(groups)
}
fn expression(node: tree_sitter::Node<'_>, source: &str) -> WrittenExpression {
    fn reference(node: tree_sitter::Node<'_>) -> bool {
        node.kind() == "identifier"
            || (node.kind() == "member_expression"
                && node.child_by_field_name("object").is_some_and(reference)
                && node
                    .child_by_field_name("property")
                    .is_some_and(|p| p.kind() == "property_identifier"))
    }
    let text = source[node.byte_range()].to_owned();
    if reference(node) {
        WrittenExpression::ReferenceText {
            text,
            span: Span::of(node),
        }
    } else {
        WrittenExpression::Unsupported {
            text,
            syntax_kind: node.kind().into(),
            span: Span::of(node),
        }
    }
}
fn extract(root: tree_sitter::Node<'_>, source: &str) -> (Vec<MappingObservation>, Vec<Omission>) {
    extract_with(root, source, || true)
}
fn extract_with(
    root: tree_sitter::Node<'_>,
    source: &str,
    mut visit: impl FnMut() -> bool,
) -> (Vec<MappingObservation>, Vec<Omission>) {
    let mut records = Vec::new();
    let mut omissions = Vec::new();
    let mut pending = vec![root];
    while let Some(object) = pending.pop() {
        if !visit() {
            break;
        }
        let mut cursor = object.walk();
        pending.extend(object.named_children(&mut cursor));
        if object.kind() != "object" {
            continue;
        }
        let omit = |reason| Omission {
            span: Span::of(object),
            reason,
        };
        let Some(fields) = pairs(object, source) else {
            omissions.push(omit(OmissionReason::UnsupportedContainer));
            continue;
        };
        let (Some(paths), Some(resolves)) = (fields.get("path"), fields.get("resolve")) else {
            continue;
        };
        if paths.len() != 1 {
            omissions.push(omit(OmissionReason::DuplicatePath));
            continue;
        }
        if resolves.len() != 1 {
            omissions.push(omit(OmissionReason::DuplicateResolve));
            continue;
        }
        let path = paths[0].child_by_field_name("value").expect("pair value");
        let Some(route_path) = (path.kind() == "string")
            .then(|| key(path, source))
            .flatten()
        else {
            omissions.push(omit(OmissionReason::UnsupportedPath));
            continue;
        };
        let resolve = resolves[0]
            .child_by_field_name("value")
            .expect("pair value");
        if resolve.kind() != "object" {
            omissions.push(omit(OmissionReason::UnsupportedContainer));
            continue;
        }
        let Some(mappings) = pairs(resolve, source) else {
            omissions.push(omit(OmissionReason::UnsupportedContainer));
            continue;
        };
        for (name, mappings) in mappings {
            if mappings.len() != 1 {
                omissions.push(Omission {
                    span: Span::of(resolve),
                    reason: OmissionReason::DuplicateMappingKey,
                });
                continue;
            }
            let pair = mappings[0];
            records.push(MappingObservation {
                object: Span::of(object),
                path_literal: Span::of(path),
                route_path: route_path.clone(),
                resolve_pair: Span::of(resolves[0]),
                mapping_pair: Span::of(pair),
                key_span: Span::of(pair.child_by_field_name("key").expect("pair key")),
                key: name,
                expression: expression(
                    pair.child_by_field_name("value").expect("pair value"),
                    source,
                ),
            });
        }
    }
    records.sort_by_key(|r| (r.object.start, r.mapping_pair.start));
    (records, omissions)
}

#[cfg(test)]
#[path = "../../tests/ir/written_source_observations_e2.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/ir/written_source_observation_measurements_e2.rs"]
mod measurement_tests;
