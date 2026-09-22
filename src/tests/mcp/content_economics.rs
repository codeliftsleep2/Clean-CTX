use super::{
    CountConfidence, SelectedRepresentation, select_economic_content, select_with_local_tokenizer,
};
use crate::tokenizer::{Tokenizer, TokenizerKind};

fn bytes(value: &str) -> usize {
    value.len()
}

#[test]
fn smaller_complete_candidate_wins() {
    let selected = select_economic_content(
        "raw source document",
        "A1".into(),
        bytes,
        CountConfidence::ExactLocal,
    );
    assert_eq!(selected.selected, SelectedRepresentation::Candidate);
    assert_eq!(selected.text, "A1");
    assert_eq!(selected.selected_tokens(), 2);
    assert_eq!(selected.saved_tokens(), 17);
}

#[test]
fn raw_wins_a_tie() {
    let selected = select_economic_content("raw", "A1!".into(), bytes, CountConfidence::ExactLocal);
    assert_eq!(selected.selected, SelectedRepresentation::RawPassthrough);
    assert_eq!(selected.text, "raw");
    assert_eq!(selected.saved_tokens(), 0);
}

#[test]
fn larger_candidate_falls_back_to_byte_exact_raw() {
    let raw = "{\r\n  const λ = '§PATHMAP';\r\n}\r\n";
    let selected = select_economic_content(
        raw,
        "schema plus a much larger candidate".into(),
        bytes,
        CountConfidence::ExactLocal,
    );
    assert_eq!(selected.selected, SelectedRepresentation::RawPassthrough);
    assert_eq!(selected.text.as_bytes(), raw.as_bytes());
    assert_eq!(selected.selected_tokens(), raw.len());
}

#[test]
fn approximate_counter_requires_savings_beyond_both_sides_of_error_bound() {
    let rejected = select_economic_content(
        &"r".repeat(100),
        "c".repeat(97),
        bytes,
        CountConfidence::Approximate {
            error_basis_points: 200,
        },
    );
    assert_eq!(rejected.selected, SelectedRepresentation::RawPassthrough);

    let accepted = select_economic_content(
        &"r".repeat(100),
        "c".repeat(95),
        bytes,
        CountConfidence::Approximate {
            error_basis_points: 200,
        },
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
