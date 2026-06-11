// src/tests/compression/rust.rs
//
// Tests for Rust language detection and compression.

#[cfg(test)]
mod rust_compression_tests {
    use crate::compression::language::{language_for_extension, looks_like_rust};

    #[test]
    fn test_rs_extension_detection() {
        let result = language_for_extension("rs");
        assert!(result.is_some());
    }

    #[test]
    fn test_rs_extension_returns_rust_language() {
        let result = language_for_extension("rs");
        assert!(result.is_some());
        // The query string should be RS_QUERY
        let (_, query) = result.unwrap();
        assert!(query.contains("struct_item"));
        assert!(query.contains("impl_item"));
    }

    #[test]
    fn test_looks_like_rust_with_fn() {
        // "fn" alone is not a strong signal — needs a second signal
        assert!(looks_like_rust("pub fn main() {}"));
    }

    #[test]
    fn test_looks_like_rust_with_struct() {
        // "struct" + "pub" = 2 signals
        assert!(looks_like_rust("pub struct MyStruct { field: i32 }"));
    }

    #[test]
    fn test_looks_like_rust_with_impl() {
        // "impl" is a strong signal — single match is enough
        assert!(looks_like_rust("impl MyStruct { fn new() {} }"));
    }

    #[test]
    fn test_looks_like_rust_with_trait() {
        // "trait" is a strong signal
        assert!(looks_like_rust("trait MyTrait { fn method(&self); }"));
    }

    #[test]
    fn test_looks_like_rust_with_pub_fn() {
        // "pub" + "fn" = 2 signals
        assert!(looks_like_rust("pub fn public_function() {}"));
    }

    #[test]
    fn test_looks_like_rust_with_use_and_fn() {
        // "use" + "fn" = 2 signals
        assert!(looks_like_rust("use crate::module;\nfn main() {}"));
    }

    #[test]
    fn test_looks_like_rust_with_mod_and_struct() {
        // "mod" + "struct" = 2 signals
        assert!(looks_like_rust("mod submodule;\nstruct Foo {}"));
    }

    #[test]
    fn test_not_looks_like_rust_single_use() {
        // Single "use" is not enough (matches Python, TypeScript, etc.)
        assert!(!looks_like_rust("use crate::module;"));
    }

    #[test]
    fn test_not_looks_like_rust_single_mod() {
        // Single "mod" is not enough (matches CSS @media)
        assert!(!looks_like_rust("mod submodule;"));
    }

    #[test]
    fn test_not_looks_like_rust_single_pub() {
        // Single "pub" is not enough
        assert!(!looks_like_rust("pub something"));
    }

    #[test]
    fn test_not_looks_like_rust_empty() {
        assert!(!looks_like_rust(""));
    }

    #[test]
    fn test_not_looks_like_rust_plain_text() {
        assert!(!looks_like_rust("Hello, world!"));
    }
}