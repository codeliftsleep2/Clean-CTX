// src/tests/ir/delta.rs
//
// Tests for Phase C: Delta Transport — instruction-level diff engine.
//
// Tests cover:
//   - Add detection (new method, param, etc.)
//   - Remove detection (deleted method, its SIG/RET ops)
//   - Modification detection (renamed method, changed type)
//   - Identical IR returns None (no unnecessary deltas)
//   - Version chain correctness
//   - JSON serialization with +/-/~ keys
//   - Edge cases: empty IRs, different files, duplicate keys

use crate::ir::compiler::CompiledIR;
use crate::ir::delta::{DeltaComputer, IRDelta};
use crate::ir::opcodes::CoreOp;

// ── Helpers ─────────────────────────────────────────────────────

/// Create a simple baseline IR with one class and one method.
fn baseline_ir(file_id: &str, version: u64) -> CompiledIR {
    CompiledIR {
        file_id: file_id.to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "SampleService".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "processData".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "input".into()),
            CoreOp::Return("M1".into(), "$b".into()),
            CoreOp::Flags("M1".into(), vec!["IF".into(), "LOOP".into()]),
            CoreOp::Import("IM1".into(), "rxjs".into(), "map".into()),
        ],
        version,
    }
}

// ── Add Detection ───────────────────────────────────────────────

#[test]
fn delta_add_method() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Add a new method M2
    current.instructions.push(CoreOp::DefMethod("C1".into(), "M2".into(), "newMethod".into()));
    current.instructions.push(CoreOp::Param("M2".into(), "P2".into(), "$n".into(), "count".into()));
    current.instructions.push(CoreOp::Return("M2".into(), "$v".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.file, "a1");
    assert_eq!(delta.from, 1);
    assert_eq!(delta.to, 2);
    assert!(!delta.ops.adds.is_empty(), "should have additions");
    assert!(delta.ops.mods.is_empty(), "should have no modifications");
    assert!(delta.ops.dels.is_empty(), "should have no deletions");

    // Verify the added instructions are present
    let added_tuples = delta.ops.adds.to_vec();
    let has_def_m = added_tuples.iter().any(|t| t[0] == "DEF_M" && t[2] == "M2");
    assert!(has_def_m, "should contain DefMethod for M2");
    let has_sig = added_tuples.iter().any(|t| t[0] == "SIG" && t[2] == "P2");
    assert!(has_sig, "should contain Param for P2");
    let has_ret = added_tuples.iter().any(|t| t[0] == "RET" && t[1] == "M2");
    assert!(has_ret, "should contain Return for M2");
}

#[test]
fn delta_add_field() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    current.instructions.push(CoreOp::DefField("C1".into(), "F1".into(), "items".into()));
    current.instructions.push(CoreOp::FieldType("F1".into(), "$n".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 2, "should have 2 additions");
    assert!(delta.ops.adds.iter().any(|t| t[0] == "DEF_F"));
    assert!(delta.ops.adds.iter().any(|t| t[0] == "FIELD_T"));
}

#[test]
fn delta_add_class() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    current.instructions.push(CoreOp::DefClass("C2".into(), "AnotherService".into()));
    current.instructions.push(CoreOp::DefMethod("C2".into(), "M3".into(), "doStuff".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 2, "should have 2 additions");
}

// ── Remove Detection ────────────────────────────────────────────

#[test]
fn delta_remove_method() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Remove M1 method instructions (and its SIG/RET/FLAGS)
    current.instructions.retain(|op| match op {
        CoreOp::DefMethod(_, mid, _) => mid != "M1",
        CoreOp::Param(mid, _, _, _) => mid != "M1",
        CoreOp::Return(mid, _) => mid != "M1",
        CoreOp::Flags(tid, _) => tid != "M1",
        _ => true,
    });

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert!(delta.ops.adds.is_empty(), "should have no additions");
    assert!(delta.ops.mods.is_empty(), "should have no modifications");
    assert!(!delta.ops.dels.is_empty(), "should have deletions");

    // Should remove DefMethod, Param, Return, and Flags for M1
    assert_eq!(delta.ops.dels.len(), 4, "should remove 4 instructions (DEF_M, SIG, RET, FLAGS)");
}

