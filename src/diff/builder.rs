// src/diff/builder.rs
//
// Snapshot construction: parse a source string with tree-sitter, walk the
// captures, and assemble a `CapturedStructure`.
//
// Phase 2: the tree-sitter capture walk, the language heuristic, and the
// marker construction logic now live in `crate::compression::*`. This
// file is reduced to:
//   1. Choosing the language (with a content-based fallback)
//   2. Driving the SHARED capture pipeline
//   3. Assembling the `CapturedStructure` from the resulting `CapEntry`s

use crate::compaction::method::find_method_params;
use crate::compaction::{
    compact_expression, compact_import, extract_class_meta, extract_class_name, extract_field,
    extract_method_sig, extract_rust_struct_name,
};
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::detect_language;
use crate::compression::markers::build_marker;
use crate::queries;

use super::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

/// A parser configuration: language factory, query string, and label.
type ParserConfig = (
    fn() -> Option<tree_sitter::Language>,
    &'static str,
    &'static str,
);

/// All supported parser configurations, in the order they should be tried.
/// G2-3 diff audit: Java was previously missing from this list entirely,
/// so a `.java` file with zero captures fell back to the csharp/rust
/// parsers and Java classes were never captured — a false negative for
/// `diff_commits`. `JAVA_QUERY` is now included.
const ALL_PARSERS: &[ParserConfig] = &[
    (
        crate::compression::language::safe_rust_language,
        queries::RS_QUERY,
        "rust",
    ),
    (
        crate::compression::language::safe_csharp_language,
        queries::CS_QUERY,
        "csharp",
    ),
    (
        crate::compression::language::safe_typescript_language,
        queries::TS_QUERY,
        "typescript",
    ),
    (
        crate::compression::language::safe_java_language,
        queries::JAVA_QUERY,
        "java",
    ),
];

