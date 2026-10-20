//! Exact same-file authority for legacy Action discriminant constants.

use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_identifier_binding::{
    ValueDeclarationKind, bind_direct_value_identifier,
};
use crate::ir::typescript_type_import_binding::bind_type_reference_to_named_import;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::BTreeMap;

pub(crate) fn has_object_string_actions(source: &str) -> bool {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return false;
    };
    let bytes = source.as_bytes();
    let mut members = std::collections::BTreeSet::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let declaration = if child.kind() == "export_statement" {
            child.child_by_field_name("declaration")
        } else {
            Some(child)
        };
        let Some(declaration) = declaration.filter(|node| node.kind() == "lexical_declaration")
        else {
            continue;
        };
        let mut declaration_cursor = declaration.walk();
        for declarator in declaration.named_children(&mut declaration_cursor) {
            let (Some(name), Some(object)) = (
                declarator.child_by_field_name("name"),
                declarator
                    .child_by_field_name("value")
                    .filter(|node| node.kind() == "object"),
            ) else {
                continue;
            };
            let Ok(owner) = name.utf8_text(bytes) else {
                continue;
            };
            let mut object_cursor = object.walk();
            for property in object.named_children(&mut object_cursor) {
                let key = if property.kind() == "shorthand_property_identifier" {
                    Some(property)
                } else if property.kind() == "pair"
                    && property
                        .child_by_field_name("value")
                        .is_some_and(|value| matches!(value.kind(), "string" | "identifier"))
                {
                    property.child_by_field_name("key")
                } else {
                    None
                };
                if let Some(Ok(key)) = key.map(|key| key.utf8_text(bytes)) {
                    members.insert(format!("{owner}.{key}"));
                }
            }
        }
    }
    !members.is_empty()
}

pub(crate) fn has_switch_action_type_handlers(source: &str) -> bool {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return false;
    };
    contains_object_action_case(tree.root_node(), source.as_bytes())
}

fn contains_object_action_case(node: tree_sitter::Node<'_>, source: &[u8]) -> bool {
    if node.kind() == "switch_statement"
        && node
            .child_by_field_name("value")
            .and_then(|value| value.utf8_text(source).ok())
            .is_some_and(|value| value.trim_matches(['(', ')', ' ']).ends_with(".type"))
    {
        let mut cursor = node.walk();
        if node.named_children(&mut cursor).any(|child| {
            child.kind() == "switch_body" && {
                let mut body_cursor = child.walk();
                child.named_children(&mut body_cursor).any(|case| {
                    case.kind() == "switch_case"
                        && case.child_by_field_name("value").is_some_and(|value| {
                            value.kind() == "identifier"
                                || (value.kind() == "member_expression"
                                    && value.child_by_field_name("object").is_some()
                                    && value.child_by_field_name("property").is_some())
                        })
                })
            }
        }) {
            return true;
        }
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| contains_object_action_case(child, source))
}

#[derive(Debug, Clone)]
pub(crate) struct LegacyDiscriminantEvidence {
    pub name: String,
    pub source_id: String,
    pub source_hash: String,
    pub declaration_span: SourceSpan,
    pub declaration_name_span: SourceSpan,
    pub case_span: SourceSpan,
    pub reducer_name: String,
}

impl LegacyDiscriminantEvidence {
    pub(crate) fn matches_source(&self, source_id: &str, source: &str) -> bool {
        self.source_id == source_id
            && self.source_hash == crate::cache::content_hash(source.as_bytes())
            && self.declaration_span.start_byte() <= self.declaration_name_span.start_byte()
            && self.declaration_name_span.end_byte() <= self.declaration_span.end_byte()
            && source
                .get(self.declaration_name_span.start_byte()..self.declaration_name_span.end_byte())
                == Some(self.name.as_str())
            && source
                .get(self.case_span.start_byte()..self.case_span.end_byte())
                .is_some_and(|text| text.trim_start().starts_with("case "))
    }

    pub(crate) fn declaration_edge(&self) -> SemanticEdge {
        let mut entity = EntityRef::new("ngrx", "Action", &self.name);
        entity.file = Some(self.source_id.clone());
        SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: entity.clone(),
            object: entity,
            layer: "ngrx",
            call_evidence: None,
        }
    }

    pub(crate) fn transition_edge(&self) -> SemanticEdge {
        let mut action = EntityRef::new("ngrx", "Action", &self.name);
        action.file = Some(self.source_id.clone());
        let mut reducer = EntityRef::new("ngrx", "Reducer", &self.reducer_name);
        reducer.file = Some(self.source_id.clone());
        SemanticEdge {
            relation: SemanticRelation::TriggersReducer,
            subject: action,
            object: reducer,
            layer: "ngrx",
            call_evidence: None,
        }
    }
}

