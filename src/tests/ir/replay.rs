// src/tests/ir/replay.rs
//
// Tests for Phase D: State Replay — client-side state machine for applying
// delta ops to reconstruct IR state.
//
// Tests cover:
//   - FileState construction, operations (append, remove, replace, contains)
//   - ContextState: load_ir, apply deltas (add, remove, modify)
//   - Version chain validation (file.version matched against from_version)
//   - Error cases: unknown file, version mismatch, symbol not found
//   - Sequential deltas: v1→v2→v3 applied in series
//   - Render: render_pretty after state application
//   - Edge cases: empty IRs, duplicate detection, multi-file state

use crate::ir::compiler::CompiledIR;
use crate::ir::delta::{DeltaComputer, IRDelta, DeltaOps, ModOp, primary_key_from_tuple};
use crate::ir::opcodes::CoreOp;
use crate::ir::replay::{ContextState, DeltaError, FileState};
use crate::compression::Fidelity;

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

/// Create an IR with two classes for complex scenarios.
fn multi_class_ir(file_id: &str, version: u64) -> CompiledIR {
    CompiledIR {
        file_id: file_id.to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "ServiceA".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "doA".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::DefClass("C2".into(), "ServiceB".into()),
            CoreOp::DefMethod("C2".into(), "M2".into(), "doB".into()),
            CoreOp::Param("M2".into(), "P1".into(), "$s".into(), "val".into()),
            CoreOp::Return("M2".into(), "$b".into()),
        ],
        version,
    }
}

/// Create a delta for adding a method to C1.
fn add_method_delta(from_version: u64, to_version: u64) -> IRDelta {
    IRDelta {
        file: "a1".to_string(),
        from_version,
        to_version,
        ops: DeltaOps {
            adds: vec![
                vec!["DEF_M".into(), "C1".into(), "M2".into(), "newMethod".into()],
                vec!["SIG".into(), "M2".into(), "P2".into(), "$n".into(), "count".into()],
                vec!["RET".into(), "M2".into(), "$v".into()],
            ],
            mods: vec![],
            dels: vec![],
        },
    }
}

/// Create a delta for removing method M1.
fn remove_method_delta(from_version: u64, to_version: u64) -> IRDelta {
    IRDelta {
        file: "a1".to_string(),
        from_version,
        to_version,
        ops: DeltaOps {
            adds: vec![],
            mods: vec![],
            dels: vec![
                vec!["DEF_M".into(), "C1".into(), "M1".into(), "processData".into()],
                vec!["SIG".into(), "M1".into(), "P1".into(), "$s".into(), "input".into()],
                vec!["RET".into(), "M1".into(), "$b".into()],
                vec!["FLAGS".into(), "M1".into(), "IF".into(), "LOOP".into()],
            ],
        },
    }
}

/// Create a delta for renaming method M1.
fn modify_method_delta(from_version: u64, to_version: u64) -> IRDelta {
    IRDelta {
        file: "a1".to_string(),
        from_version,
        to_version,
        ops: DeltaOps {
            adds: vec![],
            mods: vec![ModOp {
                key: vec!["DEF_M".into(), "C1".into(), "M1".into()],
                replace: vec!["DEF_M".into(), "C1".into(), "M1".into(), "renamedMethod".into()],
            }],
            dels: vec![],
        },
    }
}

// ── FileState Tests ──────────────────────────────────────────────

#[test]
fn file_state_new() {
    let fs = FileState::new(5);
    assert!(fs.instructions.is_empty());
    assert!(fs.index.is_empty());
    assert_eq!(fs.version, 5);
}

