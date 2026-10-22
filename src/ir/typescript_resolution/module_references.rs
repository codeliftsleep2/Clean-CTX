//! Syntax-owned module request evidence; ordinary require-spelled calls are not authority.
use super::{Resolver, module_resolution::ModuleRequestKind};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ModuleReference {
    pub start: usize,
    pub end: usize,
    pub specifier: String,
    pub kind: ModuleRequestKind,
    pub resolved: Result<PathBuf, String>,
    /// A declared module scope has no physical module-file target.
    #[serde(default)]
    pub ambient_declarations: Vec<super::DeclarationId>,
    #[serde(default)]
    pub(super) mode_valid: bool,
}

pub(super) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
) -> Result<Vec<ModuleReference>, String> {
    capture_inner(resolver, file, source, true)
}

/// Acquisition first inventories syntax, without probing a shadowed file target.
pub(super) fn capture_deferred(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
) -> Result<Vec<ModuleReference>, String> {
    capture_inner(resolver, file, source, false)
}

fn capture_inner(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    resolve: bool,
) -> Result<Vec<ModuleReference>, String> {
    let tree = resolver
        .source_tree(file)
        .ok_or("TypeScript module request parse failed")?;
    let mut pending = vec![tree.root_node()];
    let mut requests = Vec::new();
    while let Some(node) = pending.pop() {
        let request = match node.kind() {
            "import_statement" | "export_statement" => node
                .child_by_field_name("source")
                .map(|node| (node, ModuleRequestKind::StaticImport)),
            "import_require_clause" => node
                .child_by_field_name("source")
                .map(|node| (node, ModuleRequestKind::Require)),
            "call_expression"
                if node
                    .child_by_field_name("function")
                    .is_some_and(|node| node.kind() == "import") =>
            {
                node.child_by_field_name("arguments").and_then(|arguments| {
                    let mut cursor = arguments.walk();
                    let first = arguments
                        .named_children(&mut cursor)
                        .find(|child| child.kind() != "comment");
                    first
                        .filter(|node| node.kind() == "string")
                        .map(|node| (node, ModuleRequestKind::Import))
                })
            }
            _ => None,
        };
        if let Some((literal, kind)) = request
            && let Some(specifier) = literal
                .utf8_text(source.as_bytes())
                .ok()
                .and_then(super::string_value::decode)
        {
            let selected = if node.kind() == "import_statement" {
                super::import_attributes::request_mode(node, source)
            } else if node.kind() == "call_expression"
                && super::import_types::in_type_position(node)
            {
                super::import_types::call_mode(node, source)
            } else {
                Ok(kind)
            };
            let kind = selected.as_ref().copied().unwrap_or(kind);
            let mode_valid = selected.is_ok();
            let scope = selected.and_then(|kind| {
                if !resolve {
                    return Err("Deferred program module request".into());
                }
                resolver.in_source_project(file, |resolver| {
                    resolver.module_scope(file, &specifier, kind)
                })
            });
            let ambient_declarations = scope
                .as_ref()
                .map(Resolver::ambient_declarations)
                .unwrap_or_default();
            let resolved = scope.and_then(|scope| {
                if scope.blocks.is_empty() {
                    Ok(scope.file)
                } else {
                    Err("Resolved ambient module has declaration scope, not a module file".into())
                }
            });
            requests.push(ModuleReference {
                start: literal.start_byte(),
                end: literal.end_byte(),
                specifier,
                kind,
                resolved,
                ambient_declarations,
                mode_valid,
            });
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    requests.sort_by_key(|request| (request.start, request.end));
    Ok(requests)
}
