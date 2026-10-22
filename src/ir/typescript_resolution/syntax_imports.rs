//! Import binding extraction from the shared source tree.
use super::{Binding, Unit, is_type, module, symbol, text};

impl Unit {
    pub(super) fn import(&mut self, node: tree_sitter::Node<'_>, source: &str) {
        let mut cursor = node.walk();
        if let Some(clause) = node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "import_require_clause")
        {
            let mut bindings = clause.walk();
            let name = clause
                .named_children(&mut bindings)
                .find(|child| child.kind() == "identifier");
            if let (Some(name), Some(module)) = (name, module(clause, source)) {
                self.locals
                    .entry(text(name, source).into())
                    .or_default()
                    .push(Binding::Require {
                        module,
                        type_only: is_type(node),
                    });
            }
            return;
        }
        let Some(module) = module(node, source) else {
            return;
        };
        let type_only = is_type(node);
        let mode = match super::super::import_attributes::request_mode(node, source) {
            Ok(mode) => mode,
            Err(_) => {
                self.invalid = true;
                return;
            }
        };
        fn collect(
            unit: &mut Unit,
            node: tree_sitter::Node<'_>,
            source: &str,
            module: &str,
            type_only: bool,
            mode: super::super::module_resolution::ModuleRequestKind,
        ) {
            if node.kind() == "import_specifier" {
                let Some(name) = node.child_by_field_name("name") else {
                    return;
                };
                let Some(imported) = symbol(name, source) else {
                    return;
                };
                let local = node
                    .child_by_field_name("alias")
                    .map(|alias| text(alias, source).to_string())
                    .unwrap_or_else(|| imported.clone());
                unit.locals.entry(local).or_default().push(Binding::Import {
                    mode,
                    module: module.into(),
                    imported,
                    type_only: type_only || is_type(node),
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
                            mode,
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
                                mode,
                                module: module.into(),
                                imported: "default".into(),
                                type_only,
                            });
                    } else {
                        collect(unit, child, source, module, type_only, mode);
                    }
                }
                return;
            }
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                collect(unit, child, source, module, type_only, mode);
            }
        }
        collect(self, node, source, &module, type_only, mode);
    }
}
