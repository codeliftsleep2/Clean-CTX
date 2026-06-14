// src/tests/tokenizer.rs — Tests for the pluggable tokenizer module (R-19)

use crate::tokenizer::*;

#[test]
fn tokenizer_kind_from_str_cl100k() {
    assert_eq!(TokenizerKind::from_str_opt("cl100k"), Some(TokenizerKind::Cl100k));
    assert_eq!(TokenizerKind::from_str_opt("cl100k_base"), Some(TokenizerKind::Cl100k));
    assert_eq!(TokenizerKind::from_str_opt("gpt4"), Some(TokenizerKind::Cl100k));
    assert_eq!(TokenizerKind::from_str_opt("gpt-4"), Some(TokenizerKind::Cl100k));
    assert_eq!(TokenizerKind::from_str_opt("gpt35"), Some(TokenizerKind::Cl100k));
    assert_eq!(TokenizerKind::from_str_opt("gpt-3.5"), Some(TokenizerKind::Cl100k));
}

#[test]
fn tokenizer_kind_from_str_o200k() {
    assert_eq!(TokenizerKind::from_str_opt("o200k"), Some(TokenizerKind::O200k));
    assert_eq!(TokenizerKind::from_str_opt("o200k_base"), Some(TokenizerKind::O200k));
    assert_eq!(TokenizerKind::from_str_opt("gpt4o"), Some(TokenizerKind::O200k));
    assert_eq!(TokenizerKind::from_str_opt("gpt-4o"), Some(TokenizerKind::O200k));
}

#[test]
fn tokenizer_kind_from_str_claude() {
    assert_eq!(TokenizerKind::from_str_opt("claude"), Some(TokenizerKind::Claude));
    assert_eq!(TokenizerKind::from_str_opt("anthropic"), Some(TokenizerKind::Claude));
    assert_eq!(TokenizerKind::from_str_opt("claude3"), Some(TokenizerKind::Claude));
    assert_eq!(TokenizerKind::from_str_opt("claude-3"), Some(TokenizerKind::Claude));
}

#[test]
fn tokenizer_kind_from_str_llama3() {
    assert_eq!(TokenizerKind::from_str_opt("llama3"), Some(TokenizerKind::Llama3));
    assert_eq!(TokenizerKind::from_str_opt("llama-3"), Some(TokenizerKind::Llama3));
    assert_eq!(TokenizerKind::from_str_opt("llama"), Some(TokenizerKind::Llama3));
    assert_eq!(TokenizerKind::from_str_opt("meta"), Some(TokenizerKind::Llama3));
}

#[test]
fn tokenizer_kind_from_str_unknown() {
    assert_eq!(TokenizerKind::from_str_opt("unknown"), None);
    assert_eq!(TokenizerKind::from_str_opt(""), None);
    assert_eq!(TokenizerKind::from_str_opt("CL100K"), Some(TokenizerKind::Cl100k));
}

#[test]
fn tokenizer_kind_display() {
    assert_eq!(TokenizerKind::Cl100k.to_string(), "cl100k");
    assert_eq!(TokenizerKind::O200k.to_string(), "o200k");
    assert_eq!(TokenizerKind::Claude.to_string(), "claude");
    assert_eq!(TokenizerKind::Llama3.to_string(), "llama3");
}

#[test]
fn tokenizer_kind_default() {
    assert_eq!(TokenizerKind::default(), TokenizerKind::O200k);
}

#[test]
fn create_tokenizer_cl100k() {
    let tok = create_tokenizer(TokenizerKind::Cl100k).unwrap();
    assert_eq!(tok.name(), "cl100k");
    let count = tok.count_tokens("Hello, world!");
    assert!(count > 0);
}

#[test]
fn create_tokenizer_o200k() {
    let tok = create_tokenizer(TokenizerKind::O200k).unwrap();
    assert_eq!(tok.name(), "o200k");
    let count = tok.count_tokens("Hello, world!");
    assert!(count > 0);
}

