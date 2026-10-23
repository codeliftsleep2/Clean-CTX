//! Bounded, syntax-owned NgModule metadata composition for Issue #139.
use crate::angular_meta::decorators::ClassKind;
use crate::layers::meta::semantic::{
    EntityRef, SemanticDependency, SemanticEdge, SemanticRelation,
};
use std::path::Path;

#[cfg(feature = "typescript")]
mod canonical;
#[cfg(feature = "typescript")]
mod standalone;
#[cfg(feature = "typescript")]
pub(crate) const DECLARATION_FIELDS: &[&str] = &[
    "declarations",
    "imports",
    "exports",
    "standalone",
    "providers",
    "provide",
];
#[cfg(feature = "typescript")]
pub(crate) fn project_authoritative(
    authority: &crate::ir::typescript_resolution::TypeScriptAuthority,
    start: usize,
    end: usize,
) -> ModuleProjection {
    canonical::project(authority, start, end)
}

mod sources;
use sources::Contributors;

#[derive(Debug, Default)]
pub(crate) struct ModuleProjection {
    pub edges: Vec<SemanticEdge>,
    pub unresolved_spreads: Vec<String>,
    pub dependencies: Vec<SemanticDependency>,
}

pub(crate) fn project(
    owner: &str,
    raw_class: &str,
    source: &str,
    declaring_path: Option<&Path>,
    declaration_sources: &[String],
) -> ModuleProjection {
    let Some((_, ClassKind::Module, _, _, _)) =
        crate::angular_meta::decorators::extract_graph_entries(raw_class)
    else {
        return ModuleProjection::default();
    };
    let (declarations, imports, exports) =
        crate::angular_meta::decorators::extract_module_declarations(raw_class);
    let mut result = ModuleProjection::default();
    let mut contributors = Contributors::default();
    for (relation, values) in [
        (SemanticRelation::DeclaresInModule, declarations),
        (SemanticRelation::ImportsModule, imports),
        (SemanticRelation::ExportsFromModule, exports),
    ] {
        for value in compose(
            values,
            source,
            declaring_path,
            &mut result.unresolved_spreads,
            &mut contributors,
        ) {
            let Some((name, kind)) = resolve_leaf(
                &value,
                source,
                declaring_path,
                declaration_sources,
                &mut contributors,
            ) else {
                continue;
            };
            result.edges.push(SemanticEdge {
                relation,
                subject: EntityRef::new("angular", "Module", owner),
                object: EntityRef::new("angular", kind, name),
                layer: "angular",
                call_evidence: None,
            });
        }
    }
    result.dependencies = contributors.dependencies();
    result
}

pub(crate) fn unresolved_spreads(
    class_captures: &[String],
    source: &str,
    declaring_path: Option<&Path>,
) -> Vec<String> {
    let mut unresolved = Vec::new();
    for raw_class in class_captures {
        let Some((owner, ClassKind::Module, _, _, _)) =
            crate::angular_meta::decorators::extract_graph_entries(raw_class)
        else {
            continue;
        };
        unresolved.extend(
            project(&owner, raw_class, source, declaring_path, class_captures).unresolved_spreads,
        );
    }
    unresolved.sort();
    unresolved.dedup();
    unresolved
}

fn compose(
    values: Vec<String>,
    source: &str,
    declaring_path: Option<&Path>,
    unresolved: &mut Vec<String>,
    contributors: &mut Contributors,
) -> Vec<String> {
    let mut leaves = Vec::new();
    for value in values {
        let Some(spread) = value.strip_prefix("...").map(str::trim) else {
            leaves.push(value);
            continue;
        };
        if let Some(items) = const_array(source, spread) {
            leaves.extend(items);
            continue;
        }
        let Some((module, imported)) = import_binding(source, spread) else {
            unresolved.push(spread.to_string());
            continue;
        };
        let Some(path) = declaring_path.and_then(|path| contributors.resolve_module(path, &module))
        else {
            unresolved.push(spread.to_string());
            continue;
        };
        let Some(imported_source) = contributors.read(&path) else {
            unresolved.push(spread.to_string());
            continue;
        };
        if let Some(items) = const_array(&imported_source, &imported) {
            for item in items {
                if item.trim_start().starts_with("...") {
                    unresolved.push(item.trim_start_matches("...").trim().to_string());
                } else {
                    leaves.push(BoundLeaf::encode(&item, &path));
                }
            }
        } else {
            unresolved.push(spread.to_string());
        }
    }
    leaves
}

