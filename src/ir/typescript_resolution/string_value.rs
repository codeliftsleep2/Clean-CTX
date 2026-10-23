//! Decode module/export names without changing their existing Unicode contract.
pub(super) fn decode(raw: &str) -> Option<String> {
    super::string_literal::decode_module(raw)
}
