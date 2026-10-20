//! Structured evidence carriers for object-member actions and switch reducers.
//! Carriers are intercepted by `WorkspaceIndex`; none is query-visible until
//! the independent framework authorities have been reconciled.

use crate::ir::invocation_evidence::SourceSpan;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::path::{Path, PathBuf};

pub(crate) const DISPATCH_LAYER: &str = "ngrx-object-dispatch-evidence";
pub(crate) const REGISTRATION_LAYER: &str = "ngrx-reducer-registration-evidence";
pub(crate) const SWITCH_LAYER: &str = "ngrx-object-switch-evidence";

pub(crate) fn evidence(source_id: &str, source: &str) -> Vec<SemanticEdge> {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return Vec::new();
    };
    let mut output = Vec::new();
    walk(tree.root_node(), source_id, source, &mut output);
    output
}

/// Current direct-relative imports are semantic dependencies for the
/// cross-file action/reducer evidence carried by this source file.
pub(crate) fn dependencies(
    source_id: &str,
    source: &str,
) -> Vec<crate::layers::meta::semantic::SemanticDependency> {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    collect_import_paths(tree.root_node(), source_id, source.as_bytes(), &mut paths);
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(crate::layers::meta::semantic::SemanticDependency::current)
        .collect()
}

fn collect_import_paths(
    node: tree_sitter::Node<'_>,
    source_id: &str,
    source: &[u8],
    output: &mut Vec<String>,
) {
    if node.kind() == "import_statement" {
        if let Some(module) = node
            .child_by_field_name("source")
            .and_then(|string| string.named_child(0))
            .and_then(|fragment| fragment.utf8_text(source).ok())
            .filter(|module| module.starts_with("./") || module.starts_with("../"))
            && let Some(path) = resolve_module_path(source_id, module)
        {
            output.push(path);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_paths(child, source_id, source, output);
    }
}

fn resolve_module_path(source_id: &str, module: &str) -> Option<String> {
    let parent = Path::new(source_id).parent()?;
    let target = parent.join(module);
    let target = if target.extension().and_then(|suffix| suffix.to_str()) == Some("ts") {
        target
    } else {
        PathBuf::from(format!("{}.ts", target.to_string_lossy()))
    };
    target
        .canonicalize()
        .ok()
        .map(|path| crate::dictionary::path::canonical_identity_key(&path.to_string_lossy()))
}

pub(crate) fn is_evidence_layer(layer: &str) -> bool {
    matches!(layer, DISPATCH_LAYER | REGISTRATION_LAYER | SWITCH_LAYER)
}

/// Search terms sufficient to acquire the bounded evidence neighborhood from
/// the existing provider-neutral hydration path. Imported aliases normalize
/// to their exported object name before becoming discovery terms.
pub(crate) fn hydration_terms(source_id: &str, source: &str) -> Vec<String> {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return Vec::new();
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser
        .parse(source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return Vec::new();
    };
    let mut terms = Vec::new();
    collect_hydration_terms(tree.root_node(), source_id, source, &mut terms);
    terms.push("StoreModule".to_owned());
    terms.sort();
    terms.dedup();
    terms
}

fn collect_hydration_terms(
    node: tree_sitter::Node<'_>,
    source_id: &str,
    source: &str,
    output: &mut Vec<String>,
) {
    if node.kind() == "variable_declarator"
        && let Some(name) = node
            .child_by_field_name("name")
            .filter(|name| name.kind() == "identifier")
        && node
            .child_by_field_name("value")
            .is_some_and(|value| value.kind() == "object")
        && node
            .parent()
            .and_then(|parent| parent.parent())
            .is_some_and(|parent| parent.kind() == "export_statement")
        && let Ok(name) = name.utf8_text(source.as_bytes())
    {
        output.push(name.to_owned());
    } else if node.kind() == "member_expression"
        && let Ok(span) = SourceSpan::new(node.start_byte(), node.end_byte())
        && let Some(binding) =
            crate::ir::typescript_value_import_binding::bind_direct_named_object_member_import(
                source_id, source, span,
            )
    {
        output.push(binding.imported_object);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_hydration_terms(child, source_id, source, output);
    }
}

fn walk(node: tree_sitter::Node<'_>, source_id: &str, source: &str, out: &mut Vec<SemanticEdge>) {
    if node.kind() == "call_expression" {
        dispatch(node, source_id, source)
            .into_iter()
            .for_each(|edge| out.push(edge));
        registration(node, source_id, source)
            .into_iter()
            .for_each(|edge| out.push(edge));
    } else if node.kind() == "function_declaration" {
        switches(node, source_id, source, out);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        walk(child, source_id, source, out);
    }
}

fn dispatch(node: tree_sitter::Node<'_>, source_id: &str, source: &str) -> Option<SemanticEdge> {
    let function = node.child_by_field_name("function")?;
    let receiver = function
        .child_by_field_name("object")
        .filter(|_| function.kind() == "member_expression")?;
    if function
        .child_by_field_name("property")?
        .utf8_text(source.as_bytes())
        .ok()?
        != "dispatch"
    {
        return None;
    }
    let receiver_span = SourceSpan::new(receiver.start_byte(), receiver.end_byte()).ok()?;
    let property = crate::ir::typescript_this_property_binding::bind_this_property(
        source_id,
        source,
        receiver_span,
    )?;
    let type_span = property.type_reference_span()?;
    let store = crate::ir::typescript_type_import_binding::bind_type_reference_to_named_import(
        source_id, source, type_span,
    )?;
    if store.imported_symbol_name() != "Store" || store.module_specifier() != "@ngrx/store" {
        return None;
    }
    let arguments = node.child_by_field_name("arguments")?;
    let argument = arguments.named_child(0)?.filter_kind("object")?;
    if arguments.named_child_count() != 1 {
        return None;
    }
    let member =
        pair_value(argument, "type", source.as_bytes())?.filter_kind("member_expression")?;
    let resolved = resolve_member(source_id, source, member)?;
    Some(carrier(
        SemanticRelation::Dispatches,
        EntityRef::new("typescript", "DispatchSite", owner_name(node, source)?),
        member_entity(&resolved),
        DISPATCH_LAYER,
    ))
}

fn registration(
    node: tree_sitter::Node<'_>,
    source_id: &str,
    source: &str,
) -> Option<SemanticEdge> {
    let function = node.child_by_field_name("function")?;
    if function.kind() != "member_expression"
        || function
            .child_by_field_name("property")?
            .utf8_text(source.as_bytes())
            .ok()?
            != "forFeature"
    {
        return None;
    }
    let receiver = function
        .child_by_field_name("object")?
        .filter_kind("identifier")?;
    let receiver_binding =
        crate::ir::typescript_value_import_binding::bind_named_value_import_allow_alias(
            source_id,
            source,
            SourceSpan::new(receiver.start_byte(), receiver.end_byte()).ok()?,
        )?;
    if receiver_binding.imported_name != "StoreModule"
        || receiver_binding.module_specifier != "@ngrx/store"
    {
        return None;
    }
    let arguments = node.child_by_field_name("arguments")?;
    let reducer = arguments.named_child(1)?.filter_kind("identifier")?;
    let resolved = resolve_value(source_id, source, reducer)?;
    Some(carrier(
        SemanticRelation::RegistersReducer,
        EntityRef::new("ngrx", "StoreRegistration", "StoreModule.forFeature"),
        resolved_entity("ngrx", "Reducer", &resolved),
        REGISTRATION_LAYER,
    ))
}

fn switches(
    function: tree_sitter::Node<'_>,
    source_id: &str,
    source: &str,
    out: &mut Vec<SemanticEdge>,
) {
    let Some(name) = function
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
    else {
        return;
    };
    let Some(parameters) = function.child_by_field_name("parameters") else {
        return;
    };
    let parameters: Vec<_> = parameters.named_children(&mut parameters.walk()).collect();
    let Some(action) = parameters
        .last()
        .and_then(|parameter| {
            parameter
                .child_by_field_name("pattern")
                .or(Some(*parameter))
        })
        .filter(|node| node.kind() == "identifier")
    else {
        return;
    };
    collect_switches(function, action, name, source_id, source, out);
}

fn collect_switches(
    node: tree_sitter::Node<'_>,
    action: tree_sitter::Node<'_>,
    reducer: &str,
    source_id: &str,
    source: &str,
    out: &mut Vec<SemanticEdge>,
) {
    if node.kind() == "switch_statement"
        && let Some(value) = node.child_by_field_name("value")
        && discriminant_binds(value, action, source_id, source)
    {
        let Some(body) = node
            .named_children(&mut node.walk())
            .find(|child| child.kind() == "switch_body")
        else {
            return;
        };
        let mut cursor = body.walk();
        for case in body
            .named_children(&mut cursor)
            .filter(|child| child.kind() == "switch_case")
        {
            let Some(value) = case
                .child_by_field_name("value")
                .and_then(|v| v.filter_kind("member_expression"))
            else {
                continue;
            };
            let Some(member) = resolve_member(source_id, source, value) else {
                continue;
            };
            out.push(carrier(
                SemanticRelation::TriggersReducer,
                member_entity(&member),
                EntityRef::new("ngrx", "Reducer", reducer).with_file(source_id.to_owned()),
                SWITCH_LAYER,
            ));
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_switches(child, action, reducer, source_id, source, out);
    }
}

fn discriminant_binds(
    value: tree_sitter::Node<'_>,
    action: tree_sitter::Node<'_>,
    source_id: &str,
    source: &str,
) -> bool {
    let value = if value.kind() == "parenthesized_expression" {
        value.named_child(0).unwrap_or(value)
    } else {
        value
    };
    let Some(object) = value
        .child_by_field_name("object")
        .filter(|_| value.kind() == "member_expression")
    else {
        return false;
    };
    if value
        .child_by_field_name("property")
        .and_then(|p| p.utf8_text(source.as_bytes()).ok())
        != Some("type")
    {
        return false;
    }
    let Ok(span) = SourceSpan::new(object.start_byte(), object.end_byte()) else {
        return false;
    };
    crate::ir::typescript_identifier_binding::bind_direct_value_identifier(source_id, source, span)
        .is_some_and(|binding| {
            binding.declaration_span()
                == SourceSpan::new(action.start_byte(), action.end_byte())
                    .expect("ordered parser span")
        })
}

struct Resolved {
    owner: String,
    member: Option<String>,
    module: Option<String>,
}

fn resolve_member(source_id: &str, source: &str, node: tree_sitter::Node<'_>) -> Option<Resolved> {
    let span = SourceSpan::new(node.start_byte(), node.end_byte()).ok()?;
    if let Some(binding) =
        crate::ir::typescript_value_import_binding::bind_direct_named_object_member_import(
            source_id, source, span,
        )
    {
        let written_object = node
            .child_by_field_name("object")?
            .utf8_text(source.as_bytes())
            .ok()?;
        if binding.source_id != source_id
            || binding.source_hash != crate::cache::content_hash(source.as_bytes())
            || binding.reference_span != span
            || binding.local_object != written_object
        {
            return None;
        }
        return Some(Resolved {
            owner: binding.imported_object,
            member: Some(binding.member_name),
            module: Some(binding.module_specifier),
        });
    }
    let owner = node
        .child_by_field_name("object")?
        .filter_kind("identifier")?;
    let member = node
        .child_by_field_name("property")?
        .filter_kind("property_identifier")?;
    let owner_span = SourceSpan::new(owner.start_byte(), owner.end_byte()).ok()?;
    let binding = crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
        source_id, source, owner_span,
    )?;
    if !is_exported_object_declaration(source, binding.declaration_span()) {
        return None;
    }
    Some(Resolved {
        owner: owner.utf8_text(source.as_bytes()).ok()?.to_owned(),
        member: Some(member.utf8_text(source.as_bytes()).ok()?.to_owned()),
        module: Some(source_id.to_owned()),
    })
}

fn resolve_value(source_id: &str, source: &str, node: tree_sitter::Node<'_>) -> Option<Resolved> {
    let span = SourceSpan::new(node.start_byte(), node.end_byte()).ok()?;
    if let Some(binding) =
        crate::ir::typescript_value_import_binding::bind_direct_named_value_import_allow_alias(
            source_id, source, span,
        )
    {
        return Some(Resolved {
            owner: binding.imported_name,
            member: None,
            module: Some(binding.module_specifier),
        });
    }
    let _binding = crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
        source_id, source, span,
    )?;
    Some(Resolved {
        owner: node.utf8_text(source.as_bytes()).ok()?.to_owned(),
        member: None,
        module: Some(source_id.to_owned()),
    })
}