/// Build a structural snapshot by parsing the source with tree-sitter and
/// walking the captures. Mirrors the capture logic in `compressor::compress_file`
/// so the two are directly comparable.
///
/// Phase D: The fallback now tries all supported languages (Rust, C#, TS)
/// instead of only TS ↔ C#. If the first parser chosen by `detect_language`
/// yields no captures, the remaining two are tried in priority order.
///
/// G2-1/G2-4 diff audit: the snapshot is built with `Fidelity::Medium`
/// regardless of the caller's `fidelity`. `extract_field` returns an empty
/// string at `Low` and `extract_method_sig` returns the FULL raw method
/// body at `Edit`/`Verbatim` — both would corrupt the diffed state (fields
/// invisible at the default `Low` config, or multi-KB signatures at
/// `Edit`/`Verbatim`). The snapshot must be fidelity-stable; the caller's
/// `fidelity` is still honored by the *formatter* for output depth.
pub fn build_snapshot(
    source: &str,
    _fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    // G2-1/G2-4: always extract at Medium so the snapshot is
    // fidelity-stable regardless of the caller's requested level.
    // Medium is the only level where `extract_field` calls
    // `normalize_csharp_type` (converting C# `string Name` to
    // `Name:string`); High leaves `public string Name` as the field
    // key, breaking `field_key` grouping. Medium also strips method
    // bodies (no multi-KB signatures at Edit/Verbatim).
    let fidelity = Fidelity::Medium;
    let (first_lang, first_query) = detect_language(source);

    // Phase D: Build an ordered list of parsers to try, starting with the
    // detected language, followed by the others (deduplicating).
    //
    // We match against the query string to identify which parser was chosen.
    // G2-2 audit: the previous if/else chain fell through to "typescript"
    // for Java, so the dedup logic skipped the real java fallback and
    // mislabeled java == typescript. Look up the label via ALL_PARSERS.
    let first_label = ALL_PARSERS
        .iter()
        .find(|(_, q, _)| *q == first_query)
        .map(|(_, _, label)| *label)
        .unwrap_or("typescript");

    // Collect parsers to try: first choice first, then the others.
    // Only push languages whose feature is enabled (`lang_fn()` returns
    // `Some`) — otherwise `try_build_with` would panic on the
    // `Language must be Some` expect when a grammar feature is disabled.
    let mut candidates: Vec<(&str, Option<tree_sitter::Language>, &str)> = Vec::with_capacity(4);
    // Push the detected parser first
    candidates.push((first_label, Some(first_lang.clone()), first_query));
    // Push the remaining ones in ALL_PARSERS order, skipping the detected one
    for (lang_fn, query, label) in ALL_PARSERS {
        if *label != first_label {
            candidates.push((label, lang_fn(), query));
        }
    }
    // Drop candidates whose language feature is disabled.
    candidates.retain(|(_, lang, _)| lang.is_some());

    // Try each parser in order. Return the first one that produces captures.
    // On error, log a warning and continue to the next parser rather than
    // propagating the error — a single query compilation failure (e.g.,
    // `switch_statement` not recognised by the Java grammar) must not crash
    // the entire fallback chain.
    //
    // Audit fix: only `Ok` results are stored as `last_ok_result`;
    // `Err` results are logged and skipped. This ensures
    // that when all parsers fail to produce captures and the final parser
    // also fails with a compilation error, we return a valid empty
    // CapturedStructure instead of propagating the error.
    let mut last_ok_result: Option<CapturedStructure> = None;
    for (_label, lang, query) in &candidates {
        let result = match try_build_with(lang.clone(), query, source, fidelity) {
            Ok(snap) => snap,
            Err(e) => {
                tracing::warn!(
                    "build_snapshot: parser {} failed, trying next: {}",
                    _label,
                    e,
                );
                continue;
            }
        };
        // G2-2 audit: include orphan_fields and orphan_methods in the
        // success check — a file with only top-level functions or
        // top-level fields previously fell through to the wrong parser.
        if !result.classes.is_empty()
            || !result.imports.is_empty()
            || !result.orphan_fields.is_empty()
            || !result.orphan_methods.is_empty()
        {
            return Ok(result);
        }
        // Empty result — save as fallback and try next parser
        if last_ok_result.is_none() {
            last_ok_result = Some(result);
        }
    }

    // If none produced meaningful captures, return the last non-error result
    // (even if empty, so callers get a valid CapturedStructure), or construct
    // a new empty one as a safety net.
    Ok(last_ok_result.unwrap_or(CapturedStructure {
        imports: Vec::new(),
        classes: Vec::new(),
        orphan_fields: Vec::new(),
        orphan_methods: Vec::new(),
    }))
}

