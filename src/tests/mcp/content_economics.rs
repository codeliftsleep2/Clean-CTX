use super::{
    SelectedRepresentation, select_with_local_tokenizer, select_with_local_tokenizer_and_raw_tokens,
};
use crate::tokenizer::{Tokenizer, TokenizerKind};

#[test]
fn smaller_complete_candidate_wins() {
    let tokenizer = ByteTokenizer;
    let selected = select_with_local_tokenizer_and_raw_tokens(
        "raw source document",
        "A1".into(),
        TokenizerKind::Cl100k,
        Some(&tokenizer),
        19,
    );
    assert_eq!(selected.selected, SelectedRepresentation::Candidate);
    assert_eq!(selected.text, "A1");
    assert_eq!(selected.selected_tokens(), 2);
    assert_eq!(selected.saved_tokens(), 17);
}

#[test]
fn raw_wins_a_tie() {
    let tokenizer = ByteTokenizer;
    let selected = select_with_local_tokenizer_and_raw_tokens(
        "raw",
        "A1!".into(),
        TokenizerKind::Cl100k,
        Some(&tokenizer),
        3,
    );
    assert_eq!(selected.selected, SelectedRepresentation::RawPassthrough);
    assert_eq!(selected.text, "raw");
    assert_eq!(selected.saved_tokens(), 0);
}

#[test]
fn larger_candidate_falls_back_to_byte_exact_raw() {
    let raw = "{\r\n  const λ = '§PATHMAP';\r\n}\r\n";
    let tokenizer = ByteTokenizer;
    let selected = select_with_local_tokenizer_and_raw_tokens(
        raw,
        "schema plus a much larger candidate".into(),
        TokenizerKind::Cl100k,
        Some(&tokenizer),
        raw.len(),
    );
    assert_eq!(selected.selected, SelectedRepresentation::RawPassthrough);
    assert_eq!(selected.text.as_bytes(), raw.as_bytes());
    assert_eq!(selected.selected_tokens(), raw.len());
}

#[test]
fn approximate_counter_requires_savings_beyond_both_sides_of_error_bound() {
    let tokenizer = ByteTokenizer;
    let rejected = select_with_local_tokenizer_and_raw_tokens(
        &"r".repeat(100),
        "c".repeat(97),
        TokenizerKind::Claude,
        Some(&tokenizer),
        100,
    );
    assert_eq!(rejected.selected, SelectedRepresentation::RawPassthrough);

    let accepted = select_with_local_tokenizer_and_raw_tokens(
        &"r".repeat(100),
        "c".repeat(95),
        TokenizerKind::Claude,
        Some(&tokenizer),
        100,
    );
    assert_eq!(accepted.selected, SelectedRepresentation::Candidate);
}

struct ByteTokenizer;

impl Tokenizer for ByteTokenizer {
    fn name(&self) -> &str {
        "test-bytes"
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.len()
    }

    fn encode(&self, text: &str) -> Vec<u64> {
        text.bytes().map(u64::from).collect()
    }
}

#[test]
fn claude_local_approximation_applies_uncertainty_margin() {
    let tokenizer = ByteTokenizer;
    let selected = select_with_local_tokenizer(
        &"r".repeat(100),
        "c".repeat(97),
        TokenizerKind::Claude,
        Some(&tokenizer),
    );
    assert_eq!(selected.selected, SelectedRepresentation::RawPassthrough);
}

#[test]
fn unavailable_local_tokenizer_fails_safe_to_exact_raw() {
    let raw = "const λ = 1;\r\n";
    let selected =
        select_with_local_tokenizer(raw, "A1 candidate".into(), TokenizerKind::Claude, None);
    assert_eq!(selected.selected, SelectedRepresentation::RawPassthrough);
    assert_eq!(selected.text.as_bytes(), raw.as_bytes());
}

#[test]
fn c1_equivalence_counted_and_recounted_paths() {
    use crate::mcp::tool_helpers::count_tokens_with_tokenizer;
    for kind in [
        TokenizerKind::Cl100k,
        TokenizerKind::O200k,
        TokenizerKind::Claude,
        TokenizerKind::Llama3,
    ] {
        let local =
            crate::tokenizer::create_tokenizer(kind).expect("real local tokenizer prerequisite");
        for tokenizer in [Some(local.as_ref()), None] {
            for (raw, candidate) in [
                ("const λ = 1;\r\n", "A1"),
                ("same", "same"),
                ("x", "a larger candidate"),
                (
                    "rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr",
                    "ccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                ),
            ] {
                let already_counted = count_tokens_with_tokenizer(raw, tokenizer);
                let old = select_with_local_tokenizer(raw, candidate.into(), kind, tokenizer);
                let proposed = select_with_local_tokenizer_and_raw_tokens(
                    raw,
                    candidate.into(),
                    kind,
                    tokenizer,
                    already_counted,
                );
                assert_eq!(
                    old,
                    proposed,
                    "kind={kind:?} availability={} raw={raw:?}",
                    tokenizer.is_some()
                );
                assert_eq!(proposed.raw_tokens, already_counted);
            }
        }
    }
    // ByteTokenizer makes exact tie and approximation-margin boundaries explicit.
    for (raw_count, candidate_count) in [(100, 100), (100, 97), (100, 95)] {
        for kind in [TokenizerKind::Cl100k, TokenizerKind::Claude] {
            let raw = "r".repeat(raw_count);
            let candidate = "c".repeat(candidate_count);
            let count = count_tokens_with_tokenizer(&raw, Some(&ByteTokenizer));
            assert_eq!(
                select_with_local_tokenizer(&raw, candidate.clone(), kind, Some(&ByteTokenizer)),
                select_with_local_tokenizer_and_raw_tokens(
                    &raw,
                    candidate,
                    kind,
                    Some(&ByteTokenizer),
                    count
                )
            );
        }
    }
}
