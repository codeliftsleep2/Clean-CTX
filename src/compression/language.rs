// src/compression/language.rs
//
// SHARED language detection. The two orchestrators (`compress_file` and
// `build_snapshot`) used to use different strategies:
//   - `compressor.rs` matched on the file extension (`.ts` / `.js` / `.cs`)
//   - `diff/builder.rs` used a content heuristic (looks for `namespace `
//     / `using System` / `public class` / `private void`)
//
// The two strategies are not equivalent: a `namespace` is a much stronger
// signal than an extension, and a `.cs` file with no C# constructs should
// fall back to the TypeScript parser. Phase 2 funnels both call sites
// through a single heuristic that does not require a file path, and the
// orchestrators map the result to a tree-sitter `Language` + query
// string.

use tree_sitter::Language;

use crate::queries;

/// Returns `true` if the source text looks like C#. The heuristic is
/// deliberately narrow: C# has very distinctive keywords (`namespace`,
/// `using System`, `public class`, `private void`) that rarely appear
/// in TypeScript / JavaScript source. A `.cs` file containing only
/// strings or comments is rare; if that ever happens, the diff path
/// will fall back to TypeScript on its second pass.
pub fn looks_like_csharp(source: &str) -> bool {
    source.contains("namespace ")
        || source.contains("using System")
        || source.contains("public class ")
        || source.contains("private void ")
}

/// Returns `true` if the source text looks like Rust. The heuristic
/// requires multiple Rust-specific signals to reduce false positives.
/// Single keywords like `use` or `mod` appear in other languages (Python,
/// CSS, TypeScript), so we require at least two signals or one strong
/// signal (`impl`, `trait`, `fn` with `pub`).
pub fn looks_like_rust(source: &str) -> bool {
    let has_fn = source.contains("fn ");
    let has_struct = source.contains("struct ");
    let has_enum = source.contains("enum ");
    let has_impl = source.contains("impl ");
    let has_trait = source.contains("trait ");
    let has_pub = source.contains("pub ") || source.contains("pub(");
    let has_use = source.contains("use ");
    let has_mod = source.contains("mod ");

    // Strong signals: impl and trait are very Rust-specific
    let strong = has_impl || has_trait;

    // Count all signals
    let signals = [
        has_fn, has_struct, has_enum, has_impl, has_trait,
        has_pub, has_use, has_mod,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    // Require either a strong signal or at least 2 signals
    strong || signals >= 2
}

/// Returns `true` if the source text looks like Java. The heuristic
/// requires multiple Java-specific signals to reduce false positives.
/// Single keywords like `class` or `public` appear in other languages,
/// so we require at least two signals or one strong signal
/// (`package`, `import java`, `@Override`, `interface`).
pub fn looks_like_java(source: &str) -> bool {
    let has_package = source.contains("package ");
    let has_import_java = source.contains("import java.");
    let has_override = source.contains("@Override");
    let has_interface = source.contains("interface ");
    let has_class = source.contains("class ");
    let has_public = source.contains("public ");
    let has_private = source.contains("private ");
    let has_protected = source.contains("protected ");
    let has_extends = source.contains("extends ");
    let has_implements = source.contains("implements ");

    // Strong signals: package and import java are very Java-specific
    let strong = has_package || has_import_java || has_override;

    // Count all signals
    let signals = [
        has_package, has_import_java, has_override, has_interface,
        has_class, has_public, has_private, has_protected,
        has_extends, has_implements,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    // Require either a strong signal or at least 2 signals
    strong || signals >= 2
}

/// Pick the tree-sitter `Language` and query string for the given source
/// content. When `extension` is supplied it is used as a hint to break
/// ties; otherwise the content heuristic alone decides.
///
/// The returned tuple is `(Language, &'static str query)` — the static
/// query reference is safe because `crate::queries::TS_QUERY`,
/// `crate::queries::CS_QUERY`, `crate::queries::RS_QUERY`, and
/// `crate::queries::JAVA_QUERY` are all `'static` `&str` constants.
pub fn detect_language(source: &str) -> (Language, &'static str) {
    if looks_like_csharp(source) {
        (tree_sitter_c_sharp::language(), queries::CS_QUERY)
    } else if looks_like_rust(source) {
        (tree_sitter_rust::language(), queries::RS_QUERY)
    } else if looks_like_java(source) {
        (tree_sitter_java::language(), queries::JAVA_QUERY)
    } else {
        (tree_sitter_typescript::language_typescript(), queries::TS_QUERY)
    }
}

/// Map a file extension to a `(Language, query)` pair, or `None` if the
/// extension is not supported. Used by `compress_file` / streaming variant,
/// which must reject unsupported file types with a hard error.
///
/// F-FULL-16: `.js` files are rejected with a clear error because the
/// TypeScript grammar does not match all JavaScript constructs (e.g.,
/// CommonJS `require()` calls are not captured, `function` keyword
/// definitions are not recognised). Use `.ts` for full support, or open
/// an issue requesting JavaScript grammar integration.
pub fn language_for_extension(extension: &str) -> Option<(Language, &'static str)> {
    match extension {
        "ts" => Some((tree_sitter_typescript::language_typescript(), queries::TS_QUERY)),
        "cs" => Some((tree_sitter_c_sharp::language(), queries::CS_QUERY)),
        "rs" => Some((tree_sitter_rust::language(), queries::RS_QUERY)),
        "java" => Some((tree_sitter_java::language(), queries::JAVA_QUERY)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/compression/language.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/compression/rust.rs"]
mod rust_tests;
