//! Actual production bytes and actual frozen pre-reuse encoder, symmetric diagnostics.
use crate::binary_reverse_equivalence_af_perf_002::original;
use crate::default_operation_observations_af_perf_002 as work;
use crate::ir::binary_wire;
fn body(source: &str) -> &str {
    let start = source.find("pub fn encode(").unwrap();
    let end = source[start..]
        .find("/// Estimate the byte savings")
        .unwrap()
        + start;
    &source[start..end]
}
fn clean(source: &str) -> String {
    let mut source = source.replace("#[cfg(test)]", "");
    while let Some(start) = source.find("crate::default_operation_observations_af_perf_002::") {
        let end = source[start..].find(';').unwrap() + start + 1;
        source.replace_range(start..end, "");
    }
    source.split_whitespace().collect()
}
fn protected(encoder: &str, table: &str) -> bool {
    let mut expected = body(include_str!("original_binary_reverse_af_perf_002.rs")).to_owned();
    let a = expected.find("    // Build a lookup:").unwrap();
    let b = expected[a..].find("    // Helper to encode").unwrap() + a;
    expected.replace_range(a..b, "");
    let expected = clean(&expected).replace(
        "letidx=str_to_idx.get(s).copied()",
        "letidx=table.index_of(s).map(|i|iasu64)",
    );
    let table = table.split_whitespace().collect::<String>();
    let accessor = "///Readanexistingfirst-appearanceindexwithoutinterningorchangingorder.pub(crate)fnindex_of(&self,s:&str)->Option<usize>{self.indices.get(s).copied()}";
    let original_table = include_str!("fixtures/binary_reverse_original_string_table.txt")
        .split_whitespace()
        .collect::<String>();
    clean(body(encoder)) == expected
        && table.contains(accessor)
        && table.replacen(accessor, "", 1) == original_table
        && !body(encoder).contains("::binary_entry(")
        && !body(encoder).contains("to_vec()")
        && body(encoder).contains("::binary_lookup(s)")
        && body(encoder).contains("::binary_output(")
}
#[test]
fn afperf_binary_reverse_static() {
    let encoder = include_str!("../../ir/binary_wire.rs");
    let table = include_str!("../../ir/string_table.rs");
    assert!(
        protected(encoder, table),
        "Binary encoder reverse-map reuse static protection"
    );
    assert!(!protected(
        include_str!("original_binary_reverse_af_perf_002.rs"),
        table
    ));
    assert!(!protected(
        encoder,
        &table.replace(
            "self.indices.get(s).copied()",
            "self.indices.values().next().copied()"
        )
    ));
    assert!(!protected(
        &encoder.replace(".map(|i| i as u64)", ".map(|_| 0)"),
        table
    ));
    assert!(!protected(
        &encoder.replace(
            "canonical binary operand missing from string table",
            "changed panic"
        ),
        table
    ));
}
#[test]
fn af002_binary_reverse_integrated_cost() {
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
        let ir = crate::binary_reverse_design_af_perf_002::ir(methods, payload, dup);
        work::reset();
        let old = original::encode(&ir);
        let old_work = work::snapshot();
        work::reset();
        let new = binary_wire::encode(&ir);
        let new_work = work::snapshot();
        assert_eq!(old, new);
        assert_eq!(old_work.string_table_builds, new_work.string_table_builds);
        assert_eq!(old_work.tuple_visits, new_work.tuple_visits);
        assert_eq!(old_work.tuple_strings, new_work.tuple_strings);
        assert_eq!(old_work.tuple_bytes, new_work.tuple_bytes);
        assert_eq!(old_work.intern_calls, new_work.intern_calls);
        assert_eq!(
            old_work.intern_key_bytes_offered,
            new_work.intern_key_bytes_offered
        );
        assert_eq!(old_work.new_table_strings, new_work.new_table_strings);
        assert_eq!(
            old_work.new_table_string_bytes,
            new_work.new_table_string_bytes
        );
        assert_eq!(
            old_work.owned_index_capacity_max,
            new_work.owned_index_capacity_max
        );
        assert_eq!(
            old_work.ordered_strings_capacity_max,
            new_work.ordered_strings_capacity_max
        );
        assert_eq!(old_work.operand_lookups, new_work.operand_lookups);
        assert_eq!(
            old_work.operand_key_bytes_offered,
            new_work.operand_key_bytes_offered
        );
        assert_eq!(old_work.binary_output_bytes, new_work.binary_output_bytes);
        assert!(old_work.reverse_entries > 0);
        assert_eq!(old_work.copied_strings, 0);
        assert_eq!(
            [
                new_work.reverse_entries,
                new_work.reverse_capacity,
                new_work.copied_strings
            ],
            [0, 0, 0]
        );
        let samples = work::unobserved(|| {
            let mut samples: [Vec<u128>; 2] = std::array::from_fn(|_| Vec::new());
            for round in 0..27 {
                for offset in 0..2 {
                    let which = (round + offset) % 2;
                    let start = std::time::Instant::now();
                    let bytes = if which == 0 {
                        original::encode(std::hint::black_box(&ir))
                    } else {
                        binary_wire::encode(std::hint::black_box(&ir))
                    };
                    std::hint::black_box(&bytes);
                    drop(bytes);
                    if round >= 3 {
                        samples[which].push(start.elapsed().as_nanos());
                    }
                }
            }
            samples
        });
        let mut samples = samples;
        for s in &mut samples {
            s.sort_unstable();
            assert_eq!(s.len(), 24);
        }
        eprintln!(
            "AF002_BINARY_REVERSE_INTEGRATED {}",
            serde_json::json!({"methods":methods,"unicode_repetitions":payload,"duplicates":dup,
            "original_work":old_work,"production_work":new_work,"median_ns":[samples[0][12],samples[1][12]],"samples_ns":samples,
            "scope":"actual production and frozen protected pre-reuse; common setup/query/write/drop included; disabled updates on both; residual probes not release latency"})
        );
    }
}
