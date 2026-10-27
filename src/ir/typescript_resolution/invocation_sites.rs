//! Demand-scoped invocation positions use the existing resolver and lexical binder.
use super::package_evidence::PackageExportEvidence;
use super::{Resolution, Resolver};
use crate::ir::invocation_evidence::SourceSpan;
use std::path::Path;

type CapturedInvocationSites = (
    Vec<(usize, usize, Resolution)>,
    Vec<PackageExportEvidence>,
    Vec<(usize, usize)>,
);

pub(super) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    apis: &[(&str, &str)],
) -> CapturedInvocationSites {
    let mut sites = Vec::new();
    let mut packages = Vec::new();
    let mut selected = Vec::new();
    if apis.is_empty() {
        return (sites, packages, selected);
    }
    let Some(tree) = resolver.source_tree(file) else {
        return (sites, packages, selected);
    };
    let mut pending = vec![tree.root_node()];
    while let Some(node) = pending.pop() {
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
        if node.kind() != "call_expression" || node.has_error() {
            continue;
        }
        let Some(callee) = node.child_by_field_name("function") else {
            continue;
        };
        let Ok(span) = SourceSpan::new(callee.start_byte(), callee.end_byte()) else {
            continue;
        };
        let Some(resolution @ Resolution::Resolved(_)) =
            resolver.value_reference_for_invocation(file, source, span)
        else {
            continue;
        };
        let proof = super::package_evidence::capture(
            resolver,
            file,
            &[(callee.start_byte(), callee.end_byte(), resolution.clone())],
        );
        if !proof.iter().any(|package| {
            apis.iter()
                .any(|(name, export)| package.name == *name && package.declaration.name == *export)
        }) {
            continue;
        }
        selected.push((callee.start_byte(), callee.end_byte()));
        sites.push((callee.start_byte(), callee.end_byte(), resolution));
        packages.extend(proof);
        if let Some(arguments) = node.child_by_field_name("arguments") {
            let mut cursor = arguments.walk();
            for argument in arguments
                .named_children(&mut cursor)
                .filter(|a| matches!(a.kind(), "identifier" | "member_expression"))
            {
                let Ok(span) = SourceSpan::new(argument.start_byte(), argument.end_byte()) else {
                    continue;
                };
                if let Some(resolution) =
                    resolver.value_reference_for_invocation(file, source, span)
                {
                    sites.push((argument.start_byte(), argument.end_byte(), resolution));
                }
            }
        }
    }
    sites.sort_by_key(|(start, end, _)| (*start, *end));
    sites.dedup();
    selected.sort();
    selected.dedup();
    (sites, packages, selected)
}