fn span(capture: &CapturedNode) -> SourceSpan {
    SourceSpan::new(capture.start_byte, capture.end_byte).expect("ordered parser span")
}

fn is_string_literal(raw: &str) -> bool {
    raw.len() >= 2
        && ((raw.starts_with('\'') && raw.ends_with('\''))
            || (raw.starts_with('"') && raw.ends_with('"')))
}

/// Admit a top-level string constant only when an exact case reference binds
/// to it inside an exact NgRx `Action`-typed `switch(parameter.type)` reducer.
pub(crate) fn collect(
    source: &str,
    canonical_path: Option<&str>,
    captures: &[CapturedNode],
) -> Vec<LegacyDiscriminantEvidence> {
    if captures
        .iter()
        .any(|capture| capture.name == "ngrx.transition.unsupported_loop")
    {
        return Vec::new();
    }
    let Some(source_id) = canonical_path.filter(|path| !path.is_empty()) else {
        return Vec::new();
    };
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    for capture in captures {
        matches
            .entry(capture.match_index)
            .or_default()
            .insert(&capture.name, capture);
    }
    let constants: BTreeMap<_, _> = matches
        .values()
        .filter_map(|matched| {
            let name = matched.get("ngrx.type.const_name")?;
            let declaration = matched.get("ngrx.type.const")?;
            let value = matched.get("ngrx.type.const_value")?;
            is_string_literal(&value.raw_text)
                .then_some((span(name), (name.raw_text.as_str(), span(declaration))))
        })
        .collect();
    let owners: BTreeMap<_, _> = matches
        .values()
        .filter_map(|matched| {
            let declaration = matched.get("invoke.declaration.callable")?;
            let name = matched.get("invoke.declaration_name")?;
            Some((span(declaration), name.raw_text.as_str()))
        })
        .collect();
    let source_hash = crate::cache::content_hash(source.as_bytes());
    let mut result = BTreeMap::new();
    for matched in matches.values() {
        let (
            Some(owner),
            Some(parameter),
            Some(parameter_type),
            Some(receiver),
            Some(property),
            Some(reference),
            Some(case),
        ) = (
            matched.get("ngrx.transition.owner"),
            matched.get("ngrx.transition.parameter"),
            matched.get("ngrx.transition.parameter_type"),
            matched.get("ngrx.transition.receiver"),
            matched.get("ngrx.transition.property"),
            matched.get("ngrx.transition.reference"),
            matched.get("ngrx.transition.case"),
        )
        else {
            continue;
        };
        if property.raw_text != "type" {
            continue;
        }
        let Some(receiver_binding) =
            bind_direct_value_identifier(source_id, source, span(receiver))
        else {
            continue;
        };
        if receiver_binding.declaration_kind() != ValueDeclarationKind::Parameter
            || receiver_binding.declaration_span() != span(parameter)
        {
            continue;
        }
        let Some(type_binding) =
            bind_type_reference_to_named_import(source_id, source, span(parameter_type))
        else {
            continue;
        };
        if type_binding.imported_symbol_name() != "Action"
            || type_binding.module_specifier() != "@ngrx/store"
        {
            continue;
        }
        let Some(case_binding) = bind_direct_value_identifier(source_id, source, span(reference))
        else {
            continue;
        };
        if case_binding.declaration_kind() != ValueDeclarationKind::LexicalConst {
            continue;
        }
        let declaration_name_span = case_binding.declaration_span();
        let Some((name, declaration_span)) = constants.get(&declaration_name_span) else {
            continue;
        };
        let Some(reducer_name) = owners.get(&span(owner)) else {
            continue;
        };
        result.insert(
            span(case),
            LegacyDiscriminantEvidence {
                name: (*name).into(),
                source_id: source_id.into(),
                source_hash: source_hash.clone(),
                declaration_span: *declaration_span,
                declaration_name_span,
                case_span: span(case),
                reducer_name: (*reducer_name).into(),
            },
        );
    }
    result.into_values().collect()
}
