// src/tests/ir/hierarchical.rs
//
// Round-trip tests for the Scoped Hierarchical IR (Idea #4).

use crate::ir::compiler::CompiledIR;
use crate::ir::hierarchical::{
    estimate_savings, hierarchical_to_ir, ir_to_hierarchical, ir_to_hierarchical_wire, wire_to_ir,
};
use crate::ir::opcodes::CoreOp;

/// Helper: create a simple compiled IR with one class and one method.
fn make_single_class_ir() -> CompiledIR {
    CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "SampleService".to_string()),
            CoreOp::DefMethod(
                "C1".to_string(),
                "M1".to_string(),
                "processData".to_string(),
            ),
            CoreOp::Param(
                "M1".to_string(),
                "P1".to_string(),
                "$s".to_string(),
                "payload".to_string(),
            ),
            CoreOp::Return("M1".to_string(), "$b".to_string()),
            CoreOp::Flags("M1".to_string(), vec!["IF".to_string()]),
        ],
    }
}

/// Helper: create an IR with multiple classes, fields, extends, etc.
fn make_multi_class_ir() -> CompiledIR {
    CompiledIR {
        file_id: "α2".to_string(),
        version: 1,
        instructions: vec![
            // Class 1
            CoreOp::DefClass("C1".to_string(), "BaseService".to_string()),
            CoreOp::ClassFlags("C1".to_string(), vec!["EXPORT".to_string()]),
            CoreOp::DefField("C1".to_string(), "F1".to_string(), "items".to_string()),
            CoreOp::FieldType("F1".to_string(), "$s[]".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "doWork".to_string()),
            CoreOp::Param(
                "M1".to_string(),
                "P1".to_string(),
                "$n".to_string(),
                "count".to_string(),
            ),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
            // Class 2
            CoreOp::DefClass("C2".to_string(), "DerivedService".to_string()),
            CoreOp::Extends("C2".to_string(), "C1".to_string()),
            CoreOp::Implements("C2".to_string(), "IF1".to_string()),
            CoreOp::Injects(
                "C2".to_string(),
                vec!["DEP1".to_string(), "DEP2".to_string()],
            ),
            CoreOp::DefMethod(
                "C2".to_string(),
                "M2".to_string(),
                "handleEvent".to_string(),
            ),
            CoreOp::Return("M2".to_string(), "$b".to_string()),
            CoreOp::Flags("M2".to_string(), vec!["ASYNC".to_string()]),
            // Interface
            CoreOp::DefInterface("IF1".to_string(), "ServiceInterface".to_string()),
            // Imports
            CoreOp::Import("IM1".to_string(), "./module".to_string(), "Foo".to_string()),
            CoreOp::Import(
                "IM2".to_string(),
                "rxjs".to_string(),
                "Observable".to_string(),
            ),
            // Type alias
            CoreOp::TypeAlias("T1".to_string(), "$n".to_string()),
        ],
    }
}

// ── Basic Round-Trip Tests ──────────────────────────────────────

#[test]
fn test_round_trip_single_class() {
    let ir = make_single_class_ir();
    let hir = ir_to_hierarchical(&ir);
    let restored_instructions = hierarchical_to_ir(&hir);

    assert_eq!(
        ir.instructions, restored_instructions,
        "Single class: round-trip must preserve all instructions"
    );
}

