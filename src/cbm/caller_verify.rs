//! Clean-CTX verification for CBM-derived inbound caller evidence.
//!
//! CBM supplies candidate relationships. This module checks the structural
//! argument shape in candidate C# source without changing CBM's graph or
//! inserting any relationship into `WorkspaceIndex`.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CallerVerificationStatus {
    VerifiedCompatible,
    RejectedArityMismatch,
    Ambiguous,
    Unverifiable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CsharpTargetSelector<'a> {
    qualified_name: &'a str,
    declared_arity: Option<usize>,
}

impl<'a> CsharpTargetSelector<'a> {
    pub(crate) fn new(qualified_name: &'a str) -> Self {
        Self {
            qualified_name,
            declared_arity: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_declared_arity(mut self, arity: usize) -> Self {
        self.declared_arity = Some(arity);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CandidateSource<'a> {
    pub(crate) file: &'a str,
    pub(crate) text: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CallerVerificationSummary {
    pub(crate) target_qualified_name: Option<String>,
    pub(crate) target_explicit_arity: Option<usize>,
    pub(crate) raw_candidates: usize,
    pub(crate) verified: usize,
    pub(crate) rejected_arity: usize,
    pub(crate) ambiguous: usize,
    pub(crate) unverifiable: usize,
    pub(crate) verified_caller_files: usize,
    pub(crate) compatible_caller_files: usize,
    pub(crate) resolution: CallerVerificationStatus,
}

impl CallerVerificationSummary {
    pub(crate) fn unverifiable(target: Option<&str>) -> Self {
        Self {
            target_qualified_name: target.map(str::to_string),
            target_explicit_arity: None,
            raw_candidates: 0,
            verified: 0,
            rejected_arity: 0,
            ambiguous: 0,
            unverifiable: 1,
            verified_caller_files: 0,
            compatible_caller_files: 0,
            resolution: CallerVerificationStatus::Unverifiable,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ArityShape {
    declared: usize,
    minimum_explicit: usize,
    maximum_explicit: Option<usize>,
    flexible: bool,
}

impl ArityShape {
    fn accepts(self, observed: usize) -> bool {
        observed >= self.minimum_explicit
            && self
                .maximum_explicit
                .is_none_or(|maximum| observed <= maximum)
    }

    fn exact_explicit(self) -> Option<usize> {
        match self.maximum_explicit {
            Some(maximum) if maximum == self.minimum_explicit && !self.flexible => Some(maximum),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Declaration {
    arity: ArityShape,
}

pub(crate) fn verify_csharp_callers(
    target_source: &str,
    selector: CsharpTargetSelector<'_>,
    candidates: &[CandidateSource<'_>],
) -> CallerVerificationSummary {
    let method_name = selector
        .qualified_name
        .rsplit('.')
        .next()
        .unwrap_or(selector.qualified_name);
    let type_name = selector.qualified_name.rsplit('.').nth(1);
    let declarations = match parse_declarations(target_source, method_name, type_name) {
        Some(found) => found,
        None => return CallerVerificationSummary::unverifiable(Some(selector.qualified_name)),
    };
    let selected: Vec<Declaration> = declarations
        .iter()
        .copied()
        .filter(|decl| {
            selector
                .declared_arity
                .is_none_or(|arity| decl.arity.declared == arity)
        })
        .collect();
    if selected.is_empty() {
        return CallerVerificationSummary::unverifiable(Some(selector.qualified_name));
    }
    if selector.declared_arity.is_none() && selected.len() > 1 {
        let mut ambiguous = 0;
        let mut files = HashSet::new();
        let mut unverifiable = 0;
        for candidate in candidates {
            match parse_invocation_arities(candidate.text, method_name) {
                Some(invocations) => {
                    if !invocations.is_empty() {
                        files.insert(candidate.file);
                    }
                    ambiguous += invocations.len();
                }
                None => unverifiable += 1,
            }
        }
        return CallerVerificationSummary {
            target_qualified_name: Some(selector.qualified_name.to_string()),
            target_explicit_arity: None,
            raw_candidates: ambiguous,
            verified: 0,
            rejected_arity: 0,
            ambiguous,
            unverifiable,
            verified_caller_files: 0,
            compatible_caller_files: files.len(),
            resolution: if unverifiable > 0 {
                CallerVerificationStatus::Unverifiable
            } else {
                CallerVerificationStatus::Ambiguous
            },
        };
    }

    let target = selected[0].arity;
    let same_shape_ambiguity = selected.len() > 1
        || declarations
            .iter()
            .filter(|decl| arities_overlap(target, decl.arity))
            .count()
            > 1;
    let mut summary = CallerVerificationSummary {
        target_qualified_name: Some(selector.qualified_name.to_string()),
        target_explicit_arity: target.exact_explicit(),
        raw_candidates: 0,
        verified: 0,
        rejected_arity: 0,
        ambiguous: 0,
        unverifiable: 0,
        verified_caller_files: 0,
        compatible_caller_files: 0,
        resolution: CallerVerificationStatus::Unverifiable,
    };
    let mut verified_files = HashSet::new();
    let mut compatible_files = HashSet::new();

    for candidate in candidates {
        let invocations = match parse_invocation_arities(candidate.text, method_name) {
            Some(found) => found,
            None => {
                summary.unverifiable += 1;
                continue;
            }
        };
        for observed in invocations {
            summary.raw_candidates += 1;
            if !target.accepts(observed) {
                summary.rejected_arity += 1;
            } else if target.flexible || same_shape_ambiguity {
                summary.ambiguous += 1;
                compatible_files.insert(candidate.file);
            } else {
                summary.verified += 1;
                compatible_files.insert(candidate.file);
                verified_files.insert(candidate.file);
            }
        }
    }
    summary.verified_caller_files = verified_files.len();
    summary.compatible_caller_files = compatible_files.len();
    summary.resolution = if summary.unverifiable > 0 {
        CallerVerificationStatus::Unverifiable
    } else if summary.ambiguous > 0 {
        CallerVerificationStatus::Ambiguous
    } else if summary.verified > 0 {
        CallerVerificationStatus::VerifiedCompatible
    } else if summary.rejected_arity > 0 {
        CallerVerificationStatus::RejectedArityMismatch
    } else {
        CallerVerificationStatus::Unverifiable
    };
    summary
}

pub(crate) fn annotate_caller_evidence(
    payload: &mut Value,
    surface: &str,
    summary: &CallerVerificationSummary,
) {
    let mut metadata = serde_json::to_value(summary).unwrap_or(Value::Null);
    if let Some(object) = metadata.as_object_mut() {
        object.insert("surface".into(), Value::String(surface.to_string()));
        object.insert("evidence".into(), Value::String("arity".into()));
        object.insert(
            "cbm_relationships".into(),
            Value::String("raw_candidates_only".into()),
        );
    }
    if let Some(object) = payload.as_object_mut() {
        object.insert("clean_ctx_caller_verification".into(), metadata);
    }
}

fn parse_tree(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    let language = crate::compression::language::safe_csharp_language()?;
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    (!tree.root_node().has_error()).then_some(tree)
}

fn parse_declarations(
    source: &str,
    method_name: &str,
    type_name: Option<&str>,
) -> Option<Vec<Declaration>> {
    let tree = parse_tree(source)?;
    let mut nodes = Vec::new();
    collect_kind(tree.root_node(), "method_declaration", &mut nodes);
    let declarations = nodes
        .into_iter()
        .filter(|node| node_text(node.child_by_field_name("name"), source) == Some(method_name))
        .filter(|node| {
            type_name.is_none_or(|name| enclosing_type_name(*node, source) == Some(name))
        })
        .filter_map(|node| declaration_arity(node, source).map(|arity| Declaration { arity }))
        .collect();
    Some(declarations)
}

fn parse_invocation_arities(source: &str, method_name: &str) -> Option<Vec<usize>> {
    let tree = parse_tree(source)?;
    let mut nodes = Vec::new();
    collect_kind(tree.root_node(), "invocation_expression", &mut nodes);
    Some(
        nodes
            .into_iter()
            .filter(|node| invocation_name(*node, source) == Some(method_name))
            .filter_map(|node| node.child_by_field_name("arguments"))
            .map(|arguments| arguments.named_child_count())
            .collect(),
    )
}

fn declaration_arity(node: Node<'_>, source: &str) -> Option<ArityShape> {
    let parameters = node.child_by_field_name("parameters")?;
    let mut cursor = parameters.walk();
    let parameter_nodes: Vec<Node<'_>> = parameters
        .named_children(&mut cursor)
        .filter(|parameter| parameter.kind() == "parameter")
        .collect();
    let parameter_list_text = parameters.utf8_text(source.as_bytes()).ok()?;
    let has_params = has_word(parameter_list_text, "params");
    let declared = parameter_nodes.len() + usize::from(has_params);
    let texts: Vec<&str> = parameter_nodes
        .iter()
        .filter_map(|parameter| parameter.utf8_text(source.as_bytes()).ok())
        .collect();
    if texts.len() != parameter_nodes.len() {
        return None;
    }
    let extension_receiver = texts.first().is_some_and(|text| has_word(text, "this"));
    let receiver_adjustment = usize::from(extension_receiver);
    let optional = texts.iter().filter(|text| text.contains('=')).count();
    let minimum_explicit = declared
        .saturating_sub(receiver_adjustment)
        .saturating_sub(optional)
        .saturating_sub(usize::from(has_params));
    let maximum_explicit = (!has_params).then_some(declared.saturating_sub(receiver_adjustment));
    Some(ArityShape {
        declared,
        minimum_explicit,
        maximum_explicit,
        flexible: optional > 0 || has_params,
    })
}

fn invocation_name<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    let function = node.child_by_field_name("function")?;
    if function.kind() == "member_access_expression" {
        node_text(function.child_by_field_name("name"), source)
    } else {
        let text = function.utf8_text(source.as_bytes()).ok()?;
        Some(text.rsplit('.').next().unwrap_or(text))
    }
}

fn enclosing_type_name<'a>(mut node: Node<'_>, source: &'a str) -> Option<&'a str> {
    while let Some(parent) = node.parent() {
        if matches!(
            parent.kind(),
            "class_declaration" | "struct_declaration" | "record_declaration"
        ) {
            return node_text(parent.child_by_field_name("name"), source);
        }
        node = parent;
    }
    None
}

fn collect_kind<'tree>(node: Node<'tree>, kind: &str, output: &mut Vec<Node<'tree>>) {
    if node.kind() == kind {
        output.push(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_kind(child, kind, output);
    }
}

fn node_text<'a>(node: Option<Node<'_>>, source: &'a str) -> Option<&'a str> {
    node?.utf8_text(source.as_bytes()).ok()
}

fn has_word(text: &str, needle: &str) -> bool {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .any(|word| word == needle)
}

fn arities_overlap(left: ArityShape, right: ArityShape) -> bool {
    let lower = left.minimum_explicit.max(right.minimum_explicit);
    match (left.maximum_explicit, right.maximum_explicit) {
        (Some(left_max), Some(right_max)) => lower <= left_max.min(right_max),
        (Some(left_max), None) => lower <= left_max,
        (None, Some(right_max)) => lower <= right_max,
        (None, None) => true,
    }
}

#[cfg(test)]
#[path = "../tests/cbm/caller_verify.rs"]
mod tests;
