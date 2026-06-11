// src/tests/compression/rust.rs
//
// Tests for Rust language detection and compression.

#[cfg(test)]
mod rust_compression_tests {
    use crate::compression::language::{language_for_extension, looks_like_rust};
    use crate::compression::Fidelity;

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

    // ── Phase F: End-to-end pipeline tests ─────────────────────────

    /// Verify that compress_file_with_source works for .rs content.
    /// This is the pipeline provide_code_context uses for .rs files.
    #[test]
    fn test_rust_compress_file_pipeline() {
        use crate::cache::LocalStateCache;
        use crate::dictionary::PathDictionary;
        use crate::compression::pipeline::compress_file_with_source;
        use std::path::PathBuf;

        let mut dict = PathDictionary::new();
        let mut cache = LocalStateCache::new();

        let source = r#"
            use std::collections::HashMap;

            pub struct UserService {
                users: Vec<String>,
                cache: HashMap<u64, String>,
            }

            impl UserService {
                pub fn new() -> Self {
                    UserService { users: Vec::new(), cache: HashMap::new() }
                }

                pub async fn get_user(&self, id: u64) -> Option<&String> {
                    self.cache.get(&id)
                }
            }
        "#;

        let result = compress_file_with_source(
            PathBuf::from("test.rs"),
            Some(source),
            &mut dict,
            &mut cache,
            Fidelity::Low,
        );

        assert!(result.is_ok(), "compress_file_with_source should succeed for .rs: {:?}", result.err());
        let output = result.unwrap();
        assert!(!output.is_empty(), "output should not be empty");
        assert!(
            output.contains("UserService"),
            "output should contain class name 'UserService', got: {}",
            output
        );
        assert!(
            output.contains("get_user"),
            "output should contain method 'get_user', got: {}",
            output
        );
    }

    /// Verify that Medium fidelity works for .rs files.
    #[test]
    fn test_rust_compress_medium_fidelity() {
        use crate::cache::LocalStateCache;
        use crate::dictionary::PathDictionary;
        use crate::compression::pipeline::compress_file_with_source;
        use std::path::PathBuf;

        let mut dict = PathDictionary::new();
        let mut cache = LocalStateCache::new();

        let source = r#"
            pub struct Simple {
                field: i32,
            }
        "#;

        let result = compress_file_with_source(
            PathBuf::from("test.rs"),
            Some(source),
            &mut dict,
            &mut cache,
            Fidelity::Medium,
        );

        assert!(result.is_ok(), "Medium fidelity for .rs should succeed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.contains("Simple"), "output should contain 'Simple'");
    }
}