#[test]
fn test_round_trip_multi_class() {
    let ir = make_multi_class_ir();
    let hir = ir_to_hierarchical(&ir);
    let restored_instructions = hierarchical_to_ir(&hir);

    // The hierarchical format groups instructions per scope. The restored
    // order is: C1 class+flags → C1 fields → C1 methods → C2 class+rel → C2 fields → C2 methods
    // → IF1 class. This is semantically equivalent — CoreOp semantics don't
    // depend on cross-scope instruction interleaving.
    //
    // Expected order:
    //   C1: DefClass, ClassFlags, DefField, FieldType, DefMethod, Param, Return
    //   C2: DefClass, Extends, Implements, Injects, DefMethod, Return, Flags
    //   IF1: DefClass
    //   Imports, TypeAlias
    let expected = vec![
        // C1
        CoreOp::DefClass("C1".to_string(), "BaseService".to_string()),
        CoreOp::ClassFlags("C1".to_string(), vec!["EXPORT".to_string()]),
        CoreOp::DefField("C1".to_string(), "F1".to_string(), "items".to_string()),
        CoreOp::FieldType("F1".to_string(), "$s[]".to_string()),
        CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "doWork".to_string()),
        CoreOp::Param(
            "M1".to_string(),
            "P1".to_string(),
            "$n".to_string(),
            "count".to_string(),
        ),
        CoreOp::Return("M1".to_string(), "$v".to_string()),
        // C2
        CoreOp::DefClass("C2".to_string(), "DerivedService".to_string()),
        CoreOp::Extends("C2".to_string(), "C1".to_string()),
        CoreOp::Implements("C2".to_string(), "IF1".to_string()),
        CoreOp::Injects(
            "C2".to_string(),
            vec!["DEP1".to_string(), "DEP2".to_string()],
        ),
        CoreOp::DefMethod(
            "C2".to_string(),
            "M2".to_string(),
            "handleEvent".to_string(),
        ),
        CoreOp::Return("M2".to_string(), "$b".to_string()),
        CoreOp::Flags("M2".to_string(), vec!["ASYNC".to_string()]),
        // IF1
        CoreOp::DefClass("IF1".to_string(), "ServiceInterface".to_string()),
        // Imports
        CoreOp::Import("IM1".to_string(), "./module".to_string(), "Foo".to_string()),
        CoreOp::Import(
            "IM2".to_string(),
            "rxjs".to_string(),
            "Observable".to_string(),
        ),
        // Type alias
        CoreOp::TypeAlias("T1".to_string(), "$n".to_string()),
    ];

    assert_eq!(
        restored_instructions, expected,
        "Multi-class: round-trip preserves semantically equivalent ordering"
    );
}

#[test]
fn test_round_trip_empty() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![],
    };
    let hir = ir_to_hierarchical(&ir);
    let restored = hierarchical_to_ir(&hir);

    assert!(hir.classes.is_empty(), "Empty IR → no classes");
    assert!(hir.imports.is_empty(), "Empty IR → no imports");
    assert!(hir.type_aliases.is_empty(), "Empty IR → no type aliases");
    assert_eq!(ir.instructions, restored, "Empty: round-trip must be empty");
}

// ── Structural Verification Tests ───────────────────────────────

#[test]
fn test_hierarchical_structure() {
    let ir = make_multi_class_ir();
    let hir = ir_to_hierarchical(&ir);

    // Check class count
    assert_eq!(hir.classes.len(), 3, "Should have 3 classes (C1, C2, IF1)");

    // Find C1
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert_eq!(c1.name, "BaseService");
    assert_eq!(c1.class_flags, Some(vec!["EXPORT".to_string()]));
    assert_eq!(c1.fields.len(), 1);
    assert_eq!(c1.fields[0].id, "F1");
    assert_eq!(c1.fields[0].name, "items");
    assert_eq!(c1.fields[0].field_type, Some("$s[]".to_string()));
    assert_eq!(c1.methods.len(), 1);
    assert_eq!(c1.methods[0].id, "M1");
    assert_eq!(c1.methods[0].name, "doWork");
    assert!(c1.extends.is_none());
    assert!(c1.implements.is_empty());

    // Find C2
    let c2 = hir.classes.iter().find(|c| c.id == "C2").unwrap();
    assert_eq!(c2.name, "DerivedService");
    assert_eq!(c2.extends, Some("C1".to_string()));
    assert_eq!(c2.implements, vec!["IF1".to_string()]);
    assert_eq!(c2.injects, vec!["DEP1".to_string(), "DEP2".to_string()]);
    assert_eq!(c2.methods.len(), 1);
    assert_eq!(c2.methods[0].id, "M2");
    assert_eq!(c2.methods[0].return_type, Some("$b".to_string()));
    assert_eq!(c2.methods[0].flags, Some(vec!["ASYNC".to_string()]));
}

#[test]
fn test_imports_and_type_aliases() {
    let ir = make_multi_class_ir();
    let hir = ir_to_hierarchical(&ir);

    assert_eq!(hir.imports.len(), 2);
    assert_eq!(
        hir.imports[0],
        vec!["IM1".to_string(), "./module".to_string(), "Foo".to_string()]
    );
    assert_eq!(
        hir.imports[1],
        vec![
            "IM2".to_string(),
            "rxjs".to_string(),
            "Observable".to_string()
        ]
    );

    assert_eq!(hir.type_aliases.len(), 1);
    assert_eq!(
        hir.type_aliases[0],
        vec!["T1".to_string(), "$n".to_string()]
    );
}

// ── Wire Format Tests ───────────────────────────────────────────