#[test]
fn file_state_from_compiled() {
    let ir = baseline_ir("a1", 1);
    let fs = FileState::from_compiled(&ir);

    assert_eq!(fs.version, 1);
    assert_eq!(fs.instructions.len(), 6);
    assert_eq!(fs.index.len(), 6, "index should have 6 entries");

    // Verify index keys
    assert!(fs.index.contains_key("DEF_C:C1"));
    assert!(fs.index.contains_key("DEF_M:C1:M1"));
    assert!(fs.index.contains_key("SIG:M1:P1"));
    assert!(fs.index.contains_key("RET:M1"));
    assert!(fs.index.contains_key("FLAGS:M1"));
    assert!(fs.index.contains_key("IMP:IM1"));

    // Verify instruction tuples
    assert_eq!(fs.instructions[0], vec!["DEF_C", "C1", "SampleService"]);
    assert_eq!(fs.instructions[1], vec!["DEF_M", "C1", "M1", "processData"]);
}

#[test]
fn file_state_append() {
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    fs.append(vec!["DEF_F".into(), "C1".into(), "F1".into(), "items".into()]);

    assert_eq!(fs.instructions.len(), 7);
    assert!(fs.index.contains_key("DEF_F:C1:F1"));
    assert_eq!(fs.instructions[6], vec!["DEF_F", "C1", "F1", "items"]);
}

#[test]
fn file_state_remove_by_key() {
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    // Remove the import instruction
    let removed = fs.remove_by_key(&["IMP".into(), "IM1".into()]);
    assert!(removed, "remove should succeed");

    assert_eq!(fs.instructions.len(), 5);
    assert!(!fs.index.contains_key("IMP:IM1"), "index should not contain removed key");

    // Verify remaining instructions are still correct
    assert_eq!(fs.instructions[0], vec!["DEF_C", "C1", "SampleService"]);
    assert_eq!(fs.instructions[4], vec!["FLAGS", "M1", "IF", "LOOP"]);
}

#[test]
fn file_state_remove_by_key_swap_remove_preserves_index() {
    // F-13: swap_remove optimization must correctly update the index
    // for the element that was swapped into the removed position.
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    // Remove the second instruction (DefMethod at index 1)
    // swap_remove will move the last instruction (Import) into index 1
    let removed = fs.remove_by_key(&["DEF_M".into(), "C1".into(), "M1".into()]);
    assert!(removed, "remove should succeed");

    assert_eq!(fs.instructions.len(), 5, "should have 5 remaining instructions");

    // The index entry for Import (which was swapped into index 1) should be correct
    assert!(fs.index.contains_key("IMP:IM1"), "IMP:IM1 should still be in index");
    let imp_idx = fs.index.get("IMP:IM1").unwrap();
    assert_eq!(*imp_idx, 1, "Import should now be at index 1 (swapped from last position)");

    // All remaining keys should be present and point to valid indices
    assert!(fs.index.contains_key("DEF_C:C1"));
    assert!(fs.index.contains_key("SIG:M1:P1"));
    assert!(fs.index.contains_key("RET:M1"));
    assert!(fs.index.contains_key("FLAGS:M1"));

    // Verify instruction at index 1 is now the Import
    assert_eq!(fs.instructions[1], vec!["IMP", "IM1", "rxjs", "map"]);

    // Verify no duplicate keys in the index
    let mut seen_instructions = std::collections::HashSet::new();
    for (key, &idx) in &fs.index {
        assert!(idx < fs.instructions.len(), "index {} out of bounds for key {}", idx, key);
        let insn = &fs.instructions[idx];
        let computed_key = primary_key_from_tuple(insn);
        assert_eq!(key, &computed_key, "index points to wrong instruction for key {}", key);
        assert!(seen_instructions.insert(idx), "duplicate index {} in map", idx);
    }
}

