//! Ambient syntax is captured from the same immutable Unit tree and bytes.
use super::syntax::Unit;
use std::{collections::BTreeSet, path::Path};

impl Unit {
    pub(super) fn capture_ambient(
        &mut self,
        node: tree_sitter::Node<'_>,
        path: &Path,
        source: &str,
        exported: bool,
    ) -> bool {
        if node.kind() == "ambient_declaration" {
            let mut cursor = node.walk();
            let namespace = node
                .named_children(&mut cursor)
                .find(|child| child.kind() == "internal_module");
            if let Some(namespace) = namespace {
                if !self.namespace_declaration(namespace, path, source, exported) {
                    self.invalid = true;
                }
                for (id, body) in &mut self.namespace_bodies {
                    if id.start == namespace.start_byte() && id.end == namespace.end_byte() {
                        body.ambient_exports();
                    }
                }
                return true;
            }
            let mut cursor = node.walk();
            if node
                .children(&mut cursor)
                .any(|child| child.kind() == "global")
            {
                let mut cursor = node.walk();
                let body = node
                    .named_children(&mut cursor)
                    .find(|child| child.kind() == "statement_block");
                if let Some(mut unit) =
                    body.and_then(|body| Self::scope(body, path, source, &BTreeSet::new()))
                {
                    unit.ambient_exports();
                    self.ambient_globals.push(unit);
                } else {
                    self.invalid = true;
                }
                return true;
            }
        }
        if node.kind() == "module" {
            let Some(name) = node.child_by_field_name("name") else {
                self.invalid = true;
                return true;
            };
            if name.kind() != "string" {
                return false;
            }
            let specifier = name
                .utf8_text(source.as_bytes())
                .ok()
                .and_then(super::string_value::decode);
            let body = node
                .child_by_field_name("body")
                .and_then(|body| Self::scope(body, path, source, &BTreeSet::new()));
            if let (Some(specifier), Some(mut body)) = (specifier, body) {
                body.ambient_exports();
                let id = super::DeclarationId {
                    source: path.to_path_buf(),
                    source_hash: crate::cache::content_hash(source.as_bytes()),
                    start: node.start_byte(),
                    end: node.end_byte(),
                    name: specifier.clone(),
                    kind: "ambient_module".into(),
                };
                self.namespace_bodies.insert(id, body.clone());
                self.augmentations.push((specifier, body));
            } else {
                self.invalid = true;
            }
            return true;
        }
        false
    }

    fn ambient_exports(&mut self) {
        self.exports = self.locals.clone();
        for body in self.namespace_bodies.values_mut() {
            body.ambient_exports();
        }
    }

    pub(super) fn classify_program(&mut self, root: tree_sitter::Node<'_>, path: &Path) {
        self.source = Some(path.to_path_buf());
        let mut cursor = root.walk();
        self.external = root
            .named_children(&mut cursor)
            .any(|node| matches!(node.kind(), "import_statement" | "export_statement"));
        if !self.external {
            let mut pending = vec![root];
            while let Some(node) = pending.pop() {
                if node.kind() == "meta_property" {
                    let mut cursor = node.walk();
                    if node
                        .children(&mut cursor)
                        .any(|child| child.kind() == "import")
                    {
                        self.external = true;
                        break;
                    }
                }
                let mut cursor = node.walk();
                pending.extend(node.named_children(&mut cursor));
            }
        }
    }
}
