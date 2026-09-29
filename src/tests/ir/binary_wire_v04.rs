use super::*;

fn assert_ir_eq(expected: &CompiledIR, actual: &CompiledIR) {
    assert_eq!(actual.file_id, expected.file_id);
    assert_eq!(actual.version, expected.version);
    assert_eq!(actual.instructions, expected.instructions);
}

fn all_variants_with_duplicates() -> CompiledIR {
    let duplicate = CoreOp::SideEffect("M1".into(), SideEffectKind::Io);
    CompiledIR {
        file_id: "src/semantic.ts".into(),
        version: 42,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Owner".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
            CoreOp::DefMethod("C1".into(), "M2".into(), "expression".into()),
            CoreOp::DefField("C1".into(), "F1".into(), "value".into()),
            CoreOp::DefInterface("I1".into(), "Runnable".into()),
            CoreOp::DefInterfaceMethod("I1".into(), "M3".into(), "execute".into()),
            CoreOp::DefInterfaceField("I1".into(), "F2".into(), "version".into()),
            CoreOp::InterfaceModifiers("I1".into(), vec![DeclarationModifier::Export]),
            CoreOp::InterfaceExtends("I1".into(), "BaseInterface".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "input".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::FieldType("F1".into(), "$s".into()),
            CoreOp::MethodModifiers(
                "M1".into(),
                vec![DeclarationModifier::Static, DeclarationModifier::Async],
            ),
            CoreOp::ClassModifiers("C1".into(), vec![DeclarationModifier::Export]),
            CoreOp::ControlSummary(
                "M1".into(),
                vec![ControlSummary::Branch, ControlSummary::Return],
            ),
            CoreOp::PatternFacts(
                "M1".into(),
                vec![
                    PatternFact::Constructor,
                    PatternFact::Getter("value".into()),
                ],
            ),
            CoreOp::Flags("M1".into(), vec!["legacy".into(), "legacy".into()]),
            CoreOp::ClassFlags("C1".into(), vec!["metadata".into()]),
            CoreOp::Extends("C1".into(), "Base".into()),
            CoreOp::Implements("C1".into(), "I1".into()),
            CoreOp::Injects("C1".into(), vec!["Repo".into(), "Repo".into()]),
            CoreOp::Import("IM1".into(), "pkg".into(), "Thing".into()),
            CoreOp::TypeAlias("T1".into(), "Thing<string>".into()),
            CoreOp::TypeAlias("T1".into(), "Thing<string>".into()),
            CoreOp::Pattern(
                "GETTER".into(),
                vec!["C1".into(), "M1".into(), "value".into()],
            ),
            CoreOp::Body(
                "M1".into(),
                "{\r\n  return \"λ\";\r\n}".into(),
                Some(100),
                Some(124),
            ),
            CoreOp::Body("M2".into(), "expression".into(), None, None),
            CoreOp::DataFlow("M1".into(), "reads".into(), "value".into()),
            CoreOp::ControlFlow("M1".into(), "return".into(), "value".into()),
            duplicate.clone(),
            duplicate,
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::ThreadBound),
            CoreOp::Call("M1".into(), "exact".into(), 2, false),
            CoreOp::Call("M1".into(), "spread".into(), 1, true),
        ],
    }
}

fn empty_payload() -> Vec<u8> {
    let mut bytes = vec![MAGIC[0], MAGIC[1], VERSION];
    write_varint(&mut bytes, 7);
    write_varint(&mut bytes, 1);
    write_string(&mut bytes, "file.rs");
    write_varint(&mut bytes, 0);
    write_varint(&mut bytes, 0);
    bytes
}

fn single_instruction(strings: &[&str], write_instruction: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = vec![MAGIC[0], MAGIC[1], VERSION];
    write_varint(&mut bytes, 1);
    write_varint(&mut bytes, strings.len() as u64);
    for value in strings {
        write_string(&mut bytes, value);
    }
    write_varint(&mut bytes, 0);
    write_varint(&mut bytes, 1);
    write_instruction(&mut bytes);
    bytes
}

#[test]
fn v04_round_trips_complete_ir_order_and_duplicates() {
    let expected = all_variants_with_duplicates();
    let bytes = encode(&expected);
    assert_eq!(bytes[2], 0x04);
    assert_ir_eq(&expected, &decode(&bytes).expect("semantic v0x04 decode"));
    let second = encode(&expected);
    assert_eq!(bytes, second, "bytes must be deterministic");
}