struct BoundLeaf;

impl BoundLeaf {
    fn encode(name: &str, path: &Path) -> String {
        format!("{}\u{1f}{}", path.to_string_lossy(), name)
    }

    fn decode(value: &str) -> Option<(&str, &str)> {
        value.split_once('\u{1f}')
    }
}

fn resolve_leaf(
    value: &str,
    source: &str,
    declaring_path: Option<&Path>,
    declaration_sources: &[String],
    contributors: &mut Contributors,
) -> Option<(String, &'static str)> {
    if let Some((path, name)) = BoundLeaf::decode(value) {
        let array_path = Path::new(path);
        let array_source = contributors.read(array_path)?;
        if let Some(kind) = declaration_kind(&array_source, name) {
            return Some((name.to_string(), kind));
        }
        let (module, imported) = import_binding(&array_source, name)?;
        let declaration_path = contributors.resolve_module(array_path, &module)?;
        let declaration_source = contributors.read(&declaration_path)?;
        return declaration_kind(&declaration_source, &imported)
            .map(|kind| (name.to_string(), kind));
    }

    let value = value.trim();
    if let Some(base) = value.split_once('.').map(|(base, _)| base.trim())
        && value.contains('(')
    {
        return framework_module(source, base).then(|| (base.to_string(), "Module"));
    }
    if let Some(kind) = declaration_kind(source, value) {
        return Some((value.to_string(), kind));
    }
    let (module, imported) = import_binding(source, value)?;
    if module == "@angular/router" && imported == "RouterModule" {
        return Some((value.to_string(), "Module"));
    }
    if let Some(declaring_path) = declaring_path {
        let path = contributors.resolve_module(declaring_path, &module)?;
        let imported_source = contributors.read(&path)?;
        return declaration_kind(&imported_source, &imported).map(|kind| (value.to_string(), kind));
    }
    declaration_kind_from_sources(declaration_sources, &imported)
        .map(|kind| (value.to_string(), kind))
}

fn declaration_kind_from_sources(sources: &[String], name: &str) -> Option<&'static str> {
    let mut matches = sources
        .iter()
        .filter_map(|source| declaration_kind(source, name));
    let kind = matches.next()?;
    matches.next().is_none().then_some(kind)
}

fn framework_module(source: &str, name: &str) -> bool {
    import_binding(source, name)
        .is_some_and(|(module, imported)| module == "@angular/router" && imported == "RouterModule")
}

fn const_array(source: &str, name: &str) -> Option<Vec<String>> {
    let tree = parse(source)?;
    let node = find_named_declarator(tree.root_node(), source.as_bytes(), name)?;
    let declaration = node.parent()?;
    if declaration.kind() != "lexical_declaration"
        || !declaration
            .utf8_text(source.as_bytes())
            .ok()?
            .trim_start()
            .starts_with("const ")
    {
        return None;
    }
    let value = node.child_by_field_name("value")?;
    (value.kind() == "array").then(|| {
        crate::meta_util::split_top_level(
            value
                .utf8_text(source.as_bytes())
                .unwrap_or_default()
                .trim_start_matches('[')
                .trim_end_matches(']'),
            ',',
        )
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
    })
}

fn find_named_declarator<'tree>(
    node: tree_sitter::Node<'tree>,
    source: &[u8],
    name: &str,
) -> Option<tree_sitter::Node<'tree>> {
    if node.kind() == "variable_declarator"
        && node.child_by_field_name("name")?.utf8_text(source).ok()? == name
    {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find_map(|child| find_named_declarator(child, source, name))
}

pub(crate) fn declaration_kind(source: &str, name: &str) -> Option<&'static str> {
    let tree = parse(source)?;
    find_declaration_kind(tree.root_node(), source.as_bytes(), name)
}