#[test]
fn delta_remove_import() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Remove the import instruction
    current.instructions.retain(|op| !matches!(op, CoreOp::Import(_, _, _)));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.dels.len(), 1, "should have 1 deletion");
    assert_eq!(delta.ops.dels[0][0], "IMP");
}

// ── Modification Detection ──────────────────────────────────────

#[test]
fn delta_modify_method_name() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Rename M1 from "processData" to "renamedMethod"
    for op in &mut current.instructions {
        if let CoreOp::DefMethod(_cid, mid, name) = op {
            if mid == "M1" {
                *name = "renamedMethod".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert!(delta.ops.adds.is_empty(), "should have no additions");
    assert_eq!(delta.ops.mods.len(), 1, "should have 1 modification");
    assert!(delta.ops.dels.is_empty(), "should have no deletions");

    // Verify the modification
    let mod_op = &delta.ops.mods[0];
    assert_eq!(mod_op.key, vec!["DEF_M", "C1", "M1"]);
    assert_eq!(mod_op.replace, vec!["DEF_M", "C1", "M1", "renamedMethod"]);
}

#[test]
fn delta_modify_return_type() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Change return type of M1 from "$b" to "$s"
    for op in &mut current.instructions {
        if let CoreOp::Return(mid, ty) = op {
            if mid == "M1" {
                *ty = "$s".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.mods.len(), 1, "should have 1 modification");
    let mod_op = &delta.ops.mods[0];
    assert_eq!(mod_op.key, vec!["RET", "M1"]);
    assert_eq!(mod_op.replace, vec!["RET", "M1", "$s"]);
}

#[test]
fn delta_modify_param_type() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Change param type from "$s" to "$n"
    for op in &mut current.instructions {
        if let CoreOp::Param(mid, pid, ty, name) = op {
            if mid == "M1" && pid == "P1" {
                *ty = "$n".to_string();
                *name = "count".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.mods.len(), 1, "should have 1 modification");
    assert_eq!(delta.ops.mods[0].key, vec!["SIG", "M1", "P1"]);
    assert_eq!(delta.ops.mods[0].replace, vec!["SIG", "M1", "P1", "$n", "count"]);
}

#[test]
fn delta_modify_flags() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Add THROW flag to M1
    for op in &mut current.instructions {
        if let CoreOp::Flags(tid, flags) = op {
            if tid == "M1" {
                flags.push("THROW".to_string());
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.mods.len(), 1, "should have 1 modification");
    assert_eq!(delta.ops.mods[0].key, vec!["FLAGS", "M1"]);
    assert_eq!(delta.ops.mods[0].replace, vec!["FLAGS", "M1", "IF", "LOOP", "THROW"]);
}

// ── No Change Detection ─────────────────────────────────────────

#[test]
fn delta_identical_ir_returns_none() {
    let baseline = baseline_ir("a1", 1);
    let current = baseline.clone();

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current);

    assert!(delta.is_none(), "identical IR should produce no delta");
}

// ── Version Chain ───────────────────────────────────────────────

#[test]
fn delta_version_chain() {
    let v1 = baseline_ir("a1", 1);

    let mut v2 = v1.clone();
    v2.version = 2;
    v2.instructions.push(CoreOp::DefMethod("C1".into(), "M2".into(), "addedInV2".into()));
    v2.instructions.push(CoreOp::Return("M2".into(), "$v".into()));

    let mut v3 = v2.clone();
    v3.version = 3;
    // Rename M1
    for op in &mut v3.instructions {
        if let CoreOp::DefMethod(_, mid, name) = op {
            if mid == "M1" {
                *name = "renamed".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();

    // v1 → v2: adds only
    let delta_1_2 = computer.compute(&v1, &v2).expect("v1→v2 delta");
    assert_eq!(delta_1_2.from, 1);
    assert_eq!(delta_1_2.to, 2);
    assert_eq!(delta_1_2.ops.adds.len(), 2, "v1→v2: 2 adds");

    // v2 → v3: modification only
    let delta_2_3 = computer.compute(&v2, &v3).expect("v2→v3 delta");
    assert_eq!(delta_2_3.from, 2);
    assert_eq!(delta_2_3.to, 3);
    assert_eq!(delta_2_3.ops.mods.len(), 1, "v2→v3: 1 mod");

    // v1 → v3: both adds and mods
    let delta_1_3 = computer.compute(&v1, &v3).expect("v1→v3 delta");
    assert_eq!(delta_1_3.from, 1);
    assert_eq!(delta_1_3.to, 3);
    assert!(!delta_1_3.ops.adds.is_empty(), "v1→v3: has adds");
    assert!(!delta_1_3.ops.mods.is_empty(), "v1→v3: has mods");
}

// ── Combined Operations ─────────────────────────────────────────

#[test]
fn delta_add_remove_modify() {
    let mut v1 = baseline_ir("a1", 1);
    v1.instructions.push(CoreOp::DefMethod("C1".into(), "M2".into(), "toRemove".into()));
    v1.instructions.push(CoreOp::Return("M2".into(), "$v".into()));
    v1.version = 1;

    let mut v2 = v1.clone();
    v2.version = 2;

    // Remove M2 (and its RET)
    v2.instructions.retain(|op| match op {
        CoreOp::DefMethod(_, mid, _) => mid != "M2",
        CoreOp::Return(mid, _) => mid != "M2",
        _ => true,
    });

    // Add M3
    v2.instructions.push(CoreOp::DefMethod("C1".into(), "M3".into(), "added".into()));
    v2.instructions.push(CoreOp::Return("M3".into(), "$s".into()));

    // Modify M1
    for op in &mut v2.instructions {
        if let CoreOp::DefMethod(_, mid, name) = op {
            if mid == "M1" {
                *name = "modified".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&v1, &v2).expect("delta should be Some");

    assert!(!delta.ops.dels.is_empty(), "should have deletions");
    assert!(!delta.ops.adds.is_empty(), "should have additions");
    assert!(!delta.ops.mods.is_empty(), "should have modifications");

    // Verify specific ops
    assert!(delta.ops.dels.iter().any(|t| t[1] == "M2"), "M2 should be in deletions");
    assert!(delta.ops.adds.iter().any(|t| t[2] == "M3"), "M3 should be in additions");
    assert!(delta.ops.mods.iter().any(|m| m.key.len() > 2 && m.key[2] == "M1"), "M1 should be in modifications");
}

// ── JSON Serialization ──────────────────────────────────────────

#[test]
fn delta_json_serialization_add() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;
    current.instructions.push(CoreOp::DefMethod("C1".into(), "M2".into(), "newMethod".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    let json = serde_json::to_value(&delta).expect("should serialize to JSON");

    assert_eq!(json["file"], "a1");
    assert_eq!(json["from"], 1);
    assert_eq!(json["to"], 2);
    assert!(json["ops"]["+"].is_array(), "should have adds array");
    assert!(json["ops"]["~"].is_array(), "should have mods array");
    assert!(json["ops"]["-"].is_array(), "should have dels array");
    assert!(!json["ops"]["+"].as_array().unwrap().is_empty(), "adds should not be empty");
}

#[test]
fn delta_json_serialization_mod() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    for op in &mut current.instructions {
        if let CoreOp::Return(mid, ty) = op {
            if mid == "M1" {
                *ty = "$s".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");
    let json = serde_json::to_value(&delta).expect("should serialize to JSON");

    let mods = json["ops"]["~"].as_array().unwrap();
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0]["k"], serde_json::json!(["RET", "M1"]));
    assert_eq!(mods[0]["r"], serde_json::json!(["RET", "M1", "$s"]));
}

#[test]
fn delta_json_serialization_del() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;
    current.instructions.retain(|op| !matches!(op, CoreOp::Import(_, _, _)));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");
    let json = serde_json::to_value(&delta).expect("should serialize to JSON");

    let dels = json["ops"]["-"].as_array().unwrap();
    assert_eq!(dels.len(), 1);
    assert_eq!(dels[0], serde_json::json!(["IMP", "IM1", "rxjs", "map"]));
}

#[test]
fn delta_json_serialization_round_trip() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 3;
    // Add, remove, modify
    current.instructions.push(CoreOp::DefMethod("C1".into(), "M2".into(), "added".into()));
    current.instructions.retain(|op| !matches!(op, CoreOp::Import(_, _, _)));
    for op in &mut current.instructions {
        if let CoreOp::Return(mid, ty) = op {
            if mid == "M1" {
                *ty = "$n".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");
    let json = serde_json::to_value(&delta).expect("serialize");
    let restored: IRDelta = serde_json::from_value(json).expect("deserialize");

    assert_eq!(restored.file, delta.file);
    assert_eq!(restored.from, delta.from);
    assert_eq!(restored.to, delta.to);
    assert_eq!(restored.ops.adds.len(), delta.ops.adds.len());
    assert_eq!(restored.ops.mods.len(), delta.ops.mods.len());
    assert_eq!(restored.ops.dels.len(), delta.ops.dels.len());
}

// ── Edge Cases ──────────────────────────────────────────────────

#[test]
fn delta_empty_instructions() {
    let baseline = CompiledIR {
        file_id: "empty".into(),
        instructions: vec![],
        version: 1,
    };
    let mut current = baseline.clone();
    current.version = 2;
    current.instructions.push(CoreOp::DefClass("C1".into(), "NewClass".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 1, "should have 1 addition");
    assert!(delta.ops.dels.is_empty(), "should have no deletions");
}

#[test]
fn delta_both_empty_returns_none() {
    let baseline = CompiledIR {
        file_id: "empty".into(),
        instructions: vec![],
        version: 1,
    };
    let current = CompiledIR {
        file_id: "empty".into(),
        instructions: vec![],
        version: 2,
    };

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current);
    assert!(delta.is_none(), "both empty IRs should produce no delta");
}

#[test]
fn delta_interface_added() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    current.instructions.push(CoreOp::DefInterface("I1".into(), "MyInterface".into()));
    current.instructions.push(CoreOp::Implements("C1".into(), "I1".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 2, "should have 2 additions");
    assert!(delta.ops.adds.iter().any(|t| t[0] == "DEF_I"));
    assert!(delta.ops.adds.iter().any(|t| t[0] == "IMPL"));
}

#[test]
fn delta_extends_added() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    current.instructions.push(CoreOp::Extends("C1".into(), "C2".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 1);
    assert_eq!(delta.ops.adds[0], vec!["EXT", "C1", "C2"]);
}

#[test]
fn delta_injects_added() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    current.instructions.push(CoreOp::Injects("C1".into(), vec!["S1".into(), "S2".into()]));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 1);
    assert_eq!(delta.ops.adds[0], vec!["INJECTS", "C1", "S1", "S2"]);
}

#[test]
fn delta_type_alias_added() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    current.instructions.push(CoreOp::TypeAlias("T1".into(), "UserId".into()));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 1);
    assert_eq!(delta.ops.adds[0], vec!["TYPE", "T1", "UserId"]);
}

#[test]
fn delta_class_flags_modified() {
    let baseline = baseline_ir("a1", 1);
    let mut current = baseline.clone();
    current.version = 2;

    // Add class-level flags
    current.instructions.push(CoreOp::ClassFlags("C1".into(), vec!["EXPORT".into()]));

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.adds.len(), 1);
    assert_eq!(delta.ops.adds[0], vec!["FLAGS_C", "C1", "EXPORT"]);
}

#[test]
fn delta_field_type_modified() {
    let mut baseline = baseline_ir("a1", 1);
    baseline.instructions.push(CoreOp::DefField("C1".into(), "F1".into(), "count".into()));
    baseline.instructions.push(CoreOp::FieldType("F1".into(), "$n".into()));
    baseline.version = 1;

    let mut current = baseline.clone();
    current.version = 2;

    // Change field type
    for op in &mut current.instructions {
        if let CoreOp::FieldType(fid, ty) = op {
            if fid == "F1" {
                *ty = "$s".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");

    assert_eq!(delta.ops.mods.len(), 1);
    assert_eq!(delta.ops.mods[0].replace, vec!["FIELD_T", "F1", "$s"]);
}

// ── F-10: IMPL Reordering (Set Semantics) ───────────────────────
//
// IMPL keys use set semantics: `class_id:interface_id`. If the interface list is
// reordered across edits (e.g., implements A,B → implements B,A), the same three
// IMPL keys are produced and the delta shows no change. This is semantically correct
// for set semantics — the visible interface list is the same set — but the spec
// documents this behavior in §13. Tests confirm no false-positive deltas on reorder.

#[test]
fn delta_impl_reorder_no_false_positive() {
    // Baseline: class C1 implements A, B
    let baseline = CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Service".into()),
            CoreOp::Implements("C1".into(), "A".into()),
            CoreOp::Implements("C1".into(), "B".into()),
        ],
        version: 1,
    };

    // Current: class C1 implements B, A (reordered)
    let current = CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Service".into()),
            CoreOp::Implements("C1".into(), "B".into()),
            CoreOp::Implements("C1".into(), "A".into()),
        ],
        version: 2,
    };

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current);

    // Set semantics: same interface set, no delta
    assert!(delta.is_none(), "IMPL reorder should produce no delta (set semantics)");
}

#[test]
fn delta_impl_add_interface() {
    let baseline = CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Service".into()),
            CoreOp::Implements("C1".into(), "A".into()),
        ],
        version: 1,
    };

    let current = CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Service".into()),
            CoreOp::Implements("C1".into(), "A".into()),
            CoreOp::Implements("C1".into(), "B".into()),
        ],
        version: 2,
    };

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");
    assert_eq!(delta.ops.adds.len(), 1);
    assert_eq!(delta.ops.adds[0], vec!["IMPL", "C1", "B"]);
}