#[test]
fn json_wrapper_rejects_metadata_conflicts() {
    let expected = all_variants_with_duplicates();
    let valid = ir_to_binary_wire_json(&expected);
    assert_ir_eq(
        &expected,
        &binary_wire_json_to_ir(&valid).expect("matching wrapper metadata"),
    );

    let mut wrong_file = valid.clone();
    wrong_file["file"] = serde_json::json!("other.ts");
    assert!(binary_wire_json_to_ir(&wrong_file).is_none());

    let mut wrong_version = valid;
    wrong_version["v"] = serde_json::json!(43);
    assert!(binary_wire_json_to_ir(&wrong_version).is_none());
}

#[test]
fn decoder_rejects_legacy_versions_and_trailing_data() {
    for legacy in [0x01, 0x02, 0x03] {
        let mut bytes = empty_payload();
        bytes[2] = legacy;
        assert!(matches!(
            decode(&bytes),
            Err(BinaryDecodeError::UnsupportedVersion(version)) if version == legacy
        ));
    }

    let mut trailing = empty_payload();
    trailing.push(0);
    assert!(matches!(
        decode(&trailing),
        Err(BinaryDecodeError::TrailingData(1))
    ));
}

#[test]
fn decoder_rejects_invalid_indexes_opcodes_and_body_flags() {
    let mut invalid_file = empty_payload();
    let file_index = invalid_file.len() - 2;
    invalid_file[file_index] = 1;
    assert!(matches!(
        decode(&invalid_file),
        Err(BinaryDecodeError::InvalidStringIndex(1))
    ));

    let mut unknown_opcode = empty_payload();
    *unknown_opcode.last_mut().unwrap() = 1;
    unknown_opcode.push(0xFF);
    assert!(matches!(
        decode(&unknown_opcode),
        Err(BinaryDecodeError::UnknownOpcode(0xFF))
    ));

    let invalid_body = single_instruction(&["file.rs", "M1", "body"], |bytes| {
        bytes.push(OP_BODY);
        write_varint(bytes, 1);
        write_varint(bytes, 2);
        write_varint(bytes, 2);
    });
    assert!(matches!(
        decode(&invalid_body),
        Err(BinaryDecodeError::InvalidBodySpanFlag(2))
    ));
}

#[test]
fn decoder_rejects_invalid_utf8_counts_and_typed_values() {
    let mut invalid_utf8 = vec![MAGIC[0], MAGIC[1], VERSION];
    write_varint(&mut invalid_utf8, 1);
    write_varint(&mut invalid_utf8, 1);
    write_varint(&mut invalid_utf8, 1);
    invalid_utf8.push(0xFF);
    write_varint(&mut invalid_utf8, 0);
    write_varint(&mut invalid_utf8, 0);
    assert!(matches!(
        decode(&invalid_utf8),
        Err(BinaryDecodeError::InvalidUtf8(_))
    ));

    let short_fixed = single_instruction(&["file.rs", "C1"], |bytes| {
        bytes.push(OP_DEF_C);
        write_varint(bytes, 1);
    });
    assert!(matches!(
        decode(&short_fixed),
        Err(BinaryDecodeError::TruncatedData(_))
    ));

    let invalid_modifier = single_instruction(&["file.rs", "M1", "BOGUS"], |bytes| {
        bytes.push(OP_MOD_M);
        write_varint(bytes, 2);
        write_varint(bytes, 1);
        write_varint(bytes, 2);
    });
    assert!(matches!(
        decode(&invalid_modifier),
        Err(BinaryDecodeError::InvalidDeclarationModifier(value)) if value == "BOGUS"
    ));

    let invalid_summary = single_instruction(&["file.rs", "M1", "BOGUS"], |bytes| {
        bytes.push(OP_CTRL_SUM);
        write_varint(bytes, 2);
        write_varint(bytes, 1);
        write_varint(bytes, 2);
    });
    assert!(matches!(
        decode(&invalid_summary),
        Err(BinaryDecodeError::InvalidControlSummary(value)) if value == "BOGUS"
    ));

    let invalid_fact = single_instruction(&["file.rs", "M1", "BOGUS"], |bytes| {
        bytes.push(OP_PAT_FACT);
        write_varint(bytes, 2);
        write_varint(bytes, 1);
        write_varint(bytes, 2);
    });
    assert!(matches!(
        decode(&invalid_fact),
        Err(BinaryDecodeError::InvalidPatternFact)
    ));
}
