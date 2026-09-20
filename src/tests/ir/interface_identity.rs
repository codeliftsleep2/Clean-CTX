use super::*;
use crate::compression::Fidelity;
use crate::ir::compiler::CompiledIR;
use crate::ir::identity::validate_identity_graph;
use crate::ir::opcodes::{CoreOp, DeclarationModifier};

fn interface_ir() -> CompiledIR {
    CompiledIR {
        file_id: "interfaces.ts".into(),
        version: 9,
        instructions: vec![
            CoreOp::Param("M1".into(), "P1".into(), "string".into(), "value".into()),
            CoreOp::FieldType("F1".into(), "number".into()),
            CoreOp::InterfaceModifiers("I1".into(), vec![DeclarationModifier::Export]),
            CoreOp::InterfaceExtends("I1".into(), "BaseApi".into()),
            CoreOp::DefInterfaceMethod("I1".into(), "M1".into(), "run".into()),
            CoreOp::DefInterfaceField("I1".into(), "F1".into(), "version".into()),
            CoreOp::DefInterface("I1".into(), "WorkerApi".into()),
            CoreOp::DefClass("C1".into(), "Worker".into()),
            CoreOp::DefMethod("C1".into(), "M2".into(), "run".into()),
            CoreOp::DefField("C1".into(), "F2".into(), "version".into()),
        ],
    }
}

#[test]
fn interface_members_project_by_identity_independent_of_legal_order() {
    let hierarchy = try_ir_to_hierarchical(&interface_ir()).expect("checked interface projection");
    assert_eq!(hierarchy.classes.len(), 1);
    assert_eq!(hierarchy.classes[0].name, "Worker");
    assert_eq!(hierarchy.classes[0].methods[0].id, "M2");
    assert_eq!(hierarchy.classes[0].fields[0].id, "F2");
    assert_eq!(hierarchy.interfaces.len(), 1);
    let interface = &hierarchy.interfaces[0];
    assert_eq!(interface.name, "WorkerApi");
    assert_eq!(interface.methods[0].name, "run");
    assert_eq!(interface.methods[0].params[0][2], "value");
    assert_eq!(interface.fields[0].field_type.as_deref(), Some("number"));
    assert_eq!(interface.extends, ["BaseApi"]);
}

#[test]
fn interface_owners_are_not_interchangeable_with_class_owners() {
    let wrong_interface_owner = CompiledIR {
        file_id: "wrong.ts".into(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Concrete".into()),
            CoreOp::DefInterfaceMethod("C1".into(), "M1".into(), "run".into()),
        ],
    };
    assert!(validate_identity_graph(&wrong_interface_owner).is_err());

    let wrong_class_owner = CompiledIR {
        file_id: "wrong.ts".into(),
        version: 1,
        instructions: vec![
            CoreOp::DefInterface("I1".into(), "Api".into()),
            CoreOp::DefMethod("I1".into(), "M1".into(), "run".into()),
        ],
    };
    assert!(validate_identity_graph(&wrong_class_owner).is_err());
}

#[test]
fn class_and_interface_identity_collision_fails_structurally() {
    let collision = CompiledIR {
        file_id: "collision.ts".into(),
        version: 1,
        instructions: vec![
            CoreOp::DefClass("T1".into(), "Concrete".into()),
            CoreOp::DefInterface("T1".into(), "Api".into()),
        ],
    };
    assert!(validate_identity_graph(&collision).is_err());
}

#[test]
fn binary_v04_and_hierarchy_preserve_interface_semantics() {
    let ir = interface_ir();
    for operation in &ir.instructions {
        let tuple = crate::ir::wire::op_to_tuple(operation);
        assert_eq!(
            crate::ir::wire::tuple_to_op(&tuple).as_ref(),
            Some(operation)
        );
        match tuple[0].as_str() {
            "DEF_IM" => assert_eq!(
                crate::ir::delta::primary_key_from_tuple(&tuple),
                "DEF_IM:I1:M1"
            ),
            "DEF_IF" => assert_eq!(
                crate::ir::delta::primary_key_from_tuple(&tuple),
                "DEF_IF:I1:F1"
            ),
            "MOD_I" => assert_eq!(
                crate::ir::delta::key_tuple_from_tuple(&tuple),
                ["MOD_I", "I1"]
            ),
            "EXT_I" => assert_eq!(
                crate::ir::delta::key_tuple_from_tuple(&tuple),
                ["EXT_I", "I1"]
            ),
            _ => {}
        }
    }
    let bytes = crate::ir::binary_wire::encode(&ir);
    assert_eq!(&bytes[..3], &[0xCC, 0x02, 0x04]);
    assert_eq!(crate::ir::binary_wire::decode(&bytes).unwrap(), ir);

    let hierarchy = try_ir_to_hierarchical(&ir).unwrap();
    let rendered = crate::ir::render_llm::render_hierarchical_for_llm(&hierarchy, Fidelity::Low);
    assert!(rendered.contains("// Q=interface\nQ WorkerApi\n"));
    assert!(rendered.contains("// ── Worker ──\n"));
    assert!(!rendered.contains("// ── WorkerApi ──"));
}