#[test]
fn test_wire_format_round_trip() {
    let ir = make_single_class_ir();
    let wire = ir_to_hierarchical_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();

    assert_eq!(ir.file_id, decoded.file_id);
    assert_eq!(ir.version, decoded.version);
    assert_eq!(ir.instructions, decoded.instructions);
}

#[test]
fn test_wire_format_multi_class() {
    let ir = make_multi_class_ir();
    let wire = ir_to_hierarchical_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();

    assert_eq!(ir.file_id, decoded.file_id);
    assert_eq!(ir.version, decoded.version);

    // Normalize: DefInterface becomes DefClass during hierarchical round-trip
    // since interfaces are stored as ClassNode with synthetic=false.
    fn normalize(ops: &[CoreOp]) -> Vec<CoreOp> {
        ops.iter()
            .map(|op| match op {
                CoreOp::DefInterface(id, name) => CoreOp::DefClass(id.clone(), name.clone()),
                other => other.clone(),
            })
            .collect()
    }

    let mut ir_ops = normalize(&ir.instructions);
    let mut decoded_ops = normalize(&decoded.instructions);
    ir_ops.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    decoded_ops.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    assert_eq!(
        ir_ops, decoded_ops,
        "Multi-class wire: same set of ops (DefInterface→DefClass normalized)"
    );
}

#[test]
fn test_wire_format_empty() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![],
    };
    let wire = ir_to_hierarchical_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();

    assert_eq!(ir.instructions, decoded.instructions);
    assert_eq!(decoded.file_id, "α1");
    assert_eq!(decoded.version, 1);
}

#[test]
fn test_wire_format_encoding_field() {
    let ir = make_single_class_ir();
    let wire = ir_to_hierarchical_wire(&ir);

    let encoding = wire.get("encoding").and_then(|v| v.as_str());
    assert_eq!(encoding, Some("hierarchical"));
}

#[test]
fn test_wire_format_json_structure() {
    let ir = make_single_class_ir();
    let wire = ir_to_hierarchical_wire(&ir);

    // Check the JSON structure has the expected keys
    assert!(wire.get("file").is_some(), "Must have 'file' key");
    assert!(wire.get("v").is_some(), "Must have 'v' key");
    assert!(wire.get("encoding").is_some(), "Must have 'encoding' key");
    assert!(wire.get("ir").is_some(), "Must have 'ir' key");

    // Check 'ir' contains expected abbreviated fields
    let ir_val = wire.get("ir").unwrap();
    assert!(
        ir_val.get("c").is_some(),
        "Hierarchical IR must have 'c' (classes)"
    );
}

#[test]
fn test_wire_format_decode_missing_file() {
    let result = wire_to_ir(&serde_json::json!({
        "v": 1,
        "encoding": "hierarchical",
        "ir": {"c": []}
    }));
    assert!(result.is_err(), "Missing 'file' should error");
}

#[test]
fn test_wire_format_decode_missing_v() {
    let result = wire_to_ir(&serde_json::json!({
        "file": "α1",
        "encoding": "hierarchical",
        "ir": {"c": []}
    }));
    assert!(result.is_err(), "Missing 'v' should error");
}

#[test]
fn test_wire_format_decode_missing_ir() {
    let result = wire_to_ir(&serde_json::json!({
        "file": "α1",
        "v": 1,
        "encoding": "hierarchical"
    }));
    assert!(result.is_err(), "Missing 'ir' should error");
}

// ── Pattern Handling Tests ──────────────────────────────────────

#[test]
fn test_pattern_round_trip() {
    // Patterns with args already containing C1/M1 as prefix
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "MyService".to_string()),
            CoreOp::DefMethod(
                "C1".to_string(),
                "M1".to_string(),
                "constructor".to_string(),
            ),
            CoreOp::Pattern("CTOR".to_string(), vec!["C1".to_string(), "M1".to_string()]),
            CoreOp::DefMethod("C1".to_string(), "M2".to_string(), "getData".to_string()),
            CoreOp::Pattern(
                "OBSERVABLE".to_string(),
                vec!["C1".to_string(), "M2".to_string(), "data$".to_string()],
            ),
        ],
    };

    let hir = ir_to_hierarchical(&ir);
    let restored = hierarchical_to_ir(&hir);

    assert_eq!(
        ir.instructions, restored,
        "Pattern ops must survive round-trip"
    );

    // Verify pattern placement
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert_eq!(c1.methods[0].patterns.len(), 1, "M1 should have 1 pattern");
    assert_eq!(c1.methods[0].patterns[0].name, "CTOR");
    assert_eq!(c1.methods[1].patterns.len(), 1, "M2 should have 1 pattern");
    assert_eq!(c1.methods[1].patterns[0].name, "OBSERVABLE");
}

