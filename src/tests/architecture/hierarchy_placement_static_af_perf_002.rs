//! Finite classification of the two class-member placement arms, not all scans.
use super::{Source, parse, text};
use tree_sitter::Node;
fn inspect(source: &Source) -> Vec<String> {
    fn classify(node: Node<'_>, source: &Source, findings: &mut Vec<String>, gets: &mut usize) {
        if matches!(
            node.kind(),
            "for_expression" | "while_expression" | "loop_expression"
        ) {
            findings.push("hierarchy placement unclassified loop".into());
        }
        if node.kind() == "call_expression" {
            let function = node.child_by_field_name("function").unwrap();
            let called = text(function, source);
            let allowed = [
                "ClassId::from_serialized",
                "MethodId::from_serialized",
                "FieldId::from_serialized",
                "crate::work_observations_af_perf_002::update",
                "class_locations.get",
                "class_locations.get(&class_id).copied",
                "push_method",
                "push_field",
                "name.clone",
                "method_locations.insert",
                "field_locations.insert",
                "MethodLocation::Class",
                "FieldLocation::Class",
                "pending_methods.entry",
                "pending_methods.entry(class_id).or_default",
                "pending_methods.entry(class_id).or_default().push",
                "pending_fields.entry",
                "pending_fields.entry(class_id).or_default",
                "pending_fields.entry(class_id).or_default().push",
            ];
            let compact: String = called.chars().filter(|c| !c.is_whitespace()).collect();
            if !allowed.contains(&compact.as_str()) {
                findings.push(format!("hierarchy placement unclassified call: {compact}"));
            }
            if compact == "class_locations.get" {
                *gets += 1;
                if text(node.child_by_field_name("arguments").unwrap(), source) != "(&class_id)" {
                    findings.push("hierarchy placement changed lookup key".into());
                }
            }
        }
        for child in node.named_children(&mut node.walk()) {
            classify(child, source, findings, gets);
        }
    }
    fn walk(node: Node<'_>, source: &Source, findings: &mut Vec<String>, arms: &mut usize) {
        if node.kind() == "match_arm" {
            let raw = text(node, source);
            if raw.starts_with("CoreOp::DefMethod(") || raw.starts_with("CoreOp::DefField(") {
                *arms += 1;
                let mut gets = 0;
                classify(node, source, findings, &mut gets);
                if gets != 1 {
                    findings.push(
                        "hierarchy placement requires one classified map lookup per arm".into(),
                    );
                }
                return;
            }
        }
        for child in node.named_children(&mut node.walk()) {
            walk(child, source, findings, arms);
        }
    }
    let mut findings = Vec::new();
    let mut arms = 0;
    walk(source.tree.root_node(), source, &mut findings, &mut arms);
    if arms != 2 {
        findings.push(format!("hierarchy placement scope changed: {arms} arms"));
    }
    findings
}
#[test]
fn afperf_hierarchy_static_placement_sinks() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ir/hierarchical/encode.rs");
    let source = parse("hierarchy_encode", &std::fs::read_to_string(path).unwrap());
    let findings = inspect(&source);
    assert!(
        findings.is_empty(),
        "Hierarchy placement static scope: {findings:?}"
    );
    let fixture = "fn f(){match op {CoreOp::DefMethod(a,b,c)=>{classes.iter().position(|c|c.id==a);},CoreOp::DefField(a,b,c)=>{select_owner(classes);}}}";
    assert!(!inspect(&parse("fixture", fixture)).is_empty());
}