#[test]
fn file_state_remove_by_key_from_end_no_swap_issues() {
    // F-13: Removing the last element should not cause any swap issues
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    // Remove the last instruction (Import at index 5)
    let removed = fs.remove_by_key(&["IMP".into(), "IM1".into()]);
    assert!(removed, "remove should succeed");

    assert_eq!(fs.instructions.len(), 5);

    // Since we removed the last element, no swap occurred
    // Index should still be consistent
    assert!(!fs.index.contains_key("IMP:IM1"));
    assert!(fs.index.contains_key("DEF_C:C1"));
    assert!(fs.index.contains_key("DEF_M:C1:M1"));

    // Verify last instruction is now FLAGS (was at index 4)
    assert_eq!(fs.instructions[4], vec!["FLAGS", "M1", "IF", "LOOP"]);

    // FLAGS index should now point to index 4
    assert_eq!(*fs.index.get("FLAGS:M1").unwrap(), 4);
}

#[test]
fn file_state_remove_by_key_multiple_times() {
    // F-13: Multiple consecutive swap_removes should maintain a consistent index
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    // Remove Import (last, no swap effect)
    assert!(fs.remove_by_key(&["IMP".into(), "IM1".into()]));

    // Remove DEF_M (will swap with FLAGS)
    assert!(fs.remove_by_key(&["DEF_M".into(), "C1".into(), "M1".into()]));

    assert_eq!(fs.instructions.len(), 4);

    // Verify index consistency after multiple removes
    let mut seen_keys = std::collections::HashSet::new();
    for (key, &idx) in &fs.index {
        assert!(idx < fs.instructions.len(), "idx {} out of bounds", idx);
        let computed = primary_key_from_tuple(&fs.instructions[idx]);
        assert_eq!(key, &computed, "key mismatch for idx {}", idx);
        assert!(seen_keys.insert(key.clone()), "duplicate key {}", key);
    }
    assert_eq!(seen_keys.len(), 4);
}

#[test]
fn file_state_remove_nonexistent() {
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    let removed = fs.remove_by_key(&["DEF_M".into(), "C1".into(), "M99".into()]);
    assert!(!removed, "remove of nonexistent key should fail");

    assert_eq!(fs.instructions.len(), 6, "length should be unchanged");
}

#[test]
fn file_state_replace_by_key() {
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    let replaced = fs.replace_by_key(
        &["DEF_M".into(), "C1".into(), "M1".into()],
        &["DEF_M".into(), "C1".into(), "M1".into(), "renamedMethod".into()],
    );
    assert!(replaced, "replace should succeed");

    assert_eq!(fs.instructions[1], vec!["DEF_M", "C1", "M1", "renamedMethod"]);
    // Index should still be valid
    assert!(fs.index.contains_key("DEF_M:C1:M1"));
}

#[test]
fn file_state_replace_changes_key() {
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    // Replace DefClass C1 with a different class ID
    let replaced = fs.replace_by_key(
        &["DEF_C".into(), "C1".into()],
        &["DEF_C".into(), "C2".into(), "RenamedService".into()],
    );
    assert!(replaced, "replace should succeed");

    // Old key should be gone
    assert!(!fs.index.contains_key("DEF_C:C1"));
    // New key should exist
    assert!(fs.index.contains_key("DEF_C:C2"));
    assert_eq!(fs.instructions[0], vec!["DEF_C", "C2", "RenamedService"]);
}

#[test]
fn file_state_replace_nonexistent() {
    let ir = baseline_ir("a1", 1);
    let mut fs = FileState::from_compiled(&ir);

    let replaced = fs.replace_by_key(
        &["DEF_M".into(), "C1".into(), "M99".into()],
        &["DEF_M".into(), "C1".into(), "M99".into(), "ghost".into()],
    );
    assert!(!replaced, "replace of nonexistent should fail");
}

#[test]
fn file_state_contains_key() {
    let ir = baseline_ir("a1", 1);
    let fs = FileState::from_compiled(&ir);

    assert!(fs.contains_key(&["DEF_C".into(), "C1".into()]));
    assert!(fs.contains_key(&["DEF_M".into(), "C1".into(), "M1".into()]));
    assert!(!fs.contains_key(&["DEF_M".into(), "C1".into(), "M99".into()]));
    assert!(!fs.contains_key(&["DEF_C".into(), "C99".into()]));
}

