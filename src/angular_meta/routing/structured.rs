//! Parser-owned Angular route-array extraction for Issue #139.
use super::{LazyRouteTarget, RouteDecl};
use std::path::{Path, PathBuf};

pub(super) fn extract(
    source: &str,
    declaring_path: Option<&Path>,
) -> (
    Vec<RouteDecl>,
    Vec<crate::layers::meta::semantic::SemanticDependency>,
) {
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return (Vec::new(), Vec::new());
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return (Vec::new(), Vec::new());
    }
    let Some(tree) = parser.parse(source, None) else {
        return (Vec::new(), Vec::new());
    };
    if tree.root_node().has_error() {
        return (Vec::new(), Vec::new());
    }

    let mut roots = Vec::new();
    collect_route_arrays(tree.root_node(), source.as_bytes(), &mut roots);
    roots.sort_by_key(|node| node.start_byte());
    roots.dedup_by_key(|node| (node.start_byte(), node.end_byte()));

    let mut routes = Vec::new();
    let mut dependencies = Vec::new();
    for array in roots {
        parse_array(
            array,
            "",
            source,
            declaring_path,
            &mut routes,
            &mut dependencies,
        );
    }
    dependencies.sort_by(|left, right| left.file_path.cmp(&right.file_path));
    dependencies.dedup_by(|left, right| left.file_path == right.file_path);
    (routes, dependencies)
}

fn collect_route_arrays<'tree>(
    node: tree_sitter::Node<'tree>,
    source: &[u8],
    output: &mut Vec<tree_sitter::Node<'tree>>,
) {
    if node.kind() == "array" && is_route_root(node, source) {
        output.push(node);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_route_arrays(child, source, output);
    }
}

fn is_route_root(array: tree_sitter::Node<'_>, source: &[u8]) -> bool {
    let mut ancestor = array.parent();
    while let Some(node) = ancestor {
        if matches!(
            node.kind(),
            "lexical_declaration" | "export_statement" | "expression_statement"
        ) {
            let text = node.utf8_text(source).unwrap_or_default();
            return text.contains("Routes")
                || text.contains("RouterModule.forRoot")
                || text.contains("RouterModule.forChild");
        }
        if node.kind() == "pair" {
            return false;
        }
        ancestor = node.parent();
    }
    false
}

fn parse_array(
    array: tree_sitter::Node<'_>,
    parent_path: &str,
    source: &str,
    declaring_path: Option<&Path>,
    output: &mut Vec<RouteDecl>,
    dependencies: &mut Vec<crate::layers::meta::semantic::SemanticDependency>,
) {
    let mut cursor = array.walk();
    for object in array
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "object")
    {
        parse_object(
            object,
            parent_path,
            source,
            declaring_path,
            output,
            dependencies,
        );
    }
}

fn parse_object(
    object: tree_sitter::Node<'_>,
    parent_path: &str,
    source: &str,
    declaring_path: Option<&Path>,
    output: &mut Vec<RouteDecl>,
    dependencies: &mut Vec<crate::layers::meta::semantic::SemanticDependency>,
) {
    let bytes = source.as_bytes();
    let segment = pair_value(object, "path", bytes)
        .and_then(|node| quoted(node, bytes))
        .unwrap_or_default();
    let path = join_path(parent_path, &segment);
    let component = pair_value(object, "component", bytes).and_then(|node| identifier(node, bytes));
    let guards = ["canActivate", "canLoad", "canDeactivate", "canMatch"]
        .into_iter()
        .flat_map(|key| array_identifiers(pair_value(object, key, bytes), bytes))
        .collect();
    let resolvers = resolve_identifiers(pair_value(object, "resolve", bytes), bytes);

    let load_component_evidence =
        pair_value(object, "loadComponent", bytes).and_then(|node| lazy_evidence(node, bytes));
    let load_children_evidence =
        pair_value(object, "loadChildren", bytes).and_then(|node| lazy_evidence(node, bytes));
    let load_component = load_component_evidence
        .as_ref()
        .map(|(module, _)| module.clone());
    let load_children = load_children_evidence
        .as_ref()
        .map(|(module, _)| module.clone());
    let resolved_load_component = load_component_evidence.and_then(|(module, member)| {
        resolve_lazy(declaring_path?, &module, &member, "Component", dependencies)
            .map(|_| LazyRouteTarget::new("Component", member))
    });
    let resolved_load_children = load_children_evidence.and_then(|(module, member)| {
        resolve_lazy(declaring_path?, &module, &member, "Module", dependencies)
            .map(|kind| LazyRouteTarget::new(kind, member))
    });

    output.push(RouteDecl {
        path: path.clone(),
        component,
        load_component,
        load_children,
        resolved_load_component,
        resolved_load_children,
        guards,
        resolvers,
    });

    if let Some(children) = pair_value(object, "children", bytes)
        && children.kind() == "array"
    {
        parse_array(
            children,
            &path,
            source,
            declaring_path,
            output,
            dependencies,
        );
    }
}

