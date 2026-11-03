//! #174 finite reviewed writer inventory and AST insertion-scan enforcement.
use super::{Source, parse, text};
use tree_sitter::Node;
fn visit(node: Node<'_>, source: &Source, function: &str, violations: &mut Vec<String>) {
    let function = if node.kind() == "function_item" {
        text(node.child_by_field_name("name").unwrap(), source)
    } else {
        function
    };
    if node.kind() == "call_expression"
        && let Some(callee) = node.child_by_field_name("function")
        && callee.kind() == "field_expression"
    {
        let method = callee
            .child_by_field_name("field")
            .map(|n| text(n, source))
            .unwrap_or("");
        let receiver = callee
            .child_by_field_name("value")
            .map(|n| text(n, source))
            .unwrap_or("");
        let insertion = matches!(
            function,
            "register_entity" | "register_publisher_key" | "has_physical_occurrence"
        );
        if insertion && matches!(method, "iter" | "iter_mut" | "position" | "binary_search") {
            violations.push(format!(
                "AF174 prohibited insertion scan {}::{function}:{} {receiver}.{method}",
                source.module,
                node.start_position().row + 1
            ));
        }
        let storage = [
            "self.entities",
            "self.name_index",
            "self.file_map",
            "self.name_members",
            "self.occurrence_members",
            "self.publisher_members",
        ];
        if storage.iter().any(|name| receiver.starts_with(name))
            && matches!(
                method,
                "entry" | "get_mut" | "remove" | "insert" | "clear" | "retain"
            )
        {
            let reviewed = source.module == "membership"
                && matches!(
                    function,
                    "register_entity"
                        | "register_publisher_key"
                        | "take_publisher_keys"
                        | "remove_physical_occurrence"
                        | "remove_publisher_key"
                        | "clean_name_membership"
                );
            if !reviewed {
                violations.push(format!(
                    "AF174 REVIEW REQUIRED unclassified paired writer {}::{function}:{}",
                    source.module,
                    node.start_position().row + 1
                ));
            }
        }
    }
    for child in node.named_children(&mut node.walk()) {
        visit(child, source, function, violations);
    }
}
fn inspect(s: &Source) -> Vec<String> {
    let mut findings = Vec::new();
    visit(s.tree.root_node(), s, "", &mut findings);
    findings
}
#[test]
fn af174_static_registration_scans_and_writer_drift() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (module, file) in [
        ("index", "src/workspace/index.rs"),
        ("membership", "src/workspace/index/membership.rs"),
        ("edges", "src/workspace/index/edges.rs"),
        ("remove", "src/workspace/index/remove.rs"),
        ("ngrx", "src/workspace/index/ngrx_refinement.rs"),
    ] {
        let source = parse(module, &std::fs::read_to_string(root.join(file)).unwrap());
        let findings = inspect(&source);
        assert!(findings.is_empty(), "AF174 static gate: {findings:?}");
    }
    let scan = parse(
        "membership",
        "fn register_entity(){self.name_index.iter().any(|e|e==key);}",
    );
    assert!(!inspect(&scan).is_empty());
    let drift = parse(
        "unknown",
        "fn new_writer(){self.file_map.entry(file).or_default();}",
    );
    assert!(
        inspect(&drift)
            .iter()
            .any(|f| f.contains("REVIEW REQUIRED"))
    );
}
