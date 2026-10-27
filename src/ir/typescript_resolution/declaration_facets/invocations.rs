//! Only consumer-demanded native call sites enter the existing retained payload.
use super::{Demand, ExpressionFacet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct InvocationFacet {
    pub source: PathBuf,
    pub source_hash: String,
    pub expression: ExpressionFacet,
}
impl Demand<'_> {
    pub(super) fn capture_invocations(&mut self, file: &Path, selected: &[(usize, usize)]) {
        if selected.is_empty() {
            return;
        }
        let Some(file) = self.resolver.host.normalized(file) else {
            return;
        };
        let Some(source) = self.resolver.host.read(&file) else {
            return;
        };
        let Some(tree) = self.resolver.source_tree(&file) else {
            return;
        };
        let mut nodes = vec![tree.root_node()];
        while let Some(node) = nodes.pop() {
            let mut cursor = node.walk();
            nodes.extend(node.named_children(&mut cursor));
            if node.kind() != "call_expression" || node.has_error() {
                continue;
            }
            let Some(callee) = node.child_by_field_name("function") else {
                continue;
            };
            if !selected.contains(&(callee.start_byte(), callee.end_byte())) {
                continue;
            }
            self.invocation_sites
                .insert((file.clone(), callee.start_byte(), callee.end_byte()));
            if let Some(arguments) = node.child_by_field_name("arguments") {
                let mut cursor = arguments.walk();
                for argument in arguments
                    .named_children(&mut cursor)
                    .filter(|arg| matches!(arg.kind(), "identifier" | "member_expression"))
                {
                    self.invocation_sites.insert((
                        file.clone(),
                        argument.start_byte(),
                        argument.end_byte(),
                    ));
                }
            }
            let expression = self.expression(node, &file, &source);
            self.invocations.push(InvocationFacet {
                source: file.clone(),
                source_hash: self
                    .source_hashes
                    .entry(file.clone())
                    .or_insert_with(|| crate::cache::content_hash(source.as_bytes()))
                    .clone(),
                expression,
            });
        }
        self.invocations.sort_by_key(|call| call.expression.start);
    }
}