#[test]
fn file_state_empty_from_compiled() {
    let ir = CompiledIR {
        file_id: "empty".into(),
        instructions: vec![],
        version: 1,
    };
    let fs = FileState::from_compiled(&ir);

    assert_eq!(fs.instructions.len(), 0);
    assert_eq!(fs.index.len(), 0);
    assert_eq!(fs.version, 1);
}

// ── ContextState: load_ir Tests ──────────────────────────────────

#[test]
fn context_state_new() {
    let cs = ContextState::new();
    assert_eq!(cs.version(), 0);
    assert!(cs.file_ids().is_empty());
}

#[test]
fn context_state_load_ir() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    assert!(cs.has_file("a1"));
    assert_eq!(cs.version(), 1);
    assert_eq!(cs.file_ids(), vec!["a1"]);
    assert!(cs.get_ir("a1").is_some());
}

#[test]
fn context_state_load_multiple_files() {
    let ir1 = baseline_ir("a1", 1);
    let ir2 = multi_class_ir("b2", 1);

    let mut cs = ContextState::new();
    cs.load_ir(ir1);
    cs.load_ir(ir2);

    assert!(cs.has_file("a1"));
    assert!(cs.has_file("b2"));
    assert_eq!(cs.file_ids().len(), 2);
    assert_eq!(cs.version(), 1);
}

#[test]
fn context_state_load_ir_updates_version() {
    let ir_v1 = baseline_ir("a1", 1);
    let ir_v2 = multi_class_ir("b2", 3);

    let mut cs = ContextState::new();
    cs.load_ir(ir_v1);
    assert_eq!(cs.version(), 1);

    cs.load_ir(ir_v2);
    assert_eq!(cs.version(), 3, "global version should be max of all loaded IRs");
}

#[test]
fn context_state_load_overwrites_existing() {
    let ir_v1 = baseline_ir("a1", 1);
    let ir_v2 = CompiledIR {
        file_id: "a1".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "NewClass".into()),
        ],
        version: 2,
    };

    let mut cs = ContextState::new();
    cs.load_ir(ir_v1);
    assert_eq!(cs.instruction_count("a1").unwrap(), 6);

    cs.load_ir(ir_v2);
    assert_eq!(cs.instruction_count("a1").unwrap(), 1);
    assert_eq!(cs.version(), 2);
}

// ── ContextState: apply delta Tests ──────────────────────────────

#[test]
fn context_state_apply_add() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    let delta = add_method_delta(1, 2);
    let result = cs.apply(delta).expect("apply should succeed");

    assert_eq!(result, 2);
    assert_eq!(cs.version(), 2);
    assert_eq!(cs.file_version("a1").unwrap(), 2);
    assert_eq!(cs.instruction_count("a1").unwrap(), 9, "6 base + 3 adds");

    // Verify the new instructions are present
    let ir = cs.get_ir("a1").unwrap();
    let has_new_method = ir.iter().any(|t| t[0] == "DEF_M" && t[2] == "M2");
    assert!(has_new_method, "new method M2 should be present");
}

#[test]
fn context_state_apply_remove() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    let delta = remove_method_delta(1, 2);
    let result = cs.apply(delta).expect("apply should succeed");

    assert_eq!(result, 2);
    assert_eq!(cs.instruction_count("a1").unwrap(), 2, "6 base - 4 removals = 2 remaining");

    // Verify M1 instructions are gone
    let ir = cs.get_ir("a1").unwrap();
    let has_m1 = ir.iter().any(|t| t[0] == "DEF_M" && t[2] == "M1");
    assert!(!has_m1, "removed method M1 should not be present");
    let has_flags = ir.iter().any(|t| t[0] == "FLAGS" && t[1] == "M1");
    assert!(!has_flags, "removed FLAGS for M1 should not be present");
}

