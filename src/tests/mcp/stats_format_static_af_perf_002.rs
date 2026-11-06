//! Finite AST protection for session-wide context_stats text sinks and lock placement.
use tree_sitter::{Node, Parser};

fn compact(node: Node<'_>, source: &str) -> String {
    source[node.byte_range()]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}
fn guarded(mut node: Node<'_>, source: &str) -> bool {
    while let Some(parent) = node.parent() {
        if parent.kind() == "if_expression" {
            let condition = parent.child_by_field_name("condition").unwrap();
            let gate = compact(condition, source);
            if gate == "format!=\"json\""
                || gate.starts_with("format!=\"json\"&&letSome(cache_text)=render_cache_text(")
            {
                let consequence = parent.child_by_field_name("consequence").unwrap();
                if consequence.byte_range().contains(&node.start_byte())
                    || (gate.contains("&&letSome(cache_text)=render_cache_text(")
                        && condition.byte_range().contains(&node.start_byte()))
                {
                    return true;
                }
            }
        }
        node = parent;
    }
    false
}
fn inspect(source: &str) -> Vec<String> {
    let mut parser = Parser::new();
    parser
        .set_language(&crate::compression::language::safe_rust_language().unwrap())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error(), "stats AST parse error");
    let mut findings = Vec::new();
    let mut merged = Vec::new();
    let mut cache = Vec::new();
    fn walk(
        node: Node<'_>,
        source: &str,
        findings: &mut Vec<String>,
        merged: &mut Vec<usize>,
        cache: &mut Vec<usize>,
    ) {
        if node.kind() == "call_expression" {
            let function = compact(node.child_by_field_name("function").unwrap(), source);
            let args = compact(node.child_by_field_name("arguments").unwrap(), source);
            if function.ends_with("render_dashboard_text") {
                match args.as_str() {
                    "(&merged)" => {
                        merged.push(node.start_byte());
                        if !guarded(node, source) {
                            findings.push("unguarded stats text sink".into());
                        }
                    }
                    "(&session)" => {
                        // Existing required text response assembly remains in JSON's else branch.
                        let position =
                            source.find("if format == \"json\" {\n            let mut json");
                        if position.is_none_or(|p| p >= node.start_byte()) {
                            findings.push("session text placement changed".into());
                        }
                    }
                    _ => findings.push(format!("unclassified stats text sink: {args}")),
                }
            }
            if function == "render_cache_text" {
                cache.push(node.start_byte());
                if !guarded(node, source) {
                    findings.push("unguarded stats cache text sink".into());
                }
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, findings, merged, cache);
        }
    }
    walk(
        tree.root_node(),
        source,
        &mut findings,
        &mut merged,
        &mut cache,
    );
    let lock = source.find("let metrics = state.cache_metrics_lock();\n        if format");
    if merged.len() != 1
        || cache.len() != 1
        || lock.is_none()
        || !matches!((merged.first(), lock, cache.first()), (Some(m), Some(l), Some(c)) if *m < l && l < *c)
    {
        findings.push("stats sink/lock scope changed".into());
    }
    if source.matches("drop(metrics);").count() != 2 {
        findings.push("stats lock release scope changed".into());
    }
    findings
}
#[test]
fn af002_stats_static_format_sinks() {
    let source = include_str!("../../mcp/tool_handlers/stats/mod.rs");
    assert!(
        inspect(source).is_empty(),
        "stats format static: {:?}",
        inspect(source)
    );
    let mutation = source.replacen("if format != \"json\"", "if true", 1);
    assert!(!inspect(&mutation).is_empty());
}