// ── Synthetic Class Tests ───────────────────────────────────────

#[test]
fn test_method_without_prior_class_creates_synthetic() {
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
    let hir = ir_to_hierarchical(&ir);

    assert!(
        !hir.classes.is_empty(),
        "Should create synthetic class for orphan method"
    );
    let c99 = hir.classes.iter().find(|c| c.id == "C99").unwrap();
    assert_eq!(c99.methods.len(), 1);
    assert_eq!(c99.methods[0].name, "orphanMethod");
    assert!(c99.synthetic, "Synthetic class should be marked");

    // Synthetic classes skip DefClass, so restored = DefMethod + Return (no DefClass)
    let expected = vec![
        CoreOp::DefMethod(
            "C99".to_string(),
            "M1".to_string(),
            "orphanMethod".to_string(),
        ),
        CoreOp::Return("M1".to_string(), "$v".to_string()),
    ];
    let restored = hierarchical_to_ir(&hir);
    assert_eq!(
        restored, expected,
        "Synthetic class: DefClass omitted from restored"
    );
}

#[test]
fn test_field_without_prior_class_creates_synthetic() {
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
    let hir = ir_to_hierarchical(&ir);

    assert!(
        !hir.classes.is_empty(),
        "Should create synthetic class for orphan field"
    );
    let c99 = hir.classes.iter().find(|c| c.id == "C99").unwrap();
    assert_eq!(c99.fields.len(), 1);
    assert_eq!(c99.fields[0].name, "orphanField");
    assert!(c99.synthetic, "Synthetic class should be marked");

    // Synthetic classes skip DefClass
    let expected = vec![
        CoreOp::DefField(
            "C99".to_string(),
            "F1".to_string(),
            "orphanField".to_string(),
        ),
        CoreOp::FieldType("F1".to_string(), "$n".to_string()),
    ];
    let restored = hierarchical_to_ir(&hir);
    assert_eq!(
        restored, expected,
        "Synthetic class field: DefClass omitted"
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
    assert!(pct >= 0.0, "Savings percentage should be non-negative");

    // For a single-class IR, hierarchical should be smaller
    assert!(
        hier_chars <= pos_chars,
        "Hierarchical ({}) should be <= positional ({}) for simple IR",
        hier_chars,
        pos_chars
    );
}

#[test]
fn test_estimate_savings_multi_class() {
    let ir = make_multi_class_ir();
    let (pos_chars, hier_chars, pct) = estimate_savings(&ir);

    assert!(pos_chars > 0);
    assert!(hier_chars > 0);
    assert!(pct >= 0.0, "Savings: {:.1}%", pct);

    assert!(
        hier_chars < pos_chars,
        "Hierarchical ({}) should be smaller than positional ({}) for multi-class IR",
        hier_chars,
        pos_chars
    );
}

// ── Edge Cases ──────────────────────────────────────────────────

#[test]
fn test_class_with_no_methods() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "EmptyService".to_string()),
            CoreOp::ClassFlags("C1".to_string(), vec!["EXPORT".to_string()]),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert!(c1.methods.is_empty(), "Class with no methods");
    assert!(c1.fields.is_empty(), "Class with no fields");
    assert_eq!(c1.class_flags, Some(vec!["EXPORT".to_string()]));

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
fn test_wire_round_trip_with_synthetic() {
    let ir = CompiledIR {
        file_id: "α1".to_string(),
        version: 1,
        instructions: vec![CoreOp::DefMethod(
            "C99".to_string(),
            "M1".to_string(),
            "orphan".to_string(),
        )],
    };
    let wire = ir_to_hierarchical_wire(&ir);
    let decoded = wire_to_ir(&wire).unwrap();

    assert_eq!(ir.file_id, decoded.file_id);
    assert_eq!(ir.version, decoded.version);
    assert_eq!(
        ir.instructions, decoded.instructions,
        "Synthetic class wire round-trip must preserve ops without DefClass"
    );
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
            ),
        ],
    };
    let hir = ir_to_hierarchical(&ir);
    let c1 = hir.classes.iter().find(|c| c.id == "C1").unwrap();
    assert_eq!(
        c1.methods[0].body.as_deref(),
        Some("{\n  let x = 1;\n  println!(\"{}\", x);\n}")
    );

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
    assert_eq!(m1.side_effect.as_deref(), Some("mutation"));
    assert_eq!(m1.execution_context.as_deref(), Some("async"));

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
