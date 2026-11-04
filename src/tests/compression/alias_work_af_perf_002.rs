//! C2 setup/matching census; semantic assertions characterize existing behavior.
use crate::compression::type_aliases::apply_type_aliases;
use crate::work_observations_af_perf_002 as work;
use std::collections::BTreeMap;
fn aliases(n: usize) -> BTreeMap<String, String> {
    (0..n)
        .map(|i| (format!("DomainType{i:04}"), format!("$t{i}")))
        .collect()
}
fn observe(label: &str, body: &str, configured: &BTreeMap<String, String>, expected: &str) {
    work::reset();
    let start = std::time::Instant::now();
    let (actual, footer) = apply_type_aliases(body, configured);
    assert_eq!(actual, expected, "exact substitution label={label}");
    let c = work::snapshot();
    assert_eq!(c.alias_calls, 1);
    assert_eq!(c.alias_setup_entries, configured.len());
    eprintln!(
        "AF002_C2 label={label} text_bytes={} aliases={} eligible={} sort_keys={} positions={} probes={} key_bytes_considered={} matches={} output_bytes={} pair_capacity={} footer={} us={} boundary=production-matcher",
        body.len(),
        configured.len(),
        c.alias_eligible_entries,
        c.alias_sort_key_calls,
        c.alias_positions,
        c.alias_probes,
        c.alias_key_bytes_considered,
        c.alias_matches,
        c.alias_output_bytes,
        c.alias_pair_capacity,
        serde_json::to_string(&footer).unwrap(),
        start.elapsed().as_micros()
    );
}
#[test]
fn af002_c2_text_alias_match_key_and_boundary_dimensions() {
    for n in [1, 2, 4, 8, 16, 64, 256] {
        let body = "NoMatch ".repeat(n);
        observe("text-growth-negative", &body, &aliases(16), &body);
        observe(
            "alias-growth-negative",
            &"NoMatch ".repeat(16),
            &aliases(n),
            &"NoMatch ".repeat(16),
        );
    }
    for matches in [0, 1, 4, 8, 16] {
        let body = "DomainType0000 ".repeat(matches) + &"NoMatch ".repeat(16 - matches);
        let expected = "$t0 ".repeat(matches) + &"NoMatch ".repeat(16 - matches);
        observe("match-density", &body, &aliases(16), &expected);
    }
    for len in [4, 8, 16, 64, 256] {
        let key = "K".repeat(len);
        let configured = BTreeMap::from([(key.clone(), "$key".into())]);
        observe("key-length-positive", &key, &configured, "$key");
        observe(
            "key-length-negative",
            &"N".repeat(256),
            &configured,
            &"N".repeat(256),
        );
    }
    let configured = BTreeMap::from([
        ("DomainType0000".into(), "$short".into()),
        ("DomainType0000Long".into(), "$long".into()),
    ]);
    observe(
        "priority",
        "DomainType0000Long DomainType0000",
        &configured,
        "$long $short",
    );
    observe(
        "unicode-boundaries",
        "λDomainType0000 DomainType0000X $DomainType0000 DomainType0000_ [DomainType0000]",
        &aliases(1),
        "λ$t0 DomainType0000X $DomainType0000 DomainType0000_ [$t0]",
    );
    observe(
        "empty-config",
        "λ ordinary bytes",
        &aliases(0),
        "λ ordinary bytes",
    );
}

#[test]
fn af002_c2_ir_operand_setup_growth() {
    use crate::ir::opcodes::CoreOp;
    for operands in [1, 2, 4, 8, 16, 64, 256] {
        let mut instructions: Vec<_> = (0..operands)
            .map(|i| CoreOp::Return(format!("M{i}"), "DomainType0000".into()))
            .collect();
        work::reset();
        let start = std::time::Instant::now();
        crate::ir::type_aliases::apply_type_aliases_to_ir(&mut instructions, &aliases(16));
        let c = work::snapshot();
        let mut expected: Vec<_> = (0..operands)
            .map(|i| CoreOp::Return(format!("M{i}"), "$t0".into()))
            .collect();
        expected.push(CoreOp::TypeAlias("$t0".into(), "DomainType0000".into()));
        assert_eq!(
            instructions, expected,
            "ordered IR and one reversible alias declaration"
        );
        assert_eq!(c.alias_calls, operands);
        assert_eq!(c.alias_setup_entries, operands * 16);
        eprintln!(
            "AF002_C2_IR operands={operands} calls={} input_bytes={} setup_entries={} sort_keys={} probes={} pair_capacity_sum={} us={} boundary=whole-IR-transform",
            c.alias_calls,
            c.alias_input_bytes,
            c.alias_setup_entries,
            c.alias_sort_key_calls,
            c.alias_probes,
            c.alias_pair_capacity,
            start.elapsed().as_micros()
        );
    }
}
