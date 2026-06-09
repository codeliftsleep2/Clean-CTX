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

/// Pick the tree-sitter `Language` and query string for the given source
/// content. When `extension` is supplied it is used as a hint to break
/// ties; otherwise the content heuristic alone decides.
///
/// The returned tuple is `(Language, &'static str query)` — the static
/// query reference is safe because `crate::queries::TS_QUERY` and
/// `crate::queries::CS_QUERY` are both `'static` `&str` constants.
pub fn detect_language(source: &str) -> (Language, &'static str) {
    if looks_like_csharp(source) {
        (tree_sitter_c_sharp::language(), queries::CS_QUERY)
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
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/compression/language.rs"]
mod tests;
