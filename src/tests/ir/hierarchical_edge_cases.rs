// Hierarchical projection edge cases, savings, and metadata round trips.

use super::{make_multi_class_ir, make_single_class_ir};
use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    HierarchicalProjectionError, ProjectionIdentityKind, estimate_savings, hierarchical_to_ir,
    ir_to_hierarchical, try_ir_to_hierarchical,
};
use crate::ir::opcodes::CoreOp;
// ── Identity-Failure Tests ─────────────────────────────────────

#[test]
fn method_without_declared_class_returns_structured_failure() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefMethod(
                "C99".to_string(),
                "M1".to_string(),
                "orphanMethod".to_string(),
            ),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
        ],
    };
    let error = try_ir_to_hierarchical(&ir).expect_err("orphan method must be rejected");

    assert_eq!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "DEF_M owner",
            expected: ProjectionIdentityKind::Class,
            id: "C99".to_string(),
            instruction: 0,
        }
    );
}

#[test]
fn field_without_declared_class_returns_structured_failure() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefField(
                "C99".to_string(),
                "F1".to_string(),
                "orphanField".to_string(),
            ),
            CoreOp::FieldType("F1".to_string(), "$n".to_string()),
        ],
    };
    let error = try_ir_to_hierarchical(&ir).expect_err("orphan field must be rejected");

    assert_eq!(
        error,
        HierarchicalProjectionError::UnresolvedIdentity {
            operation: "DEF_F owner",
            expected: ProjectionIdentityKind::Class,
            id: "C99".to_string(),
            instruction: 0,
        }
    );
}

// ── Savings Estimation Test ─────────────────────────────────────

#[test]
fn test_estimate_savings_non_empty() {
    let ir = make_single_class_ir();
    let (pos_chars, hier_chars, pct) = estimate_savings(&ir);

    assert!(
        pos_chars > 0,
        "Positional encoding should produce characters"
    );
    assert!(
        hier_chars > 0,
        "Hierarchical encoding should produce characters"
    );
    let expected = ((pos_chars as f64 - hier_chars as f64) / pos_chars as f64) * 100.0;
    assert!((pct - expected).abs() < f64::EPSILON);
}

#[test]
fn test_estimate_savings_multi_class() {
    let ir = make_multi_class_ir();
    let (pos_chars, hier_chars, pct) = estimate_savings(&ir);

    assert!(pos_chars > 0);
    assert!(hier_chars > 0);
    let expected = ((pos_chars as f64 - hier_chars as f64) / pos_chars as f64) * 100.0;
    assert!((pct - expected).abs() < f64::EPSILON);
}

// ── Edge Cases ──────────────────────────────────────────────────

#[test]
fn test_class_with_no_methods() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "EmptyService".to_string()),
            CoreOp::ClassModifiers(
                "C1".to_string(),
                vec![crate::ir::opcodes::DeclarationModifier::Export],
            ),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert!(c1.methods.is_empty(), "Class with no methods");
    assert!(c1.fields.is_empty(), "Class with no fields");
    assert_eq!(
        c1.modifiers,
        vec![vec![crate::ir::opcodes::DeclarationModifier::Export]]
    );

    let restored = hierarchical_to_ir(&hir);
    assert_eq!(ir.instructions, restored);
}

#[test]
fn test_imports_only() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::Import("IM1".to_string(), "fs".to_string(), "readFile".to_string()),
            CoreOp::Import("IM2".to_string(), "path".to_string(), "join".to_string()),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    assert!(hir.classes.is_empty(), "No classes for imports-only IR");
    assert_eq!(hir.imports.len(), 2);

    let restored = hierarchical_to_ir(&hir);
    assert_eq!(ir.instructions, restored);
}

