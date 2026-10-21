use super::{DeclarationId, Namespace};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug)]
pub(super) enum Binding {
    Declaration(DeclarationId, bool),
    Import {
        module: String,
        imported: String,
        type_only: bool,
    },
    Namespace {
        module: String,
        type_only: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub(super) struct Unit {
    pub locals: BTreeMap<String, Vec<Binding>>,
    pub exports: BTreeMap<String, Vec<Binding>>,
    pub stars: Vec<(String, bool)>,
}

fn text<'a>(node: tree_sitter::Node<'_>, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or_default()
}
fn module(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    Some(
        text(node.child_by_field_name("source")?, source)
            .trim_matches(['\'', '"'])
            .to_string(),
    )
}
fn is_type(node: tree_sitter::Node<'_>, source: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .any(|child| child.kind() == "type" || text(child, source) == "type")
}

impl Unit {
    pub(super) fn parse(path: &Path, source: &str) -> Option<Self> {
        let language = crate::compression::language::safe_typescript_language()?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).ok()?;
        let tree = parser.parse(source, None)?;
        if tree.root_node().has_error() {
            return None;
        }
        let mut unit = Self::default();
        let root = tree.root_node();
        let mut cursor = root.walk();
        for node in root.named_children(&mut cursor) {
            match node.kind() {
                "import_statement" => unit.import(node, source),
                "export_statement" => {
                    let external = module(node, source);
                    let type_only = is_type(node, source);
                    let mut cursor = node.walk();
                    for child in node.named_children(&mut cursor) {
                        if child.kind() == "export_clause" {
                            let mut cursor = child.walk();
                            for entry in child.named_children(&mut cursor) {
                                let Some(name) = entry.child_by_field_name("name") else {
                                    continue;
                                };
                                let name = text(name, source).to_string();
                                let alias = entry
                                    .child_by_field_name("alias")
                                    .map(|alias| text(alias, source).to_string())
                                    .unwrap_or_else(|| name.clone());
                                let binding = external.as_ref().map(|module| Binding::Import {
                                    module: module.clone(),
                                    imported: name.clone(),
                                    type_only: type_only || is_type(entry, source),
                                });
                                if let Some(binding) = binding {
                                    unit.exports.entry(alias).or_default().push(binding);
                                } else {
                                    // Local export lists are linked after all locals are captured.
                                    unit.exports
                                        .entry(alias)
                                        .or_default()
                                        .push(Binding::Import {
                                            module: String::new(),
                                            imported: name,
                                            type_only: type_only || is_type(entry, source),
                                        });
                                }
                            }
                        } else {
                            unit.declarations(child, path, source, true);
                        }
                    }
                    if let Some(module) = external
                        && text(node, source).contains('*')
                    {
                        let mut cursor = node.walk();
                        if node
                            .named_children(&mut cursor)
                            .any(|child| child.kind() == "namespace_export")
                        {
                            // Namespace export identity needs a module-symbol node.
                            // It must not be misinterpreted as an ordinary star.
                            return None;
                        }
                        unit.stars.push((module, type_only));
                    }
                }
                _ => unit.declarations(node, path, source, false),
            }
        }
        Some(unit)
    }

    fn import(&mut self, node: tree_sitter::Node<'_>, source: &str) {
        let Some(module) = module(node, source) else {
            return;
        };
        let type_only = is_type(node, source);
        fn collect(
            unit: &mut Unit,
            node: tree_sitter::Node<'_>,
            source: &str,
            module: &str,
            type_only: bool,
        ) {
            if node.kind() == "import_specifier" {
                let Some(name) = node.child_by_field_name("name") else {
                    return;
                };
                let imported = text(name, source).to_string();
                let local = node
                    .child_by_field_name("alias")
                    .map(|alias| text(alias, source).to_string())
                    .unwrap_or_else(|| imported.clone());
                unit.locals.entry(local).or_default().push(Binding::Import {
                    module: module.into(),
                    imported,
                    type_only: type_only || is_type(node, source),
                });
                return;
            }
            if node.kind() == "namespace_import" {
                let mut cursor = node.walk();
                if let Some(name) = node
                    .named_children(&mut cursor)
                    .find(|child| child.kind() == "identifier")
                {
                    unit.locals
                        .entry(text(name, source).to_string())
                        .or_default()
                        .push(Binding::Namespace {
                            module: module.into(),
                            type_only,
                        });
                }
                return;
            }
            if node.kind() == "import_clause" {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    if child.kind() == "identifier" {
                        unit.locals
                            .entry(text(child, source).to_string())
                            .or_default()
                            .push(Binding::Import {
                                module: module.into(),
                                imported: "default".into(),
                                type_only,
                            });
                    } else {
                        collect(unit, child, source, module, type_only);
                    }
                }
                return;
            }
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect(unit, child, source, module, type_only);
            }
        }
        collect(self, node, source, &module, type_only);
    }

    fn declarations(
        &mut self,
        node: tree_sitter::Node<'_>,
        path: &Path,
        source: &str,
        exported: bool,
    ) {
        if matches!(
            node.kind(),
            "ambient_declaration" | "lexical_declaration" | "variable_declaration"
        ) {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                self.declarations(child, path, source, exported);
            }
            return;
        }
        if !matches!(
            node.kind(),
            "class_declaration"
                | "abstract_class_declaration"
                | "function_declaration"
                | "function_signature"
                | "interface_declaration"
                | "type_alias_declaration"
                | "enum_declaration"
                | "variable_declarator"
        ) {
            return;
        }
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        if !matches!(name.kind(), "identifier" | "type_identifier") {
            return;
        }
        let name = text(name, source).to_string();
        let binding = Binding::Declaration(
            DeclarationId {
                source: path.to_path_buf(),
                source_hash: crate::cache::content_hash(source.as_bytes()),
                start: node.start_byte(),
                end: node.end_byte(),
                name: name.clone(),
                kind: node.kind().into(),
            },
            !matches!(
                node.kind(),
                "interface_declaration" | "type_alias_declaration"
            ),
        );
        self.locals
            .entry(name.clone())
            .or_default()
            .push(binding.clone());
        if exported {
            self.exports.entry(name).or_default().push(binding);
        }
    }
}

pub(super) fn admits(binding: &Binding, namespace: Namespace) -> bool {
    match binding {
        Binding::Declaration(_, value) => *value || namespace == Namespace::Type,
        Binding::Import { type_only, .. } | Binding::Namespace { type_only, .. } => {
            !*type_only || namespace == Namespace::Type
        }
    }
}
