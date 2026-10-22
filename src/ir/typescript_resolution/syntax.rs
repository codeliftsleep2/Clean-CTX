#[path = "syntax_imports.rs"]
mod imports;
use super::{DeclarationId, Namespace};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug)]
pub(super) enum Binding {
    Alias(Vec<String>),
    Declaration(DeclarationId, bool),
    Import {
        module: String,
        imported: String,
        mode: super::module_resolution::ModuleRequestKind,
        type_only: bool,
    },
    Require {
        module: String,
        type_only: bool,
    },
    Namespace {
        mode: super::module_resolution::ModuleRequestKind,
        module: String,
        type_only: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub(super) struct Unit {
    pub tree: Option<tree_sitter::Tree>,
    pub locals: BTreeMap<String, Vec<Binding>>,
    pub exports: BTreeMap<String, Vec<Binding>>,
    pub stars: Vec<(String, bool)>,
    pub assignment: Vec<Binding>,
    pub namespace_bodies: BTreeMap<DeclarationId, Unit>,
    pub(super) invalid: bool,
    pub ambient_globals: Vec<Unit>,
    pub augmentations: Vec<(String, Unit)>,
    pub external: bool,
    pub source: Option<std::path::PathBuf>,
}

fn text<'a>(node: tree_sitter::Node<'_>, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or_default()
}
fn module(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    super::string_value::decode(text(node.child_by_field_name("source")?, source))
}
fn symbol(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    if node.kind() == "string" {
        super::string_value::decode(text(node, source))
    } else {
        Some(text(node, source).to_string())
    }
}
fn is_type(node: tree_sitter::Node<'_>) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .any(|child| child.kind() == "type")
}

impl Unit {
    pub(super) fn namespace_declaration(
        &mut self,
        node: tree_sitter::Node<'_>,
        path: &Path,
        source: &str,
        exported: bool,
    ) -> bool {
        fn segments<'a>(node: tree_sitter::Node<'a>, out: &mut Vec<tree_sitter::Node<'a>>) -> bool {
            match node.kind() {
                "identifier" | "property_identifier" => {
                    out.push(node);
                    true
                }
                "nested_identifier" => {
                    let mut cursor = node.walk();
                    node.named_children(&mut cursor)
                        .filter(|child| child.kind() != "comment")
                        .all(|child| segments(child, out))
                }
                _ => false,
            }
        }
        let Some(name) = node.child_by_field_name("name") else {
            return false;
        };
        let Some(body) = node.child_by_field_name("body") else {
            return false;
        };
        let mut names = Vec::new();
        if !segments(name, &mut names) || names.is_empty() {
            return false;
        }
        let Some(mut scope) = Self::scope(body, path, source, &std::collections::BTreeSet::new())
        else {
            return false;
        };
        // A.B is declaration syntax for A containing an exported namespace B.
        // Every qualifier retains its own AST segment span; the outer owner
        // keeps the existing whole-declaration identity used by simple forms.
        for (index, segment) in names.iter().enumerate().rev() {
            let id = DeclarationId {
                source: path.to_path_buf(),
                source_hash: crate::cache::content_hash(source.as_bytes()),
                start: if index == 0 {
                    node.start_byte()
                } else {
                    segment.start_byte()
                },
                end: if index == 0 {
                    node.end_byte()
                } else {
                    segment.end_byte()
                },
                name: text(*segment, source).into(),
                kind: "internal_module".into(),
            };
            let mut parent = Self::default();
            let binding = Binding::Declaration(id.clone(), true);
            parent
                .locals
                .entry(id.name.clone())
                .or_default()
                .push(binding.clone());
            if index != 0 || exported {
                parent
                    .exports
                    .entry(id.name.clone())
                    .or_default()
                    .push(binding);
            }
            parent.namespace_bodies.insert(id, scope);
            scope = parent;
        }
        for (name, bindings) in scope.locals {
            self.locals.entry(name).or_default().extend(bindings);
        }
        for (name, bindings) in scope.exports {
            self.exports.entry(name).or_default().extend(bindings);
        }
        self.namespace_bodies.extend(scope.namespace_bodies);
        true
    }

