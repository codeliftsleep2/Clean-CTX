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

use std::sync::OnceLock;
use tree_sitter::Language;

use crate::queries;

/// Thread-safe wrapper around `tree_sitter_typescript::LANGUAGE_TYPESCRIPT`.
/// Uses `OnceLock` to ensure only one thread ever initializes the WASM parser,
/// preventing the Windows deadlock that occurs when multiple threads race to
/// initialize tree-sitter's internal `OnceLock` simultaneously.
pub fn safe_typescript_language() -> Language {
    static LANG: OnceLock<Language> = OnceLock::new();
    LANG.get_or_init(|| tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()).clone()
}

/// Thread-safe wrapper around `tree_sitter_c_sharp::LANGUAGE`.
pub fn safe_csharp_language() -> Language {
    static LANG: OnceLock<Language> = OnceLock::new();
    LANG.get_or_init(|| tree_sitter_c_sharp::LANGUAGE.into()).clone()
}

/// Thread-safe wrapper around `tree_sitter_rust::LANGUAGE`.
pub fn safe_rust_language() -> Language {
    static LANG: OnceLock<Language> = OnceLock::new();
    LANG.get_or_init(|| tree_sitter_rust::LANGUAGE.into()).clone()
}

/// Thread-safe wrapper around `tree_sitter_java::LANGUAGE`.
pub fn safe_java_language() -> Language {
    static LANG: OnceLock<Language> = OnceLock::new();
    LANG.get_or_init(|| tree_sitter_java::LANGUAGE.into()).clone()
}

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
/// Single keywords like `class` or `public` appear in other languages
/// (especially TypeScript), so we require at least three signals or
/// one strong signal (`package`, `import java`, `@Override`, etc.).
///
/// FAANG audit: Added `javax.`, `jakarta.`, and `org.springframework`
/// as additional strong signals so Spring Boot / Jakarta EE files
/// are correctly detected as Java even without `import java.`.
/// Also added a TypeScript anti-signal check: files containing `=>`
/// (arrow functions), `: boolean`, `: string` (TypeScript type
/// annotations in method signatures), or TypeScript-style `import {`
/// are unlikely to be Java.
pub fn looks_like_java(source: &str) -> bool {
    // Anti-signal: TypeScript/JavaScript constructs that should NEVER appear in Java.
    if source.contains("=>") && !source.contains("->") {
        return false; // Arrow functions → TypeScript, not Java
    }
    if source.contains("export ") || source.contains("export default") {
        return false; // `export` keyword → TypeScript module, not Java
    }
    if let Some(import_line) = source.lines().find(|l| l.trim().starts_with("import ")) {
        if import_line.contains('{') && !import_line.contains("static ") {
            return false; // "import { Foo }" is TypeScript, not Java
        }
    }

    let has_package = source.contains("package ");
    let has_import_java = source.contains("import java.");
    let has_import_javax = source.contains("import javax.");
    let has_import_jakarta = source.contains("import jakarta.");
    let has_import_spring = source.contains("import org.springframework");
    let has_override = source.contains("@Override");
    let has_interface = source.contains("interface ");
    let has_class = source.contains("class ");
    let has_public = source.contains("public ");
    let has_private = source.contains("private ");
    let has_protected = source.contains("protected ");
    let has_extends = source.contains("extends ");
    let has_implements = source.contains("implements ");

    // Strong signals: Java-specific constructs
    let strong = has_package
        || has_import_java
        || has_import_javax
        || has_import_jakarta
        || has_import_spring
        || has_override;

    // Count all signals
    let signals = [
        has_package, has_import_java, has_import_javax, has_import_jakarta,
        has_import_spring, has_override, has_interface,
        has_class, has_public, has_private, has_protected,
        has_extends, has_implements,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    // Require either a strong signal or at least 3 weak signals
    // (increased from 2 to avoid false-positive on TypeScript files
    //  that use `class` + `public` + `private`)
    strong || signals >= 3
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
        (safe_csharp_language(), queries::CS_QUERY)
    } else if looks_like_rust(source) {
        (safe_rust_language(), queries::RS_QUERY)
    } else if looks_like_java(source) {
        (safe_java_language(), queries::JAVA_QUERY)
    } else {
        (safe_typescript_language(), queries::TS_QUERY)
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
        "ts" => Some((safe_typescript_language(), queries::TS_QUERY)),
        "cs" => Some((safe_csharp_language(), queries::CS_QUERY)),
        "rs" => Some((safe_rust_language(), queries::RS_QUERY)),
        "java" => Some((safe_java_language(), queries::JAVA_QUERY)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/compression/language.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/compression/rust.rs"]
mod rust_tests;