#[test]
fn context_state_apply_modify() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    let delta = modify_method_delta(1, 2);
    let result = cs.apply(delta).expect("apply should succeed");

    assert_eq!(result, 2);
    assert_eq!(cs.instruction_count("a1").unwrap(), 6, "count should remain same after modify");

    // Verify M1 was renamed
    let ir = cs.get_ir("a1").unwrap();
    let m1_op = ir.iter().find(|t| t[0] == "DEF_M" && t[2] == "M1").unwrap();
    assert_eq!(m1_op[3], "renamedMethod", "M1 should be renamed");
}

#[test]
fn context_state_apply_combined_delta() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // Combined: add, remove, modify in one delta
    let delta = IRDelta {
        file: "a1".to_string(),
        from_version: 1,
        to_version: 2,
        ops: DeltaOps {
            adds: vec![
                vec!["DEF_F".into(), "C1".into(), "F1".into(), "newField".into()],
            ],
            mods: vec![ModOp {
                key: vec!["DEF_M".into(), "C1".into(), "M1".into()],
                replace: vec!["DEF_M".into(), "C1".into(), "M1".into(), "modifiedMethod".into()],
            }],
            dels: vec![
                vec!["IMP".into(), "IM1".into(), "rxjs".into(), "map".into()],
            ],
        },
    };

    cs.apply(delta).expect("combined delta should apply");

    assert_eq!(cs.instruction_count("a1").unwrap(), 6, "6 base - 1 del + 1 add = 6");
    assert!(cs.get_ir("a1").unwrap().iter().any(|t| t[0] == "DEF_F"), "DEF_F should be present");
    assert!(!cs.get_ir("a1").unwrap().iter().any(|t| t[0] == "IMP"), "IMP should be removed");
}

// ── Error Cases ──────────────────────────────────────────────────

#[test]
fn context_state_apply_unknown_file() {
    let mut cs = ContextState::new();

    let delta = IRDelta {
        file: "nonexistent".to_string(),
        from_version: 0,
        to_version: 1,
        ops: DeltaOps::default(),
    };

    let result = cs.apply(delta);
    match result {
        Err(DeltaError::UnknownFile(file)) => assert_eq!(file, "nonexistent"),
        other => panic!("expected UnknownFile error, got: {:?}", other),
    }
}

#[test]
fn context_state_apply_version_mismatch() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // Delta targeting version 0 when file is at version 1
    let delta = IRDelta {
        file: "a1".to_string(),
        from_version: 0,
        to_version: 2,
        ops: DeltaOps::default(),
    };

    let result = cs.apply(delta);
    match result {
        Err(DeltaError::VersionMismatch { expected, got }) => {
            assert_eq!(expected, 1, "expected current version is 1");
            assert_eq!(got, 0, "got from_version 0");
        }
        other => panic!("expected VersionMismatch error, got: {:?}", other),
    }
}

#[test]
fn context_state_apply_symbol_not_found_on_remove() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // Try to remove an instruction that doesn't exist
    let delta = IRDelta {
        file: "a1".to_string(),
        from_version: 1,
        to_version: 2,
        ops: DeltaOps {
            adds: vec![],
            mods: vec![],
            dels: vec![
                vec!["DEF_M".into(), "C1".into(), "M99".into(), "ghostMethod".into()],
            ],
        },
    };

    let result = cs.apply(delta);
    match result {
        Err(DeltaError::SymbolNotFound(key)) => {
            assert!(key.contains("M99"), "error should mention M99");
        }
        other => panic!("expected SymbolNotFound error, got: {:?}", other),
    }
}

#[test]
fn context_state_apply_symbol_not_found_on_modify() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // Try to modify an instruction that doesn't exist
    let delta = IRDelta {
        file: "a1".to_string(),
        from_version: 1,
        to_version: 2,
        ops: DeltaOps {
            adds: vec![],
            mods: vec![ModOp {
                key: vec!["DEF_M".into(), "C1".into(), "M99".into()],
                replace: vec!["DEF_M".into(), "C1".into(), "M99".into(), "ghost".into()],
            }],
            dels: vec![],
        },
    };

    let result = cs.apply(delta);
    match result {
        Err(DeltaError::SymbolNotFound(key)) => {
            assert!(key.contains("M99"), "error should mention M99");
        }
        other => panic!("expected SymbolNotFound error, got: {:?}", other),
    }
}

