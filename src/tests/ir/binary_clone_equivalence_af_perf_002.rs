//! Agent MaxHeadRoom: frozen original encoder, independent ordered-table checks.
#[path = "binary_clone_fixtures_af_perf_002.rs"]
mod fixtures;
#[path = "original_binary_encoder_af_perf_002.rs"]
pub(crate) mod original;
use crate::ir::{CompiledIR, CoreOp, binary_wire as current};

pub(crate) fn assert_binary(ir: &CompiledIR) -> Vec<u8> {
    let bytes = current::encode(ir);
    assert_eq!(bytes, original::encode(ir), "complete frozen binary");
    let decoded = current::decode(&bytes).unwrap();
    assert_eq!(decoded.file_id, ir.file_id);
    assert_eq!(decoded.version, ir.version);
    assert_eq!(decoded.instructions, ir.instructions);
    bytes
}
fn varint(bytes: &[u8], offset: &mut usize) -> u64 {
    let mut value = 0;
    let mut shift = 0;
    loop {
        let b = bytes[*offset];
        *offset += 1;
        value |= u64::from(b & 127) << shift;
        if b < 128 {
            return value;
        }
        shift += 7;
    }
}
#[test]
fn af002_binary_frozen_semantics() {
    let all = fixtures::all_variants_with_duplicates();
    let mut cases = vec![
        all.clone(),
        CompiledIR {
            file_id: String::new(),
            version: 0,
            instructions: vec![],
        },
    ];
    for body in [String::new(), "共有λ\r\n".into(), "共有".repeat(65536)] {
        cases.push(CompiledIR {
            file_id: "α1".into(),
            version: u64::MAX,
            instructions: vec![
                CoreOp::DefClass("".into(), "".into()),
                CoreOp::Body("".into(), body, None, None),
                CoreOp::Flags("".into(), vec!["".into(), "α1".into(), "".into()]),
            ],
        });
    }
    // Each typed opcode independently, plus original and reversed complete streams.
    for op in &all.instructions {
        cases.push(CompiledIR {
            file_id: "λ.ts".into(),
            version: 7,
            instructions: vec![op.clone(), op.clone()],
        });
    }
    let mut reversed = all;
    reversed.instructions.reverse();
    cases.push(reversed);
    for (case, ir) in cases.into_iter().enumerate() {
        let bytes = assert_binary(&ir);
        eprintln!(
            "AF002_BINARY_CASE {}",
            serde_json::json!({"case":case,"bytes":bytes,
            "decoded_file":ir.file_id,"decoded_version":ir.version,"instructions":format!("{:?}",ir.instructions)})
        );
        // Independent first-appearance dictionary (not StringTable intern/lookup).
        let mut strings = Vec::new();
        for op in &ir.instructions {
            for s in crate::ir::wire::op_to_tuple(op) {
                if !strings.contains(&s) {
                    strings.push(s);
                }
            }
        }
        if !strings.contains(&ir.file_id) {
            strings.push(ir.file_id.clone());
        }
        let mut at = 3;
        assert_eq!(varint(&bytes, &mut at), ir.version);
        assert_eq!(varint(&bytes, &mut at) as usize, strings.len());
        for s in &strings {
            let len = varint(&bytes, &mut at) as usize;
            assert_eq!(&bytes[at..at + len], s.as_bytes());
            at += len;
        }
        let index = varint(&bytes, &mut at) as usize;
        assert_eq!(&strings[index], &ir.file_id);
        assert_eq!(
            current::ir_to_binary_wire_json(&ir),
            original::ir_to_binary_wire_json(&ir)
        );
        let wrapper = original::ir_to_binary_wire_json(&ir);
        assert_eq!(
            format!("{:?}", current::binary_wire_json_to_ir(&wrapper)),
            format!("{:?}", original::binary_wire_json_to_ir(&wrapper))
        );
        assert_eq!(
            current::estimate_savings(&ir),
            original::estimate_savings(&ir)
        );
        assert_eq!(
            current::is_binary_wire(&bytes),
            original::is_binary_wire(&bytes)
        );
        // Every truncated prefix must retain the same decoder error/panic distinction.
        for n in [0, 1, 2, 3, bytes.len() / 2, bytes.len() - 1] {
            let a = std::panic::catch_unwind(|| current::decode(&bytes[..n]));
            let b = std::panic::catch_unwind(|| original::decode(&bytes[..n]));
            assert_eq!(a.is_err(), b.is_err());
            if let (Ok(a), Ok(b)) = (a, b) {
                assert_eq!(format!("{a:?}"), format!("{b:?}"));
            }
        }
        let mut invalid = bytes.clone();
        invalid.push(0);
        assert_eq!(
            format!("{:?}", current::decode(&invalid)),
            format!("{:?}", original::decode(&invalid))
        );
    }
    let invalid = serde_json::json!({"file":"α1","v":1,"data":"!","encoding":"binary"});
    assert!(current::binary_wire_json_to_ir(&invalid).is_none());
    assert!(original::binary_wire_json_to_ir(&invalid).is_none());
    // A missing table key is unreachable from complete typed IR. Exercise the
    // verbatim frozen closure in a faulted empty-map probe, not public authority.
    let panic = std::panic::catch_unwind(original::missing_probe::missing_operand).unwrap_err();
    let message = panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap();
    assert_eq!(
        message,
        "canonical binary operand missing from string table"
    );
    let production = include_str!("../../ir/binary_wire.rs").replace("\r\n", "\n");
    let probe = include_str!("binary_encoder_missing_probe_af_perf_002.rs");
    let start = production.find("    let encode_operand =").unwrap();
    let end = production[start..].find("    // 4. File identity").unwrap() + start;
    let body = production[start..end].replace(
        "        #[cfg(test)]\n        crate::default_operation_observations_af_perf_002::binary_lookup(s);\n", "");
    let normalize = |s: &str| s.split_whitespace().collect::<String>();
    assert!(
        normalize(probe).contains(&normalize(&body)),
        "fault probe must preserve actual lookup closure"
    );
}
