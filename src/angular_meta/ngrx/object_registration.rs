//! Exact NgRx reducer registration evidence.

use crate::ir::invocation_evidence::SourceSpan;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) const MAP_LAYER: &str = "ngrx-reducer-map-registration-evidence";
pub(crate) const MAP_ENTITY_TYPE: &str = "ObjectMap";

pub(crate) fn evidence(source_id: &str, source: &str) -> Vec<SemanticEdge> {
    parse(source).map_or_else(Vec::new, |tree| {
        let mut output = Vec::new();
        walk_calls(tree.root_node(), &mut |call| {
            output.extend(registration(call, source_id, source));
        });
        output
    })
}

pub(crate) fn hydration_terms(source_id: &str, source: &str) -> Vec<String> {
    let mut terms = vec!["StoreModule".to_owned()];
    if let Some(tree) = parse(source) {
        walk_calls(tree.root_node(), &mut |call| {
            if let Some((_, argument)) = registration_call(call, source_id, source)
                && let Some(resolved) = resolve_value(source_id, source, argument)
            {
                terms.push(resolved.name.clone());
                if let Some(path) = resolve_relative_path(source_id, &resolved.module)
                    && let Ok(map_source) = std::fs::read_to_string(path)
                {
                    terms.extend(
                        crate::ir::typescript_object_members::referenced_value_names(&map_source),
                    );
                }
            }
        });
    }
    terms.sort();
    terms.dedup();
    terms
}

pub(crate) fn has_registration(source_id: &str, source: &str) -> bool {
    evidence(source_id, source).into_iter().next().is_some()
}

fn registration(call: tree_sitter::Node<'_>, source_id: &str, source: &str) -> Vec<SemanticEdge> {
    let Some((api, argument)) = registration_call(call, source_id, source) else {
        return Vec::new();
    };
    let Some(resolved) = resolve_value(source_id, source, argument) else {
        return Vec::new();
    };
    let mut kinds = vec![(MAP_ENTITY_TYPE, MAP_LAYER, "typescript")];
    if api == "forFeature" && !resolved_object_declaration(source_id, source, argument) {
        kinds.push(("Reducer", super::object_actions::REGISTRATION_LAYER, "ngrx"));
    }
    kinds
        .into_iter()
        .map(|(entity_type, layer, domain)| SemanticEdge {
            relation: SemanticRelation::RegistersReducer,
            subject: EntityRef::new("ngrx", "StoreRegistration", format!("StoreModule.{api}")),
            object: EntityRef::new(domain, entity_type, resolved.name.clone())
                .with_file(resolved.module.clone()),
            layer,
            call_evidence: None,
        })
        .collect()
}

fn registration_call<'a>(
    call: tree_sitter::Node<'a>,
    source_id: &str,
    source: &str,
) -> Option<(String, tree_sitter::Node<'a>)> {
    let function = call.child_by_field_name("function")?;
    if function.kind() != "member_expression" {
        return None;
    }
    let api = function
        .child_by_field_name("property")?
        .utf8_text(source.as_bytes())
        .ok()?;
    if !matches!(api, "forFeature" | "forRoot") {
        return None;
    }
    let receiver = function.child_by_field_name("object")?;
    let binding = crate::ir::typescript_value_import_binding::bind_named_value_import_allow_alias(
        source_id,
        source,
        SourceSpan::new(receiver.start_byte(), receiver.end_byte()).ok()?,
    )?;
    if binding.imported_name != "StoreModule" || binding.module_specifier != "@ngrx/store" {
        return None;
    }
    let arguments = call.child_by_field_name("arguments")?;
    let argument = arguments.named_child(u32::from(api == "forFeature"))?;
    (argument.kind() == "identifier").then_some((api.to_owned(), argument))
}

struct ResolvedValue {
    name: String,
    module: String,
}

fn resolve_value(
    source_id: &str,
    source: &str,
    node: tree_sitter::Node<'_>,
) -> Option<ResolvedValue> {
    let span = SourceSpan::new(node.start_byte(), node.end_byte()).ok()?;
    if let Some(binding) =
        crate::ir::typescript_value_import_binding::bind_direct_named_value_import_allow_alias(
            source_id, source, span,
        )
    {
        return Some(ResolvedValue {
            name: binding.imported_name,
            module: binding.module_specifier,
        });
    }
    crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
        source_id, source, span,
    )?;
    Some(ResolvedValue {
        name: node.utf8_text(source.as_bytes()).ok()?.to_owned(),
        module: source_id.to_owned(),
    })
}

fn resolved_object_declaration(source_id: &str, source: &str, node: tree_sitter::Node<'_>) -> bool {
    let Ok(span) = SourceSpan::new(node.start_byte(), node.end_byte()) else {
        return false;
    };
    crate::ir::typescript_identifier_binding::bind_direct_value_identifier(source_id, source, span)
        .is_some_and(|binding| {
            super::object_actions::is_object_declaration(source, binding.declaration_span())
        })
}

fn parse(source: &str) -> Option<tree_sitter::Tree> {
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
}

fn walk_calls<'a>(node: tree_sitter::Node<'a>, callback: &mut impl FnMut(tree_sitter::Node<'a>)) {
    if node.kind() == "call_expression" {
        callback(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        walk_calls(child, callback);
    }
}

fn resolve_relative_path(source_id: &str, module: &str) -> Option<std::path::PathBuf> {
    if !(module.starts_with("./") || module.starts_with("../")) {
        return None;
    }
    let target = std::path::Path::new(source_id).parent()?.join(module);
    let target = if target.extension().is_some() {
        target
    } else {
        std::path::PathBuf::from(format!("{}.ts", target.to_string_lossy()))
    };
    target.canonicalize().ok()
}