fn try_build_with(
    language: Option<tree_sitter::Language>,
    query_string: &str,
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    // Run the SHARED capture pipeline. The closure maps each
    // (capture_name, raw_text) pair to the normalised text the diff
    // path wants stored in the resulting CapEntry. F-08: pass the
    // real `fidelity` through so the per-capture closures honour it.
    let all_captures = run_capture_pipeline(
        language.expect("Language must be Some - check feature flags"),
        query_string,
        source,
        fidelity,
        |capture_name, raw, f| {
            match capture_name {
                "class.root" => Some(extract_class_name(raw)),
                // C# interfaces and records are distinct AST nodes but
                // share the same class-like shape. F-01 diff audit.
                "interface.root" | "record.root" => Some(extract_class_name(raw)),
                // Rust type declarations: struct, enum, trait, impl
                "struct.root" | "enum.root" | "trait.root" | "impl.root" => {
                    Some(extract_rust_struct_name(raw))
                }
                // C# constructors and TS top-level functions are method-like.
                "method.root" | "constructor.root" | "func.root" => {
                    Some(extract_method_sig(raw, f))
                }
                "field.root" => Some(extract_field(raw, f)),
                "import.root" | "mod.root" => Some(compact_import(raw, f)),
                "type.root" => Some(compact_expression(raw, f)),
                _ => Some(compact_expression(raw, f)),
            }
        },
    )?;

    let mut classes: Vec<CapturedClass> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let mut orphan_fields: Vec<String> = Vec::new();
    let mut orphan_methods: Vec<CapturedMethod> = Vec::new();
    let mut pending_fields: Vec<String> = Vec::new();
    let mut pending_markers: Vec<String> = Vec::new();

    for cap in &all_captures {
        match cap.name.as_str() {
            "import.root" => {
                if !cap.text.is_empty() {
                    imports.push(cap.text.clone());
                }
            }
            "class.root" | "struct.root" | "enum.root" | "trait.root" | "impl.root"
            | "interface.root" | "record.root" => {
                if let Some(last) = classes.last_mut() {
                    if last.fields.is_empty() && !pending_fields.is_empty() {
                        last.fields = std::mem::take(&mut pending_fields);
                    }
                } else if !pending_fields.is_empty() {
                    orphan_fields.append(&mut pending_fields);
                }
                classes.push(CapturedClass {
                    name: cap.text.clone(),
                    class_meta: extract_class_meta(&cap.raw_text),
                    fields: Vec::new(),
                    methods: Vec::new(),
                });
                pending_markers.clear();
            }
            "method.root" | "constructor.root" | "func.root" => {
                let method = CapturedMethod {
                    sig: cap.text.clone(),
                    markers: std::mem::take(&mut pending_markers),
                    body: extract_method_body(&cap.raw_text),
                };
                if let Some(last) = classes.last_mut() {
                    last.methods.push(method);
                } else {
                    // G2-2 audit: top-level functions (TS `function` /
                    // `export function`, C# top-level statements) were
                    // previously dropped when no class was open — a false
                    // negative for files with only top-level functions.
                    orphan_methods.push(method);
                }
            }
            "field.root" => {
                if !cap.text.is_empty() {
                    pending_fields.push(cap.text.clone());
                }
            }
            _ => {
                if fidelity == Fidelity::Low {
                    continue;
                }
                // Delegate marker construction to the SHARED module.
                if let Some(marker) = build_marker(&cap.name, &cap.text)
                    && pending_markers.last().map(|m| m != &marker).unwrap_or(true)
                {
                    pending_markers.push(marker);
                }
            }
        }
    }

    if let Some(last) = classes.last_mut() {
        if last.fields.is_empty() && !pending_fields.is_empty() {
            last.fields = pending_fields;
        }
    } else if !pending_fields.is_empty() {
        orphan_fields.extend(pending_fields);
    }

    Ok(CapturedStructure {
        imports,
        classes,
        orphan_fields,
        orphan_methods,
    })
}

/// Extract a normalized body fingerprint from a method's raw source text.
///
/// Tree-sitter's `method.root` capture includes the full method declaration
/// (signature + body). From that raw text we take everything after the
/// closing `)` of the parameter list, collapse whitespace, and trim. The
/// result is compared in `diff_snapshots` so **body-only** changes (logic
/// fixes with unchanged signatures) are no longer reported as `Unchanged`.
///
/// Returns `None` when no body follows the signature (e.g. abstract
/// methods ending in `;`, or malformed input).
fn extract_method_body(raw: &str) -> Option<String> {
    // Only consider the signature portion (everything before the first
    // `{`). Parens in the method BODY (e.g. `api.get(id)` call
    // arguments) would otherwise be found by `find_method_params` as the
    // "last" depth-0 group, producing a wrong body fingerprint. F-03 diff
    // audit: the previous implementation used `raw.find('(')` which found
    // the FIRST `(`, breaking body extraction for methods with tuple
    // return types. Scanning only the signature part solves both.
    let body_start = raw.find('{')?;
    let sig_part = &raw[..body_start];
    let (_open, close) = find_method_params(sig_part)?;
    let after = &raw[close + 1..];
    // If nothing follows the signature (or it's just a `;`), there's no
    // body — abstract/interface methods.
    let trimmed = after.trim();
    if trimmed.is_empty() || trimmed == ";" {
        return None;
    }
    // Collapse all whitespace runs to a single space so cosmetic
    // reformatting doesn't produce a spurious diff, but real
    // body-content changes still do.
    let normalized = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/diff/builder.rs"]
mod tests;
