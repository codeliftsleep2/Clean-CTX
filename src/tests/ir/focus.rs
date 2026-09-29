use crate::ir::focus::{FocusResolutionError, resolve_focus_method_ids, retain_focused_bodies};
use crate::ir::opcodes::CoreOp;
use crate::ir::{ClassNode, CompiledIR, HierarchicalIR, InterfaceNode, MethodNode};
use std::collections::HashSet;

fn method(id: &str, name: &str) -> MethodNode {
    MethodNode {
        id: id.into(),
        name: name.into(),
        params: Vec::new(),
        return_type: None,
        modifiers: Vec::new(),
        control_summaries: Vec::new(),
        pattern_facts: Vec::new(),
        flags: Vec::new(),
        patterns: Vec::new(),
        body: Some(format!("{{ /* {id} */ }}")),
        body_start: Some(1),
        body_end: Some(2),
        control_flow: Vec::new(),
        data_flow: Vec::new(),
        side_effect: Vec::new(),
        execution_context: Vec::new(),
    }
}

fn class(id: &str, name: &str, methods: Vec<MethodNode>) -> ClassNode {
    ClassNode {
        id: id.into(),
        name: name.into(),
        methods,
        fields: Vec::new(),
        modifiers: Vec::new(),
        class_flags: Vec::new(),
        extends: None,
        implements: Vec::new(),
        injects: Vec::new(),
        patterns: Vec::new(),
        synthetic: false,
    }
}

fn interface(id: &str, name: &str, methods: Vec<MethodNode>) -> InterfaceNode {
    InterfaceNode {
        id: id.into(),
        name: name.into(),
        methods,
        fields: Vec::new(),
        modifiers: Vec::new(),
        extends: Vec::new(),
    }
}

fn selectors(values: &[&str]) -> HashSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn qualified_selector_resolves_typed_owner_to_canonical_method_ids() {
    let hierarchy = HierarchicalIR {
        classes: vec![
            class("C1", "Left", vec![method("M1", "run")]),
            class("C2", "Right", vec![method("M2", "run")]),
        ],
        interfaces: vec![interface("I1", "Contract", vec![method("M3", "run")])],
        imports: Vec::new(),
        type_aliases: Vec::new(),
        calls: Vec::new(),
    };

    let resolved = resolve_focus_method_ids(&hierarchy, &selectors(&["Right.run"])).unwrap();
    assert_eq!(resolved, selectors(&["M2"]));
}

#[test]
fn bare_selector_rejects_cross_owner_ambiguity() {
    let hierarchy = HierarchicalIR {
        classes: vec![
            class("C1", "Left", vec![method("M1", "run")]),
            class("C2", "Right", vec![method("M2", "run")]),
        ],
        interfaces: Vec::new(),
        imports: Vec::new(),
        type_aliases: Vec::new(),
        calls: Vec::new(),
    };

    assert_eq!(
        resolve_focus_method_ids(&hierarchy, &selectors(&["run"])),
        Err(FocusResolutionError::AmbiguousBareMethod("run".into()))
    );
}

#[test]
fn qualified_selector_rejects_duplicate_typed_owner_names() {
    let hierarchy = HierarchicalIR {
        classes: vec![class("C1", "Owner", vec![method("M1", "run")])],
        interfaces: vec![interface("I1", "Owner", vec![method("M2", "run")])],
        imports: Vec::new(),
        type_aliases: Vec::new(),
        calls: Vec::new(),
    };

    assert_eq!(
        resolve_focus_method_ids(&hierarchy, &selectors(&["Owner.run"])),
        Err(FocusResolutionError::AmbiguousOwner("Owner".into()))
    );
}

#[test]
fn qualified_overload_family_resolves_every_canonical_occurrence() {
    let hierarchy = HierarchicalIR {
        classes: vec![class(
            "C1",
            "Owner",
            vec![method("M1", "find"), method("M2", "find")],
        )],
        interfaces: Vec::new(),
        imports: Vec::new(),
        type_aliases: Vec::new(),
        calls: Vec::new(),
    };

    let resolved = resolve_focus_method_ids(&hierarchy, &selectors(&["Owner.find"])).unwrap();
    assert_eq!(resolved, selectors(&["M1", "M2"]));
}

#[test]
fn body_filter_uses_only_resolved_canonical_ids() {
    let mut ir = CompiledIR {
        file_id: "alpha".into(),
        version: 1,
        instructions: vec![
            CoreOp::DefMethod("C1".into(), "M1".into(), "run".into()),
            CoreOp::Body("M1".into(), "{ left(); }".into(), Some(1), Some(12)),
            CoreOp::DefMethod("C2".into(), "M2".into(), "run".into()),
            CoreOp::Body("M2".into(), "{ right(); }".into(), Some(20), Some(32)),
        ],
    };

    retain_focused_bodies(&mut ir, &selectors(&["M2"]));
    assert!(
        !ir.instructions
            .iter()
            .any(|op| matches!(op, CoreOp::Body(id, ..) if id == "M1"))
    );
    assert!(
        ir.instructions
            .iter()
            .any(|op| matches!(op, CoreOp::Body(id, ..) if id == "M2"))
    );
}
