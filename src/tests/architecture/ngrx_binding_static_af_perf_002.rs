//! Finite collector call/guard/position inspection; no whole-program completeness claim.
use super::{Source, parse, text};
use tree_sitter::Node;

fn inspect(source: &Source) -> Vec<String> {
    fn walk(node: Node<'_>, source: &Source, function: &str, findings: &mut Vec<String>) {
        let function = if node.kind() == "function_item" {
            text(node.child_by_field_name("name").unwrap(), source)
        } else {
            function
        };
        if node.kind() == "call_expression"
            && let Some(callee) = node.child_by_field_name("function")
            && callee.kind() == "field_expression"
            && callee
                .child_by_field_name("field")
                .is_some_and(|n| text(n, source) == "object_member_value_bindings")
        {
            let mut ancestor = node.parent();
            let mut guarded = false;
            while let Some(parent) = ancestor {
                if parent.kind() == "if_expression"
                    && parent
                        .child_by_field_name("condition")
                        .is_some_and(|n| text(n, source) == "needs_value_bindings")
                {
                    guarded = true;
                }
                if parent.kind() == "function_item" {
                    break;
                }
                ancestor = parent.parent();
            }
            if function != "rebuild_ngrx_import_refinements" || !guarded {
                findings.push("AF173 unguarded or unclassified binding collector".into());
            }
        }
        if node.kind() == "function_item" && function == "rebuild_ngrx_import_refinements" {
            let body = text(node, source);
            let demand = body.find("let needs_value_bindings = candidates.iter().any");
            let collector = body.find("self.object_member_value_bindings()");
            let pass = body.find("for (asserting_file, edge) in &candidates");
            if !matches!((demand, collector, pass), (Some(d), Some(c), Some(p)) if d < c && c < p)
                || body.matches("self.object_member_value_bindings()").count() != 1
                || !body.contains("object_registration::MAP_LAYER")
                || !body.contains("object_actions::SWITCH_VALUE_LAYER")
            {
                findings.push("AF173 REVIEW REQUIRED demand/observation position drift".into());
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, function, findings);
        }
    }
    let mut result = Vec::new();
    walk(source.tree.root_node(), source, "", &mut result);
    result
}

#[test]
fn af173_binding_static_guard_and_observation_position() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/workspace/index/ngrx_refinement.rs");
    let source = parse("ngrx_refinement", &std::fs::read_to_string(path).unwrap());
    let findings = inspect(&source);
    assert!(
        findings.is_empty(),
        "AF173 static binding guard: {findings:?}"
    );
    for fixture in [
        "fn rebuild_ngrx_import_refinements(){let value_bindings=self.object_member_value_bindings();}",
        "fn delegated(){self.object_member_value_bindings();}",
    ] {
        assert!(!inspect(&parse("fixture", fixture)).is_empty());
    }
}
