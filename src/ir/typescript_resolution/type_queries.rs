//! Non-emitting typeof queries bind value declarations without granting runtime use.
use super::{
    Namespace, Resolution, Resolver,
    syntax::{Binding, admits},
};
use crate::ir::invocation_evidence::SourceSpan;
use std::{collections::BTreeSet, path::Path};
use tree_sitter::Node;

impl Resolver {
    pub(super) fn type_query_reference(
        &mut self,
        file: &Path,
        source: &str,
        node: Node<'_>,
    ) -> Option<Resolution> {
        let mut cursor = node.walk();
        let operands: Vec<_> = node
            .named_children(&mut cursor)
            .filter(|child| child.kind() != "comment")
            .collect();
        let [operand] = operands.as_slice() else {
            return Some(Resolution::Unresolved("Unproved typeof operand".into()));
        };
        if super::import_types::call_and_members(*operand, source).is_some() {
            return self.import_reference_in_namespace(file, source, *operand, Namespace::Value);
        }
        if !matches!(operand.kind(), "identifier" | "member_expression") {
            return Some(Resolution::Unresolved(
                "Type query operand requires additional authority".into(),
            ));
        }
        let span = SourceSpan::new(operand.start_byte(), operand.end_byte()).ok()?;
        self.value_reference_for_query(file, source, span, true)
    }

    pub(super) fn query_value_binding(
        &mut self,
        file: &Path,
        name: &str,
        members: &[&str],
    ) -> Resolution {
        self.in_source_project(file, |resolver| {
            let Some(unit) = resolver.unit(file) else {
                return Resolution::Unresolved("Type query source absent".into());
            };
            let eligible: Vec<_> = unit
                .locals
                .get(name)
                .into_iter()
                .flatten()
                .filter(|binding| {
                    admits(binding, Namespace::Value)
                        || matches!(
                            binding,
                            Binding::Import { .. }
                                | Binding::Namespace { .. }
                                | Binding::Require { .. }
                        )
                })
                .collect();
            if eligible.iter().any(|binding| {
                matches!(
                    binding,
                    Binding::Import {
                        type_only: true,
                        ..
                    } | Binding::Namespace {
                        type_only: true,
                        ..
                    } | Binding::Require {
                        type_only: true,
                        ..
                    }
                )
            }) {
                let [binding] = eligible.as_slice() else {
                    return Resolution::Unresolved("Competing type-query bindings".into());
                };
                if members.is_empty() {
                    let mut binding = (**binding).clone();
                    match &mut binding {
                        Binding::Import { type_only, .. }
                        | Binding::Namespace { type_only, .. }
                        | Binding::Require { type_only, .. } => *type_only = false,
                        _ => {}
                    }
                    let (ids, complete) =
                        resolver.binding(file, &binding, Namespace::Value, &mut BTreeSet::new());
                    return resolver.outcome(ids, complete);
                }
                if let Binding::Namespace { module, mode, .. } = binding {
                    let target = match resolver.module_scope(file, module, *mode) {
                        Ok(target) => target,
                        Err(error) => return Resolution::Unresolved(error),
                    };
                    let (first, rest) = members.split_first().expect("nonempty members");
                    if rest.is_empty() {
                        return resolver.scope_export(&target, first, Namespace::Value);
                    }
                    return resolver.namespace_chain_from(
                        &target,
                        first,
                        rest,
                        Namespace::Value,
                        true,
                    );
                }
                return Resolution::Unresolved(
                    "Qualified type-only alias requires scope authority".into(),
                );
            }
            if members.is_empty() {
                resolver.local(file, name, Namespace::Value)
            } else {
                resolver.namespace_chain(file, name, members, Namespace::Value)
            }
        })
    }
}
