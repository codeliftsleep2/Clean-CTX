//! Finite successful compression IR clone/sink classification, not a global clone ban.
use tree_sitter::{Node, Parser};
fn compact(node: Node<'_>, source: &str) -> String {
    source[node.byte_range()]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}
fn inspect(source: &str) -> Vec<String> {
    let mut parser = Parser::new();
    parser
        .set_language(&crate::compression::language::safe_rust_language().unwrap())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error(), "A1 handler AST parse error");
    let mut findings = Vec::new();
    let mut clones = Vec::new();
    let mut persistence = Vec::new();
    fn walk(
        node: Node<'_>,
        source: &str,
        findings: &mut Vec<String>,
        clones: &mut Vec<usize>,
        persistence: &mut Vec<usize>,
    ) {
        if node.kind() == "let_declaration"
            && let Some(value) = node.child_by_field_name("value")
            && [
                "ir",
                "&ir",
                "&mutir",
                "&ir.instructions",
                "&mutir.instructions",
            ]
            .contains(&compact(value, source).as_str())
        {
            findings.push("A1 unclassified IR alias".into());
        }
        if node.kind() == "call_expression" {
            let function = compact(node.child_by_field_name("function").unwrap(), source);
            let args = node.child_by_field_name("arguments").unwrap();
            if function.starts_with("ir.")
                && function != "ir.clone"
                && function != "ir.file_id.clone_from"
            {
                findings.push(format!("A1 unclassified IR storage call: {function}"));
            }
            if function == "ir.clone" {
                clones.push(node.start_byte());
                let mut parent = node.parent();
                let mut live = false;
                while let Some(ancestor) = parent {
                    if ancestor.kind() == "call_expression" {
                        let f = compact(ancestor.child_by_field_name("function").unwrap(), source);
                        if f.ends_with(".load_ir")
                            && compact(ancestor.child_by_field_name("arguments").unwrap(), source)
                                == "(ir.clone(),Some(source_hash.clone()))"
                        {
                            live = true;
                        }
                    }
                    parent = ancestor.parent();
                }
                if !live {
                    findings.push("A1 redundant or unclassified IR clone".into());
                }
            }
            let allowed = [
                "checked_hierarchy_or_respond",
                "super::content::presentation_document",
                "super::provide_persistence::persist_read_baseline_with_authority",
                "crate::work_observations_af_perf_002::publication_ir_copy",
                "crate::ir::hierarchical::hierarchy_to_wire_reduced",
                "ir_to_wire",
                "crate::ir::positional::ir_to_positional_wire",
            ];
            for argument in args.named_children(&mut args.walk()) {
                if ["ir", "&ir", "&mutir", "ir.instructions", "&ir.instructions"]
                    .contains(&compact(argument, source).as_str())
                    && !allowed.contains(&function.as_str())
                {
                    findings.push(format!("A1 unclassified IR consumer: {function}"));
                }
            }
            if function == "super::provide_persistence::persist_read_baseline_with_authority" {
                persistence.push(node.start_byte());
                let arguments: Vec<_> = args
                    .named_children(&mut args.walk())
                    .map(|n| compact(n, source))
                    .collect();
                if arguments.get(3).map(String::as_str) != Some("&ir") {
                    findings.push("A1 persistence must borrow the existing IR".into());
                }
                let mut parent = node.parent();
                while let Some(ancestor) = parent {
                    if ancestor.kind() == "if_expression" {
                        let condition =
                            compact(ancestor.child_by_field_name("condition").unwrap(), source);
                        if !condition.starts_with("letOk(") || !condition.ends_with("=ir_result") {
                            findings.push("A1 persistence call newly conditional".into());
                        }
                    }
                    parent = ancestor.parent();
                }
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, findings, clones, persistence);
        }
    }
    walk(
        tree.root_node(),
        source,
        &mut findings,
        &mut clones,
        &mut persistence,
    );
    if clones.len() != 1
        || persistence.len() != 1
        || !matches!((persistence.first(),clones.first()),(Some(p),Some(c)) if p<c)
    {
        findings.push("A1 requires one live IR clone after the same persistence call".into());
    }
    findings
}
#[test]
fn afperf_a1_static_handler_clone_sinks() {
    let source = include_str!("../../mcp/tool_handlers/core/compress.rs");
    let findings = inspect(source);
    assert!(findings.is_empty(), "A1 clone static: {findings:?}");
    let mutation = source.replacen(
        "let path_alias =",
        "let redundant = ir.clone();\n        let path_alias =",
        1,
    );
    assert!(!inspect(&mutation).is_empty());
    let delegated = source.replacen(
        "&ir,\n                &semantic_edges",
        "&another_ir,\n                &semantic_edges",
        1,
    );
    assert!(!inspect(&delegated).is_empty());
    let normalizer = include_str!("../../mcp/persistence_ir.rs");
    let normalized: String = normalizer.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalized
            .contains("letmutdurable=ir.clone();durable.file_id=file_path.to_string();durable"),
        "existing persistence normalization clone/owner replacement must remain"
    );
}