fn member_entity(value: &Resolved) -> EntityRef {
    resolved_entity("ngrx", "Action", value)
}
fn resolved_entity(domain: &'static str, ty: &'static str, value: &Resolved) -> EntityRef {
    let name = value.member.as_ref().map_or_else(
        || value.owner.clone(),
        |member| format!("{}.{}", value.owner, member),
    );
    let mut entity = EntityRef::new(domain, ty, name);
    entity.file.clone_from(&value.module);
    entity
}
fn carrier(
    relation: SemanticRelation,
    subject: EntityRef,
    object: EntityRef,
    layer: &'static str,
) -> SemanticEdge {
    SemanticEdge {
        relation,
        subject,
        object,
        layer,
        call_evidence: None,
    }
}
fn pair_value<'a>(
    object: tree_sitter::Node<'a>,
    key: &str,
    source: &[u8],
) -> Option<tree_sitter::Node<'a>> {
    object
        .named_children(&mut object.walk())
        .filter(|n| n.kind() == "pair")
        .find_map(|pair| {
            (pair
                .child_by_field_name("key")?
                .utf8_text(source)
                .ok()?
                .trim_matches(['\'', '"'])
                == key)
                .then(|| pair.child_by_field_name("value"))
                .flatten()
        })
}
fn owner_name<'a>(mut node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    loop {
        if node.kind() == "class_declaration" {
            return node
                .child_by_field_name("name")?
                .utf8_text(source.as_bytes())
                .ok();
        }
        node = node.parent()?;
    }
}
fn is_exported_object_declaration(source: &str, declaration: SourceSpan) -> bool {
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
    let Some(name) = tree
        .root_node()
        .named_descendant_for_byte_range(declaration.start_byte(), declaration.end_byte())
    else {
        return false;
    };
    let Some(declarator) = name
        .parent()
        .filter(|node| node.kind() == "variable_declarator")
    else {
        return false;
    };
    declarator
        .child_by_field_name("value")
        .is_some_and(|value| value.kind() == "object")
        && declarator
            .parent()
            .and_then(|node| node.parent())
            .is_some_and(|node| node.kind() == "export_statement")
}
trait NodeKindExt<'a> {
    fn filter_kind(self, kind: &str) -> Option<tree_sitter::Node<'a>>;
}
impl<'a> NodeKindExt<'a> for tree_sitter::Node<'a> {
    fn filter_kind(self, kind: &str) -> Option<Self> {
        (self.kind() == kind).then_some(self)
    }
}