#[test]
fn context_state_apply_duplicate_symbol() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // Try to add an instruction with a key that already exists
    let delta = IRDelta {
        file: "a1".to_string(),
        from_version: 1,
        to_version: 2,
        ops: DeltaOps {
            adds: vec![
                vec!["DEF_M".into(), "C1".into(), "M1".into(), "duplicate".into()],
            ],
            mods: vec![],
            dels: vec![],
        },
    };

    let result = cs.apply(delta);
    match result {
        Err(DeltaError::DuplicateSymbol(key)) => {
            assert!(key.contains("M1"), "error should mention M1");
        }
        other => panic!("expected DuplicateSymbol error, got: {:?}", other),
    }
}

// ── Sequential Deltas (v1→v2→v3) ─────────────────────────────────

#[test]
fn context_state_sequential_deltas() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // v1 → v2: add method M2
    let delta_1_2 = add_method_delta(1, 2);
    cs.apply(delta_1_2).expect("v1→v2 apply");

    assert_eq!(cs.file_version("a1").unwrap(), 2);
    assert_eq!(cs.instruction_count("a1").unwrap(), 9);
    assert!(cs.get_ir("a1").unwrap().iter().any(|t| t[2] == "M2"));

    // v2 → v3: rename M1
    let delta_2_3 = modify_method_delta(2, 3);
    cs.apply(delta_2_3).expect("v2→v3 apply");

    assert_eq!(cs.file_version("a1").unwrap(), 3);
    assert_eq!(cs.instruction_count("a1").unwrap(), 9); // same count
    let m1 = cs.get_ir("a1").unwrap().iter().find(|t| t[0] == "DEF_M" && t[2] == "M1").unwrap();
    assert_eq!(m1[3], "renamedMethod");

    // v3 → v4: remove M2
    let delta_3_4 = IRDelta {
        file: "a1".to_string(),
        from_version: 3,
        to_version: 4,
        ops: DeltaOps {
            adds: vec![],
            mods: vec![],
            dels: vec![
                vec!["DEF_M".into(), "C1".into(), "M2".into(), "newMethod".into()],
                vec!["SIG".into(), "M2".into(), "P2".into(), "$n".into(), "count".into()],
                vec!["RET".into(), "M2".into(), "$v".into()],
            ],
        },
    };
    cs.apply(delta_3_4).expect("v3→v4 apply");

    assert_eq!(cs.file_version("a1").unwrap(), 4);
    assert_eq!(cs.instruction_count("a1").unwrap(), 6); // back to original count
    assert!(!cs.get_ir("a1").unwrap().iter().any(|t| t[2] == "M2"), "M2 should be gone");

    // Final state should contain: DEF_C C1, DEF_M M1 (renamed), SIG, RET, FLAGS, no import
    let final_ir = cs.get_ir("a1").unwrap();
    assert!(final_ir.iter().any(|t| t[0] == "DEF_C" && t[1] == "C1"));
    assert!(final_ir.iter().any(|t| t[0] == "DEF_M" && t[1] == "C1" && t[2] == "M1" && t[3] == "renamedMethod"));
    assert!(final_ir.iter().any(|t| t[0] == "SIG" && t[1] == "M1"));
    assert!(final_ir.iter().any(|t| t[0] == "RET" && t[1] == "M1"));
    assert!(final_ir.iter().any(|t| t[0] == "FLAGS" && t[1] == "M1"));
}

// ── Render Tests ─────────────────────────────────────────────────

