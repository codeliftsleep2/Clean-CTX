//! Finite encoder-body protection; not a repository-wide allocation prohibition.
fn body(source: &str) -> &str {
    let start = source.find("pub fn encode(").expect("encoder");
    let end = source[start..]
        .find("/// Estimate the byte savings")
        .expect("encoder end")
        + start;
    &source[start..end]
}
fn canonical(source: &str) -> String {
    let mut text = body(source).replace("#[cfg(test)]", "");
    while let Some(start) = text.find("crate::default_operation_observations_af_perf_002::") {
        let end = text[start..].find(';').expect("observation terminator") + start + 1;
        text.replace_range(start..end, "");
    }
    text.split_whitespace().collect()
}
fn protected(source: &str) -> bool {
    let original = include_str!("original_binary_encoder_af_perf_002.rs");
    // Separately authorized reverse-map reuse; retain the independent clone ban.
    let mut original = original.to_owned();
    let start = original
        .find("    // Build a lookup:")
        .expect("original map");
    let end = original[start..]
        .find("    // Helper to encode")
        .expect("lookup")
        + start;
    original.replace_range(start..end, "");
    let expected = canonical(&original)
        .replace(
            "letstrings:Vec<String>=table.strings().to_vec();",
            "letstrings=table.strings();",
        )
        .replace("forsin&strings{", "forsinstrings{")
        .replace(
            "letidx=str_to_idx.get(s).copied()",
            "letidx=table.index_of(s).map(|i|iasu64)",
        );
    let active = body(source);
    canonical(source) == expected
        && !active.contains("::binary_copy(")
        && ["::binary_lookup(s)", "::binary_output("]
            .iter()
            .all(|site| active.contains(*site))
}
#[test]
fn afperf_binary_clone_static() {
    let _: Option<crate::binary_clone_equivalence_af_perf_002::original::BinaryDecodeError> = None;
    assert_eq!(
        include_bytes!("binary_clone_original_decode_af_perf_002.rs"),
        include_bytes!("../../ir/binary_wire/decode.rs"),
        "frozen decoder copy identity"
    );
    let source = include_str!("../../ir/binary_wire.rs");
    assert!(
        protected(source),
        "Binary encoder slice-borrow static protection"
    );
    let restored = source
        .replace(
            "let strings = table.strings();",
            "let strings: Vec<String> = table.strings().to_vec();",
        )
        .replace("for s in strings {", "for s in &strings {");
    assert!(
        !protected(&restored),
        "real ownership restoration must be rejected"
    );
    for (from, to) in [
        (
            "canonical binary operand missing from string table",
            "changed panic",
        ),
        (".map(|i| i as u64)", ".map(|_| 0)"),
        ("::binary_lookup(s)", "::removed_observer(s)"),
    ] {
        assert!(source.contains(from));
        assert!(
            !protected(&source.replace(from, to)),
            "finite guard sensitivity: {from}"
        );
    }
}
