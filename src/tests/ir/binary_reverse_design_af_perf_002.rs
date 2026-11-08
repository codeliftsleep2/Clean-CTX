//! Agent MaxHeadRoom: test-only reverse-map design comparison, no production change.
use crate::ir::{CompiledIR, CoreOp, binary_wire};
use std::cell::{Cell, RefCell};
#[path = "binary_reverse_encoder_af_perf_002.rs"]
mod encoder;
#[path = "binary_reverse_fixtures_af_perf_002.rs"]
mod fixtures;
#[path = "binary_reverse_table_af_perf_002.rs"]
mod table;
#[derive(Clone, Debug, Default, serde::Serialize)]
struct Work {
    intern_calls: usize,
    intern_key_bytes: usize,
    new_strings: usize,
    new_string_bytes: usize,
    reverse_entries: usize,
    reverse_key_bytes: usize,
    lookups: usize,
    lookup_key_bytes: usize,
    output_bytes: usize,
    table_strings: usize,
    table_capacity: usize,
    string_capacity: usize,
    reverse_capacity: usize,
    common_container_header_bytes: usize,
    extra_map_header_bytes: usize,
}
thread_local! {
    static WORK: RefCell<Work> = RefCell::new(Work::default());
    static ENABLED: Cell<bool> = const { Cell::new(true) };
}
fn observe(f: impl FnOnce(&mut Work)) {
    if ENABLED.with(Cell::get) {
        WORK.with(|w| f(&mut w.borrow_mut()));
    }
}
fn measured(ir: &CompiledIR, mode: encoder::Mode) -> (Vec<u8>, Work) {
    WORK.with(|w| *w.borrow_mut() = Work::default());
    let bytes = encoder::encode(ir, mode);
    (bytes, WORK.with(|w| w.borrow().clone()))
}
pub(crate) fn ir(methods: usize, payload: usize, duplicate: bool) -> CompiledIR {
    let mut instructions = vec![CoreOp::DefClass("C".into(), "共有Owner".into())];
    for i in 0..methods {
        let id = if duplicate {
            "M".into()
        } else {
            format!("M{i}")
        };
        instructions.push(CoreOp::DefMethod(
            "C".into(),
            id.clone(),
            format!("run{}", if duplicate { 0 } else { i }),
        ));
        instructions.push(CoreOp::Body(
            id,
            format!(
                "{}{}",
                "共有".repeat(payload),
                if duplicate { 0 } else { i }
            ),
            None,
            None,
        ));
    }
    CompiledIR {
        file_id: "λ.ts".into(),
        version: 42,
        instructions,
    }
}
pub(crate) fn all_opcode_fixture() -> CompiledIR {
    fixtures::all_variants_with_duplicates()
}
#[test]
fn af002_binary_reverse_design_semantics() {
    let values = ["", "a", "共有", "a", "共有", "", "λ"];
    let mut t = table::StringTable::new();
    assert!(t.is_empty());
    let mut ordered = Vec::new();
    for s in values {
        let expected = ordered.iter().position(|v| v == &s).unwrap_or_else(|| {
            ordered.push(s);
            ordered.len() - 1
        });
        assert_eq!(t.intern(s), expected);
        let rebuilt: std::collections::HashMap<_, _> = t
            .strings()
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();
        for key in values {
            assert_eq!(t.index_of(key), rebuilt.get(key).copied());
        }
    }
    assert_eq!(t.lookup(0), Some(""));
    assert_eq!(t.lookup(999), None);
    assert_eq!(t.index_of("absent"), None);
    assert_eq!(t.len(), ordered.len());
    assert_eq!(
        table::StringTable::from_json(&t.to_json())
            .unwrap()
            .strings(),
        t.clone().strings()
    );
    assert!(table::StringTable::from_json(&serde_json::json!([1])).is_none());
    let mut cases = vec![
        fixtures::all_variants_with_duplicates(),
        ir(0, 0, false),
        ir(1, 0, false),
        ir(8, 0, true),
        ir(64, 4096, false),
    ];
    cases.push(CompiledIR {
        file_id: String::new(),
        version: u64::MAX,
        instructions: vec![CoreOp::Body(String::new(), String::new(), None, None)],
    });
    for ir in cases {
        let expected = binary_wire::encode(&ir);
        for mode in [encoder::Mode::Rebuild, encoder::Mode::Reuse] {
            assert_eq!(encoder::encode(&ir, mode), expected);
        }
    }
    // All40 frozen public identity records retain complete binary payloads where present.
    // This is payload equivalence, not new registered public integration of a test design.
    let records: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../mcp/fixtures/binary_clone_public_identities.json"
    ))
    .unwrap();
    let mut count = 0;
    for record in records {
        if let Some(bytes) = record.get("normalized_binary") {
            let bytes: Vec<u8> = serde_json::from_value(bytes.clone()).unwrap();
            let ir = binary_wire::decode(&bytes).unwrap();
            for mode in [encoder::Mode::Rebuild, encoder::Mode::Reuse] {
                assert_eq!(encoder::encode(&ir, mode), bytes);
            }
            count += 1;
        }
    }
    assert_eq!(count, 39);
    for mode in [encoder::Mode::Rebuild, encoder::Mode::Reuse] {
        let p = std::panic::catch_unwind(|| encoder::missing_operand(mode)).unwrap_err();
        let message = p
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| p.downcast_ref::<String>().map(String::as_str))
            .unwrap();
        assert_eq!(
            message,
            "canonical binary operand missing from string table"
        );
    }
}
#[test]
fn af002_binary_reverse_design_cost() {
    let axes = [
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
    ];
    for (methods, payload, duplicates) in axes {
        let ir = ir(methods, payload, duplicates);
        crate::default_operation_observations_af_perf_002::reset();
        let expected = binary_wire::encode(&ir);
        let production_work = crate::default_operation_observations_af_perf_002::snapshot();
        let (a, wa) = measured(&ir, encoder::Mode::Rebuild);
        let (b, wb) = measured(&ir, encoder::Mode::Reuse);
        assert_eq!(a, expected);
        assert_eq!(b, expected);
        assert_eq!(wa.intern_calls, wb.intern_calls);
        assert_eq!(wa.intern_key_bytes, wb.intern_key_bytes);
        assert_eq!(wa.lookups, wb.lookups);
        assert_eq!(wa.lookup_key_bytes, wb.lookup_key_bytes);
        assert_eq!(wa.reverse_entries, wa.table_strings);
        assert_eq!(wb.reverse_entries, 0);
        assert_eq!(wa.table_strings, production_work.new_table_strings);
        assert_eq!(wa.lookups, production_work.operand_lookups);
        let mut samples: [Vec<u128>; 2] = std::array::from_fn(|_| Vec::new());
        ENABLED.with(|v| v.set(false));
        for round in 0..27 {
            for offset in 0..2 {
                let which = (round + offset) % 2;
                let start = std::time::Instant::now();
                let bytes = encoder::encode(
                    std::hint::black_box(&ir),
                    if which == 0 {
                        encoder::Mode::Rebuild
                    } else {
                        encoder::Mode::Reuse
                    },
                );
                std::hint::black_box(&bytes);
                drop(bytes);
                if round >= 3 {
                    samples[which].push(start.elapsed().as_nanos());
                }
            }
        }
        ENABLED.with(|v| v.set(true));
        for s in &mut samples {
            s.sort_unstable();
            assert_eq!(s.len(), 24);
        }
        eprintln!(
            "AF002_BINARY_REVERSE {}",
            serde_json::json!({"methods":methods,"unicode_repetitions":payload,"duplicates":duplicates,"production_work":production_work,"rebuild_work":wa,"reuse_work":wb,"median_ns":[samples[0][12],samples[1][12]],"samples_ns":samples,"scope":"full encode setup/query/write/drop; symmetric test copies only timed; production byte/work control untimed; no allocator/public latency claims"})
        );
    }
}
