//! Existing source export inspection, relocated without policy changes.
pub(super) fn exports_const(path: &str, name: &str) -> bool {
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|s| s.export_reads += 1);
    let Ok(source) = std::fs::read_to_string(path) else {
        return false;
    };
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|s| s.export_parses += 1);
    let Some(tree) = parser.parse(&source, None) else {
        return false;
    };
    let root = tree.root_node();
    let mut cursor = root.walk();
    root.named_children(&mut cursor)
        .filter(|node| node.kind() == "export_statement")
        .any(|export| exported_const_descendant(export, source.as_bytes(), name))
}

pub(super) fn exports_function(path: &str, name: &str) -> bool {
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|s| s.export_reads += 1);
    let Ok(source) = std::fs::read_to_string(path) else {
        return false;
    };
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|s| s.export_parses += 1);
    let Some(tree) = parser
        .parse(&source, None)
        .filter(|tree| !tree.root_node().has_error())
    else {
        return false;
    };
    let root = tree.root_node();
    root.named_children(&mut root.walk())
        .filter(|node| node.kind() == "export_statement")
        .filter_map(|export| export.named_child(0))
        .any(|declaration| {
            declaration.kind() == "function_declaration"
                && declaration
                    .child_by_field_name("name")
                    .and_then(|identifier| identifier.utf8_text(source.as_bytes()).ok())
                    == Some(name)
        })
}

fn exported_const_descendant(node: tree_sitter::Node<'_>, source: &[u8], name: &str) -> bool {
    if node.kind() == "variable_declarator"
        && node
            .child_by_field_name("name")
            .and_then(|name| name.utf8_text(source).ok())
            == Some(name)
        && node
            .parent()
            .is_some_and(|parent| parent.kind() == "lexical_declaration")
    {
        return true;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| exported_const_descendant(child, source, name))
}
