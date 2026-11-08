//! Frozen protected pre-reuse implementation, independent from design-copy timings.
#[path = "original_binary_reverse_af_perf_002.rs"]
pub(crate) mod original;
use crate::default_operation_observations_af_perf_002 as work;
use crate::ir::{CompiledIR, binary_wire};
#[test]
fn af002_binary_reverse_frozen_equivalence() {
    let _: Option<original::BinaryDecodeError> = None;
    let mut cases = vec![
        crate::binary_reverse_design_af_perf_002::all_opcode_fixture(),
        CompiledIR {
            file_id: String::new(),
            version: u64::MAX,
            instructions: vec![],
        },
    ];
    for (methods, payload, dup) in [
        (0, 0, false),
        (1, 0, false),
        (8, 0, false),
        (64, 0, false),
        (256, 0, false),
        (1, 4096, false),
        (16, 64, false),
        (16, 4096, false),
        (64, 4096, false),
        (8, 0, true),
        (256, 0, true),
        (64, 4096, true),
    ] {
        cases.push(crate::binary_reverse_design_af_perf_002::ir(
            methods, payload, dup,
        ));
    }
    for ir in cases {
        work::unobserved(|| {
            let bytes = binary_wire::encode(&ir);
            assert_eq!(bytes, original::encode(&ir));
            let decoded = binary_wire::decode(&bytes).unwrap();
            assert_eq!(decoded.file_id, ir.file_id);
            assert_eq!(decoded.version, ir.version);
            assert_eq!(decoded.instructions, ir.instructions);
            assert_eq!(
                binary_wire::ir_to_binary_wire_json(&ir),
                original::ir_to_binary_wire_json(&ir)
            );
            let wrapper = original::ir_to_binary_wire_json(&ir);
            assert_eq!(
                format!("{:?}", binary_wire::binary_wire_json_to_ir(&wrapper)),
                format!("{:?}", original::binary_wire_json_to_ir(&wrapper))
            );
            assert_eq!(
                binary_wire::estimate_savings(&ir),
                original::estimate_savings(&ir)
            );
            assert!(original::is_binary_wire(&bytes));
        });
    }
}