fn pair_value<'tree>(
    object: tree_sitter::Node<'tree>,
    key: &str,
    source: &[u8],
) -> Option<tree_sitter::Node<'tree>> {
    let mut cursor = object.walk();
    object
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "pair")
        .find_map(|pair| {
            (pair
                .child_by_field_name("key")?
                .utf8_text(source)
                .ok()?
                .trim_matches(|ch| ch == '\'' || ch == '"')
                == key)
                .then(|| pair.child_by_field_name("value"))
                .flatten()
        })
}

fn quoted(node: tree_sitter::Node<'_>, source: &[u8]) -> Option<String> {
    let text = node.utf8_text(source).ok()?.trim();
    (text.len() >= 2
        && ((text.starts_with('\'') && text.ends_with('\''))
            || (text.starts_with('"') && text.ends_with('"'))))
    .then(|| text[1..text.len() - 1].to_string())
}

fn identifier(node: tree_sitter::Node<'_>, source: &[u8]) -> Option<String> {
    matches!(node.kind(), "identifier" | "type_identifier")
        .then(|| node.utf8_text(source).ok().map(str::to_owned))
        .flatten()
}

fn array_identifiers(node: Option<tree_sitter::Node<'_>>, source: &[u8]) -> Vec<String> {
    let Some(array) = node.filter(|node| node.kind() == "array") else {
        return Vec::new();
    };
    let mut cursor = array.walk();
    array
        .named_children(&mut cursor)
        .filter_map(|node| identifier(node, source))
        .collect()
}

fn resolve_identifiers(node: Option<tree_sitter::Node<'_>>, source: &[u8]) -> Vec<String> {
    let Some(object) = node.filter(|node| node.kind() == "object") else {
        return Vec::new();
    };
    let mut cursor = object.walk();
    object
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "pair")
        .filter_map(|pair| pair.child_by_field_name("value"))
        .filter_map(|node| identifier(node, source))
        .collect()
}

fn lazy_evidence(node: tree_sitter::Node<'_>, source: &[u8]) -> Option<(String, String)> {
    let text = node.utf8_text(source).ok()?;
    let import = text.find("import(")?;
    let module = crate::angular_meta::util::extract_first_quoted(&text[import + 7..])?;
    let then = text.find(".then(")?;
    let arrow = text[then..].find("=>")? + then;
    let selection = text[arrow + 2..].trim();
    let member = selection
        .rsplit_once('.')?
        .1
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '$');
    is_identifier(member).then(|| (module, member.to_string()))
}

fn resolve_lazy(
    path: &Path,
    module: &str,
    member: &str,
    expected: &str,
    dependencies: &mut Vec<crate::layers::meta::semantic::SemanticDependency>,
) -> Option<&'static str> {
    let target = resolve_module_path(path, module)?;
    dependencies.push(crate::layers::meta::semantic::SemanticDependency::current(
        target.to_string_lossy().into_owned(),
    ));
    let source = std::fs::read_to_string(target).ok()?;
    let kind = crate::angular_meta::module_metadata::declaration_kind(&source, member);
    if expected == "Component" && kind == Some("Component") {
        return kind;
    }
    if expected == "Module" && kind == Some("Module") {
        return kind;
    }
    if expected == "Module"
        && crate::angular_meta::module_metadata::has_routes_const(&source, member)
    {
        return Some("RouteConfig");
    }
    None
}
fn resolve_module_path(path: &Path, module: &str) -> Option<PathBuf> {
    if !(module.starts_with("./") || module.starts_with("../")) {
        return None;
    }
    let target = path.parent()?.join(module);
    if target.is_file() {
        return Some(target);
    }
    let mut typescript_path = target.into_os_string();
    typescript_path.push(".ts");
    let typescript_target = PathBuf::from(typescript_path);
    Some(typescript_target)
}

fn join_path(parent: &str, child: &str) -> String {
    match (parent.trim_matches('/'), child.trim_matches('/')) {
        ("", child) => child.to_string(),
        (parent, "") => parent.to_string(),
        (parent, child) => format!("{parent}/{child}"),
    }
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
