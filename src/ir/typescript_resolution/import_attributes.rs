//! Syntax-owned type-import mode, shared by binding and request evidence.
use super::module_resolution::ModuleRequestKind;
use tree_sitter::Node;

pub(super) fn request_mode(node: Node<'_>, source: &str) -> Result<ModuleRequestKind, String> {
    let mut cursor = node.walk();
    let attributes: Vec<_> = node
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "import_attribute")
        .collect();
    if attributes.is_empty() {
        return Ok(ModuleRequestKind::StaticImport);
    }
    if attributes.len() != 1 {
        return Err("Multiple import attribute clauses".into());
    }
    let attribute = attributes[0];
    let mut cursor = attribute.walk();
    let object = attribute
        .named_children(&mut cursor)
        .find(|child| child.kind() == "object")
        .ok_or("Import attributes require an object")?;
    let selected = object_mode(object, source)?;
    if let Some(mode) = selected {
        let mut cursor = node.walk();
        if !node
            .children(&mut cursor)
            .any(|child| child.kind() == "type")
        {
            return Err("resolution-mode requires a type-only import".into());
        }
        Ok(mode)
    } else {
        Ok(ModuleRequestKind::StaticImport)
    }
}

pub(super) fn object_mode(
    object: Node<'_>,
    source: &str,
) -> Result<Option<ModuleRequestKind>, String> {
    if object.kind() != "object" {
        return Err("Import attributes require an object".into());
    }
    let mut cursor = object.walk();
    let mut selected = None;
    for pair in object
        .named_children(&mut cursor)
        .filter(|child| child.kind() != "comment")
    {
        if pair.kind() != "pair" {
            return Err("Unproved import attribute".into());
        }
        let key = pair
            .child_by_field_name("key")
            .ok_or("Missing import attribute key")?;
        let key_text = key
            .utf8_text(source.as_bytes())
            .map_err(|_| "Invalid attribute key")?;
        let key = if key.kind() == "string" {
            super::string_value::decode(key_text).ok_or("Invalid attribute string")?
        } else if key.kind() == "property_identifier" {
            key_text.to_string()
        } else {
            return Err("Unproved import attribute key".into());
        };
        if key != "resolution-mode" {
            continue;
        }
        if selected.is_some() {
            return Err("Duplicate resolution-mode".into());
        }
        let value = pair
            .child_by_field_name("value")
            .ok_or("Missing attribute value")?;
        if value.kind() != "string" {
            return Err("Nonliteral resolution-mode".into());
        }
        selected = Some(
            match super::string_value::decode(
                value
                    .utf8_text(source.as_bytes())
                    .map_err(|_| "Invalid attribute value")?,
            )
            .as_deref()
            {
                Some("import") => ModuleRequestKind::Import,
                Some("require") => ModuleRequestKind::Require,
                _ => return Err("Invalid resolution-mode".into()),
            },
        );
    }
    Ok(selected)
}