// ── F-12: FLAGS Collision Prevention ─────────────────────────────
//
// FLAGS key uses `target_id` only. Two methods in the same class can each have
// one FLAGS op with unique target_id (M1, M2), so keys do NOT collide across methods.
// However, if the compiler emits two FLAGS ops for the same target (e.g., one from
// control flow and one from a pattern recognizer), the BTreeMap index will overwrite.
// The fix is to ensure the compiler merges FLAGS for the same target — tested at
// the compiler level in compiler.rs tests. At the delta level, we test that two
// distinct methods with distinct FLAGS keys are handled correctly.

#[test]
fn delta_flags_two_methods_no_collision() {
    let baseline = CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Service".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "doA".into()),
            CoreOp::Flags("M1".into(), vec!["IF".into()]),
            CoreOp::DefMethod("C1".into(), "M2".into(), "doB".into()),
            CoreOp::Flags("M2".into(), vec!["LOOP".into()]),
        ],
        version: 1,
    };

    // Modify M1's flags
    let mut current = baseline.clone();
    current.version = 2;
    for op in &mut current.instructions {
        if let CoreOp::Flags(tid, flags) = op {
            if tid == "M1" {
                flags.push("THROW".to_string());
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");
    assert_eq!(delta.ops.mods.len(), 1);
    assert_eq!(delta.ops.mods[0].key, vec!["FLAGS", "M1"]);
    assert!(delta.ops.mods[0].replace.contains(&"THROW".to_string()));
    // M2's flags should be untouched
    assert!(delta.ops.adds.is_empty());
    assert!(delta.ops.dels.is_empty());
}

// ── F-11: INJECTS Dep Changes (replace behavior) ────────────────
//
// INJECTS key is `class_id` only — the entire deps list is replaced on change.
// There is no per-dep add/remove granularity. This is a documented deviation.

#[test]
fn delta_injects_dep_change_replaces() {
    let baseline = CompiledIR {
        file_id: "a1".into(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Service".into()),
            CoreOp::Injects("C1".into(), vec!["S1".into(), "S2".into()]),
        ],
        version: 1,
    };

    let mut current = baseline.clone();
    current.version = 2;
    // Change deps: remove S1, add S3
    for op in &mut current.instructions {
        if let CoreOp::Injects(cid, deps) = op {
            if cid == "C1" {
                *deps = vec!["S2".into(), "S3".into()];
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&baseline, &current).expect("delta should be Some");
    assert_eq!(delta.ops.mods.len(), 1);
    assert_eq!(delta.ops.mods[0].key, vec!["INJECTS", "C1"]);
    assert_eq!(delta.ops.mods[0].replace, vec!["INJECTS", "C1", "S2", "S3"]);
    assert!(delta.ops.adds.is_empty());
    assert!(delta.ops.dels.is_empty());
}

// ── Primary Key Helpers ─────────────────────────────────────────

#[test]
fn test_primary_key_from_tuple() {
    use crate::ir::delta::primary_key_from_tuple;

    assert_eq!(primary_key_from_tuple(["DEF_C".into(), "C1".into(), "Foo".into()].as_slice()), "DEF_C:C1");
    assert_eq!(
        primary_key_from_tuple(["DEF_M".into(), "C1".into(), "M1".into(), "bar".into()].as_slice()),
        "DEF_M:C1:M1"
    );
    assert_eq!(
        primary_key_from_tuple(["DEF_F".into(), "C1".into(), "F1".into(), "x".into()].as_slice()),
        "DEF_F:C1:F1"
    );
    assert_eq!(
        primary_key_from_tuple(["SIG".into(), "M1".into(), "P1".into(), "$s".into(), "name".into()].as_slice()),
        "SIG:M1:P1"
    );
    assert_eq!(
        primary_key_from_tuple(["RET".into(), "M1".into(), "$b".into()].as_slice()),
        "RET:M1"
    );
    assert_eq!(
        primary_key_from_tuple(["FLAGS".into(), "M1".into(), "IF".into()].as_slice()),
        "FLAGS:M1"
    );
    assert_eq!(
        primary_key_from_tuple(["IMP".into(), "IM1".into(), "rxjs".into(), "map".into()].as_slice()),
        "IMP:IM1"
    );
    assert!(primary_key_from_tuple([].as_slice()).is_empty());
}

#[test]
fn test_key_tuple_from_tuple() {
    use crate::ir::delta::key_tuple_from_tuple;

    assert_eq!(
        key_tuple_from_tuple(["DEF_C".into(), "C1".into(), "Foo".into()].as_slice()),
        vec!["DEF_C", "C1"]
    );
    assert_eq!(
        key_tuple_from_tuple(["DEF_M".into(), "C1".into(), "M1".into(), "bar".into()].as_slice()),
        vec!["DEF_M", "C1", "M1"]
    );
    assert_eq!(
        key_tuple_from_tuple(["IMPL".into(), "C1".into(), "I1".into()].as_slice()),
        vec!["IMPL", "C1", "I1"]
    );
    assert!(key_tuple_from_tuple([].as_slice()).is_empty());
}