    pub(super) fn parse(path: &Path, source: &str) -> Option<Self> {
        let language = if matches!(
            path.extension().and_then(|part| part.to_str()),
            Some("tsx" | "jsx")
        ) {
            static TSX: std::sync::OnceLock<tree_sitter::Language> = std::sync::OnceLock::new();
            TSX.get_or_init(|| tree_sitter_typescript::LANGUAGE_TSX.into())
                .clone()
        } else {
            crate::compression::language::safe_typescript_language()?
        };
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).ok()?;
        let (tree, type_stars) = super::syntax_frontend::parse(&mut parser, source)?;
        let mut unit = Self::scope(tree.root_node(), path, source, &type_stars)?;
        unit.classify_program(tree.root_node(), path);
        unit.tree = Some(tree);
        Some(unit)
    }

    pub(super) fn scope(
        root: tree_sitter::Node<'_>,
        path: &Path,
        source: &str,
        type_stars: &std::collections::BTreeSet<usize>,
    ) -> Option<Self> {
        let mut unit = Self {
            source: Some(path.to_path_buf()),
            ..Self::default()
        };
        let mut cursor = root.walk();
        for node in root.named_children(&mut cursor) {
            // An undecodable external source is not an absent source clause.
            // In particular it must never turn `export ... from` into a local
            // export and borrow a same-spelling declaration from this unit.
            if node.child_by_field_name("source").is_some() && module(node, source).is_none() {
                return None;
            }
            match node.kind() {
                "import_statement" => unit.import(node, source),
                "export_statement" => {
                    if has_token(node, "=") {
                        let mut cursor = node.walk();
                        if let Some(value) = node
                            .named_children(&mut cursor)
                            .find(|child| child.kind() != "comment")
                            && value.kind() == "identifier"
                        {
                            unit.assignment.push(Binding::Import {
                                mode: super::module_resolution::ModuleRequestKind::StaticImport,
                                module: String::new(),
                                imported: text(value, source).into(),
                                type_only: false,
                            });
                        }
                        continue;
                    }
                    let external = module(node, source);
                    let type_only = is_type(node) || type_stars.contains(&node.start_byte());
                    let default_export = has_token(node, "default");
                    let mut namespace_export = false;
                    let mut cursor = node.walk();
                    for child in node.named_children(&mut cursor) {
                        if child.kind() == "namespace_export" {
                            namespace_export = true;
                            let mut cursor = child.walk();
                            let name = child
                                .named_children(&mut cursor)
                                .find(|node| matches!(node.kind(), "identifier" | "string"))?;
                            unit.exports.entry(symbol(name, source)?).or_default().push(
                                Binding::Namespace {
                                    mode: super::module_resolution::ModuleRequestKind::StaticImport,
                                    module: external.clone()?,
                                    type_only,
                                },
                            );
                        } else if child.kind() == "export_clause" {
                            let mut cursor = child.walk();
                            for entry in child.named_children(&mut cursor) {
                                let Some(name) = entry.child_by_field_name("name") else {
                                    continue;
                                };
                                let name = symbol(name, source)?;
                                let alias = entry
                                    .child_by_field_name("alias")
                                    .map(|alias| symbol(alias, source))
                                    .unwrap_or_else(|| Some(name.clone()))?;
                                let binding = external.as_ref().map(|module| Binding::Import {
                                    mode: super::module_resolution::ModuleRequestKind::StaticImport,
                                    module: module.clone(),
                                    imported: name.clone(),
                                    type_only: type_only || is_type(entry),
                                });
                                if let Some(binding) = binding {
                                    unit.exports.entry(alias).or_default().push(binding);
                                } else {
                                    // Local export lists are linked after all locals are captured.
                                    unit.exports
                                        .entry(alias)
                                        .or_default()
                                        .push(Binding::Import {
                                            mode: super::module_resolution::ModuleRequestKind::StaticImport,
                                            module: String::new(),
                                            imported: name,
                                            type_only: type_only || is_type(entry),
                                        });
                                }
                            }
                        } else if default_export {
                            unit.default_declaration(child, path, source);
                        } else {
                            unit.declarations(child, path, source, true);
                        }
                    }
                    if let Some(module) = external
                        && has_token(node, "*")
                        && !namespace_export
                    {
                        unit.stars.push((module, type_only));
                    }
                }
                _ => unit.declarations(node, path, source, false),
            }
        }
        drop(cursor);
        if unit.invalid { None } else { Some(unit) }
    }

    fn default_declaration(&mut self, node: tree_sitter::Node<'_>, path: &Path, source: &str) {
        if node.kind() == "identifier" {
            self.exports
                .entry("default".into())
                .or_default()
                .push(Binding::Import {
                    mode: super::module_resolution::ModuleRequestKind::StaticImport,
                    module: String::new(),
                    imported: text(node, source).into(),
                    type_only: false,
                });
            return;
        }
        if !matches!(
            node.kind(),
            "class_declaration"
                | "abstract_class_declaration"
                | "function_declaration"
                | "generator_function_declaration"
                | "function_signature"
                | "interface_declaration"
                | "class"
                | "function_expression"
                | "generator_function"
        ) {
            // Expressions need expression-owned identity, not a guessed declaration.
            return;
        }
        if node.child_by_field_name("name").is_none()
            || matches!(
                node.kind(),
                "class" | "function_expression" | "generator_function"
            )
        {
            // A default expression's internal name is not a module-local binding.
            // Identity comes from its physical source and exact syntax span.
            let binding = Binding::Declaration(
                DeclarationId {
                    source: path.to_path_buf(),
                    source_hash: crate::cache::content_hash(source.as_bytes()),
                    start: node.start_byte(),
                    end: node.end_byte(),
                    name: node
                        .child_by_field_name("name")
                        .map(|name| text(name, source).to_string())
                        .unwrap_or_else(|| "default".into()),
                    kind: node.kind().into(),
                },
                node.kind() != "interface_declaration",
            );
            self.exports
                .entry("default".into())
                .or_default()
                .push(binding);
            return;
        }
        self.declarations(node, path, source, false);
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        if let Some(bindings) = self.locals.get(text(name, source)) {
            let exact = bindings.iter().filter(|binding| matches!(binding,
                Binding::Declaration(id, _) if id.start == node.start_byte() && id.end == node.end_byte()
            )).cloned().collect::<Vec<_>>();
            self.exports
                .entry("default".into())
                .or_default()
                .extend(exact);
        }
    }

    fn declarations(
        &mut self,
        node: tree_sitter::Node<'_>,
        path: &Path,
        source: &str,
        exported: bool,
    ) {
        if self.capture_ambient(node, path, source, exported) {
            return;
        }
        if node.kind() == "import_alias" {
            fn path(node: tree_sitter::Node<'_>, source: &str, parts: &mut Vec<String>) -> bool {
                match node.kind() {
                    "identifier" | "property_identifier" => {
                        parts.push(text(node, source).into());
                        true
                    }
                    "nested_identifier" => {
                        let mut cursor = node.walk();
                        node.named_children(&mut cursor)
                            .filter(|n| n.kind() != "comment")
                            .all(|n| path(n, source, parts))
                    }
                    _ => false,
                }
            }
            let mut cursor = node.walk();
            let children: Vec<_> = node
                .named_children(&mut cursor)
                .filter(|n| n.kind() != "comment")
                .collect();
            if children.len() != 2 || children[0].kind() != "identifier" {
                self.invalid = true;
                return;
            }
            let mut parts = Vec::new();
            if !path(children[1], source, &mut parts) || parts.is_empty() {
                self.invalid = true;
                return;
            }
            let name = text(children[0], source).to_string();
            let binding = Binding::Alias(parts);
            self.locals
                .entry(name.clone())
                .or_default()
                .push(binding.clone());
            if exported {
                self.exports.entry(name).or_default().push(binding);
            }
            return;
        }
        // The pinned grammar also admits internal_module as an expression.
        // A standalone namespace may therefore have this statement wrapper.
        // Unwrap only that exact shape, never declarations inside expressions.
        if node.kind() == "expression_statement" {
            let mut cursor = node.walk();
            let mut children = node
                .named_children(&mut cursor)
                .filter(|child| child.kind() != "comment");
            if let Some(child) = children.next()
                && child.kind() == "internal_module"
                && children.next().is_none()
            {
                self.declarations(child, path, source, exported);
            }
            return;
        }
        if node.kind() == "internal_module" {
            if !self.namespace_declaration(node, path, source, exported) {
                self.invalid = true;
            }
            return;
        }
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
                | "generator_function_declaration"
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
        if node.kind() == "variable_declarator"
            && matches!(name.kind(), "object_pattern" | "array_pattern")
        {
            let Some(names) = crate::ir::typescript_binding_pattern::names(name) else {
                return;
            };
            for slot in names {
                let name = text(slot, source).to_string();
                let binding = Binding::Declaration(
                    DeclarationId {
                        source: path.to_path_buf(),
                        source_hash: crate::cache::content_hash(source.as_bytes()),
                        start: slot.start_byte(),
                        end: slot.end_byte(),
                        name: name.clone(),
                        kind: "binding_pattern".into(),
                    },
                    true,
                );
                self.locals
                    .entry(name.clone())
                    .or_default()
                    .push(binding.clone());
                if exported {
                    self.exports.entry(name).or_default().push(binding);
                }
            }
            return;
        }
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

fn has_token(node: tree_sitter::Node<'_>, token: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .any(|child| child.kind() == token)
}

pub(super) fn admits(binding: &Binding, namespace: Namespace) -> bool {
    match binding {
        Binding::Alias(_) => true,
        Binding::Declaration(declaration, value) => match namespace {
            Namespace::Value => *value,
            Namespace::Type => matches!(
                declaration.kind.as_str(),
                "class_declaration"
                    | "class"
                    | "abstract_class_declaration"
                    | "interface_declaration"
                    | "type_alias_declaration"
                    | "enum_declaration"
            ),
        },
        Binding::Import { type_only, .. }
        | Binding::Namespace { type_only, .. }
        | Binding::Require { type_only, .. } => !*type_only || namespace == Namespace::Type,
    }
}
