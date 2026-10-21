//! Source-local EF model registration syntax; shared by projection and obligations.
use std::collections::BTreeSet;
use tree_sitter::Node;

#[derive(Default)]
pub(super) struct OnModelCreating {
    pub(super) direct_entities: BTreeSet<String>,
    pub(super) explicit_configurations: BTreeSet<String>,
    pub(super) assembly_markers: BTreeSet<String>,
}

impl OnModelCreating {
    pub(super) fn parse(source: &str) -> Option<Self> {
        let language = crate::compression::language::safe_csharp_language()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).ok()?;
        let tree = parser.parse(source, None)?;
        let mut model = Self::default();
        collect_on_model_creating(tree.root_node(), source.as_bytes(), &mut model);
        Some(model)
    }
}

fn collect_on_model_creating(node: Node<'_>, source: &[u8], model: &mut OnModelCreating) {
    if node.kind() == "method_declaration"
        && node
            .child_by_field_name("name")
            .and_then(|name| name.utf8_text(source).ok())
            == Some("OnModelCreating")
    {
        let Some(builder) = model_builder_parameter(node, source) else {
            return;
        };
        collect_model_invocations(node, source, &builder, model);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_on_model_creating(child, source, model);
    }
}

fn model_builder_parameter(node: Node<'_>, source: &[u8]) -> Option<String> {
    let parameters = node.child_by_field_name("parameters")?;
    let mut cursor = parameters.walk();
    parameters
        .named_children(&mut cursor)
        .find_map(|parameter| {
            let type_name = parameter
                .child_by_field_name("type")?
                .utf8_text(source)
                .ok()?;
            if !type_name.trim_end().ends_with("ModelBuilder") {
                return None;
            }
            parameter
                .child_by_field_name("name")?
                .utf8_text(source)
                .ok()
                .map(str::to_string)
        })
}

fn collect_model_invocations(
    node: Node<'_>,
    source: &[u8],
    builder: &str,
    model: &mut OnModelCreating,
) {
    if node.kind() == "invocation_expression"
        && let Ok(text) = node.utf8_text(source)
    {
        let direct = format!("{builder}.Entity<");
        let explicit = format!("{builder}.ApplyConfiguration(");
        let assembly = format!("{builder}.ApplyConfigurationsFromAssembly(");
        if let Some(written) = generic_argument_after(text, &direct) {
            model.direct_entities.insert(written);
        } else if let Some(rest) = text.strip_prefix(&explicit)
            && let Some(written) = constructed_type(rest)
        {
            model.explicit_configurations.insert(written);
        } else if let Some(rest) = text.strip_prefix(&assembly)
            && let Some(written) = local_assembly_marker(rest)
        {
            model.assembly_markers.insert(written);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_model_invocations(child, source, builder, model);
    }
}

fn generic_argument_after(text: &str, prefix: &str) -> Option<String> {
    let rest = text.strip_prefix(prefix)?;
    let end = rest.find('>')?;
    let written = rest[..end].trim();
    (!written.is_empty()).then(|| written.to_string())
}

fn constructed_type(rest: &str) -> Option<String> {
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("new ")?;
    let end = rest.find('(')?;
    let written = rest[..end].trim();
    (!written.is_empty()).then(|| written.to_string())
}

fn local_assembly_marker(rest: &str) -> Option<String> {
    let rest = rest.trim_start().strip_prefix("typeof(")?;
    let end = rest.find(')')?;
    let written = rest[..end].trim();
    rest[end + 1..]
        .trim_start()
        .starts_with(".Assembly")
        .then(|| written.to_string())
}