#[test]
fn context_state_render_pretty() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    let rendered = cs.render_pretty("a1", Fidelity::Low);
    assert!(rendered.is_some(), "should render existing file");

    let text = rendered.unwrap();
    // Low fidelity should include $c, method name, etc.
    assert!(text.contains("$c SampleService"), "low fidelity should show class");
    assert!(text.contains("processData"), "should contain method name");
}

#[test]
fn context_state_render_pretty_nonexistent_file() {
    let cs = ContextState::new();
    let rendered = cs.render_pretty("nonexistent", Fidelity::Low);
    assert!(rendered.is_none(), "should return None for unknown file");
}

#[test]
fn context_state_render_after_apply() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    // Add a method
    let delta = add_method_delta(1, 2);
    cs.apply(delta).expect("apply");

    // Render after state change
    let rendered = cs.render_pretty("a1", Fidelity::Low).unwrap();
    assert!(rendered.contains("newMethod"), "should contain added method name");
}

// ── Multi-file State Tests ───────────────────────────────────────

#[test]
fn context_state_multi_file_operations() {
    let ir1 = baseline_ir("file1", 1);
    let ir2 = multi_class_ir("file2", 1);

    let mut cs = ContextState::new();
    cs.load_ir(ir1);
    cs.load_ir(ir2);

    // Both files loaded
    assert!(cs.has_file("file1"));
    assert!(cs.has_file("file2"));
    assert_eq!(cs.file_ids().len(), 2);

    // Apply delta to file1
    let delta = modify_method_delta(1, 2);
    // Need to update file_id in delta to match
    let delta = IRDelta {
        file: "file1".to_string(),
        ..delta
    };
    cs.apply(delta).expect("apply to file1");

    // file2 should be unaffected
    assert_eq!(cs.file_version("file2").unwrap(), 1);
    let file2_ir = cs.get_ir("file2").unwrap();
    assert!(file2_ir.iter().any(|t| t[0] == "DEF_M" && t[2] == "M2")); // M2 still present

    // file1 should have renamed method
    let file1_ir = cs.get_ir("file1").unwrap();
    let m1 = file1_ir.iter().find(|t| t[0] == "DEF_M" && t[2] == "M1").unwrap();
    assert_eq!(m1[3], "renamedMethod");
}

// ── Edge Cases ───────────────────────────────────────────────────

#[test]
fn context_state_empty_ir() {
    let ir = CompiledIR {
        file_id: "empty".into(),
        instructions: vec![],
        version: 1,
    };
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    assert!(cs.has_file("empty"));
    assert_eq!(cs.instruction_count("empty").unwrap(), 0);

    // Apply an empty delta (adds nothing)
    let delta = IRDelta {
        file: "empty".to_string(),
        from_version: 1,
        to_version: 2,
        ops: DeltaOps::default(),
    };

    let result = cs.apply(delta).expect("empty delta should apply");
    assert_eq!(result, 2);
    assert_eq!(cs.instruction_count("empty").unwrap(), 0);
}

#[test]
fn context_state_version_tracking() {
    let mut cs = ContextState::new();
    assert_eq!(cs.version(), 0);

    let ir1 = baseline_ir("a1", 5);
    cs.load_ir(ir1);
    assert_eq!(cs.version(), 5);

    let ir2 = multi_class_ir("b2", 3);
    cs.load_ir(ir2);
    assert_eq!(cs.version(), 5, "should stay at max version (5)");

    let ir3 = CompiledIR {
        file_id: "c3".into(),
        instructions: vec![],
        version: 10,
    };
    cs.load_ir(ir3);
    assert_eq!(cs.version(), 10, "should update to 10");
}

#[test]
fn context_state_remove_file() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    assert!(cs.has_file("a1"));
    let removed = cs.remove_file("a1");
    assert!(removed, "remove_file should return true");
    assert!(!cs.has_file("a1"));

    let removed_again = cs.remove_file("a1");
    assert!(!removed_again, "second remove should return false");
}