#[test]
fn test_method_param_search_across_methods() {
    // Test that Param/Return ops are correctly matched to their method even
    // when the current_method_idx doesn't match the method's ID (cross-method
    // interleaving).
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "MultiMethodService".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "first".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M2".to_string(), "second".to_string()),
            CoreOp::Param(
                "M2".to_string(),
                "P1".to_string(),
                "$s".to_string(),
                "data".to_string(),
            ),
            CoreOp::Return("M2".to_string(), "$v".to_string()),
            CoreOp::Param(
                "M1".to_string(),
                "P2".to_string(),
                "$n".to_string(),
                "count".to_string(),
            ),
            CoreOp::Return("M1".to_string(), "$b".to_string()),
        ],
    };

    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert_eq!(c1.methods.len(), 2);

    // M1 should have 1 param (P2) and return type $b
    let m1 = c1.methods.iter().find(|m| m.id == "M1").unwrap();
    assert_eq!(m1.params.len(), 1, "M1 should have 1 param");
    assert_eq!(m1.params[0][0], "P2", "M1 param should be P2");
    assert_eq!(m1.return_type, Some("$b".to_string()));

    // M2 should have 1 param (P1) and return type $v
    let m2 = c1.methods.iter().find(|m| m.id == "M2").unwrap();
    assert_eq!(m2.params.len(), 1, "M2 should have 1 param");
    assert_eq!(m2.params[0][0], "P1", "M2 param should be P1");
    assert_eq!(m2.return_type, Some("$v".to_string()));

    // The restored order groups M1's ops together and M2's ops together.
    let restored = hierarchical_to_ir(&hir);
    let expected = vec![
        CoreOp::DefClass("C1".to_string(), "MultiMethodService".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "first".to_string()),
        CoreOp::Param(
            "M1".to_string(),
            "P2".to_string(),
            "$n".to_string(),
            "count".to_string(),
        ),
        CoreOp::Return("M1".to_string(), "$b".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M2".to_string(), "second".to_string()),
        CoreOp::Param(
            "M2".to_string(),
            "P1".to_string(),
            "$s".to_string(),
            "data".to_string(),
        ),
        CoreOp::Return("M2".to_string(), "$v".to_string()),
    ];
    assert_eq!(restored, expected, "Cross-method params correctly grouped");
}

#[test]
fn wire_projection_rejects_method_without_declared_class() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![CoreOp::DefMethod(
            "C99".to_string(),
            "M1".to_string(),
            "orphan".to_string(),
        )],
    };
    assert!(matches!(
        try_ir_to_hierarchical(&ir),
        Err(HierarchicalProjectionError::UnresolvedIdentity {
            operation: "DEF_M owner",
            expected: ProjectionIdentityKind::Class,
            ref id,
            instruction: 0,
        }) if id == "C99"
    ));
}

// ── Edit Mode & R-43a Round-Trip Tests (Phase 4) ──────────────────

/// Edit Mode: verbatim method body must survive a hierarchical round-trip.
#[test]
fn test_body_round_trip() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "MyService".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "doWork".to_string()),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
            CoreOp::Body(
                "M1".to_string(),
                "{\n  let x = 1;\n  println!(\"{}\", x);\n}".to_string(),
                // apply_edit Phase 1: absolute byte span of the body slice.
                Some(64),
                Some(101),
            ),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert_eq!(
        c1.methods[0].body.as_deref(),
        Some("{\n  let x = 1;\n  println!(\"{}\", x);\n}")
    );
    // Phase 1: spans must fold into the MethodNode alongside the text.
    assert_eq!(c1.methods[0].body_start, Some(64));
    assert_eq!(c1.methods[0].body_end, Some(101));

    let restored = hierarchical_to_ir(&hir);
    assert_eq!(
        ir.instructions, restored,
        "Body op must survive hierarchical round-trip"
    );
}

/// R-43a: ControlFlow, DataFlow, SideEffect, and ExecutionContext ops
/// must NOT be silently discarded during hierarchical conversion.
#[test]
fn test_r43a_metadata_round_trip() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "MyService".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "process".to_string()),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
            CoreOp::ControlFlow("M1".to_string(), "if".to_string(), "x > 0".to_string()),
            CoreOp::DataFlow("M1".to_string(), "reads".to_string(), "config".to_string()),
            CoreOp::SideEffect("M1".to_string(), "mutation".to_string()),
            CoreOp::ExecutionContext("M1".to_string(), "async".to_string()),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    let m1 = &c1.methods[0];
    assert_eq!(
        m1.control_flow,
        vec![vec!["if".to_string(), "x > 0".to_string()]]
    );
    assert_eq!(
        m1.data_flow,
        vec![vec!["reads".to_string(), "config".to_string()]]
    );
    assert_eq!(m1.side_effect, vec!["mutation".to_string()]);
    assert_eq!(m1.execution_context, vec!["async".to_string()]);

    let restored = hierarchical_to_ir(&hir);
    assert_eq!(
        ir.instructions, restored,
        "R-43a metadata must survive hierarchical round-trip"
    );
}

#[test]
fn test_class_patterns_at_class_level() {
    // Class-level pattern (no current_method)
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "MyService".to_string()),
            CoreOp::Pattern(
                "SOME_PAT".to_string(),
                vec!["C1".to_string(), "arg1".to_string()],
            ),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert_eq!(c1.patterns.len(), 1, "Class-level pattern stored on class");
    assert_eq!(c1.patterns[0].name, "SOME_PAT");
    assert_eq!(
        c1.patterns[0].args,
        vec!["C1".to_string(), "arg1".to_string()]
    );

    let restored = hierarchical_to_ir(&hir);
    assert_eq!(ir.instructions, restored);
}
