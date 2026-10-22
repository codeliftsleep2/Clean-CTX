//! Syntax-owned binding slots; keys and default-value expressions are not names.
pub(crate) fn names(node: tree_sitter::Node<'_>) -> Option<Vec<tree_sitter::Node<'_>>> {
    match node.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => Some(vec![node]),
        "pair_pattern" => names(node.child_by_field_name("value")?),
        "assignment_pattern" | "object_assignment_pattern" => {
            names(node.child_by_field_name("left")?)
        }
        "rest_pattern" => names(node.named_child(0)?),
        "array_pattern" | "object_pattern" => {
            let mut output = Vec::new();
            let mut cursor = node.walk();
            for child in node
                .named_children(&mut cursor)
                .filter(|child| child.kind() != "comment")
            {
                output.extend(names(child)?);
            }
            Some(output)
        }
        _ => None,
    }
}

pub(crate) fn is_binding_name(reference: tree_sitter::Node<'_>) -> bool {
    let mut ancestor = reference.parent();
    while let Some(node) = ancestor {
        let pattern = match node.kind() {
            "variable_declarator" => node.child_by_field_name("name"),
            "required_parameter" | "optional_parameter" => node.child_by_field_name("pattern"),
            "catch_clause" => node.child_by_field_name("parameter"),
            "arrow_function" => node.child_by_field_name("parameter"),
            "function_expression" | "generator_function" => node.child_by_field_name("name"),
            "for_in_statement" if node.child_by_field_name("kind").is_some() => {
                node.child_by_field_name("left")
            }
            _ => {
                ancestor = node.parent();
                continue;
            }
        };
        return pattern
            .and_then(names)
            .is_some_and(|names| names.contains(&reference));
    }
    false
}