#[test]
fn create_tokenizer_claude() {
    let tok = create_tokenizer(TokenizerKind::Claude).unwrap();
    assert_eq!(tok.name(), "claude");
    let count = tok.count_tokens("Hello, world!");
    assert!(count > 0);
}

#[test]
fn create_tokenizer_llama3() {
    let tok = create_tokenizer(TokenizerKind::Llama3).unwrap();
    assert_eq!(tok.name(), "llama3");
    let count = tok.count_tokens("Hello, world!");
    assert!(count > 0);
}

#[test]
fn tokenizer_encode_returns_nonempty() {
    let tok = create_tokenizer(TokenizerKind::Cl100k).unwrap();
    let tokens = tok.encode("Hello, world!");
    assert!(!tokens.is_empty());
}

#[test]
fn tokenizer_count_tokens_empty_string() {
    let tok = create_tokenizer(TokenizerKind::Cl100k).unwrap();
    let count = tok.count_tokens("");
    assert_eq!(count, 0);
}

#[test]
fn resolve_tokenizer_kind_tool_arg_priority() {
    // Tool arg takes priority over config default
    let kind = resolve_tokenizer_kind(Some("o200k"), Some("claude"));
    assert_eq!(kind, TokenizerKind::O200k);
}

#[test]
fn resolve_tokenizer_kind_config_default() {
    // Config default used when tool arg is None
    let kind = resolve_tokenizer_kind(None, Some("llama3"));
    assert_eq!(kind, TokenizerKind::Llama3);
}

#[test]
fn resolve_tokenizer_kind_fallback() {
    // Falls back to o200k (the new default) when both are None
    let kind = resolve_tokenizer_kind(None, None);
    assert_eq!(kind, TokenizerKind::O200k);
}

#[test]
fn resolve_tokenizer_kind_invalid_tool_arg() {
    // Invalid tool arg falls through to config default
    let kind = resolve_tokenizer_kind(Some("invalid"), Some("o200k"));
    assert_eq!(kind, TokenizerKind::O200k);
}

#[test]
fn resolve_tokenizer_kind_invalid_config() {
    // Invalid config falls through to default (o200k)
    let kind = resolve_tokenizer_kind(None, Some("invalid"));
    assert_eq!(kind, TokenizerKind::O200k);
}

#[test]
fn init_all_tokenizers_succeeds() {
    // Should not panic
    let result = init_all_tokenizers();
    assert!(result.is_ok());
}

#[test]
fn tokenizer_kind_serialization_roundtrip() {
    let kind = TokenizerKind::O200k;
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, "\"o200k\"");
    let deserialized: TokenizerKind = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, TokenizerKind::O200k);
}

#[test]
fn tokenizer_kind_aliases_via_from_str_opt() {
    // Aliases like "gpt-4o" are handled by from_str_opt, not serde.
    // Config files should use canonical names ("o200k"), but tool
    // arguments support aliases via from_str_opt.
    assert_eq!(TokenizerKind::from_str_opt("gpt-4o"), Some(TokenizerKind::O200k));
    assert_eq!(TokenizerKind::from_str_opt("gpt-4"), Some(TokenizerKind::Cl100k));
    assert_eq!(TokenizerKind::from_str_opt("anthropic"), Some(TokenizerKind::Claude));
    assert_eq!(TokenizerKind::from_str_opt("meta"), Some(TokenizerKind::Llama3));
}

#[test]
fn llama3_ratio_adjustment() {
    // Llama-3 should produce more tokens than o200k for the same text
    let o200k = create_tokenizer(TokenizerKind::O200k).unwrap();
    let llama3 = create_tokenizer(TokenizerKind::Llama3).unwrap();
    let text = "fn main() { println!(\"Hello, world!\"); }";
    let o200k_count = o200k.count_tokens(text);
    let llama3_count = llama3.count_tokens(text);
    // Llama-3 should have more tokens due to ratio adjustment
    assert!(llama3_count >= o200k_count, "llama3 ({}) should be >= o200k ({})", llama3_count, o200k_count);
}