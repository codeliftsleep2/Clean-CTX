//! Finite A3 authority-selection scope, not a type-resolved whole-program graph.
use super::{Source, parse, text};
use tree_sitter::Node;

fn inspect(source: &Source) -> Vec<String> {
    fn walk(node: Node<'_>, source: &Source, owner: &str, findings: &mut Vec<String>) {
        let mut owner = owner.to_owned();
        if node.kind() == "impl_item" {
            owner = node
                .child_by_field_name("type")
                .map(|n| text(n, source))
                .unwrap_or("")
                .into();
        }
        if node.kind() == "function_item" {
            let name = text(node.child_by_field_name("name").unwrap(), source);
            let identity = format!("{owner}::{name}");
            let parameters = node.child_by_field_name("parameters").unwrap();
            if text(parameters, source).contains("[Declaration]")
                && !matches!(
                    identity.as_str(),
                    "::project_with_authority" | "::resolve" | "DeclarationLookup<'a>::new"
                )
            {
                findings.push(format!(
                    "AF002_A3 REVIEW REQUIRED authority consumer {identity}"
                ));
            }
            owner = identity;
        }
        if node.kind() == "call_expression"
            && let Some(function) = node.child_by_field_name("function")
            && function.kind() == "field_expression"
        {
            let receiver = function
                .child_by_field_name("value")
                .map(|n| text(n, source))
                .unwrap_or("");
            let method = function
                .child_by_field_name("field")
                .map(|n| text(n, source))
                .unwrap_or("");
            if receiver == "declarations"
                && matches!(method, "iter" | "iter_mut" | "into_iter")
                && owner == "::resolve"
            {
                findings.push("AF002_A3 prohibited full-catalog selection".into());
            }
        }
        if owner == "::resolve"
            && node.kind() == "call_expression"
            && let Some(arguments) = node.child_by_field_name("arguments")
            && arguments
                .named_children(&mut arguments.walk())
                .any(|n| text(n, source) == "declarations")
            && let Some(function) = node.child_by_field_name("function")
            && text(function, source) != "DeclarationLookup::new"
        {
            findings.push("AF002_A3 REVIEW REQUIRED delegated authority selection".into());
        }
        // Explicit new aliases/delegation of the complete authority require classification.
        if owner == "::resolve"
            && node.kind() == "let_declaration"
            && let Some(value) = node.child_by_field_name("value")
            && text(value, source) == "declarations"
        {
            findings.push("AF002_A3 REVIEW REQUIRED authority alias".into());
        }
        if owner == "::resolve"
            && node.kind() == "for_expression"
            && let Some(value) = node.child_by_field_name("value")
            && text(value, source).contains("declarations")
        {
            findings.push("AF002_A3 prohibited full-catalog selection".into());
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, &owner, findings);
        }
    }
    let mut findings = Vec::new();
    walk(source.tree.root_node(), source, "", &mut findings);
    findings
}

#[test]
fn af002_a3_static_selection_scope() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ir/csharp_exact_calls.rs");
    let source = parse(
        "csharp_exact_calls",
        &std::fs::read_to_string(path).unwrap(),
    );
    let findings = inspect(&source);
    assert!(findings.is_empty(), "AF002_A3 static scope: {findings:?}");
    for fixture in [
        "fn resolve(declarations: &[Declaration]) { declarations.iter().filter(|d| true); }",
        "fn resolve(declarations: &[Declaration]) { for d in declarations {} }",
        "fn resolve(declarations: &[Declaration]) { let alias = declarations; }",
        "fn new_selection(declarations: &[Declaration]) {}",
        "fn resolve(declarations: &[Declaration]) { unknown(declarations); }",
    ] {
        assert!(!inspect(&parse("fixture", fixture)).is_empty());
    }
}