#[test]
fn context_state_file_version() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    assert_eq!(cs.file_version("a1").unwrap(), 1);
    assert!(cs.file_version("nonexistent").is_none());

    // After delta, version increases
    let delta = add_method_delta(1, 2);
    cs.apply(delta).expect("apply");
    assert_eq!(cs.file_version("a1").unwrap(), 2);
}

#[test]
fn context_state_get_ir() {
    let ir = baseline_ir("a1", 1);
    let mut cs = ContextState::new();
    cs.load_ir(ir);

    let ir_ref = cs.get_ir("a1");
    assert!(ir_ref.is_some());
    assert_eq!(ir_ref.unwrap().len(), 6);

    assert!(cs.get_ir("nonexistent").is_none());
}

// ── DeltaError Display ───────────────────────────────────────────

#[test]
fn delta_error_display_unknown_file() {
    let err = DeltaError::UnknownFile("test.ts".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("test.ts"));
}

#[test]
fn delta_error_display_version_mismatch() {
    let err = DeltaError::VersionMismatch { expected: 3, got: 1 };
    let msg = format!("{}", err);
    assert!(msg.contains("3"));
    assert!(msg.contains("1"));
}

#[test]
fn delta_error_display_symbol_not_found() {
    let err = DeltaError::SymbolNotFound("M99".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("M99"));
}

#[test]
fn delta_error_display_duplicate() {
    let err = DeltaError::DuplicateSymbol("M1".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("M1"));
}

// ── End-to-End: Full Replay Cycle ────────────────────────────────

#[test]
fn full_replay_cycle() {
    // Simulate a complete edit cycle:
    // 1. Compile original source → IR
    // 2. Load into ContextState
    // 3. Edit source → compile new IR
    // 4. Compute delta
    // 5. Apply delta to ContextState
    // 6. Render and verify

    let v1 = baseline_ir("main.ts", 1);

    // Step 2: Load v1 into state
    let mut cs = ContextState::new();
    cs.load_ir(v1.clone());
    assert_eq!(cs.instruction_count("main.ts").unwrap(), 6);

    // Step 3-4: Create v2 (modified) and compute delta
    let mut v2 = v1.clone();
    v2.version = 2;
    // Remove import
    v2.instructions.retain(|op| !matches!(op, CoreOp::Import(_, _, _)));
    // Add field
    v2.instructions.push(CoreOp::DefField("C1".into(), "F1".into(), "items".into()));
    v2.instructions.push(CoreOp::FieldType("F1".into(), "$n".into()));
    // Rename M1
    for op in &mut v2.instructions {
        if let CoreOp::DefMethod(_, mid, name) = op {
            if mid == "M1" {
                *name = "refactored".to_string();
            }
        }
    }

    let computer = DeltaComputer::new();
    let delta = computer.compute(&v1, &v2).expect("delta should be Some");

    // Step 5: Apply delta
    cs.apply(delta).expect("apply should succeed");

    // Step 6: Verify final state
    assert_eq!(cs.file_version("main.ts").unwrap(), 2);
    assert_eq!(cs.instruction_count("main.ts").unwrap(), 7); // 6 - 1 (imp) + 2 (def_f, field_t) = 7

    let final_ir = cs.get_ir("main.ts").unwrap();
    // No import
    assert!(!final_ir.iter().any(|t| t[0] == "IMP"));
    // Has field
    assert!(final_ir.iter().any(|t| t[0] == "DEF_F" && t[2] == "F1"));
    assert!(final_ir.iter().any(|t| t[0] == "FIELD_T" && t[1] == "F1"));
    // Method renamed
    let m1 = final_ir.iter().find(|t| t[0] == "DEF_M" && t[2] == "M1").unwrap();
    assert_eq!(m1[3], "refactored");

    // Render at low fidelity should show updated content
    let rendered = cs.render_pretty("main.ts", Fidelity::Low).unwrap();
    assert!(rendered.contains("refactored"), "rendered output should have new method name");
}