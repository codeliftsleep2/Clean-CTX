//! Finite public compression economics caller/sink protection.
use super::{Source, parse, text};
use tree_sitter::Node;
fn inspect(source: &Source) -> Vec<String> {
    fn walk(node: Node<'_>, source: &Source, findings: &mut Vec<String>) {
        if node.kind() == "call_expression"
            && let Some(function) = node.child_by_field_name("function")
        {
            let called = text(function, source);
            if called.ends_with("::economical_presentation_document")
                || called.ends_with("::select_with_local_tokenizer")
            {
                findings.push("C1 prohibited raw recount in public compression".into());
            }
            if called.ends_with("::select_with_local_tokenizer_and_raw_tokens") {
                let arguments = node.child_by_field_name("arguments").unwrap();
                let values: Vec<_> = arguments.named_children(&mut arguments.walk()).collect();
                if values.len() != 5 || text(values[4], source) != "raw_tokens" {
                    findings.push("C1 REVIEW REQUIRED precomputed count wiring".into());
                }
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, findings);
        }
    }
    let mut findings = Vec::new();
    walk(source.tree.root_node(), source, &mut findings);
    findings
}
#[test]
fn afperf_c1_static_no_public_recount() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/mcp/tool_handlers/core/compress.rs");
    let source = parse("core_compress", &std::fs::read_to_string(path).unwrap());
    let findings = inspect(&source);
    assert!(findings.is_empty(), "C1 static scope: {findings:?}");
    assert!(
        !inspect(&parse(
            "fixture",
            "fn entry(){content::economical_presentation_document();}"
        ))
        .is_empty()
    );
    assert!(!inspect(&parse("fixture", "fn entry(){economics::select_with_local_tokenizer_and_raw_tokens(raw,candidate,kind,tok,other);}")).is_empty());
}
