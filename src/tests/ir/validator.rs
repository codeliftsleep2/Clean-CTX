// src/tests/ir/validator.rs
//
// Tests for R-43b Phase 5: IR Validation Engine

use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use crate::ir::validator::{DefaultValidator, IRValidator, ValidationError};

fn valid_ir() -> CompiledIR {
    CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "UserService".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "getUser".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "id".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::Flags("M1".into(), vec!["ASYNC".into()]),
            CoreOp::Extends("C1".into(), "BaseService".into()),
            CoreOp::Implements("C1".into(), "IUserService".into()),
            CoreOp::Injects("C1".into(), vec!["IUserRepo".into()]),
            CoreOp::DataFlow("M1".into(), "reads".into(), "userRepo".into()),
            CoreOp::ControlFlow("M1".into(), "if".into(), "condition".into()),
            CoreOp::SideEffect("M1".into(), "async".into()),
            CoreOp::ExecutionContext("M1".into(), "async".into()),
        ],
        version: 1,
    }
}

#[test]
fn test_valid_ir_no_errors() {
    let ir = valid_ir();
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(
        errors.is_empty(),
        "valid IR should have no errors: {:?}",
        errors
    );
}

#[test]
fn test_ret_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::Return("M99".into(), "$v".into()),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E001"));
}

#[test]
fn test_param_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::Param("M99".into(), "P1".into(), "$s".into(), "x".into()),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E002"));
}

#[test]
fn test_flags_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::Flags("M99".into(), vec!["ASYNC".into()]),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E003"));
}

#[test]
fn test_extends_unknown_class() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![CoreOp::Extends("C99".into(), "Base".into())],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E004"));
}

#[test]
fn test_implements_unknown_class() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![CoreOp::Implements("C99".into(), "Iface".into())],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E005"));
}

#[test]
fn test_injects_unknown_class() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![CoreOp::Injects("C99".into(), vec!["Dep".into()])],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E006"));
}

#[test]
fn test_dataflow_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::DataFlow("M99".into(), "reads".into(), "x".into()),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E007"));
}

#[test]
fn test_controlflow_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::ControlFlow("M99".into(), "if".into(), "x".into()),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E008"));
}

#[test]
fn test_side_effect_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::SideEffect("M99".into(), "io".into()),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E009"));
}

#[test]
fn test_ctx_unknown_method() {
    let ir = CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Test".into()),
            CoreOp::ExecutionContext("M99".into(), "async".into()),
        ],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == "E010"));
}

// ── E011: native call facts (RED-CALL22) ────────────────────────────

#[test]
fn test_call_with_declared_caller_is_valid() {
    let ir = CompiledIR {
        file_id: "Example.cs".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Example".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
            // An unresolved CALLEE is a truthful fact: it is a call-site NAME,
            // never resolved, and must never be reported as an error.
            CoreOp::Call("M1".into(), "NeverDeclaredHere".into(), 3),
        ],
        version: 1,
    };
    let errors = DefaultValidator::new().validate(&ir);
    assert!(
        errors.is_empty(),
        "a call whose caller exists must validate: {errors:?}"
    );
}

#[test]
fn test_call_with_unknown_caller_is_e011() {
    let ir = CompiledIR {
        file_id: "Example.cs".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Example".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
            CoreOp::Call("M99".into(), "Save".into(), 1),
        ],
        version: 1,
    };
    let errors = DefaultValidator::new().validate(&ir);
    assert!(
        errors.iter().any(|e| e.code == "E011"),
        "an orphaned caller must be reported as E011: {errors:?}"
    );
    let e011 = errors
        .iter()
        .find(|e| e.code == "E011")
        .expect("E011 present");
    assert!(
        e011.message.contains("M99"),
        "the diagnostic must name the caller: {}",
        e011.message
    );
    assert_eq!(e011.instruction_index, Some(2));
}

#[test]
fn test_empty_ir() {
    let ir = CompiledIR {
        file_id: "empty.ts".to_string(),
        instructions: vec![],
        version: 1,
    };
    let validator = DefaultValidator::new();
    let errors = validator.validate(&ir);
    assert!(errors.is_empty());
}

#[test]
fn test_validation_error_display() {
    let err = ValidationError {
        code: "E001".to_string(),
        message: "test error".to_string(),
        instruction_index: Some(5),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("E001"));
    assert!(msg.contains("test error"));
}