fn find_declaration_kind(
    node: tree_sitter::Node<'_>,
    source: &[u8],
    name: &str,
) -> Option<&'static str> {
    if matches!(
        node.kind(),
        "class_declaration" | "abstract_class_declaration"
    ) && node.child_by_field_name("name")?.utf8_text(source).ok()? == name
    {
        let node_text = node.utf8_text(source).ok()?;
        let class_offset = node_text.find("class ")?;
        let class_start = node.start_byte() + class_offset;
        let declaration_start = crate::meta_util::find_class_source_start(
            std::str::from_utf8(source).ok()?,
            class_start,
        );
        let raw_declaration =
            std::str::from_utf8(source.get(declaration_start..node.end_byte())?).ok()?;
        let (proven_name, kind, _, _, _) =
            crate::angular_meta::decorators::extract_graph_entries(raw_declaration)?;
        if proven_name != name {
            return None;
        }
        return Some(match kind {
            ClassKind::Component => "Component",
            ClassKind::Directive => "Directive",
            ClassKind::Pipe => "Pipe",
            ClassKind::Module => "Module",
            ClassKind::Service => return None,
        });
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find_map(|child| find_declaration_kind(child, source, name))
}

pub(crate) fn has_routes_const(source: &str, name: &str) -> bool {
    let Some(tree) = parse(source) else {
        return false;
    };
    let Some(node) = find_named_declarator(tree.root_node(), source.as_bytes(), name) else {
        return false;
    };
    if node
        .child_by_field_name("value")
        .is_none_or(|value| value.kind() != "array")
        || !import_binding(source, "Routes")
            .is_some_and(|(module, imported)| module == "@angular/router" && imported == "Routes")
    {
        return false;
    }
    let mut ancestor = node.parent();
    while let Some(node) = ancestor {
        if node.kind() == "export_statement" {
            return true;
        }
        if node.kind() == "program" {
            break;
        }
        ancestor = node.parent();
    }
    false
}
fn import_binding(source: &str, local: &str) -> Option<(String, String)> {
    let tree = parse(source)?;
    let mut matches = Vec::new();
    collect_import_bindings(tree.root_node(), source.as_bytes(), local, &mut matches);
    let [binding] = matches.as_slice() else {
        return None;
    };
    Some(binding.clone())
}

fn collect_import_bindings(
    node: tree_sitter::Node<'_>,
    source: &[u8],
    local: &str,
    output: &mut Vec<(String, String)>,
) {
    if node.kind() == "import_statement" {
        if node
            .children(&mut node.walk())
            .any(|child| child.kind() == "type")
        {
            return;
        }
        let Some(module) = node
            .child_by_field_name("source")
            .and_then(|string| string.named_child(0))
            .and_then(|fragment| fragment.utf8_text(source).ok())
        else {
            return;
        };
        collect_import_specifiers(node, source, local, module, output);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_bindings(child, source, local, output);
    }
}

fn collect_import_specifiers(
    node: tree_sitter::Node<'_>,
    source: &[u8],
    local: &str,
    module: &str,
    output: &mut Vec<(String, String)>,
) {
    if node.kind() == "import_specifier" {
        if node
            .children(&mut node.walk())
            .any(|child| child.kind() == "type")
        {
            return;
        }
        let Some(imported) = node
            .child_by_field_name("name")
            .and_then(|name| name.utf8_text(source).ok())
        else {
            return;
        };
        let local_name = node
            .child_by_field_name("alias")
            .and_then(|alias| alias.utf8_text(source).ok())
            .unwrap_or(imported);
        if local_name == local {
            output.push((module.to_string(), imported.to_string()));
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_specifiers(child, source, local, module, output);
    }
}
fn parse(source: &str) -> Option<tree_sitter::Tree> {
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    (!tree.root_node().has_error()).then_some(tree)
}
#[cfg(test)]
#[path = "../tests/angular_meta/module_metadata_issue_139.rs"]
mod module_metadata_issue_139;

#[cfg(test)]
#[path = "../tests/angular_meta/module_metadata_issue_157.rs"]
mod module_metadata_issue_157;
